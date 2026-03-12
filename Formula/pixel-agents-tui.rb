# This file is updated automatically by the release workflow.
class PixelAgentsTui < Formula
  desc "Terminal UI visualizing Claude Code agents as animated pixel characters"
  homepage "https://github.com/esumerfd/pixel-agents-tui"
  version "0.3.0"

  on_macos do
    on_arm do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.3.0/pixel-agents-tui-v0.3.0-aarch64-apple-darwin.tar.gz"
      sha256 "acc9ca33a12800a503d6cc4b45b0fbe83d4f72b7376f21abefabeba3dce8a9be"
    end
    on_intel do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.3.0/pixel-agents-tui-v0.3.0-x86_64-apple-darwin.tar.gz"
      sha256 "23e71e75b8650e3a78192842f8c65ba4c040769e60ba440c2ed278b2decc7f9f"
    end
  end

  on_linux do
    url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.3.0/pixel-agents-tui-v0.3.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "4dc248468e6617d7097f9d2dcf65470e206b0ce80ffbd9b20c5c847d74566039"
  end

  def install
    bin.install "pixel-agents-tui"
  end

  test do
    system "#{bin}/pixel-agents-tui", "--version"
  end
end
