# setup_prysmview.ps1
# Script de setup inicial do PrysmView (React + Tailwind + Tauri)
# Autor: PrysmView Team
# Versão: 2.0.0

param(
    [switch]$SkipGitInit = $false
)

$ErrorActionPreference = "Stop"

# Função para imprimir mensagens coloridas
function Write-ColorMessage {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Função para verificar se um comando existe
function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    }
    catch {
        return $false
    }
}

# Função para verificar dependências
function Test-Dependencies {
    Write-ColorMessage ">>> Verificando dependências do sistema..." -ForegroundColor Cyan

    $dependencies = @(
        @{Name = "Node.js"; Command = "node"; MinVersion = "18.0.0"; InstallUrl = "https://nodejs.org/"},
        @{Name = "pnpm"; Command = "pnpm"; MinVersion = "8.0.0"; InstallUrl = "https://pnpm.io/installation"},
        @{Name = "Rust"; Command = "cargo"; MinVersion = "1.70.0"; InstallUrl = "https://rustup.rs/"}
    )

    $missing = @()

    foreach ($dep in $dependencies) {
        if (Test-Command $dep.Command) {
            $version = & $dep.Command --version 2>&1
            Write-ColorMessage "  ✓ $($dep.Name): $version" -ForegroundColor Green
        }
        else {
            Write-ColorMessage "  ✗ $($dep.Name): não encontrado" -ForegroundColor Red
            $missing += $dep
        }
    }

    if ($missing.Count -gt 0) {
        Write-ColorMessage "`n❌ Dependências faltando:" -ForegroundColor Red
        foreach ($dep in $missing) {
            Write-ColorMessage "  • $($dep.Name) - Instale em: $($dep.InstallUrl)" -ForegroundColor Yellow
        }
        Write-ColorMessage "`nPor favor, instale as dependências faltantes antes de continuar." -ForegroundColor Yellow
        exit 1
    }

    Write-ColorMessage "✓ Todas as dependências estão instaladas.`n" -ForegroundColor Green
}

# Função para criar estrutura de diretórios
function Initialize-DirectoryStructure {
    Write-ColorMessage ">>> Criando estrutura de diretórios..." -ForegroundColor Cyan

    $directories = @(
        "apps/host/src",
        "apps/host/src-tauri/src",
        "apps/admin/src",
        "apps/admin/src-tauri/src",
        "packages/ui/src/components",
        "packages/ui/src/lib",
        "packages/config",
        "packages/types/src",
        "server/cmd/api",
        "server/internal",
        "docs/adr",
        "docs/spikes",
        "Releases"
    )

    foreach ($dir in $directories) {
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Force -Path $dir | Out-Null
            Write-ColorMessage "  ✓ Criado: $dir" -ForegroundColor Green
        }
    }

    Write-ColorMessage "✓ Estrutura de diretórios criada.`n" -ForegroundColor Green
}

# Função para criar package.json root
function Initialize-RootPackageJson {
    Write-ColorMessage ">>> Criando package.json root..." -ForegroundColor Cyan

    $packageJson = @{
        name = "prysmview"
        version = "2.0.0"
        private = true
        description = "PrysmView - Software de acesso remoto peer-to-peer (React + Tailwind + Tauri)"
        scripts = @{
            dev = "turbo run dev"
            build = "turbo run build"
            lint = "turbo run lint"
            format = "turbo run format"
            typecheck = "turbo run typecheck"
            test = "turbo run test"
            clean = "turbo run clean"
        }
        devDependencies = @{
            turbo = "^1.11.0"
            typescript = "^5.3.0"
            prettier = "^3.1.0"
            eslint = "^8.56.0"
        }
        workspaces = @(
            "apps/*",
            "packages/*"
        )
    } | ConvertTo-Json -Depth 10

    $packageJson | Out-File -FilePath "package.json" -Encoding UTF8
    Write-ColorMessage "✓ package.json root criado.`n" -ForegroundColor Green
}

# Função para criar pnpm-workspace.yaml
function Initialize-PnpmWorkspace {
    Write-ColorMessage ">>> Criando pnpm-workspace.yaml..." -ForegroundColor Cyan

    $workspaceConfig = @"
packages:
  - 'apps/*'
  - 'packages/*'
"@

    $workspaceConfig | Out-File -FilePath "pnpm-workspace.yaml" -Encoding UTF8
    Write-ColorMessage "✓ pnpm-workspace.yaml criado.`n" -ForegroundColor Green
}

