# Plantilla del cask de Apolo (ADR 0026). publicar.yml la rellena en cada versión y la empuja a
# webcafeina/homebrew-apolo como Casks/apolo.rb.
#
# Apolo no está firmada ni notarizada (ADR 0009). Homebrew solo rechaza eso en su repositorio
# oficial; en un tap propio la instala, y es macOS quien la bloquearía al abrirla. Por eso se le
# quita la cuarentena al instalar (postflight_steps: el «postflight do» de antes está obsoleto desde
# Homebrew 7, y --no-quarantine ya no existe).
cask "apolo" do
  version "{{VERSION}}"
  sha256 "{{SHA_DMG}}"

  url "https://github.com/webcafeina/Apolo/releases/download/v#{version}/Apolo_#{version}_universal.dmg"
  name "Apolo"
  desc "Image optimizer with Squoosh's engines"
  homepage "https://apolo.webcafeina.com/"

  livecheck do
    url :url
    strategy :github_latest
  end

  # Apolo se actualiza sola desde la aplicación; Homebrew no tiene que hacerlo por ella.
  auto_updates true
  depends_on macos: :big_sur

  app "Apolo.app"

  postflight_steps do
    run "/usr/bin/xattr",
        args:         ["-dr", "com.apple.quarantine", "{{appdir}}/Apolo.app"],
        must_succeed: false
  end

  zap trash: [
    "~/Library/Application Support/Apolo",
    "~/Library/Application Support/com.webcafeina.apolo",
    "~/Library/Caches/com.webcafeina.apolo",
    "~/Library/WebKit/com.webcafeina.apolo",
  ]

  caveats <<~EOS
    Apolo isn't signed or notarized by Apple. This cask removes the quarantine attribute when it
    installs it. If macOS still blocks it:
      xattr -dr com.apple.quarantine /Applications/Apolo.app
    or System Settings › Privacy & Security › "Open Anyway".
  EOS
end
