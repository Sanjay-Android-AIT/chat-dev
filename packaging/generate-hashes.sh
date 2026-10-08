#!/bin/bash
set -e

VERSION=${1:-"0.1.0"}
REPO="Sanjay-Android-AIT/chat-dev"
RELEASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

echo "Fetching SHA256 checksums for v${VERSION}..."

TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

curl -fsSL "${RELEASE_URL}/SHA256SUMS.txt" -o "${TEMP_DIR}/SHA256SUMS.txt" || {
  echo "Error: Could not download SHA256SUMS.txt from GitHub release v${VERSION}."
  echo "Ensure the release v${VERSION} is published first."
  exit 1
}

ARM64_HASH=$(grep "tiktik-macos-arm64.tar.gz" "${TEMP_DIR}/SHA256SUMS.txt" | awk '{print $1}')
WIN_HASH=$(grep "tiktik-windows-x86_64.zip" "${TEMP_DIR}/SHA256SUMS.txt" | awk '{print $1}')

echo "macOS ARM64 SHA256: ${ARM64_HASH}"
echo "Windows SHA256:     ${WIN_HASH}"

# Update Homebrew formula
sed -i '' "s/PUT_ARM64_SHA256_HERE/${ARM64_HASH}/g" packaging/homebrew/Formula/tiktik.rb

# Update Scoop manifest
sed -i '' "s/PUT_WINDOWS_ZIP_SHA256_HERE/${WIN_HASH}/g" packaging/scoop/bucket/tiktik.json

echo "Successfully updated packaging/homebrew/Formula/tiktik.rb and packaging/scoop/bucket/tiktik.json!"