# Função para criar turbo.json
function Initialize-TurboConfig {
    Write-ColorMessage ">>> Criando turbo.json..." -ForegroundColor Cyan

    $turboConfig = @{
        $schema = "https://turbo.build/schema.json"
        globalDependencies = @(
            "**/.env.*local"
        )
        pipeline = @{
            build = @{
                dependsOn = @("^build")
                outputs = @(
                    "dist/**",
                    ".next/**",
                    "!.next/cache/**"
                )
            }
            lint = @{
                dependsOn = @("^build")
            }
            dev = @{
                cache = $false
                persistent = $true
            }
            clean = @{
                cache = $false
            }
        }
    } | ConvertTo-Json -Depth 10

    $turboConfig | Out-File -FilePath "turbo.json" -Encoding UTF8
    Write-ColorMessage "✓ turbo.json criado.`n" -ForegroundColor Green
}

# Função para criar app Host
function Initialize-HostApp {
    Write-ColorMessage ">>> Inicializando app Host..." -ForegroundColor Cyan

    Set-Location -Path "apps\host"

    # Criar package.json do Host
    $hostPackageJson = @{
        name = "prysmview-host"
        version = "2.0.0"
        type = "module"
        scripts = @{
            dev = "vite"
            build = "tsc && vite build"
            preview = "vite preview"
            tauri = "tauri"
            "tauri:dev" = "tauri dev"
            "tauri:build" = "tauri build"
            lint = "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0"
            format = "prettier --write \"src/**/*.{ts,tsx,css}\""
            typecheck = "tsc --noEmit"
            test = "vitest"
        }
        dependencies = @{
            react = "^18.2.0"
            "react-dom" = "^18.2.0"
            "@tauri-apps/api" = "^1.5.0"
            zustand = "^4.4.0"
            "react-router-dom" = "^6.20.0"
            "@radix-ui/react-tabs" = "^1.0.0"
            "@radix-ui/react-dialog" = "^1.0.0"
            "lucide-react" = "^0.300.0"
            clsx = "^2.0.0"
            "tailwind-merge" "^2.0.0"
        }
        devDependencies = @{
            "@tauri-apps/cli" = "^1.5.0"
            "@types/react" = "^18.2.0"
            "@types/react-dom" = "^18.2.0"
            "@vitejs/plugin-react" = "^4.2.0"
            typescript = "^5.3.0"
            vite = "^5.0.0"
            "@typescript-eslint/eslint-plugin" = "^6.0.0"
            "@typescript-eslint/parser" = "^6.0.0"
            eslint = "^8.56.0"
            "eslint-plugin-react-hooks" = "^4.6.0"
            "eslint-plugin-react-refresh" = "^0.4.0"
            prettier = "^3.1.0"
            "tailwindcss" = "^3.4.0"
            "autoprefixer" = "^10.4.0"
            "postcss" = "^8.4.0"
            vitest = "^1.0.0"
            "@testing-library/react" = "^14.0.0"
        }
    } | ConvertTo-Json -Depth 10

    $hostPackageJson | Out-File -FilePath "package.json" -Encoding UTF8

    # Criar vite.config.ts
    $viteConfig = @"
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
"@

    $viteConfig | Out-File -FilePath "vite.config.ts" -Encoding UTF8

    # Criar tsconfig.json
    $tsConfig = @{
        compilerOptions = @{
            target = "ES2020"
            useDefineForClassFields = $true
            lib = @("ES2020", "DOM", "DOM.Iterable")
            module = "ESNext"
            skipLibCheck = $true
            moduleResolution = "bundler"
            allowImportingTsExtensions = $true
            resolveJsonModule = $true
            isolatedModules = $true
            noEmit = $true
            jsx = "react-jsx"
            strict = $true
            noUnusedLocals = $true
            noUnusedParameters = $true
            noFallthroughCasesInSwitch = $true
        }
        include = @("src")
        references = @(@{ path = "../packages/types" })
    } | ConvertTo-Json -Depth 10

    $tsConfig | Out-File -FilePath "tsconfig.json" -Encoding UTF8

    # Criar tailwind.config.js
    $tailwindConfig = @"
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
"@

    $tailwindConfig | Out-File -FilePath "tailwind.config.js" -Encoding UTF8

    # Criar postcss.config.js
    $postcssConfig = @"
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"@

    $postcssConfig | Out-File -FilePath "postcss.config.js" -Encoding UTF8

    # Criar index.html
    $indexHtml = @"
<!DOCTYPE html>
<html lang="pt-BR">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PrysmView Host</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"@

    $indexHtml | Out-File -FilePath "index.html" -Encoding UTF8

    # Criar src/main.tsx
    $mainTsx = @"
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"@

    $mainTsx | Out-File -FilePath "src\main.tsx" -Encoding UTF8

    # Criar src/App.tsx
    $appTsx = @"
import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)

  return (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">PrysmView Host</h1>
        <p className="text-gray-400 mb-4">Interface do Host - React + Tailwind + Tauri</p>
        <button
          onClick={() => setCount((c) => c + 1)}
          className="px-4 py-2 bg-blue-600 rounded hover:bg-blue-700 transition"
        >
          Count is {count}
        </button>
      </div>
    </div>
  )
}

