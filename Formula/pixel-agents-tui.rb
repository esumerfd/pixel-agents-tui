# This file is updated automatically by the release workflow.
class PixelAgentsTui < Formula
  desc "Terminal UI visualizing Claude Code agents as animated pixel characters"
  homepage "https://github.com/esumerfd/pixel-agents-tui"
  version "0.5.0"

  on_macos do
    on_arm do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.5.0/pixel-agents-tui-v0.5.0-aarch64-apple-darwin.tar.gz"
      sha256 "15ed5011d55079c7359ce5d76aba2446e85930d25ab5b78f67e2ba195d609297"
    end
    on_intel do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.5.0/pixel-agents-tui-v0.5.0-x86_64-apple-darwin.tar.gz"
      sha256 "e726d245f0f17e310dd9f9fbc9f00125c0587dfb968059290059bb5a7ef20978"
    end
  end

  on_linux do
    url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.5.0/pixel-agents-tui-v0.5.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "35879edea24ecdcc9416e103c10aa9562752caa84a1873dc3ff28a92abfc8e29"
  end

  def install
    bin.install "pixel-agents-tui"
  end

  test do
    system "#{bin}/pixel-agents-tui", "--version"
  end
end
