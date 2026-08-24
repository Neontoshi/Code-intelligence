@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  Code Intelligence Installer (Windows)
echo ========================================
echo.

set "INSTALL_DIR=%ProgramFiles%\CodeIntelligence"

echo 📁 Installation directory: %INSTALL_DIR%
echo.

REM Create directory if it doesn't exist
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo 📥 Downloading ci.exe...
powershell -Command "& { Invoke-WebRequest -Uri 'https://github.com/neontoshi/Code-intelligence/releases/latest/download/ci_windows_x86_64.exe' -OutFile '%TEMP%\ci.exe' }"

if errorlevel 1 (
    echo ❌ Download failed. Please check your internet connection.
    pause
    exit /b 1
)

echo 📦 Installing to %INSTALL_DIR%...
move "%TEMP%\ci.exe" "%INSTALL_DIR%\ci.exe"

echo 🔧 Adding to PATH...
powershell -Command "& { [Environment]::SetEnvironmentVariable('Path', $env:Path + ';%INSTALL_DIR%', 'User') }"

echo.
echo ✅ Installation complete!
echo.
echo 🚀 Quick Start:
echo   1. Open a new Command Prompt
echo   2. Run: ci analyze <path-to-your-project>
echo.
echo 📖 Documentation: https://github.com/neontoshi/Code-intelligence
echo.
pause
