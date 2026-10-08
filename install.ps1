# TikTik Windows 1-Liner Installer
# Usage: irm https://raw.githubusercontent.com/Sanjay-Android-AIT/chat-dev/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "Sanjay-Android-AIT/chat-dev"
$version = "0.1.0"
$zipUrl = "https://github.com/$repo/releases/download/v$version/tiktik-windows-x86_64.zip"
$installDir = "$HOME\.tiktik\bin"

Write-Host "==> Installing tiktik v$version for Windows..." -ForegroundColor Cyan

# Create install directory
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$tempZip = "$env:TEMP\tiktik-$version.zip"
Write-Host "==> Downloading $zipUrl..."
Invoke-WebRequest -Uri $zipUrl -OutFile $tempZip

Write-Host "==> Extracting tiktik.exe to $installDir..."
Expand-Archive -Path $tempZip -DestinationPath $installDir -Force
Remove-Item $tempZip -Force

# Add to User PATH if not already present
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "==> Adding $installDir to User PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
}

Write-Host ""
Write-Host "✓ tiktik v$version successfully installed!" -ForegroundColor Green
Write-Host "Restart your terminal (or PowerShell) and type:"
Write-Host "  tiktik <room_name>" -ForegroundColor Cyan
