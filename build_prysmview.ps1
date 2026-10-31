# build_prysmview.ps1
# Script de build do PrysmView com Tauri (React + Tailwind)
# Autor: PrysmView Team
# Versão: 2.0.0

param(
    [string]$Mode = "production",
    [switch]$SkipTests = $false
)

$ErrorActionPreference = "Stop"

# Função para imprimir mensagens coloridas
function Write-ColorMessage {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Função para verificar se um comando existe
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

# Função para verificar dependências
function Test-Dependencies {
    Write-ColorMessage ">>> Verificando dependências..." -ForegroundColor Cyan

    $dependencies = @(
        @{Name = "Node.js"; Command = "node"; MinVersion = "18.0.0"},
        @{Name = "pnpm"; Command = "pnpm"; MinVersion = "8.0.0"},
        @{Name = "Rust"; Command = "cargo"; MinVersion = "1.70.0"}
    )

    $missing = @()

    foreach ($dep in $dependencies) {
        if (Test-Command $dep.Command) {
            $version = & $dep.Command --version 2>&1
            Write-ColorMessage "  ✓ $($dep.Name): $version" -ForegroundColor Green
        }
        else {
            Write-ColorMessage "  ✗ $($dep.Name): não encontrado" -ForegroundColor Red
            $missing += $dep.Name
        }
    }

    if ($missing.Count -gt 0) {
        Write-ColorMessage "`n❌ Dependências faltando: $($missing -join ', ')" -ForegroundColor Red
        Write-ColorMessage "Por favor, instale as dependências faltantes antes de continuar." -ForegroundColor Yellow
        exit 1
    }

    Write-ColorMessage "✓ Todas as dependências estão instaladas.`n" -ForegroundColor Green
}

# Função para instalar dependências
function Install-Dependencies {
    param([string]$AppPath)

    Write-ColorMessage ">>> Instalando dependências em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        pnpm install
        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "❌ Falha ao instalar dependências em $AppPath" -ForegroundColor Red
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ Dependências instaladas em $AppPath`n" -ForegroundColor Green
}

# Função para executar testes
function Run-Tests {
    param([string]$AppPath)

    Write-ColorMessage ">>> Executando testes em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        pnpm test
        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "❌ Testes falharam em $AppPath" -ForegroundColor Red
            Set-Location -Path "..\.."
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ Testes passaram em $AppPath`n" -ForegroundColor Green
}

# Função para fazer lint
function Run-Lint {
    param([string]$AppPath)

    Write-ColorMessage ">>> Executando lint em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        pnpm lint
        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "❌ Lint falhou em $AppPath" -ForegroundColor Red
            Set-Location -Path "..\.."
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ Lint passou em $AppPath`n" -ForegroundColor Green
}

# Função para fazer typecheck
function Run-TypeCheck {
    param([string]$AppPath)

    Write-ColorMessage ">>> Executando typecheck em $AppPath..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    if (Test-Path "package.json") {
        pnpm typecheck
        if ($LASTEXITCODE -ne 0) {
            Write-ColorMessage "❌ Typecheck falhou em $AppPath" -ForegroundColor Red
            Set-Location -Path "..\.."
            exit 1
        }
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ Typecheck passou em $AppPath`n" -ForegroundColor Green
}

# Função para build do app Tauri
function Build-TauriApp {
    param(
        [string]$AppPath,
        [string]$AppName
    )

    Write-ColorMessage ">>> Compilando $AppName..." -ForegroundColor Yellow
    Set-Location -Path $AppPath

    # Build do frontend
    Write-ColorMessage "  Build do frontend (Vite)..." -ForegroundColor Cyan
    pnpm build
    if ($LASTEXITCODE -ne 0) {
        Write-ColorMessage "❌ Falha no build do frontend de $AppName" -ForegroundColor Red
        Set-Location -Path "..\.."
        exit 1
    }

    # Build do Tauri
    Write-ColorMessage "  Build do Tauri..." -ForegroundColor Cyan
    pnpm tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-ColorMessage "❌ Falha no build do Tauri de $AppName" -ForegroundColor Red
        Set-Location -Path "..\.."
        exit 1
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ $AppName compilado com sucesso`n" -ForegroundColor Green
}

# Função principal
function Build-PrysmView {
    $StartTime = Get-Date

    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-ColorMessage "  PRYSMVIEW BUILD SCRIPT v2.0.0" -ForegroundColor Cyan
    Write-ColorMessage "  React + Tailwind + Tauri" -ForegroundColor Cyan
    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-ColorMessage ""

    # Verificar dependências
    Test-Dependencies

    # Criar diretório de releases
    $ReleaseDir = ".\Releases"
    if (!(Test-Path $ReleaseDir)) {
        New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
        Write-ColorMessage "✓ Diretório de releases criado: $ReleaseDir`n" -ForegroundColor Green
    }

    # Verificar se as pastas existem
    $HostPath = ".\apps\host"
    $AdminPath = ".\apps\admin"

    if (!(Test-Path $HostPath)) {
        Write-ColorMessage "❌ Pasta do Host não encontrada: $HostPath" -ForegroundColor Red
        Write-ColorMessage "Execute o script de setup antes de fazer o build." -ForegroundColor Yellow
        exit 1
    }

    if (!(Test-Path $AdminPath)) {
        Write-ColorMessage "❌ Pasta do Admin não encontrada: $AdminPath" -ForegroundColor Red
        Write-ColorMessage "Execute o script de setup antes de fazer o build." -ForegroundColor Yellow
        exit 1
    }

    # ========================
    # BUILD DO HOST
    # ========================
    Write-ColorMessage "───────────────────────────────────────────────────────────────" -ForegroundColor Cyan
    Write-ColorMessage "  BUILD DO HOST" -ForegroundColor Cyan
    Write-ColorMessage "───────────────────────────────────────────────────────────────" -ForegroundColor Cyan
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
        Write-ColorMessage "✓ Instalador MSI do Host copiado: $($HostMsi.Name)" -ForegroundColor Green
    }

    if ($HostExe) {
        Copy-Item $HostExe.FullName -Destination $ReleaseDir -Force
        Write-ColorMessage "✓ Executável do Host copiado: $($HostExe.Name)" -ForegroundColor Green
    }

    Write-ColorMessage ""

    # ========================
    # BUILD DO ADMIN
    # ========================
    Write-ColorMessage "───────────────────────────────────────────────────────────────" -ForegroundColor Cyan
    Write-ColorMessage "  BUILD DO ADMIN" -ForegroundColor Cyan
    Write-ColorMessage "───────────────────────────────────────────────────────────────" -ForegroundColor Cyan
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
        Write-ColorMessage "✓ Instalador MSI do Admin copiado: $($AdminMsi.Name)" -ForegroundColor Green
    }

    if ($AdminExe) {
        Copy-Item $AdminExe.FullName -Destination $ReleaseDir -Force
        Write-ColorMessage "✓ Executável do Admin copiado: $($AdminExe.Name)" -ForegroundColor Green
    }

    Write-ColorMessage ""

    # ========================
    # RESUMO
    # ========================
    $EndTime = Get-Date
    $Duration = $EndTime - $StartTime

    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Green
    Write-ColorMessage "  BUILD CONCLUÍDO COM SUCESSO!" -ForegroundColor Green
    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Green
    Write-ColorMessage ""
    Write-ColorMessage "Tempo total: $($Duration.ToString('mm\:ss'))" -ForegroundColor Cyan
    Write-ColorMessage "Os executáveis (.exe) e instaladores (.msi) estão na pasta:" -ForegroundColor Cyan
    Write-ColorMessage "  $ReleaseDir" -ForegroundColor White
    Write-ColorMessage ""
    Write-ColorMessage "Arquivos gerados:" -ForegroundColor Cyan

    Get-ChildItem -Path $ReleaseDir | ForEach-Object {
        $size = [math]::Round($_.Length / 1MB, 2)
        Write-ColorMessage "  • $($_.Name) ($size MB)" -ForegroundColor White
    }

    Write-ColorMessage ""
}

# Executar build
try {
    Build-PrysmView
}
catch {
    Write-ColorMessage "`n❌ ERRO FATAL: $_" -ForegroundColor Red
    Write-ColorMessage "Stack trace: $($_.ScriptStackTrace)" -ForegroundColor Red
    exit 1
}
