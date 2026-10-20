// SSIMULACRA 2 de libjxl (tools/ssimulacra2.cc) para Apolo (ADR 0022): la
// misma nota que la herramienta `ssimulacra2` oficial, pero desde píxeles en
// memoria en vez de ficheros. Con transparencia, como la herramienta: se mide
// sobre un fondo oscuro (0,1) y uno claro (0,9) y vale la peor de las dos.
//
// Con menos memoria (entrega 8b): la de libjxl tiene a la vez unas once
// imágenes de tres canales en coma flotante, 3,5 GB con una foto de 24 MP. Su
// cálculo es independiente por canal (cada canal se desenfoca y se suma aparte),
// así que aquí va canal a canal, con las mismas operaciones en el mismo orden,
// y la imagen lineal de la escala siguiente se prepara en cuanto se ha pasado a
// XYB la actual, para soltar la grande. La nota es la misma bit a bit: lo
// comprueba `crates/nucleo/tests/ssimulacra2_ligero.rs` frente a la función de libjxl, que sigue aquí.
//
// Se incluye el .cc entero, y no solo su cabecera, para usar sus funciones
// internas (Downsample, Blur, ToXYB…) sin copiarlas.

#include "tools/ssimulacra2.cc"

#include <jxl/encode.h>
#include <jxl/types.h>

#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <memory>
#include <utility>

#include "lib/extras/codec_in_out.h"
#include "lib/extras/packed_image.h"
#include "lib/extras/packed_image_convert.h"
#include "tools/no_memory_manager.h"

