class Tiktik < Formula
  desc "Stealth encrypted terminal chat disguised as React Native and Kotlin dev logs"
  homepage "https://github.com/Sanjay-Android-AIT/chat-dev"
  version "0.1.0"
  license "MIT"

  url "https://github.com/Sanjay-Android-AIT/chat-dev/releases/download/v#{version}/tiktik-macos-arm64.tar.gz"
  # Calculate with: shasum -a 256 tiktik-macos-arm64.tar.gz
  sha256 "898f923ce418723ddf2f40b1ce0da2e9568ad82f231b833af6ee563d5cc77da0"

  def install
    bin.install "tiktik"
  end

  test do
    assert_match "tiktik", shell_output("#{bin}/tiktik --help")
  end
end
