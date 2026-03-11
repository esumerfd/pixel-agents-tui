# This file is updated automatically by the release workflow.
class PixelAgentsTui < Formula
  desc "Terminal UI visualizing Claude Code agents as animated pixel characters"
  homepage "https://github.com/esumerfd/pixel-agents-tui"
  version "0.2.0"

  on_macos do
    on_arm do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.2.0/pixel-agents-tui-v0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "28e4643c913376b47c10f66ee531d1476caecfc4107fb3b57027d40ba5ec2b2a"
    end
    on_intel do
      url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.2.0/pixel-agents-tui-v0.2.0-x86_64-apple-darwin.tar.gz"
      sha256 "827dbb14e0100ef61f97d09bab74dacff1444eac3ccf396c4d8dfa8df513d920"
    end
  end

  on_linux do
    url "https://github.com/esumerfd/pixel-agents-tui/releases/download/v0.2.0/pixel-agents-tui-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "c7c15ac594faa9de92d3507e16e4714b981242619e95c8ba69f59bea32a995b3"
  end

  def install
    bin.install "pixel-agents-tui"
  end

  test do
    system "#{bin}/pixel-agents-tui", "--version"
  end
end
