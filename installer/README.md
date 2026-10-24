# Instalador Windows — PrysmView (Inno Setup 6)

Em conformidade com a decisão arquitetural **ADR 0002** e a **Fase 10** da especificação `AGENTS.md`.

## 1. Visão Geral

O instalador do PrysmView empacota e configura os componentes necessários para a operação em caixas de loja (PDVs) rodando Windows 10 e Windows 11:

1. **`prysmview-svc.exe`**: Serviço Windows em `LocalSystem` com inicialização automática (`SERVICE_AUTO_START`) e recuperação automática em falha (`Restart=on-failure`).
2. **`prysmview-tray.exe`**: Aplicativo da bandeja do sistema (sem elevação de privilégios) executado na sessão interativa do usuário para gerenciamento de consentimento e encerramento de sessão.
3. **Diretório seguro de dados**: `C:\ProgramData\PrysmView` configurado com permissões restritas (apenas `SYSTEM` e Administradores possuem permissão de gravação).

---

## 2. Como Gerar o Instalador

### Pré-requisitos
- Compilador Rust configurado (`cargo`).
- **Inno Setup 6** instalado (disponível gratuitamente em [jrsoftware.org](https://jrsoftware.org/isdl.php)).

### Passo a passo
1. Compile os binários do Host e da Bandeja em modo de otimização `release`:
   ```powershell
   cargo build --release -p prysmview-host --bin prysmview-svc
   cargo build --release -p prysmview-tray
   ```

2. Compile o pacote instalador com o utilitário de linha de comando `iscc`:
   ```powershell
   & "C:\Program Files (x86)\Inno Setup 6\iscc.exe" installer\prysmview.iss
   ```

3. O arquivo final compilado será gerado em:
   `dist\PrysmView-Setup-0.1.0.exe` (tamanho esperado abaixo de 15 MB, cumprindo o teto de 30 MB).

---

## 3. Instalação e Desinstalação Automatizada (PDVs)

Para implantação em massa em parques de PDVs sem intervenção humana:

### Instalação Silenciosa
```powershell
.\PrysmView-Setup-0.1.0.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

### Desinstalação Silenciosa
```powershell
& "C:\Program Files\PrysmView\unins000.exe" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```
A desinstalação para o serviço Windows, remove o registro do SCM, finaliza o processo da bandeja e limpa os arquivos temporários do diretório de dados de forma limpa.
