# setup-desktop.ps1 - Configura o PrysmView Desktop (Tauri) na raiz do projeto

Write-Host ">>> Configurando o PrysmView Desktop..." -ForegroundColor Cyan

# 1. Garantir pasta src-tauri
New-Item -ItemType Directory -Force -Path "src-tauri\src" | Out-Null
New-Item -ItemType Directory -Force -Path "src-tauri\icons" | Out-Null

# 2. Copiar icones
Copy-Item "assets\icon.ico" "src-tauri\icons\icon.ico" -Force -ErrorAction SilentlyContinue
Copy-Item "assets\logo.png" "src-tauri\icons\icon.png" -Force -ErrorAction SilentlyContinue
Copy-Item "assets\icon.ico" "src-tauri\icons\32x32.png" -Force -ErrorAction SilentlyContinue
Copy-Item "assets\icon.ico" "src-tauri\icons\128x128.png" -Force -ErrorAction SilentlyContinue
Copy-Item "assets\icon.ico" "src-tauri\icons\128x128@2x.png" -Force -ErrorAction SilentlyContinue

Write-Host "[OK] Pastas e icones configurados." -ForegroundColor Green
Write-Host "Para abrir o aplicativo nativo agora, execute:" -ForegroundColor Yellow
Write-Host "  npm run tauri:dev" -ForegroundColor White
Write-Host "Para compilar o instalador MSI e executavel final, execute:" -ForegroundColor Yellow
Write-Host "  npm run tauri:build" -ForegroundColor White
