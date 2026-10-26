<#
.SYNOPSIS
    Configura o Windows Defender e SmartScreen para reconhecer o PrysmView como software corporativo confiavel.
.DESCRIPTION
    1. Instala o certificado digital oficial do PrysmView em 'Autoridades de Certificacao Raiz Confiaveis'
       e 'Editores Confiaveis' (TrustedPublisher).
    2. Adiciona regras de exclusao para os processos e pastas do PrysmView no Windows Defender.
    3. Remove o flag Zone.Identifier (Mark-of-the-Web) de executaveis baixados da internet para evitar alertas.
#>

param(
    [switch]$Uninstall = $false
)

$ErrorActionPreference = "SilentlyContinue"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   PrysmView - Configuracao de Confiabilidade Windows" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warning "Este script necessita de privilegios elevados (Executar como Administrador) para configurar o Windows Defender e certificados da maquina."
    Write-Host "Solicitando elevacao UAC..." -ForegroundColor Yellow
    Start-Process powershell.exe -ArgumentList ("-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" + $(if ($Uninstall) { " -Uninstall" } else { "" })) -Verb RunAs
    exit
}

$certFile = Join-Path $PSScriptRoot "..\dist\PrysmView-Certificate.cer"
if (-not (Test-Path $certFile)) {
    $certFile = Join-Path $PSScriptRoot "..\assets\PrysmView-Certificate.cer"
}

$processes = @("PrysmAdmin.exe", "prysmview-svc.exe", "PrysmHost.exe")
$paths = @(
    "${env:ProgramFiles}\PrysmView Admin",
    "${env:ProgramFiles(x86)}\PrysmView Admin",
    "${env:ProgramFiles}\PrysmView Host",
    "${env:ProgramFiles(x86)}\PrysmView Host",
    "${env:ProgramData}\PrysmView"
)

if ($Uninstall) {
    Write-Host "[1/2] Removendo regras do Windows Defender..." -ForegroundColor Yellow
    foreach ($p in $processes) {
        Remove-MpPreference -ExclusionProcess $p -ErrorAction SilentlyContinue
    }
    foreach ($path in $paths) {
        Remove-MpPreference -ExclusionPath $path -ErrorAction SilentlyContinue
    }
    Write-Host "[2/2] Regras removidas com sucesso." -ForegroundColor Green
    exit
}

# 1. Instalar Certificado Raiz e Editor Confiavel
if (Test-Path $certFile) {
    Write-Host "[1/3] Instalando certificado oficial em Editores Confiaveis..." -ForegroundColor Yellow
    $cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($certFile)
    
    $rootStore = New-Object System.Security.Cryptography.X509Certificates.X509Store([System.Security.Cryptography.X509Certificates.StoreName]::Root, [System.Security.Cryptography.X509Certificates.StoreLocation]::LocalMachine)
    $rootStore.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    $rootStore.Add($cert)
    $rootStore.Close()

    $pubStore = New-Object System.Security.Cryptography.X509Certificates.X509Store([System.Security.Cryptography.X509Certificates.StoreName]::TrustedPublisher, [System.Security.Cryptography.X509Certificates.StoreLocation]::LocalMachine)
    $pubStore.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    $pubStore.Add($cert)
    $pubStore.Close()

    Write-Host "      [OK] Certificado registrado como Autoridade e Editor Confiavel." -ForegroundColor Green
} else {
    Write-Host "[1/3] [AVISO] Certificado $certFile nao encontrado para importacao." -ForegroundColor DarkGray
}

# 2. Configurar Windows Defender
Write-Host "[2/3] Configurando integridade e exclusoes no Windows Defender..." -ForegroundColor Yellow
foreach ($p in $processes) {
    Add-MpPreference -ExclusionProcess $p -ErrorAction SilentlyContinue
}
foreach ($path in $paths) {
    Add-MpPreference -ExclusionPath $path -ErrorAction SilentlyContinue
}
Write-Host "      [OK] Processos ($($processes -join ', ')) autorizados no Defender." -ForegroundColor Green

# 3. Desbloquear binarios baixados (remover Mark of the Web - Zone.Identifier)
Write-Host "[3/3] Desbloqueando instaladores contra filtros de SmartScreen..." -ForegroundColor Yellow
$distDir = Join-Path $PSScriptRoot "..\dist"
if (Test-Path $distDir) {
    Get-ChildItem -Path $distDir -Filter "*.exe" -Recurse | ForEach-Object {
        Unblock-File -Path $_.FullName -ErrorAction SilentlyContinue
    }
}
Write-Host "      [OK] Binarios desbloqueados para execucao direta." -ForegroundColor Green

Write-Host "`nConfiguracao concluida com sucesso!" -ForegroundColor Cyan
Write-Host "O Windows 10/11 agora reconhece o PrysmView como um aplicativo seguro e confiavel." -ForegroundColor Green
