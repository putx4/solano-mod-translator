param(
    [switch]$SkipAppInstall
)

$ErrorActionPreference = "Continue"
$host.UI.RawUI.WindowTitle = "Solano Mod Translator - Instalador completo"

function Write-Header($text) {
    Write-Host ""
    Write-Host "==============================================" -ForegroundColor Cyan
    Write-Host "  $text" -ForegroundColor Cyan
    Write-Host "==============================================" -ForegroundColor Cyan
}

function Write-Step($text) {
    Write-Host ""
    Write-Host " > $text" -ForegroundColor Yellow
}

function Write-Ok($text) {
    Write-Host "   OK: $text" -ForegroundColor Green
}

function Write-Fail($text) {
    Write-Host "   ERROR: $text" -ForegroundColor Red
}

function Confirm-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-Command($cmd) {
    return [bool](Get-Command $cmd -ErrorAction SilentlyContinue)
}

function Test-Url($url) {
    try {
        $r = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 3
        return $r.StatusCode -eq 200
    } catch {
        return $false
    }
}

# ============================================================
Write-Header "Solano Mod Translator - Instalacion completa"
Write-Host "  Este asistente instalara todo lo necesario:" -ForegroundColor White
Write-Host "  - Node.js (si falta)" -ForegroundColor White
Write-Host "  - opencode (servidor local de IA)" -ForegroundColor White
Write-Host "  - La aplicacion Solano Mod Translator" -ForegroundColor White
Write-Host ""
Write-Host "  Requiere permisos de administrador para instalar opencode." -ForegroundColor DarkGray

# ============================================================
# 1. Verificar / instalar Node.js
# ============================================================
Write-Header "PASO 1/4 - Verificar Node.js"
if (Test-Command node) {
    $nodeVer = node --version
    Write-Ok "Node.js instalado: $nodeVer"
} else {
    Write-Step "Node.js no encontrado. Intentando instalarlo via winget..."
    if (Test-Command winget) {
        winget install OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements --silent
        if (Test-Command node) {
            Write-Ok "Node.js instalado exitosamente."
        } else {
            Write-Fail "No se pudo instalar Node.js automaticamente."
            Write-Host "   Descargalo manualmente desde: https://nodejs.org/es/download" -ForegroundColor Yellow
            Write-Host "   Una vez instalado, cierra y vuelve a ejecutar este asistente." -ForegroundColor Yellow
            exit 1
        }
    } else {
        Write-Fail "winget no esta disponible y Node.js no esta instalado."
        Write-Host "   Descargalo manualmente desde: https://nodejs.org/es/download" -ForegroundColor Yellow
        exit 1
    }
}

# ============================================================
# 2. Verificar / instalar opencode
# ============================================================
Write-Header "PASO 2/4 - Verificar opencode"
$opencodeCmd = "opencode"
if (-not (Confirm-Admin)) {
    Write-Fail "Se necesitan permisos de administrador para instalar opencode."
    Write-Host "   Re-ejecuta este script como administrador." -ForegroundColor Yellow
    exit 1
}

if (Test-Command opencode) {
    $ocVer = opencode --version 2>$null
    Write-Ok "opencode instalado: $ocVer"
} else {
    Write-Step "Instalando opencode globalmente..."
    npm install -g opencode-ai
    if (Test-Command opencode) {
        Write-Ok "opencode instalado exitosamente."
    } else {
        Write-Fail "No se pudo instalar opencode via npm."
        Write-Host "   Prueba manualmente con: npm install -g opencode-ai" -ForegroundColor Yellow
        exit 1
    }
}

# ============================================================
# 3. Verificar que opencode sirva
# ============================================================
Write-Header "PASO 3/4 - Verificar servidor opencode"

$running = Test-Url "http://127.0.0.1:4096/global/health"
if ($running) {
    Write-Ok "El servidor opencode ya esta corriendo."
} else {
    Write-Step "Iniciando opencode serve..."
    $ocExe = (Get-Command opencode).Source
    Start-Process -FilePath $ocExe -ArgumentList "serve" -WindowStyle Minimized

    $attempts = 0
    $started = $false
    while ($attempts -lt 45) {
        $attempts++
        Start-Sleep -Milliseconds 1500
        $running = Test-Url "http://127.0.0.1:4096/global/health"
        if ($running) { $started = $true; break }
    }

    if ($started) {
        Write-Ok "opencode serve esta listo."
    } else {
        Write-Fail "opencode no respondio a tiempo. Revisa que tengas un proveedor de IA configurado."
        Write-Host "   Ejecuta: opencode" -ForegroundColor Yellow
        Write-Host "   Y sigue el asistente para configurar el proveedor (opencode auth)." -ForegroundColor Yellow
    }
}

# ============================================================
# 4. Instalar / localizar la aplicacion
# ============================================================
Write-Header "PASO 4/4 - Instalar la aplicacion Solano Mod Translator"

$appExe = Join-Path $env:LOCALAPPDATA "Solano Mod Translator\solano-mod-translator.exe"

if ($SkipAppInstall -and (Test-Path $appExe)) {
    Write-Ok "La app ya esta instalada (omitiendo instalacion via -SkipAppInstall)."
} elseif (Test-Path $appExe) {
    Write-Step "La app ya esta instalada. Se omitira la reinstalacion."
} else {
    $setupExe = "src-tauri\target\release\bundle\nsis\Solano Mod Translator_1.0.0_x64-setup.exe"
    $setupExe = Join-Path $PSScriptRoot $setupExe

    if (Test-Path $setupExe) {
        Write-Step "Ejecutando instalador de la app..."
        Write-Host "   Se abrira el asistente de instalacion de la app." -ForegroundColor DarkGray
        Start-Process -FilePath $setupExe -Wait
        Write-Ok "Instalador de la app finalizado."
    } else {
        Write-Fail "No se encontro el instalador de la app: $setupExe"
        Write-Host "   Compilalo con: npm run tauri build" -ForegroundColor Yellow
    }
}

# ============================================================
# Resumen final
# ============================================================
Write-Header "Instalacion completada"
Write-Host ""
Write-Host " Para empezar a traducir:" -ForegroundColor White
Write-Host "   1. Abre la app 'Solano Mod Translator' desde el menu Inicio." -ForegroundColor White
Write-Host "   2. Ve a la pestana 'Configuracion'." -ForegroundColor White
Write-Host "   3. Pega tu API Key en el proveedor que quieras usar (Gemini/OpenAI)." -ForegroundColor White
Write-Host "      - Gemini: https://aistudio.google.com/apikey" -ForegroundColor DarkGray
Write-Host "      - OpenAI: https://platform.openai.com/api-keys" -ForegroundColor DarkGray
Write-Host "   4. Pulsa 'Guardar' y luego 'Probar conexion'." -ForegroundColor White
Write-Host "   5. Selecciona un mod y traduce." -ForegroundColor White
Write-Host ""
Write-Host " La API Key se guarda de forma segura en el almacen de credenciales de Windows." -ForegroundColor DarkGray
Write-Host " No es necesario compartir tu clave: cada persona usa la suya." -ForegroundColor DarkGray
Write-Host ""
Write-Host " Pulsa cualquier tecla para salir..." -ForegroundColor DarkGray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
