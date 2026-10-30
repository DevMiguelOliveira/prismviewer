$dst = "$env:LOCALAPPDATA\Programs\PrysmView Admin\PrysmAdmin.exe"
if (Test-Path "target\release\PrysmAdmin.exe") {
    Copy-Item -Force "target\release\PrysmAdmin.exe" -Destination $dst
    Start-Process -FilePath $dst
    Write-Host "PrysmAdmin atualizado e aberto com sucesso: $dst"
} else {
    Write-Host "Arquivo target\release\PrysmAdmin.exe nao encontrado."
}
