# Build script for Windows
# PowerShell script to build the application

$ErrorActionPreference = "Stop"

Write-Host "🚀 Building ファイル仕訳け君 for Windows..." -ForegroundColor Blue

# Check if Node.js is installed
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js version: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Node.js is not installed. Please install Node.js first." -ForegroundColor Red
    exit 1
}

# Check if Rust is installed
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust version: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust is not installed. Please install Rust first." -ForegroundColor Red
    exit 1
}

# Install dependencies if needed
if (-not (Test-Path "node_modules")) {
    Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
    npm install
}

# Build the application
Write-Host "📦 Building for Windows (x86_64)..." -ForegroundColor Blue
npm run tauri:build:windows

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Windows build completed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Artifacts location:" -ForegroundColor Cyan
    Write-Host "  - src-tauri\target\release\bundle\msi\" -ForegroundColor White
    Write-Host "  - src-tauri\target\release\bundle\nsis\" -ForegroundColor White
    Write-Host ""
    Write-Host "🎉 Build process completed successfully!" -ForegroundColor Green
} else {
    Write-Host "❌ Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}
