# build_prysmview.ps1
# Script de build do PrysmView com Tauri (React + Tailwind)
# Autor: PrysmView Team
# Versao: 2.0.0

param(
    [string]$Mode = "production",
    [switch]$SkipTests = $false
)

$ErrorActionPreference = "Stop"

# Funcao para imprimir mensagens coloridas
function Write-ColorMessage {
    param(
        [string]$Message,
        [Alias("Color")]
        [string]$ForegroundColor = "White"
    )
    Write-Host $Message -ForegroundColor $ForegroundColor
}

# Funcao para verificar se um comando existe
function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    }
    catch {
        return $false
    }
}

# Detectar gerenciador de pacotes preferido (pnpm ou npm)
$global:PkgManager = "npm"
if (Test-Command "pnpm") {
    $global:PkgManager = "pnpm"
}

# Funcao para verificar dependencias
function Test-Dependencies {
    Write-ColorMessage ">>> Verificando dependencias do sistema..." -ForegroundColor Cyan

    $dependencies = @(
        @{Name = "Node.js"; Command = "node"},
        @{Name = "Rust / Cargo"; Command = "cargo"}
    )

    $missing = @()

    foreach ($dep in $dependencies) {
        if (Test-Command $dep.Command) {
            $ver = & $dep.Command --version 2>&1
            Write-ColorMessage "  [OK] $($dep.Name): $ver" -ForegroundColor Green
        }
        else {
            Write-ColorMessage "  [FALHA] $($dep.Name): nao encontrado" -ForegroundColor Red
            $missing += $dep.Name
        }
    }

    if (Test-Command "pnpm") {
        $pnpmVer = pnpm --version 2>&1
        Write-ColorMessage "  [OK] pnpm: $pnpmVer" -ForegroundColor Green
    }
    else {
        Write-ColorMessage "  [AVISO] pnpm nao encontrado globalmente; utilizando npm como fallback." -ForegroundColor Yellow
    }

    if ($missing.Count -gt 0) {
        $missingList = $missing -join ", "
        Write-ColorMessage "`n[ERRO] Dependencias obrigatorias faltando: $missingList" -ForegroundColor Red
        Write-ColorMessage "Por favor, instale Node.js e Rust (rustup.rs) antes de continuar." -ForegroundColor Yellow
        exit 1
    }

    Write-ColorMessage "[OK] Verificacao de dependencias concluida.`n" -ForegroundColor Green
}

# Funcao para instalar dependencias
function Install-Dependencies {
    param([string]$AppPath)

    Write-ColorMessage ">>> Instalando dependencias em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        if ($global:PkgManager -eq "pnpm") {
            pnpm install
        }
        else {
            npm install
        }

        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "[ERRO] Falha ao instalar dependencias em $AppPath" -ForegroundColor Red
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "[OK] Dependencias instaladas em $AppPath`n" -ForegroundColor Green
}

# Funcao para executar testes
function Run-Tests {
    param([string]$AppPath)

    Write-ColorMessage ">>> Executando testes em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        if ($global:PkgManager -eq "pnpm") {
            pnpm test
        }
        else {
            npm test
        }

        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "[ERRO] Testes falharam em $AppPath" -ForegroundColor Red
            Set-Location -Path "..\.."
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "[OK] Testes passaram em $AppPath`n" -ForegroundColor Green
}

# Funcao para fazer lint
function Run-Lint {
    param([string]$AppPath)

    Write-ColorMessage ">>> Executando lint em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        if ($global:PkgManager -eq "pnpm") {
            pnpm lint
        }
        else {
            npm run lint
        }

        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "[ERRO] Lint falhou em $AppPath" -ForegroundColor Red
            Set-Location -Path "..\.."
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "[OK] Lint passou em $AppPath`n" -ForegroundColor Green
}

# Funcao para fazer typecheck
function Run-TypeCheck {
    param([string]$AppPath)

    Write-ColorMessage ">>> Executando typecheck em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        if ($global:PkgManager -eq "pnpm") {
            pnpm typecheck
        }
        else {
            npm run typecheck
        }

        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "[ERRO] Typecheck falhou em $AppPath" -ForegroundColor Red
            Set-Location -Path "..\.."
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "[OK] Typecheck passou em $AppPath`n" -ForegroundColor Green
}

