class Speck < Formula
  desc "Spec-driven agentic compiler, built on top of zerostack"
  homepage "https://github.com/gi-dellav/speck"
  version "1.0.0-rc2"
  license "GPL-3.0-only"

  depends_on "zerostack"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/gi-dellav/speck/releases/download/v1.0.0-rc2/speck-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/gi-dellav/speck/releases/download/v1.0.0-rc2/speck-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/gi-dellav/speck/releases/download/v1.0.0-rc2/speck-x86_64-unknown-linux-musl.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    else
      url "https://github.com/gi-dellav/speck/releases/download/v1.0.0-rc2/speck-aarch64-unknown-linux-musl.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install Dir["speck*"].first => "speck"
  end

  test do
    assert_match(/^speck /, shell_output("#{bin}/speck --version"))
  end
end
