# install.ps1 - Windows Installer for Code Intelligence
[CmdletBinding()]
param(
    [switch]$Help,
    [string]$InstallDir = "$env:ProgramFiles\CodeIntelligence"
)

$ErrorActionPreference = "Stop"

# Auto-elevate to administrator if needed (compatible with both file run and iex pipe)
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "⚠️ Administrator privileges required. Requesting elevation..." -ForegroundColor Yellow
    if ($PSCommandPath) {
        Start-Process powershell -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -NoProfile -File `"$PSCommandPath`""
    } else {
        Start-Process powershell -Verb RunAs -ArgumentList "-ExecutionPolicy Bypass -NoProfile -Command `"irm https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.ps1 | iex`""
    }
    exit
}

if ($Help) {
    Write-Host "Code Intelligence Installer`nUsage: .\install.ps1 [-InstallDir <path>]" -ForegroundColor Cyan
    exit 0
}

Write-Host "🔍 Installing Code Intelligence..." -ForegroundColor Cyan

# Force TLS 1.2+ for reliable downloads
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

$assetName = "ci_windows_x86_64.exe"
$downloadUrl = "https://github.com/neontoshi/Code-intelligence/releases/latest/download/$assetName"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$tempFile = Join-Path $env:TEMP "$([System.Guid]::NewGuid()).exe"
Write-Host "📥 Downloading $assetName..." -ForegroundColor Gray
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing

$targetFile = Join-Path $InstallDir "ci.exe"
Move-Item -Path $tempFile -Destination $targetFile -Force
Write-Host "✅ Binary installed to: $targetFile" -ForegroundColor Green

# Manage System PATH cleanly
$machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
$pathParts = $machinePath -split ';' | Where-Object { $_ -ne "" }
if ($pathParts -notcontains $InstallDir) {
    Write-Host "🔧 Updating Machine PATH..." -ForegroundColor Gray
    $newPath = ($pathParts + $InstallDir) -join ';'
    [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
}

$env:Path = "$env:Path;$InstallDir"

Write-Host "`n✅ Installation complete!" -ForegroundColor Green
try {
    & "$targetFile" --version
} catch {
    Write-Host "Restart your terminal to use 'ci'." -ForegroundColor Yellow
}
