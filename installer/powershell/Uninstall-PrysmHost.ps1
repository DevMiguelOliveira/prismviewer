<#
.SYNOPSIS
    Script de Desinstalação Limpa do PrysmHost no PDV (D-01).
.DESCRIPTION
    Para o serviço Windows, encerra processos em execução, remove o registro do serviço,
    apaga arquivos temporários e remove entradas do Windows sem deixar artefatos órfãos.
#>
[CmdletBinding()]
param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

Write-Host "==================================================================" -ForegroundColor Yellow
Write-Host " PrysmView - Desinstalação Limpa do PrysmHost" -ForegroundColor Yellow
Write-Host "==================================================================" -ForegroundColor Yellow

# 1. Parar o serviço Windows
Write-Host "[1/4] Interrompendo serviço 'prysmview-svc'..." -ForegroundColor Yellow
$service = Get-Service -Name "prysmview-svc" -ErrorAction SilentlyContinue
if ($service) {
    if ($service.Status -eq 'Running') {
        Stop-Service -Name "prysmview-svc" -Force
        Start-Sleep -Seconds 2
    }
}

# 2. Encerrar qualquer processo visual do PrysmHost em execução
Write-Host "[2/4] Encerrando processos visuais residuais..." -ForegroundColor Yellow
Get-Process -Name "PrysmHost", "prysmview-svc" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

# 3. Execução do desinstalador oficial se presente
$InstallDir = "$env:ProgramFiles\PrysmView"
$Uninstaller = "$InstallDir\unins000.exe"

if (Test-Path $Uninstaller) {
    Write-Host "[3/4] Executando desinstalador oficial silenciosamente..." -ForegroundColor Yellow
    Start-Process -FilePath $Uninstaller -ArgumentList "/VERYSILENT /NORESTART" -Wait
} else {
    Write-Host "[3/4] Desinstalador não encontrado em $Uninstaller. Limpando serviço via sc.exe..." -ForegroundColor Yellow
    if ($service) {
        & sc.exe delete "prysmview-svc" | Out-Null
    }
}

# 4. Limpeza de diretórios de configuração e temporários
Write-Host "[4/4] Limpando pastas de dados temporários e credenciais..." -ForegroundColor Yellow
$ConfigDir = "$env:ProgramData\PrysmView"
if (Test-Path $ConfigDir) {
    Remove-Item -Path $ConfigDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "==================================================================" -ForegroundColor Green
Write-Host " Desinstalação do PrysmHost concluída com sucesso." -ForegroundColor Green
Write-Host "==================================================================" -ForegroundColor Green
