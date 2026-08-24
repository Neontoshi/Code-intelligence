# install.ps1 - Windows Installer for Code Intelligence
# Run with: powershell -ExecutionPolicy Bypass -File install.ps1

param(
    [switch]$Help,
    [string]$InstallDir = "$env:ProgramFiles\CodeIntelligence"
)

# Help message
if ($Help) {
    Write-Host @"
Code Intelligence Installer

Usage: .\install.ps1 [-InstallDir <path>] [-Help]

Options:
  -InstallDir <path>  Installation directory (default: $env:ProgramFiles\CodeIntelligence)
  -Help              Show this help message

Examples:
  .\install.ps1
  .\install.ps1 -InstallDir "C:\Tools\ci"
"@ -ForegroundColor Cyan
    exit 0
}

Write-Host "🔍 Installing Code Intelligence..." -ForegroundColor Cyan
Write-Host ""

# 1. Detect system architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-Host "📐 Architecture: $arch" -ForegroundColor Gray

# 2. Determine binary name
$assetName = "ci_windows_x86_64.exe"
$downloadUrl = "https://github.com/neontoshi/Code-intelligence/releases/latest/download/$assetName"

# 3. Create installation directory
Write-Host "📁 Installation directory: $InstallDir" -ForegroundColor Gray
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# 4. Download the binary
Write-Host "📥 Downloading $assetName..." -ForegroundColor Gray
$tempFile = Join-Path $env:TEMP $assetName
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -ErrorAction Stop
} catch {
    Write-Host "❌ Failed to download: $_" -ForegroundColor Red
    Write-Host "   Please check your internet connection and try again." -ForegroundColor Yellow
    exit 1
}

# 5. Verify download
if (-not (Test-Path $tempFile) -or (Get-Item $tempFile).Length -eq 0) {
    Write-Host "❌ Downloaded file is empty or corrupted." -ForegroundColor Red
    exit 1
}

# 6. Move to installation directory
$targetFile = Join-Path $InstallDir "ci.exe"
Move-Item -Path $tempFile -Destination $targetFile -Force
Write-Host "✅ Installed to: $targetFile" -ForegroundColor Green

# 7. Add to PATH (User level, no admin required)
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Host "🔧 Adding to PATH..." -ForegroundColor Gray
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir", "User")

    # Update current session's PATH
    $env:Path = "$env:Path;$InstallDir"
}

# 8. Verify installation
Write-Host ""
Write-Host "✅ Installation complete!" -ForegroundColor Green
Write-Host ""

# Try to run ci --version
try {
    $version = & "$InstallDir\ci.exe" --version 2>$null
    if ($version) {
        Write-Host "📦 Installed version: $version" -ForegroundColor Cyan
    } else {
        Write-Host "⚠️ Could not verify version. Please open a new terminal and run: ci --version" -ForegroundColor Yellow
    }
} catch {
    Write-Host "⚠️ Could not verify version. Please open a new terminal and run: ci --version" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🚀 Quick Start:" -ForegroundColor Cyan
Write-Host "  1. Open a new terminal (or restart your current one)" -ForegroundColor Gray
Write-Host "  2. Run: ci analyze <path-to-your-project>" -ForegroundColor Gray
Write-Host ""
Write-Host "📖 Documentation: https://github.com/neontoshi/Code-intelligence" -ForegroundColor Gray