# Funcao para build do app Tauri
function Build-TauriApp {
    param(
        [string]$AppPath,
        [string]$AppName
    )

    Write-ColorMessage ">>> Compilando $AppName..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    # Build do frontend
    Write-ColorMessage "  Build do frontend (Vite)..." -ForegroundColor Cyan
    if ($global:PkgManager -eq "pnpm") {
        pnpm build
    }
    else {
        npm run build
    }

    if ($LASTEXITCODE -ne 0) {
        Write-ColorMessage "[ERRO] Falha no build do frontend de $AppName" -ForegroundColor Red
        Set-Location -Path "..\.."
        exit 1
    }

    # Build do Tauri
    Write-ColorMessage "  Build do Tauri (Rust + WebView)..." -ForegroundColor Cyan
    if ($global:PkgManager -eq "pnpm") {
        pnpm tauri build
    }
    else {
        npx @tauri-apps/cli build
    }

    if ($LASTEXITCODE -ne 0) {
        Write-ColorMessage "[ERRO] Falha no build do Tauri de $AppName" -ForegroundColor Red
        Set-Location -Path "..\.."
        exit 1
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "[OK] $AppName compilado com sucesso`n" -ForegroundColor Green
}

# Funcao principal
function Build-PrysmView {
    $StartTime = Get-Date

    Write-ColorMessage "===============================================================" -ForegroundColor Cyan
    Write-ColorMessage "  PRYSMVIEW BUILD SCRIPT v2.0.0" -ForegroundColor Cyan
    Write-ColorMessage "  React + Tailwind + Tauri" -ForegroundColor Cyan
    Write-ColorMessage "===============================================================" -ForegroundColor Cyan
    Write-ColorMessage ""

    # Verificar dependencias
    Test-Dependencies

    # Criar diretorio de releases
    $ReleaseDir = ".\Releases"
    if (!(Test-Path $ReleaseDir)) {
        New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
        Write-ColorMessage "[OK] Diretorio de releases criado: $ReleaseDir`n" -ForegroundColor Green
    }

    # Verificar se as pastas existem
    $HostPath = ".\apps\host"
    $AdminPath = ".\apps\admin"

    if (!(Test-Path $HostPath)) {
        Write-ColorMessage "[ERRO] Pasta do Host nao encontrada: $HostPath" -ForegroundColor Red
        Write-ColorMessage "Execute o script de setup antes de fazer o build." -ForegroundColor Yellow
        exit 1
    }

    if (!(Test-Path $AdminPath)) {
        Write-ColorMessage "[ERRO] Pasta do Admin nao encontrada: $AdminPath" -ForegroundColor Red
        Write-ColorMessage "Execute o script de setup antes de fazer o build." -ForegroundColor Yellow
        exit 1
    }

    # ========================
    # BUILD DO HOST
    # ========================
    Write-ColorMessage "---------------------------------------------------------------" -ForegroundColor Cyan
    Write-ColorMessage "  BUILD DO HOST (PDV)" -ForegroundColor Cyan
    Write-ColorMessage "---------------------------------------------------------------" -ForegroundColor Cyan
    Write-ColorMessage ""

    Install-Dependencies -AppPath $HostPath

    if (-not $SkipTests) {
        Run-Lint -AppPath $HostPath
        Run-TypeCheck -AppPath $HostPath
        Run-Tests -AppPath $HostPath
    }

    Build-TauriApp -AppPath $HostPath -AppName "PrysmView Host"

    # Copiar instalador do Host
    $HostMsi = Get-ChildItem -Path "$HostPath\src-tauri\target\release\bundle\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
    $HostExe = Get-ChildItem -Path "$HostPath\src-tauri\target\release\PrysmView Host.exe" -ErrorAction SilentlyContinue

    if ($HostMsi) {
        Copy-Item $HostMsi.FullName -Destination $ReleaseDir -Force
        Write-ColorMessage "[OK] Instalador MSI do Host copiado: $($HostMsi.Name)" -ForegroundColor Green
    }

    if ($HostExe) {
        Copy-Item $HostExe.FullName -Destination $ReleaseDir -Force
        Write-ColorMessage "[OK] Executavel do Host copiado: $($HostExe.Name)" -ForegroundColor Green
    }

    Write-ColorMessage ""

    # ========================
    # BUILD DO ADMIN
    # ========================
    Write-ColorMessage "---------------------------------------------------------------" -ForegroundColor Cyan
    Write-ColorMessage "  BUILD DO ADMIN (Console do Operador)" -ForegroundColor Cyan
    Write-ColorMessage "---------------------------------------------------------------" -ForegroundColor Cyan
    Write-ColorMessage ""

    Install-Dependencies -AppPath $AdminPath

    if (-not $SkipTests) {
        Run-Lint -AppPath $AdminPath
        Run-TypeCheck -AppPath $AdminPath
        Run-Tests -AppPath $AdminPath
    }

    Build-TauriApp -AppPath $AdminPath -AppName "PrysmView Admin"

    # Copiar instalador do Admin
    $AdminMsi = Get-ChildItem -Path "$AdminPath\src-tauri\target\release\bundle\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
    $AdminExe = Get-ChildItem -Path "$AdminPath\src-tauri\target\release\PrysmView Admin.exe" -ErrorAction SilentlyContinue

    if ($AdminMsi) {
        Copy-Item $AdminMsi.FullName -Destination $ReleaseDir -Force
        Write-ColorMessage "[OK] Instalador MSI do Admin copiado: $($AdminMsi.Name)" -ForegroundColor Green
    }

    if ($AdminExe) {
        Copy-Item $AdminExe.FullName -Destination $ReleaseDir -Force
        Write-ColorMessage "[OK] Executavel do Admin copiado: $($AdminExe.Name)" -ForegroundColor Green
    }

    Write-ColorMessage ""

    # ========================
    # RESUMO
    # ========================
    $EndTime = Get-Date
    $Duration = $EndTime - $StartTime

    Write-ColorMessage "===============================================================" -ForegroundColor Green
    Write-ColorMessage "  BUILD CONCLUIDO COM SUCESSO!" -ForegroundColor Green
    Write-ColorMessage "===============================================================" -ForegroundColor Green
    Write-ColorMessage ""
    Write-ColorMessage "Tempo total: $($Duration.ToString('mm\:ss'))" -ForegroundColor Cyan
    Write-ColorMessage "Os executaveis (.exe) e instaladores (.msi) estao na pasta:" -ForegroundColor Cyan
    Write-ColorMessage "  $ReleaseDir" -ForegroundColor White
    Write-ColorMessage ""
    Write-ColorMessage "Arquivos gerados:" -ForegroundColor Cyan

    Get-ChildItem -Path $ReleaseDir | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        Write-ColorMessage "  * $($_.Name) ($size MB)" -ForegroundColor White
    }

    Write-ColorMessage ""
}

# Executar build
try {
    Build-PrysmView
}
catch {
    Write-ColorMessage "`n[ERRO FATAL]: $_" -ForegroundColor Red
    Write-ColorMessage "Stack trace: $($_.ScriptStackTrace)" -ForegroundColor Red
    exit 1
}
