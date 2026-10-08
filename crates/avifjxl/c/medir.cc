// SSIMULACRA 2 de libjxl (tools/ssimulacra2.cc) para Apolo (ADR 0022): la
// misma nota que la herramienta `ssimulacra2` oficial, pero desde píxeles en
// memoria en vez de ficheros. Con transparencia, como la herramienta: se mide
// sobre un fondo oscuro (0,1) y uno claro (0,9) y vale la peor de las dos.

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
#include "tools/ssimulacra2.h"

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

}  // namespace

// La nota SSIMULACRA 2 (de −∞ a 100) de `distorsionada` frente a `original`,
// del mismo tamaño y con los mismos canales (3 o 4). NaN si no se puede medir
// (menos de 8×8 píxeles, o un fallo de libjxl).
extern "C" double apolo_ssimulacra2(const uint8_t* original,
                                    const uint8_t* distorsionada, size_t ancho,
                                    size_t alto, int canales) {
  if (ancho < 8 || alto < 8 || (canales != 3 && canales != 4)) return NAN;
  JxlMemoryManager* memoria = jpegxl::tools::NoMemoryManager();
  jxl::CodecInOut a(memoria), b(memoria);
  if (!Cargar(original, ancho, alto, canales, &a) ||
      !Cargar(distorsionada, ancho, alto, canales, &b)) {
    return NAN;
  }
  if (!a.Main().HasAlpha()) {
    auto m = ComputeSSIMULACRA2(a.Main(), b.Main());
    return m.ok() ? std::move(m).value_().Score() : NAN;
  }
  auto oscuro = ComputeSSIMULACRA2(a.Main(), b.Main(), 0.1f);
  auto claro = ComputeSSIMULACRA2(a.Main(), b.Main(), 0.9f);
  if (!oscuro.ok() || !claro.ok()) return NAN;
  return std::min(std::move(oscuro).value_().Score(),
                  std::move(claro).value_().Score());
}