export default App
"@

    $appTsx | Out-File -FilePath "src\App.tsx" -Encoding UTF8

    # Criar src/index.css
    $indexCss = @"
@tailwind base;
@tailwind components;
@tailwind utilities;
"@

    $indexCss | Out-File -FilePath "src\index.css" -Encoding UTF8

    # Inicializar Tauri
    Write-ColorMessage "  Inicializando Tauri..." -ForegroundColor Yellow
    pnpm tauri init --app-name "PrysmView Host" --window-title "PrysmView Host" --dist-dir "../dist"

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ App Host inicializado.`n" -ForegroundColor Green
}

# Função para criar app Admin
function Initialize-AdminApp {
    Write-ColorMessage ">>> Inicializando app Admin..." -ForegroundColor Cyan

    Set-Location -Path "apps\admin"

    # Criar package.json do Admin (similar ao Host)
    $adminPackageJson = @{
        name = "prysmview-admin"
        version = "2.0.0"
        type = "module"
        scripts = @{
            dev = "vite"
            build = "tsc && vite build"
            preview = "vite preview"
            tauri = "tauri"
            "tauri:dev" = "tauri dev"
            "tauri:build" = "tauri build"
            lint = "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0"
            format = "prettier --write \"src/**/*.{ts,tsx,css}\""
            typecheck = "tsc --noEmit"
            test = "vitest"
        }
        dependencies = @{
            react = "^18.2.0"
            "react-dom" = "^18.2.0"
            "@tauri-apps/api" = "^1.5.0"
            zustand = "^4.4.0"
            "react-router-dom" = "^6.20.0"
            "@radix-ui/react-tabs" = "^1.0.0"
            "@radix-ui/react-dialog" = "^1.0.0"
            "lucide-react" = "^0.300.0"
            clsx = "^2.0.0"
            "tailwind-merge" "^2.0.0"
        }
        devDependencies = @{
            "@tauri-apps/cli" = "^1.5.0"
            "@types/react" = "^18.2.0"
            "@types/react-dom" = "^18.2.0"
            "@vitejs/plugin-react" = "^4.2.0"
            typescript = "^5.3.0"
            vite = "^5.0.0"
            "@typescript-eslint/eslint-plugin" = "^6.0.0"
            "@typescript-eslint/parser" = "^6.0.0"
            eslint = "^8.56.0"
            "eslint-plugin-react-hooks" = "^4.6.0"
            "eslint-plugin-react-refresh" = "^0.4.0"
            prettier = "^3.1.0"
            "tailwindcss" = "^3.4.0"
            "autoprefixer" = "^10.4.0"
            "postcss" = "^8.4.0"
            vitest = "^1.0.0"
            "@testing-library/react" = "^14.0.0"
        }
    } | ConvertTo-Json -Depth 10

    $adminPackageJson | Out-File -FilePath "package.json" -Encoding UTF8

    # Criar vite.config.ts (igual ao Host)
    $viteConfig = @"
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1421,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
"@

    $viteConfig | Out-File -FilePath "vite.config.ts" -Encoding UTF8

    # Criar tsconfig.json
    $tsConfig = @{
        compilerOptions = @{
            target = "ES2020"
            useDefineForClassFields = $true
            lib = @("ES2020", "DOM", "DOM.Iterable")
            module = "ESNext"
            skipLibCheck = $true
            moduleResolution = "bundler"
            allowImportingTsExtensions = $true
            resolveJsonModule = $true
            isolatedModules = $true
            noEmit = $true
            jsx = "react-jsx"
            strict = $true
            noUnusedLocals = $true
            noUnusedParameters = $true
            noFallthroughCasesInSwitch = $true
        }
        include = @("src")
        references = @(@{ path = "../packages/types" })
    } | ConvertTo-Json -Depth 10

    $tsConfig | Out-File -FilePath "tsconfig.json" -Encoding UTF8

    # Criar tailwind.config.js
    $tailwindConfig = @"
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
"@

    $tailwindConfig | Out-File -FilePath "tailwind.config.js" -Encoding UTF8

    # Criar postcss.config.js
    $postcssConfig = @"
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"@

    $postcssConfig | Out-File -FilePath "postcss.config.js" -Encoding UTF8

    # Criar index.html
    $indexHtml = @"
<!DOCTYPE html>
<html lang="pt-BR">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>PrysmView Admin</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"@

    $indexHtml | Out-File -FilePath "index.html" -Encoding UTF8

    # Criar src/main.tsx
    $mainTsx = @"
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"@

    $mainTsx | Out-File -FilePath "src\main.tsx" -Encoding UTF8

    # Criar src/App.tsx
    $appTsx = @"
import { useState } from 'react'

function App() {
  const [count, setCount] = useState(0)

  return (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">PrysmView Admin</h1>
        <p className="text-gray-400 mb-4">Painel Admin - React + Tailwind + Tauri</p>
        <button
          onClick={() => setCount((c) => c + 1)}
          className="px-4 py-2 bg-blue-600 rounded hover:bg-blue-700 transition"
        >
          Count is {count}
        </button>
      </div>
    </div>
  )
}

export default App
"@

    $appTsx | Out-File -FilePath "src\App.tsx" -Encoding UTF8

    # Criar src/index.css
    $indexCss = @"
@tailwind base;
@tailwind components;
@tailwind utilities;
"@

    $indexCss | Out-File -FilePath "src\index.css" -Encoding UTF8

    # Inicializar Tauri
    Write-ColorMessage "  Inicializando Tauri..." -ForegroundColor Yellow
    pnpm tauri init --app-name "PrysmView Admin" --window-title "PrysmView Admin" --dist-dir "../dist"

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ App Admin inicializado.`n" -ForegroundColor Green
}

