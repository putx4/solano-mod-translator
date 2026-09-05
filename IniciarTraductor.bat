@echo off
setlocal EnableExtensions

REM ============================================
REM  Solano Mod Translator - Iniciador
REM  Arranca opencode serve (si falta) y abre la app.
REM ============================================

REM -------- Localizar la app: instalada primero, luego release del repo --------
set "APP_EXE=%LOCALAPPDATA%\Solano Mod Translator\solano-mod-translator.exe"
if not exist "%APP_EXE%" set "APP_EXE=%~dp0src-tauri\target\release\solano-mod-translator.exe"
if not exist "%APP_EXE%" goto :no_app

REM -------- Localizar opencode.exe --------
set "OPENCODE_EXE=%APPDATA%\npm\node_modules\opencode-ai\bin\opencode.exe"
if not exist "%OPENCODE_EXE%" set "OPENCODE_EXE=opencode"

REM -------- Si opencode ya responde en :4096, no lo duplicamos --------
set "RUNNING=0"
powershell -NoProfile -Command "try { Invoke-RestMethod -Uri 'http://127.0.0.1:4096/global/health' -TimeoutSec 3 | Out-Null; exit 0 } catch { exit 1 }" >nul 2>&1
if %ERRORLEVEL% EQU 0 set "RUNNING=1"

if %RUNNING% EQU 1 goto :launch

start "" /min "%OPENCODE_EXE%" serve

set /a ATTEMPTS=0
set /a RUNNING=0

:wait_loop
if %RUNNING% EQU 1 goto :launch
set /a ATTEMPTS+=1
if %ATTEMPTS% GTR 40 goto :no_opencode
powershell -NoProfile -Command "try { Invoke-RestMethod -Uri 'http://127.0.0.1:4096/global/health' -TimeoutSec 2 | Out-Null; exit 0 } catch { exit 1 }" >nul 2>&1
if %ERRORLEVEL% EQU 0 set "RUNNING=1"
timeout /t 1 /nobreak >nul
goto :wait_loop

:launch
start "" "%APP_EXE%"
exit /b 0

:no_opencode
echo [ERROR] opencode serve no respondio a tiempo.
echo         Revisa que tengas un proveedor configurado (opencode auth / /connect).
pause
exit /b 1

:no_app
echo [ERROR] No se encontro la app instalada ni el release del repo.
echo         %APP_EXE%
echo         Compilala/instalala primero.
pause
exit /b 1
