# Plantilla de la fórmula de Apolo (ADR 0026). publicar.yml la rellena en cada versión y la empuja a
# webcafeina/homebrew-apolo como Formula/apolo.rb: {{VERSION}} y las sumas {{SHA_*}} salen de la
# Release. Binarios ya compilados (los paquetes de la CLI), sin bottles: una fórmula no se pone en
# cuarentena, así que macOS no los bloquea.
class Apolo < Formula
  desc "Image optimizer CLI and MCP server: lighter images, just as good"
  homepage "https://apolo.webcafeina.com/"
  version "{{VERSION}}"
  license "GPL-3.0-only"

  livecheck do
    url :stable
    strategy :github_latest
  end

  # Un binario por sistema, dentro de on_macos y on_linux. `brew style` lo marca (es una regla del
  # repositorio oficial, que compila desde el código), pero en un tap propio es lo normal y funciona:
  # la prueba del tap lo instala de verdad en macOS y Linux.

  on_macos do
    url "https://github.com/webcafeina/Apolo/releases/download/v{{VERSION}}/apolo-cli-{{VERSION}}-macos-universal.tar.gz"
    sha256 "{{SHA_MACOS}}"
  end

  on_linux do
    on_intel do
      url "https://github.com/webcafeina/Apolo/releases/download/v{{VERSION}}/apolo-cli-{{VERSION}}-linux-amd64.tar.gz"
      sha256 "{{SHA_LINUX_AMD64}}"
    end
    on_arm do
      url "https://github.com/webcafeina/Apolo/releases/download/v{{VERSION}}/apolo-cli-{{VERSION}}-linux-arm64.tar.gz"
      sha256 "{{SHA_LINUX_ARM64}}"
    end
  end

  def install
    bin.install "apolo", "apolo-mcp"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/apolo --version")
    assert_match version.to_s, shell_output("#{bin}/apolo-mcp --version 2>&1")
  end
end