# Função para criar pacote UI compartilhado
function Initialize-UIPackage {
    Write-ColorMessage ">>> Inicializando pacote UI compartilhado..." -ForegroundColor Cyan

    Set-Location -Path "packages\ui"

    $uiPackageJson = @{
        name = "@prysmview/ui"
        version = "2.0.0"
        type = "module"
        main = "./dist/index.js"
        types = "./dist/index.d.ts"
        exports = @{
            "." = @{
                import = "./dist/index.js"
                types = "./dist/index.d.ts"
            }
        }
        scripts = @{
            build = "tsc"
            dev = "tsc --watch"
            lint = "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0"
            format = "prettier --write \"src/**/*.{ts,tsx}\""
        }
        dependencies = @{
            react = "^18.2.0"
            "react-dom" = "^18.2.0"
            clsx = "^2.0.0"
            "tailwind-merge" = "^2.0.0"
            "@radix-ui/react-tabs" = "^1.0.0"
            "@radix-ui/react-dialog" = "^1.0.0"
            "lucide-react" = "^0.300.0"
        }
        devDependencies = @{
            "@types/react" = "^18.2.0"
            "@types/react-dom" = "^18.2.0"
            typescript = "^5.3.0"
            eslint = "^8.56.0"
            prettier = "^3.1.0"
            tailwindcss = "^3.4.0"
        }
        peerDependencies = @{
            react = "^18.2.0"
            "react-dom" = "^18.2.0"
        }
    } | ConvertTo-Json -Depth 10

    $uiPackageJson | Out-File -FilePath "package.json" -Encoding UTF8

    # Criar tsconfig.json
    $tsConfig = @{
        compilerOptions = @{
            target = "ES2020"
            lib = @("ES2020", "DOM", "DOM.Iterable")
            module = "ESNext"
            skipLibCheck = $true
            moduleResolution = "bundler"
            allowImportingTsExtensions = $true
            resolveJsonModule = $true
            isolatedModules = $true
            noEmit = $false
            jsx = "react-jsx"
            strict = $true
            noUnusedLocals = $true
            noUnusedParameters = $true
            noFallthroughCasesInSwitch = $true
            declaration = $true
            outDir = "./dist"
        }
        include = @("src")
        exclude = @("node_modules", "dist")
    } | ConvertTo-Json -Depth 10

    $tsConfig | Out-File -FilePath "tsconfig.json" -Encoding UTF8

    # Criar src/index.ts
    $indexTs = @"
export * from './components/Button'
export * from './components/Input'
export * from './components/Tabs'
export * from './components/Modal'
"@

    $indexTs | Out-File -FilePath "src\index.ts" -Encoding UTF8

    # Criar src/components/Button.tsx (placeholder)
    $buttonTsx = @"
import React from 'react'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  loading?: boolean
}

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function Button({ variant = 'primary', loading, disabled, className, children, ...props }: ButtonProps) {
  const baseStyles = 'px-4 py-2 rounded font-medium transition focus:outline-none focus:ring-2 focus:ring-offset-2'
  const variants = {
    primary: 'bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500',
    secondary: 'bg-gray-600 text-white hover:bg-gray-700 focus:ring-gray-500',
    ghost: 'bg-transparent text-gray-300 hover:bg-gray-800 focus:ring-gray-500',
    danger: 'bg-red-600 text-white hover:bg-red-700 focus:ring-red-500',
  }

  return (
    <button
      className={cn(baseStyles, variants[variant], disabled && 'opacity-50 cursor-not-allowed', className)}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? 'Loading...' : children}
    </button>
  )
}
"@

    $buttonTsx | Out-File -FilePath "src\components\Button.tsx" -Encoding UTF8

    # Criar outros componentes placeholders
    "src\components\Input.tsx", "src\components\Tabs.tsx", "src\components\Modal.tsx" | ForEach-Object {
        $content = @"
import React from 'react'

export default function Placeholder() {
  return <div>Component placeholder</div>
}
"@
        $content | Out-File -FilePath $_ -Encoding UTF8
    }

    Set-Location -Path "..\.."
    Write-ColorMessage "✓ Pacote UI compartilhado inicializado.`n" -ForegroundColor Green
}

