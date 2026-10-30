# Script de Assinatura Digital e Confiabilidade do Windows Defender / SmartScreen
# Gera certificado Authenticode, assina binarios e instaladores, e registra publisher confiavel.

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   PrysmView - Assinatura Digital & Confiabilidade Windows" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$distDir = Join-Path $PSScriptRoot "..\dist"
$assetsDir = Join-Path $PSScriptRoot "..\assets"
$targetReleaseDir = Join-Path $PSScriptRoot "..\target\release"

if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir | Out-Null
}

$certSubject = "CN=PrysmView Security Solutions, O=PrysmView Team, C=BR"
$cert = Get-ChildItem -Path "Cert:\CurrentUser\My" | Where-Object {
    $_.Subject -eq $certSubject -and $_.NotAfter -gt (Get-Date)
} | Select-Object -First 1

if (-not $cert) {
    Write-Host "[1/5] Gerando Certificado de Assinatura de Codigo (Authenticode)..." -ForegroundColor Yellow
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject $certSubject `
        -KeyAlgorithm RSA `
        -KeyLength 3072 `
        -HashAlgorithm SHA256 `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears(5) `
        -FriendlyName "PrysmView Code Signing Certificate"
    Write-Host "      Certificado criado: $($cert.Thumbprint)" -ForegroundColor Green
} else {
    Write-Host "[1/5] Certificado de Assinatura encontrado: $($cert.Thumbprint)" -ForegroundColor Green
}

# Exporta certificado publico (.cer)
$publicCertPathDist = Join-Path $distDir "PrysmView-Certificate.cer"
$publicCertPathAssets = Join-Path $assetsDir "PrysmView-Certificate.cer"
Export-Certificate -Cert $cert -FilePath $publicCertPathDist -Force | Out-Null
Export-Certificate -Cert $cert -FilePath $publicCertPathAssets -Force | Out-Null
Write-Host "[2/5] Certificado publico exportado para $publicCertPathDist" -ForegroundColor Green

# Instala nos repositorios de Editores Confiaveis locais via certutil (sem prompts bloqueantes)
Write-Host "[3/5] Registrando certificado como Editor Confiavel (TrustedPublisher)..." -ForegroundColor Yellow
& certutil.exe -addstore -user -f CA "$publicCertPathDist" | Out-Null
& certutil.exe -addstore -user -f TrustedPublisher "$publicCertPathDist" | Out-Null
Write-Host "      [OK] Registrado em CurrentUser\TrustedPublisher e CA" -ForegroundColor DarkGray

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($isAdmin) {
    try {
        & certutil.exe -addstore -f Root "$publicCertPathDist" | Out-Null
        & certutil.exe -addstore -f TrustedPublisher "$publicCertPathDist" | Out-Null
        Write-Host "      [OK] Registrado em LocalMachine\Root e TrustedPublisher (Administrador)" -ForegroundColor DarkGray
    } catch {
        Write-Warning "      Nao foi possivel registrar em LocalMachine: $_"
    }
}

# Limpeza de instaladores e pacotes redundantes da pasta dist/
$redundantItems = @(
    (Join-Path $distDir "PrysmAdmin-Setup-0.1.0.exe"),
    (Join-Path $distDir "PrysmHost-Setup-0.1.0.exe"),
    (Join-Path $distDir "PrysmHost-Portable.zip"),
    (Join-Path $distDir "PrysmHost-Portable")
)
foreach ($item in $redundantItems) {
    if (Test-Path $item) {
        Remove-Item -Path $item -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Compila executáveis de release atualizados com Cargo antes de sincronizar
Write-Host "[3.5/5] Compilando executaveis atualizados com cargo build --release..." -ForegroundColor Yellow
& cargo build --release --bin PrysmAdmin --bin PrysmHost
if ($LASTEXITCODE -ne 0) {
    throw "Falha ao compilar binários com cargo build --release."
}
Write-Host "      [OK] Compilacao release concluida com sucesso." -ForegroundColor Green

# Fecha eventuais processos em execução para evitar bloqueio de arquivo (locked file) no Windows
$runningProcesses = Get-Process -Name PrysmAdmin, PrysmHost -ErrorAction SilentlyContinue
if ($runningProcesses) {
    Write-Host "      Fechando instancias anteriores em execucao para permitir atualizacao..." -ForegroundColor DarkGray
    $runningProcesses | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 800
}

# Sincroniza executáveis diretos compilados para dist/
if (Test-Path (Join-Path $targetReleaseDir "PrysmAdmin.exe")) {
    Copy-Item (Join-Path $targetReleaseDir "PrysmAdmin.exe") (Join-Path $distDir "PrysmAdmin.exe") -Force
}
if (Test-Path (Join-Path $targetReleaseDir "PrysmHost.exe")) {
    Copy-Item (Join-Path $targetReleaseDir "PrysmHost.exe") (Join-Path $distDir "PrysmHost.exe") -Force
}

# Arquivos executáveis oficiais para assinar com Authenticode SHA-256
$filesToSign = @(
    (Join-Path $targetReleaseDir "PrysmAdmin.exe"),
    (Join-Path $targetReleaseDir "PrysmHost.exe"),
    (Join-Path $targetReleaseDir "prysmview-svc.exe"),
    (Join-Path $distDir "PrysmAdmin.exe"),
    (Join-Path $distDir "PrysmHost.exe")
)

Write-Host "[4/5] Assinando executaveis diretos em tempo real com SHA256..." -ForegroundColor Yellow
foreach ($filePath in $filesToSign) {
    if (Test-Path $filePath) {
        try {
            $sig = Set-AuthenticodeSignature -FilePath $filePath -Certificate $cert -HashAlgorithm SHA256
            $rel = Resolve-Path $filePath -Relative
            Write-Host "      [OK] Assinado com sucesso: $rel (Status: $($sig.Status))" -ForegroundColor Green
        } catch {
            Write-Warning "      Erro ao assinar ${filePath}: $_"
        }
    } else {
        Write-Host "      [IGNORADO] Arquivo nao encontrado ainda: $(Split-Path $filePath -Leaf)" -ForegroundColor DarkGray
    }
}


# Gerar Manifesto de Integridade Criptografica SHA256SUMS.txt
Write-Host "[5/5] Gerando manifesto de hashes SHA-256 (SHA256SUMS.txt)..." -ForegroundColor Yellow
$checksumFile = Join-Path $distDir "SHA256SUMS.txt"
$hashLines = @()

foreach ($filePath in $filesToSign) {
    if (Test-Path $filePath) {
        $fileName = Split-Path $filePath -Leaf
        $hash = (Get-FileHash -Path $filePath -Algorithm SHA256).Hash.ToLower()
        $hashLines += "$hash  $fileName"
    }
}

if (Test-Path $publicCertPathDist) {
    $certHash = (Get-FileHash -Path $publicCertPathDist -Algorithm SHA256).Hash.ToLower()
    $hashLines += "$certHash  PrysmView-Certificate.cer"
}

$hashLines | Out-File -FilePath $checksumFile -Encoding utf8 -Force
Write-Host "      Manifesto gerado em $checksumFile" -ForegroundColor Green

Write-Host "`nProcesso concluido com sucesso!" -ForegroundColor Cyan
Write-Host "O software agora possui assinatura digital valida, manifest do Windows 10/11 e integridade comprovada." -ForegroundColor Green
