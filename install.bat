@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  Code Intelligence Installer (Windows)
echo ========================================
echo.

REM Check for administrator privileges
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo ⚠️ This installer requires administrator privileges.
    echo Please right-click Command Prompt and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

set "INSTALL_DIR=%ProgramFiles%\CodeIntelligence"
set "TEMP_FILE=%TEMP%\ci.exe"

echo 📁 Installation directory: %INSTALL_DIR%
echo.

REM Create directory if it doesn't exist
if not exist "%INSTALL_DIR%" (
    echo 📁 Creating directory...
    mkdir "%INSTALL_DIR%"
    if errorlevel 1 (
        echo ❌ Failed to create directory. Please check permissions.
        pause
        exit /b 1
    )
)

echo 📥 Downloading ci.exe...
powershell -Command "& { Invoke-WebRequest -Uri 'https://github.com/neontoshi/Code-intelligence/releases/latest/download/ci_windows_x86_64.exe' -OutFile '%TEMP_FILE%' }"

if errorlevel 1 (
    echo ❌ Download failed. Please check your internet connection.
    pause
    exit /b 1
)

if not exist "%TEMP_FILE%" (
    echo ❌ Download failed - file not found.
    pause
    exit /b 1
)

echo 📦 Installing to %INSTALL_DIR%...
move /Y "%TEMP_FILE%" "%INSTALL_DIR%\ci.exe"

if errorlevel 1 (
    echo ❌ Failed to move file. Please check permissions.
    pause
    exit /b 1
)

echo 🔧 Adding to PATH...
powershell -Command "& { [Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';%INSTALL_DIR%', 'Machine') }"

echo.
echo ✅ Installation complete!
echo.
echo 📦 Installed version:
"%INSTALL_DIR%\ci.exe" --version
echo.
echo 🚀 Quick Start:
echo   1. Open a new Command Prompt
echo   2. Run: ci analyze ^<path-to-your-project^>
echo.
echo 📖 Documentation: https://github.com/neontoshi/Code-intelligence
echo.
pause