# Função para instalar dependências
function Install-AllDependencies {
    Write-ColorMessage ">>> Instalando dependências (isso pode levar alguns minutos)..." -ForegroundColor Cyan

    pnpm install

    if ($LASTEXITCODE -ne 0) {
        Write-ColorMessage "❌ Falha ao instalar dependências" -ForegroundColor Red
        exit 1
    }

    Write-ColorMessage "✓ Dependências instaladas.`n" -ForegroundColor Green
}

# Função principal
function Setup-PrysmView {
    $StartTime = Get-Date

    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-ColorMessage "  PRYSMVIEW SETUP SCRIPT v2.0.0" -ForegroundColor Cyan
    Write-ColorMessage "  React + Tailwind + Tauri" -ForegroundColor Cyan
    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-ColorMessage ""

    # Verificar dependências
    Test-Dependencies

    # Criar estrutura
    Initialize-DirectoryStructure

    # Criar configurações root
    Initialize-RootPackageJson
    Initialize-PnpmWorkspace
    Initialize-TurboConfig

    # Inicializar apps
    Initialize-HostApp
    Initialize-AdminApp

    # Inicializar pacotes
    Initialize-UIPackage

    # Instalar dependências
    Install-AllDependencies

    # Git init (opcional)
    if (-not $SkipGitInit) {
        Write-ColorMessage ">>> Inicializando repositório Git..." -ForegroundColor Cyan
        git init
        git add .
        git commit -m "Initial commit: PrysmView v2.0.0 setup"
        Write-ColorMessage "✓ Repositório Git inicializado.`n" -ForegroundColor Green
    }

    # Resumo
    $EndTime = Get-Date
    $Duration = $EndTime - $StartTime

    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Green
    Write-ColorMessage "  SETUP CONCLUÍDO COM SUCESSO!" -ForegroundColor Green
    Write-ColorMessage "═══════════════════════════════════════════════════════════════" -ForegroundColor Green
    Write-ColorMessage ""
    Write-ColorMessage "Tempo total: $($Duration.ToString('mm\:ss'))" -ForegroundColor Cyan
    Write-ColorMessage ""
    Write-ColorMessage "Próximos passos:" -ForegroundColor Cyan
    Write-ColorMessage "  1. Executar o Host em modo dev:  pnpm --filter prysmview-host dev" -ForegroundColor White
    Write-ColorMessage "  2. Executar o Admin em modo dev:  pnpm --filter prysmview-admin dev" -ForegroundColor White
    Write-ColorMessage "  3. Executar ambos em modo dev:   pnpm dev" -ForegroundColor White
    Write-ColorMessage "  4. Build de produção:            pnpm build" -ForegroundColor White
    Write-ColorMessage "  5. Build de instaladores:       .\build_prysmview.ps1" -ForegroundColor White
    Write-ColorMessage ""
    Write-ColorMessage "Documentação: AGENTS.md" -ForegroundColor Yellow
    Write-ColorMessage ""
}

# Executar setup
try {
    Setup-PrysmView
}
catch {
    Write-ColorMessage "`n❌ ERRO FATAL: $_" -ForegroundColor Red
    Write-ColorMessage "Stack trace: $($_.ScriptStackTrace)" -ForegroundColor Red
    exit 1
}
