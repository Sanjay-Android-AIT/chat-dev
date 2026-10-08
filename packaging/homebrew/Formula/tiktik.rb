class Tiktik < Formula
  desc "Stealth encrypted terminal chat disguised as React Native and Kotlin dev logs"
  homepage "https://github.com/Sanjay-Android-AIT/chat-dev"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Sanjay-Android-AIT/chat-dev/releases/download/v#{version}/tiktik-macos-arm64.tar.gz"
      # Calculate with: shasum -a 256 tiktik-macos-arm64.tar.gz
      sha256 "PUT_ARM64_SHA256_HERE"
    else
      url "https://github.com/Sanjay-Android-AIT/chat-dev/releases/download/v#{version}/tiktik-macos-x86_64.tar.gz"
      # Calculate with: shasum -a 256 tiktik-macos-x86_64.tar.gz
      sha256 "PUT_X86_64_SHA256_HERE"
    end
  end

  def install
    bin.install "tiktik"
  end

  test do
    assert_match "tiktik", shell_output("#{bin}/tiktik --help")
  end
end
