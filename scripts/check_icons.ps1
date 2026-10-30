Add-Type -AssemblyName System.Drawing

Write-Host "=== VERIFICACAO DE ICONES NOS EXECUTAVEIS ===" -ForegroundColor Cyan

function Inspect-Icon-File([string]$path) {
    if (-not (Test-Path $path)) { return }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $count = [System.BitConverter]::ToUInt16($bytes, 4)
    Write-Host "$path contem $count resolucoes/frames:" -ForegroundColor Yellow
    for ($i=0; $i -lt $count; $i++) {
        $offset = 6 + ($i * 16)
        $w = $bytes[$offset]
        if ($w -eq 0) { $w = 256 }
        $h = $bytes[$offset+1]
        if ($h -eq 0) { $h = 256 }
        $bpp = [System.BitConverter]::ToUInt16($bytes, $offset+6)
        $size = [System.BitConverter]::ToUInt32($bytes, $offset+8)
        Write-Host "  - Resoluçao $w x $h ($bpp bpp, $size bytes)"
    }
}

Inspect-Icon-File "assets\icon.ico"

$files = @(
    "dist\PrysmAdmin.exe",
    "dist\PrysmHost.exe",
    "dist\PrysmAdmin-Setup-0.1.0.exe",
    "dist\PrysmHost-Setup-0.1.0.exe",
    "target\release\prysmview-svc.exe",
    "dist\PrysmHost-Portable\PrysmHost.exe",
    "dist\PrysmHost-Portable\prysmview-svc.exe"
)

foreach ($f in $files) {
    if (Test-Path $f) {
        $full = (Resolve-Path $f).Path
        try {
            $ico = [System.Drawing.Icon]::ExtractAssociatedIcon($full)
            if ($ico) {
                Write-Host "$f : ICONE OK ($($ico.Width)x$($ico.Height))" -ForegroundColor Green
            } else {
                Write-Host "$f : SEM ICONE" -ForegroundColor Red
            }
        } catch {
            Write-Host "$f : Erro ao extrair - $_" -ForegroundColor Red
        }
    } else {
        Write-Host "$f : NAO ENCONTRADO" -ForegroundColor DarkGray
    }
}

