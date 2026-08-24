@echo off
setlocal enabledelayedexpansion

echo ========================================
echo  Code Intelligence Installer (Windows)
echo ========================================
echo.

net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [!] Administrator privileges required. Elevating...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b 0
)

set "INSTALL_DIR=%ProgramFiles%\CodeIntelligence"
if defined ProgramW6432 set "INSTALL_DIR=%ProgramW6432%\CodeIntelligence"
set "TEMP_FILE=%TEMP%\ci_%RANDOM%.exe"

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo [^>] Downloading ci_windows_x86_64.exe...
powershell -NoProfile -ExecutionPolicy Bypass -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13; Invoke-WebRequest -Uri 'https://github.com/neontoshi/Code-intelligence/releases/latest/download/ci_windows_x86_64.exe' -OutFile '%TEMP_FILE%' -UseBasicParsing"

if not exist "%TEMP_FILE%" (
    echo [X] Download failed. Please verify your internet connection.
    pause
    exit /b 1
)

echo [^>] Installing binary to %INSTALL_DIR%...
move /Y "%TEMP_FILE%" "%INSTALL_DIR%\ci.exe" >nul

echo [^>] Registering PATH environment variable...
powershell -NoProfile -Command "$dir = '%INSTALL_DIR%'; $p = [Environment]::GetEnvironmentVariable('Path', 'Machine'); if (-not ($p -split ';' -contains $dir)) { [Environment]::SetEnvironmentVariable('Path', ($p.TrimEnd(';') + ';' + $dir), 'Machine') }"

echo.
echo [V] Installation complete!
"%INSTALL_DIR%\ci.exe" --version
echo.
pause
