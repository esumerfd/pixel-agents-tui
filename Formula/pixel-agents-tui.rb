# This file is updated automatically by the release workflow.
class PixelAgentsTui < Formula
  desc "Terminal UI visualizing Claude Code agents as animated pixel characters"
  homepage "https://github.com/esumerfd/pixel-agents-tui"
  version "0.6.0"

  on_macos do
    on_arm do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.6.0/pixel-agents-tui-v0.6.0-aarch64-apple-darwin.tar.gz"
      sha256 "32e437fc9ce3e8756ee5e0983282f45b41227f0eee0c8987342cab0c7060cbb1"
    end
    on_intel do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.6.0/pixel-agents-tui-v0.6.0-x86_64-apple-darwin.tar.gz"
      sha256 "a71d232bb3f1ebadf85b7f97e1cbd1262bab447342fa4b80ed252696fa1bde8a"
    end
  end

  on_linux do
    url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.6.0/pixel-agents-tui-v0.6.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "85b1db61816da62eeba0e00a441839970f8129e752e8eaa8303270d935d09c90"
  end

  def install
    bin.install "pixel-agents-tui"
  end

  test do
    system "#{bin}/pixel-agents-tui", "--version"
  end
end
