@echo off
chcp 65001 >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0packaging\windows\install.ps1"
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Installation finished with errors.
    pause
) else (
    echo.
    echo Done!
    timeout /t 3 >nul
)
