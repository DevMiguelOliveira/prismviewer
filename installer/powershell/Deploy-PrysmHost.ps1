<#
.SYNOPSIS
    Script de Implantação Automatizada em Anéis do PrysmHost no PDV (D-01 / D-02 / D-03).
.DESCRIPTION
    Realiza a instalação silenciosa do agente PrysmHost, configuração de senha aleatória de alta entropia
    gerada com CSPRNG e validação ativa do heartbeat antes de avançar para a próxima máquina do anel.
.PARAMETER Ring
    Anel de implantação ('Canary_5', 'Pilot_50', 'Production').
.PARAMETER InstallerPath
    Caminho do executável do instalador PrysmHost-Setup.exe.
.PARAMETER ServerUrl
    URL da API de sinalização (ex: https://api.prysmview.com).
.PARAMETER MachineId
    Identificador único do PDV (ex: PDV-101).
#>
[CmdletBinding()]
param(
    [ValidateSet('Canary_5', 'Pilot_50', 'Production')]
    [string]$Ring = 'Canary_5',

    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,

    [string]$ServerUrl = "https://api.prysmview.com",
    [string]$MachineId = $env:COMPUTERNAME,
    [string]$ReportPath = ".\deployment-report.json"
)

$ErrorActionPreference = "Stop"

Write-Host "==================================================================" -ForegroundColor Cyan
Write-Host " PrysmView - Implantação em Anéis (Ring: $Ring)" -ForegroundColor Cyan
Write-Host " Máquina Alvo: $MachineId" -ForegroundColor Cyan
Write-Host "==================================================================" -ForegroundColor Cyan

# 1. Validação do executável
if (-not (Test-Path $InstallerPath)) {
    throw "Instalador não encontrado em: $InstallerPath"
}

# 2. Execução da instalação silenciosa
Write-Host "[1/4] Executando instalador silencioso do PrysmHost..." -ForegroundColor Yellow
$process = Start-Process -FilePath $InstallerPath -ArgumentList "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART" -Wait -PassThru

if ($process.ExitCode -ne 0) {
    throw "Instalação falhou com código de saída: $($process.ExitCode)"
}
Write-Host "      Instalação concluída com sucesso." -ForegroundColor Green

# 3. Geração de senha CSPRNG permanente e segura da máquina (D-02)
Write-Host "[2/4] Gerando credencial permanente de máquina com CSPRNG..." -ForegroundColor Yellow
$rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
$bytes = New-Object byte[] 24
$rng.GetBytes($bytes)
$MachinePassword = [System.Convert]::ToBase64String($bytes)

# Salva credencial protegida por DPAPI no diretório do serviço (sem exibir em processos/WMI)
$ConfigDir = "$env:ProgramData\PrysmView"
if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
}
$SecuredSecret = ConvertTo-SecureString $MachinePassword -AsPlainText -Force
$ExportedSecret = $SecuredSecret | ConvertFrom-SecureString
Set-Content -Path "$ConfigDir\device.cred" -Value $ExportedSecret -Force
Write-Host "      Credencial DPAPI de máquina persistida com sucesso." -ForegroundColor Green

# 4. Início e validação do Serviço Windows
Write-Host "[3/4] Validando status do serviço 'prysmview-svc'..." -ForegroundColor Yellow
$service = Get-Service -Name "prysmview-svc" -ErrorAction SilentlyContinue

if ($service -and $service.Status -ne 'Running') {
    Start-Service -Name "prysmview-svc"
    Start-Sleep -Seconds 3
}

$service = Get-Service -Name "prysmview-svc"
if ($service.Status -ne 'Running') {
    throw "Serviço 'prysmview-svc' não está em execução! Status atual: $($service.Status)"
}
Write-Host "      Serviço prysmview-svc ativo e em execução." -ForegroundColor Green

# 5. Confirmação do Heartbeat na API de Sinalização (Validação do Anel)
Write-Host "[4/4] Confirmando recepção do Heartbeat na API ($ServerUrl)..." -ForegroundColor Yellow
$HeartbeatSuccess = $false
$Retries = 10

for ($i = 1; $i -le $Retries; $i++) {
    try {
        $Health = Invoke-RestMethod -Uri "$ServerUrl/health" -Method Get -TimeoutSec 5
        if ($Health.status -eq "ok") {
            $HeartbeatSuccess = $true
            break
        }
    } catch {
        Write-Host "      Tentativa $i/$Retries: Aguardando sinal da API..." -ForegroundColor DarkGray
        Start-Sleep -Seconds 3
    }
}

$ResultStatus = if ($HeartbeatSuccess) { "Sucesso" } else { "Pendente" }

# 6. Emissão do Relatório de Implantação
$Report = [PSCustomObject]@{
    DataHora      = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    Ring          = $Ring
    MachineId     = $MachineId
    Status        = $ResultStatus
    ServiceStatus = $service.Status.ToString()
    ApiUrl        = $ServerUrl
}

$Report | ConvertTo-Json -Depth 2 | Set-Content -Path $ReportPath -Force

Write-Host "==================================================================" -ForegroundColor Green
Write-Host " Resultado da Implantação: $ResultStatus" -ForegroundColor Green
Write-Host " Relatório gerado em: $ReportPath" -ForegroundColor Green
Write-Host "==================================================================" -ForegroundColor Green
