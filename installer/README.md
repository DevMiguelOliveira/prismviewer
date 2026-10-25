# Instaladores Windows — PrysmView (Inno Setup 6)

Em conformidade com a decisão arquitetural **ADR 0002**, a **Fase 10** e a **Fase 18** da especificação `AGENTS.md`.

O PrysmView disponibiliza **dois instaladores totalmente distintos e especializados**:

---

## 1. PrysmView Host Setup (Cliente / Caixa PDV)

- **Arquivo:** `dist/PrysmHost-Setup-0.1.0.exe` (~4.8 MB)
- **Script:** `installer/prysm-host.iss`
- **Destino:** Computadores de caixa e PDVs nas lojas físicas.
- **Componentes instalados:**
  1. **`prysmview-svc.exe`**: Serviço Windows em `LocalSystem` com inicialização automática (`SERVICE_AUTO_START`) e recuperação automática em falha (`Restart=on-failure`).
  2. **`PrysmHost.exe`**: Aplicativo da bandeja do sistema (sem elevação de privilégios) executado na sessão interativa do operador do caixa para exibição do ID, código de 8 dígitos, gerenciamento de consentimento e encerramento de emergência.
  3. **Diretório seguro de dados**: `C:\ProgramData\PrysmView` configurado com permissões restritas (apenas `SYSTEM` e Administradores possuem permissão de gravação).
- **Segurança:** **NÃO contém** e **NÃO instala** nenhum executável de administração ou acesso a outros caixas.

### Instalação Silenciosa em Massa no PDV
```powershell
.\PrysmHost-Setup-0.1.0.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

---

## 2. PrysmView Admin Setup (Técnico / Suporte TI)

- **Arquivo:** `dist/PrysmAdmin-Setup-0.1.0.exe` (~4.6 MB)
- **Script:** `installer/prysm-admin.iss`
- **Destino:** Computadores e notebooks da equipe técnica e central de suporte.
- **Componentes instalados:**
  1. **`PrysmAdmin.exe`**: Console administrativo completo com Sidebar de navegação, 4 KPI cards de telemetria, grade de caixas com dados de hardware/SO/IP, Conexão Rápida direta por ID, agrupamento por lojas, histórico de auditoria e configurações da nuvem Azure.
  2. Atalho criado na Área de Trabalho e no Menu Iniciar: *"PrysmView Admin"*.
- **Segurança:** **NÃO instala** serviço Windows em background nem aplicativo de host.

---

## 3. Como Compilar os Instaladores

```powershell
# 1. Compilar os binários em modo release
cargo build --release -p prysmview-host --bin prysmview-svc
cargo build --release -p prysmview-tray --bin PrysmHost
cargo build --release -p prysmview-viewer --bin PrysmAdmin

# 2. Gerar o Instalador do Host (Cliente / PDV)
& "C:\Users\luism\AppData\Local\Programs\Inno Setup 6\ISCC.exe" installer\prysm-host.iss

# 3. Gerar o Instalador do Admin (Técnico)
& "C:\Users\luism\AppData\Local\Programs\Inno Setup 6\ISCC.exe" installer\prysm-admin.iss
```
