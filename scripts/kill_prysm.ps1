$procs = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "*Prysm*" -or $_.ProcessName -like "*prysm*" }
if ($procs) {
    Write-Host "Processos encontrados:"
    $procs | ForEach-Object {
        Write-Host "PID $($_.Id): $($_.ProcessName)"
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        Write-Host "  Encerrado."
    }
} else {
    Write-Host "Nenhum processo Prysm ativo."
}
