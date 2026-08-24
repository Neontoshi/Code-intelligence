# install.ps1 - Windows Installer for Code Intelligence
# Run with: powershell -ExecutionPolicy Bypass -File install.ps1

# Auto-elevate to administrator if needed
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "⚠️ This installer requires administrator privileges." -ForegroundColor Yellow
    Write-Host "🔁 Restarting as administrator..." -ForegroundColor Cyan
    Start-Process powershell -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit
}

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
    try {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Write-Host "   Created directory" -ForegroundColor Gray
    } catch {
        Write-Host "❌ Failed to create directory: $_" -ForegroundColor Red
        exit 1
    }
}

# 4. Download the binary
Write-Host "📥 Downloading $assetName..." -ForegroundColor Gray
$tempFile = Join-Path $env:TEMP $assetName
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -ErrorAction Stop
    Write-Host "   Download complete" -ForegroundColor Gray
} catch {
    Write-Host "❌ Failed to download: $_" -ForegroundColor Red
    Write-Host "   Please check your internet connection and try again." -ForegroundColor Yellow
    exit 1
}

# 5. Verify download
if (-not (Test-Path $tempFile)) {
    Write-Host "❌ Downloaded file not found." -ForegroundColor Red
    exit 1
}

$fileSize = (Get-Item $tempFile).Length
if ($fileSize -eq 0) {
    Write-Host "❌ Downloaded file is empty (0 bytes)." -ForegroundColor Red
    exit 1
}
Write-Host "   File size: $([math]::Round($fileSize / 1MB, 2)) MB" -ForegroundColor Gray

# 6. Move to installation directory
$targetFile = Join-Path $InstallDir "ci.exe"
Write-Host "📦 Installing to: $targetFile" -ForegroundColor Gray
try {
    # Remove existing file if present
    if (Test-Path $targetFile) {
        Remove-Item -Path $targetFile -Force
    }
    Move-Item -Path $tempFile -Destination $targetFile -Force
    Write-Host "✅ Installed successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to install: $_" -ForegroundColor Red
    exit 1
}

# 7. Add to PATH (Machine level)
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if ($currentPath -notlike "*$InstallDir*") {
    Write-Host "🔧 Adding to system PATH..." -ForegroundColor Gray
    try {
        $newPath = "$currentPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
        Write-Host "   PATH updated successfully" -ForegroundColor Gray

        # Update current session's PATH
        $env:Path = "$env:Path;$InstallDir"
    } catch {
        Write-Host "⚠️ Failed to update PATH: $_" -ForegroundColor Yellow
        Write-Host "   You may need to add $InstallDir to your PATH manually." -ForegroundColor Yellow
    }
} else {
    Write-Host "✅ PATH already contains $InstallDir" -ForegroundColor Gray
}

Write-Host ""
Write-Host "✅ Installation complete!" -ForegroundColor Green
Write-Host ""

# 8. Verify installation
Write-Host "🔍 Verifying installation..." -ForegroundColor Gray
try {
    $version = & "$InstallDir\ci.exe" --version 2>$null
    if ($version) {
        Write-Host "📦 Installed version: $version" -ForegroundColor Cyan
    } else {
        Write-Host "⚠️ Could not verify version." -ForegroundColor Yellow
        Write-Host "   Please open a new terminal and run: ci --version" -ForegroundColor Yellow
    }
} catch {
    Write-Host "⚠️ Could not verify version." -ForegroundColor Yellow
    Write-Host "   Please open a new terminal and run: ci --version" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🚀 Quick Start:" -ForegroundColor Cyan
Write-Host "  1. Open a new terminal (or restart your current one)" -ForegroundColor Gray
Write-Host "  2. Run: ci analyze <path-to-your-project>" -ForegroundColor Gray
Write-Host ""
Write-Host "📖 Documentation: https://github.com/neontoshi/Code-intelligence" -ForegroundColor Gray
Write-Host "🐛 Report issues: https://github.com/neontoshi/Code-intelligence/issues" -ForegroundColor Gray
Write-Host ""

# Pause so users can see the output
Read-Host "Press Enter to exit"
