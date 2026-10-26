<#
.SYNOPSIS
    Script de Sincronização e Retentativa de Credencial de Máquina (D-03).
.DESCRIPTION
    Garante que a credencial da máquina (senha CSPRNG protegida por DPAPI) seja
    sincronizada com o servidor central. Se a rede falhar, registra uma pendência
    persistida e realiza novas tentativas com backoff até a recuperação da rede.
.PARAMETER ServerUrl
    URL da API de sinalização (ex: https://api.prysmview.com).
.PARAMETER MachineId
    Identificador único do PDV.
.PARAMETER SimulateNetworkFailure
    Flag para testes automatizados simulando queda de rede.
#>
[CmdletBinding()]
param(
    [string]$ServerUrl = "https://api.prysmview.com",
    [string]$MachineId = $env:COMPUTERNAME,
    [switch]$SimulateNetworkFailure,
    [int]$MaxRetries = 3
)

$ErrorActionPreference = "Stop"

$ConfigDir = "$env:ProgramData\PrysmView"
$CredFile = "$ConfigDir\device.cred"
$QueueFile = "$ConfigDir\pending_cred.queue"

if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
}

# 1. Recuperar ou gerar a credencial DPAPI local
if (-not (Test-Path $CredFile)) {
    Write-Host "[1/3] Gerando nova credencial CSPRNG de máquina..." -ForegroundColor Yellow
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    $bytes = New-Object byte[] 24
    $rng.GetBytes($bytes)
    $plain = [System.Convert]::ToBase64String($bytes)
    $sec = ConvertTo-SecureString $plain -AsPlainText -Force
    $enc = $sec | ConvertFrom-SecureString
    Set-Content -Path $CredFile -Value $enc -Force
}

$encryptedCred = (Get-Content -Path $CredFile -Raw).Trim()
$securePassword = $encryptedCred | ConvertTo-SecureString
$bstr = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($securePassword)
$plainPassword = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($bstr)
[System.Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)

# 2. Tentativa de envio para a API
Write-Host "[2/3] Sincronizando credencial da máquina $MachineId com API central..." -ForegroundColor Yellow
$syncSuccess = $false

if ($SimulateNetworkFailure) {
    Write-Warning "Simulação de falha de rede ativada para teste (D-03)."
} else {
    for ($attempt = 1; $attempt -le $MaxRetries; $attempt++) {
        try {
            $payload = @{
                machine_id = $MachineId
                credential_hash = (Get-FileHash -InputStream ([IO.MemoryStream]::new([Text.Encoding]::UTF8.GetBytes($plainPassword))) -Algorithm SHA256).Hash
                timestamp = (Get-Date).ToUniversalTime().ToString("o")
            } | ConvertTo-Json

            $resp = Invoke-RestMethod -Uri "$ServerUrl/api/v1/host/register-credential" -Method Post -Body $payload -ContentType "application/json" -TimeoutSec 5 -ErrorAction Stop
            $syncSuccess = $true
            Write-Host "      Credencial sincronizada com sucesso na tentativa $attempt." -ForegroundColor Green
            break
        } catch {
            Write-Host "      Falha na sincronização (tentativa $attempt/$MaxRetries): $($_.Exception.Message)" -ForegroundColor DarkGray
            Start-Sleep -Seconds 1
        }
    }
}

# 3. Tratamento de fila de pendências (D-03)
if (-not $syncSuccess) {
    Write-Warning "[3/3] Falha de comunicação com o servidor. Enfileirando sincronização pendente..."
    $queueItem = @{
        MachineId = $MachineId
        EncryptedCred = $encryptedCred
        EnqueuedAt = (Get-Date).ToUniversalTime().ToString("o")
        Attempts = 1
    }
    $queueItem | ConvertTo-Json | Set-Content -Path $QueueFile -Force
    Write-Host "      Pendência registrada em: $QueueFile. A máquina mantém a senha conhecida e tentará no próximo ciclo." -ForegroundColor Yellow
    return [PSCustomObject]@{
        Status = "Pending"
        QueueFile = $QueueFile
        MachineId = $MachineId
    }
} else {
    if (Test-Path $QueueFile) {
        Remove-Item -Path $QueueFile -Force
        Write-Host "[3/3] Fila de sincronizações pendentes limpa com sucesso." -ForegroundColor Green
    }
    return [PSCustomObject]@{
        Status = "Synchronized"
        MachineId = $MachineId
    }
}
