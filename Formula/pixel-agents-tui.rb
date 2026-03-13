# This file is updated automatically by the release workflow.
class PixelAgentsTui < Formula
  desc "Terminal UI visualizing Claude Code agents as animated pixel characters"
  homepage "https://github.com/esumerfd/pixel-agents-tui"
  version "0.4.0"

  on_macos do
    on_arm do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.4.0/pixel-agents-tui-v0.4.0-aarch64-apple-darwin.tar.gz"
      sha256 "5719d86b58c330ac13a2fdf7842c373317a7671e903531355ea2bb4695256504"
    end
    on_intel do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.4.0/pixel-agents-tui-v0.4.0-x86_64-apple-darwin.tar.gz"
      sha256 "9098f976ef39f5e15f9c123ff1d32b679d042824912d3b6278d182cecfcd6e8e"
    end
  end

  on_linux do
    url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.4.0/pixel-agents-tui-v0.4.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "4a180d2d76459f00dfe5478d5c1dc2f3071ae178377afde470aa45efa551786d"
  end

  def install
    bin.install "pixel-agents-tui"
  end

  test do
    system "#{bin}/pixel-agents-tui", "--version"
  end
end