namespace {

// Píxeles de 8 bits en sRGB, RGB (3 canales) o RGBA (4), a un CodecInOut.
bool Cargar(const uint8_t* pixeles, size_t ancho, size_t alto, int canales,
            jxl::CodecInOut* io) {
  jxl::extras::PackedPixelFile ppf;
  ppf.info.xsize = ancho;
  ppf.info.ysize = alto;
  ppf.info.bits_per_sample = 8;
  ppf.info.num_color_channels = 3;
  ppf.info.alpha_bits = canales == 4 ? 8 : 0;
  ppf.info.num_extra_channels = canales == 4 ? 1 : 0;
  ppf.info.orientation = JXL_ORIENT_IDENTITY;
  ppf.info.uses_original_profile = JXL_TRUE;
  JxlColorEncodingSetToSRGB(&ppf.color_encoding, JXL_FALSE);
  JxlPixelFormat formato = {static_cast<uint32_t>(canales), JXL_TYPE_UINT8,
                            JXL_NATIVE_ENDIAN, 0};
  auto imagen = jxl::extras::PackedImage::Create(ancho, alto, formato);
  if (!imagen.ok()) return false;
  jxl::extras::PackedImage im = std::move(imagen).value_();
  std::memcpy(im.pixels(), pixeles, ancho * alto * canales);
  ppf.frames.emplace_back(std::move(im));
  return static_cast<bool>(
      jxl::extras::ConvertPackedPixelFileToCodecInOut(ppf, nullptr, io));
}

// Multiply de libjxl, de un solo canal.
void MultiplicarCanal(const Image3F& a, const Image3F& b, size_t c,
                      ImageF* mul) {
  for (size_t y = 0; y < a.ysize(); ++y) {
    const float* JXL_RESTRICT in1 = a.PlaneRow(c, y);
    const float* JXL_RESTRICT in2 = b.PlaneRow(c, y);
    float* JXL_RESTRICT out = mul->Row(y);
    for (size_t x = 0; x < a.xsize(); ++x) {
      out[x] = in1[x] * in2[x];
    }
  }
}

// SSIMMap de libjxl, de un solo canal: escribe sus dos medias.
void SSIMMapCanal(const ImageF& m1, const ImageF& m2, const ImageF& s11,
                  const ImageF& s22, const ImageF& s12,
                  double* plane_averages) {
  const double onePerPixels = 1.0 / (m1.ysize() * m1.xsize());
  double sum1[2] = {0.0};
  for (size_t y = 0; y < m1.ysize(); ++y) {
    const float* JXL_RESTRICT row_m1 = m1.ConstRow(y);
    const float* JXL_RESTRICT row_m2 = m2.ConstRow(y);
    const float* JXL_RESTRICT row_s11 = s11.ConstRow(y);
    const float* JXL_RESTRICT row_s22 = s22.ConstRow(y);
    const float* JXL_RESTRICT row_s12 = s12.ConstRow(y);
    for (size_t x = 0; x < m1.xsize(); ++x) {
      float mu1 = row_m1[x];
      float mu2 = row_m2[x];
      float mu11 = mu1 * mu1;
      float mu22 = mu2 * mu2;
      float mu12 = mu1 * mu2;
      float num_m = 1.0 - (mu1 - mu2) * (mu1 - mu2);
      float num_s = 2 * (row_s12[x] - mu12) + kC2;
      float denom_s = (row_s11[x] - mu11) + (row_s22[x] - mu22) + kC2;
      double d = 1.0 - (num_m * num_s / denom_s);
      d = std::max(d, 0.0);
      sum1[0] += d;
      sum1[1] += quartic(d);
    }
  }
  plane_averages[0] = onePerPixels * sum1[0];
  plane_averages[1] = sqrt(sqrt(onePerPixels * sum1[1]));
}

// EdgeDiffMap de libjxl, de un solo canal: escribe sus cuatro medias.
void EdgeDiffMapCanal(const Image3F& img1, const ImageF& mu1,
                      const Image3F& img2, const ImageF& mu2, size_t c,
                      double* plane_averages) {
  const double onePerPixels = 1.0 / (img1.ysize() * img1.xsize());
  double sum1[4] = {0.0};
  for (size_t y = 0; y < img1.ysize(); ++y) {
    const float* JXL_RESTRICT row1 = img1.PlaneRow(c, y);
    const float* JXL_RESTRICT row2 = img2.PlaneRow(c, y);
    const float* JXL_RESTRICT rowm1 = mu1.ConstRow(y);
    const float* JXL_RESTRICT rowm2 = mu2.ConstRow(y);
    for (size_t x = 0; x < img1.xsize(); ++x) {
      double d1 = (1.0 + std::abs(row2[x] - rowm2[x])) /
                      (1.0 + std::abs(row1[x] - rowm1[x])) -
                  1.0;
      double artifact = std::max(d1, 0.0);
      sum1[0] += artifact;
      sum1[1] += quartic(artifact);
      double detail_lost = std::max(-d1, 0.0);
      sum1[2] += detail_lost;
      sum1[3] += quartic(detail_lost);
    }
  }
  plane_averages[0] = onePerPixels * sum1[0];
  plane_averages[1] = sqrt(sqrt(onePerPixels * sum1[1]));
  plane_averages[2] = onePerPixels * sum1[2];
  plane_averages[3] = sqrt(sqrt(onePerPixels * sum1[3]));
}

// ComputeSSIMULACRA2 de libjxl, canal a canal. Usa y cambia `orig` y `dist`
// (la de libjxl trabaja sobre copias): quien la llama ya no las necesita.
StatusOr<Msssim> Ligero(ImageBundle& orig, ImageBundle& dist, float bg) {
  JxlMemoryManager* memory_manager = jpegxl::tools::NoMemoryManager();
  Msssim msssim;

  if (orig.HasAlpha()) AlphaBlend(orig, bg);
  if (dist.HasAlpha()) AlphaBlend(dist, bg);
  orig.ClearExtraChannels();
  dist.ClearExtraChannels();

  JXL_RETURN_IF_ERROR(orig.TransformTo(
      jxl::ColorEncoding::LinearSRGB(orig.IsGray()), *JxlGetDefaultCms()));
  JXL_RETURN_IF_ERROR(dist.TransformTo(
      jxl::ColorEncoding::LinearSRGB(dist.IsGray()), *JxlGetDefaultCms()));

  Image3F img1;
  Image3F img2;
  JXL_RETURN_IF_ERROR(ToXYB(orig, &img1));
  JXL_RETURN_IF_ERROR(ToXYB(dist, &img2));
  MakePositiveXYB(img1);
  MakePositiveXYB(img2);

  JXL_ASSIGN_OR_RETURN(Blur blur, Blur::Create(img1.xsize(), img1.ysize()));

  for (int scale = 0; scale < kNumScales; scale++) {
    // Como en libjxl: aquí img1 tiene todavía el tamaño de la escala anterior.
    if (img1.xsize() < 8 || img1.ysize() < 8) {
      break;
    }
    if (scale) {
      // `orig` y `dist` ya están reducidas: se hizo al final de la anterior.
      JXL_RETURN_IF_ERROR(ToXYB(orig, &img1));
      JXL_RETURN_IF_ERROR(ToXYB(dist, &img2));
      MakePositiveXYB(img1);
      MakePositiveXYB(img2);
    }
    // La lineal de la escala siguiente, ya: libjxl la saca de la misma imagen
    // al principio de la siguiente vuelta, pero así la grande se suelta antes
    // de reservar los canales de esta.
    if (scale + 1 < kNumScales) {
      JXL_ASSIGN_OR_RETURN(Image3F tmp, Downsample(*orig.color(), 2, 2));
      JXL_RETURN_IF_ERROR(orig.SetFromImage(
          std::move(tmp), jxl::ColorEncoding::LinearSRGB(orig.IsGray())));
      JXL_ASSIGN_OR_RETURN(tmp, Downsample(*dist.color(), 2, 2));
      JXL_RETURN_IF_ERROR(dist.SetFromImage(
          std::move(tmp), jxl::ColorEncoding::LinearSRGB(dist.IsGray())));
    }
    const size_t xs = img1.xsize();
    const size_t ys = img1.ysize();
    JXL_RETURN_IF_ERROR(blur.ShrinkTo(xs, ys));
    JXL_ASSIGN_OR_RETURN(ImageF mul, ImageF::Create(memory_manager, xs, ys));
    JXL_ASSIGN_OR_RETURN(ImageF sigma1_sq, ImageF::Create(memory_manager, xs, ys));
    JXL_ASSIGN_OR_RETURN(ImageF sigma2_sq, ImageF::Create(memory_manager, xs, ys));
    JXL_ASSIGN_OR_RETURN(ImageF sigma12, ImageF::Create(memory_manager, xs, ys));
    JXL_ASSIGN_OR_RETURN(ImageF mu1, ImageF::Create(memory_manager, xs, ys));
    JXL_ASSIGN_OR_RETURN(ImageF mu2, ImageF::Create(memory_manager, xs, ys));

    MsssimScale sscale;
    for (size_t c = 0; c < 3; ++c) {
      MultiplicarCanal(img1, img1, c, &mul);
      JXL_RETURN_IF_ERROR(blur.BlurPlane(mul, &sigma1_sq));
      MultiplicarCanal(img2, img2, c, &mul);
      JXL_RETURN_IF_ERROR(blur.BlurPlane(mul, &sigma2_sq));
      MultiplicarCanal(img1, img2, c, &mul);
      JXL_RETURN_IF_ERROR(blur.BlurPlane(mul, &sigma12));
      JXL_RETURN_IF_ERROR(blur.BlurPlane(img1.Plane(c), &mu1));
      JXL_RETURN_IF_ERROR(blur.BlurPlane(img2.Plane(c), &mu2));
      SSIMMapCanal(mu1, mu2, sigma1_sq, sigma2_sq, sigma12,
                   &sscale.avg_ssim[c * 2]);
      EdgeDiffMapCanal(img1, mu1, img2, mu2, c, &sscale.avg_edgediff[c * 4]);
    }
    msssim.scales.push_back(sscale);
  }
  return msssim;
}

// La nota con `medir`: Ligero, o la de libjxl tal cual (para comprobar).
template <typename Medir>
double Nota(const uint8_t* original, const uint8_t* distorsionada,
            size_t ancho, size_t alto, int canales, Medir medir) {
  if (ancho < 8 || alto < 8 || (canales != 3 && canales != 4)) return NAN;
  JxlMemoryManager* memoria = jpegxl::tools::NoMemoryManager();
  jxl::CodecInOut a(memoria), b(memoria);
  if (!Cargar(original, ancho, alto, canales, &a) ||
      !Cargar(distorsionada, ancho, alto, canales, &b)) {
    return NAN;
  }
  return medir(a.Main(), b.Main());
}

}  // namespace

