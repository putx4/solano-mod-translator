@echo off
setlocal

REM ============================================
REM  Solano Mod Translator - Instalador completo
REM  Instala Node.js, opencode y la app.
REM  Se auto-eleva a administrador si es necesario.
REM ============================================

title Solano Mod Translator - Instalador completo

REM -------- Auto-elevar a administrador --------
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo Solicitando permisos de administrador...
    powershell -NoProfile -Command "Start-Process -FilePath '%~f0' -Verb RunAs"
    exit /b
)

cd /d "%~dp0"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0InstalarTodo.ps1"

endlocal
