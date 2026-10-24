# Inkk Windows Release Packaging Script
# Builds release binary, creates distribution folder, calculates SHA256, and generates zip archive.

param (
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
Set-Location $ProjectRoot

Write-Host "=================================================" -ForegroundColor Cyan
Write-Host "  Packaging Inkk v$Version for Windows x64 Distribution" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

# 1. Terminate running instance if any
Stop-Process -Name inkk -Force -ErrorAction SilentlyContinue

# 2. Build release binary
Write-Host "`n[1/4] Compiling optimized release binary..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build release failed!"
}

# 3. Setup distribution directory
$DistRoot = Join-Path $ProjectRoot "dist"
$BundleName = "Inkk-v$Version-Windows-x64"
$BundleDir = Join-Path $DistRoot $BundleName

if (Test-Path $BundleDir) {
    Remove-Item $BundleDir -Recurse -Force
}
New-Item -ItemType Directory -Path $BundleDir -Force | Out-Null

Write-Host "`n[2/4] Assembling release assets..." -ForegroundColor Yellow

# Copy binary
Copy-Item (Join-Path $ProjectRoot "target\release\inkk.exe") (Join-Path $BundleDir "inkk.exe") -Force

# Copy assets if present
$AssetsDir = Join-Path $ProjectRoot "assets"
if (Test-Path $AssetsDir) {
    Copy-Item $AssetsDir (Join-Path $BundleDir "assets") -Recurse -Force
}

# Copy documentation, licenses and update manifest
Copy-Item (Join-Path $ProjectRoot "packaging\README.txt") (Join-Path $BundleDir "README.txt") -Force
Copy-Item (Join-Path $ProjectRoot "LICENSE-MIT") (Join-Path $BundleDir "LICENSE-MIT") -Force
Copy-Item (Join-Path $ProjectRoot "LICENSE-APACHE") (Join-Path $BundleDir "LICENSE-APACHE") -Force
Copy-Item (Join-Path $ProjectRoot "packaging\update_manifest.json") (Join-Path $BundleDir "update_manifest.json") -Force

# 4. Generate SHA256 checksum
Write-Host "`n[3/4] Computing SHA256 checksums..." -ForegroundColor Yellow
$ExePath = Join-Path $BundleDir "inkk.exe"
$Hash = (Get-FileHash -Path $ExePath -Algorithm SHA256).Hash
$ChecksumFile = Join-Path $BundleDir "SHA256SUMS.txt"
"SHA256 (inkk.exe) = $Hash" | Out-File -FilePath $ChecksumFile -Encoding utf8

# Update checksum in manifest
$ManifestPath = Join-Path $BundleDir "update_manifest.json"
$ManifestJson = Get-Content $ManifestPath -Raw | ConvertFrom-Json
$ManifestJson | Add-Member -NotePropertyName "sha256" -NotePropertyValue $Hash -Force
$ManifestJson | ConvertTo-Json -Depth 5 | Out-File -FilePath $ManifestPath -Encoding utf8
Copy-Item $ManifestPath (Join-Path $ProjectRoot "packaging\update_manifest.json") -Force

# 5. Compress into ZIP bundle
Write-Host "`n[4/4] Creating distribution ZIP archive..." -ForegroundColor Yellow
$ZipPath = Join-Path $DistRoot "$BundleName.zip"
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}
Compress-Archive -Path "$BundleDir\*" -DestinationPath $ZipPath -CompressionLevel Optimal

Write-Host "`n=================================================" -ForegroundColor Green
Write-Host "  SUCCESS! Release package generated:" -ForegroundColor Green
Write-Host "  Directory: $BundleDir" -ForegroundColor White
Write-Host "  Zip Bundle: $ZipPath" -ForegroundColor White
Write-Host "  SHA256:    $Hash" -ForegroundColor White
Write-Host "=================================================" -ForegroundColor Green