// La nota SSIMULACRA 2 (de −∞ a 100) de `distorsionada` frente a `original`,
// del mismo tamaño y con los mismos canales (3 o 4). NaN si no se puede medir
// (menos de 8×8 píxeles, o un fallo de libjxl).
extern "C" double apolo_ssimulacra2(const uint8_t* original,
                                    const uint8_t* distorsionada, size_t ancho,
                                    size_t alto, int canales) {
  return Nota(original, distorsionada, ancho, alto, canales,
              [](ImageBundle& a, ImageBundle& b) -> double {
                if (!a.HasAlpha()) {
                  auto m = Ligero(a, b, 0.5f);
                  return m.ok() ? std::move(m).value_().Score() : NAN;
                }
                // Dos fondos: el primero sobre copias, el segundo ya sobre
                // las de verdad, que no se vuelven a usar.
                auto a2 = a.Copy();
                auto b2 = b.Copy();
                if (!a2.ok() || !b2.ok()) return NAN;
                ImageBundle ca = std::move(a2).value_();
                ImageBundle cb = std::move(b2).value_();
                auto oscuro = Ligero(ca, cb, 0.1f);
                if (!oscuro.ok()) return NAN;
                const double nota_oscuro = std::move(oscuro).value_().Score();
                auto claro = Ligero(a, b, 0.9f);
                if (!claro.ok()) return NAN;
                return std::min(nota_oscuro, std::move(claro).value_().Score());
              });
}

// La misma nota con la función de libjxl tal cual: solo para comprobar que la
// de arriba da lo mismo (crates/nucleo/tests/ssimulacra2_ligero.rs).
extern "C" double apolo_ssimulacra2_libjxl(const uint8_t* original,
                                           const uint8_t* distorsionada,
                                           size_t ancho, size_t alto,
                                           int canales) {
  return Nota(original, distorsionada, ancho, alto, canales,
              [](ImageBundle& a, ImageBundle& b) -> double {
                if (!a.HasAlpha()) {
                  auto m = ComputeSSIMULACRA2(a, b);
                  return m.ok() ? std::move(m).value_().Score() : NAN;
                }
                auto oscuro = ComputeSSIMULACRA2(a, b, 0.1f);
                auto claro = ComputeSSIMULACRA2(a, b, 0.9f);
                if (!oscuro.ok() || !claro.ok()) return NAN;
                return std::min(std::move(oscuro).value_().Score(),
                                std::move(claro).value_().Score());
              });
}
