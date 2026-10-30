Write-Host "=== VERIFICACAO DE ASSINATURAS DIGITAIS (AUTHENTICODE) ===" -ForegroundColor Cyan
$files = Get-ChildItem -Path "dist" -Filter "*.exe"
foreach ($f in $files) {
    $sig = Get-AuthenticodeSignature $f.FullName
    $statusColor = if ($sig.Status -eq "Valid") { "Green" } else { "Yellow" }
    Write-Host "$($f.Name) : Status=$($sig.Status), Signer=$($sig.SignerCertificate.Subject)" -ForegroundColor $statusColor
}

Write-Host "`n=== VERIFICACAO DE DLLs IMPORTADAS (DEPENDENCIAS) ===" -ForegroundColor Cyan

function Get-ImportedDlls([string]$filePath) {
    $bytes = [System.IO.File]::ReadAllBytes($filePath)
    # Check MZ
    if ($bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) { return @() }
    $peOffset = [System.BitConverter]::ToInt32($bytes, 0x3C)
    if ($bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset+1] -ne 0x45) { return @() }
    
    # Optional Header magic (0x20B for PE32+)
    $optHeaderMagic = [System.BitConverter]::ToUInt16($bytes, $peOffset + 24)
    $is64 = ($optHeaderMagic -eq 0x20B)
    
    # Import Table RVA
    $importDirOffset = if ($is64) { $peOffset + 24 + 112 + 8 } else { $peOffset + 24 + 96 + 8 }
    $importRva = [System.BitConverter]::ToUInt32($bytes, $importDirOffset)
    if ($importRva -eq 0) { return @() }
    
    # Section headers to convert RVA to file offset
    $numSections = [System.BitConverter]::ToUInt16($bytes, $peOffset + 6)
    $optHeaderSize = [System.BitConverter]::ToUInt16($bytes, $peOffset + 20)
    $sectionHeaderStart = $peOffset + 24 + $optHeaderSize
    
    function RvaToOffset([uint32]$rva) {
        for ($s = 0; $s -lt $numSections; $s++) {
            $hdr = $sectionHeaderStart + ($s * 40)
            $vSize = [System.BitConverter]::ToUInt32($bytes, $hdr + 8)
            $vAddr = [System.BitConverter]::ToUInt32($bytes, $hdr + 12)
            $rawSize = [System.BitConverter]::ToUInt32($bytes, $hdr + 16)
            $rawPtr = [System.BitConverter]::ToUInt32($bytes, $hdr + 20)
            
            if ($rva -ge $vAddr -and $rva -lt ($vAddr + [Math]::Max($vSize, $rawSize))) {
                return $rawPtr + ($rva - $vAddr)
            }
        }
        return 0
    }
    
    $importFileOffset = RvaToOffset $importRva
    if ($importFileOffset -eq 0) { return @() }
    
    $dlls = [System.Collections.Generic.List[string]]::new()
    $curr = $importFileOffset
    while ($true) {
        $nameRva = [System.BitConverter]::ToUInt32($bytes, $curr + 12)
        if ($nameRva -eq 0) { break }
        $nameOffset = RvaToOffset $nameRva
        if ($nameOffset -ne 0) {
            $nameChars = [System.Collections.Generic.List[char]]::new()
            $idx = $nameOffset
            while ($bytes[$idx] -ne 0) {
                $nameChars.Add([char]$bytes[$idx])
                $idx++
            }
            $dlls.Add([string]::new($nameChars.ToArray()))
        }
        $curr += 20
    }
    return $dlls
}

foreach ($f in @("dist\PrysmAdmin.exe", "dist\PrysmHost.exe", "target\release\prysmview-svc.exe")) {
    if (Test-Path $f) {
        $dlls = Get-ImportedDlls (Resolve-Path $f).Path
        Write-Host "`nDependencias de $f :" -ForegroundColor Yellow
        foreach ($d in $dlls) {
            $isStandardWin = $d -match '^(KERNEL32|USER32|GDI32|ADVAPI32|SHELL32|OLE32|OLEAUT32|WS2_32|IPHLPAPI|BCRYPT|CRYPT32|SHLWAPI|NTDLL|DXGI|D3D11|SECUR32|VERSION|USERENV|WTSAPI32|SETUPAPI|CFGMGR32|IMM32|COMCTL32|COMDLG32|WINMM|DNSAPI|DWMPAPI|UXTHEME|WLDAP32|NCrypt|DWrite)\.dll$'
            $color = if ($isStandardWin) { "DarkGray" } elseif ($d -match 'VCRUNTIME|MSVCP') { "Red" } else { "Cyan" }
            Write-Host "  - $d" -ForegroundColor $color
        }
    }
}
