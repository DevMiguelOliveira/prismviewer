# Guia de Confiabilidade: Windows Defender & SmartScreen

Este documento descreve todas as medidas implementadas no PrysmView para garantir que o sistema operacional Windows 10 e Windows 11 reconheça o software como legítimo, seguro e auditado, eliminando bloqueios por falso positivo.

---

## 1. O que foi implementado no Software

### A. Recursos Nativos e Metadados PE (VERSIONINFO)
Executáveis sem metadados são penalizados pelos motores heurísticos de antivírus. Todos os executáveis (`PrysmAdmin.exe`, `prysmview-svc.exe`, `PrysmHost.exe`) e os instaladores agora possuem blocos completos de `VERSIONINFO`:
- **Empresa:** `PrysmView Security Solutions`
- **Descrição do Arquivo:** Identificação clara da função do executável
- **Versão do Produto:** `0.1.0.0`
- **Copyright:** `Copyright (C) 2026 PrysmView Team`
- **Idioma:** Português (Brasil) e Inglês

### B. Manifesto de Aplicação Windows 10/11 (`app.manifest`)
Embutido diretamente na seção de recursos do executável (`RT_MANIFEST = 24`):
- Declaração explícita de compatibilidade com **Windows 10 e Windows 11** (`{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}`)
- Nível de privilégio seguro: `<requestedExecutionLevel level="asInvoker" uiAccess="false" />`
- Suporte a alta resolução: `PerMonitorV2` DPI Awareness
- Suporte a caminhos longos: `longPathAware`

### C. Assinatura Digital Authenticode (SHA-256)
- Os binários e instaladores são assinados digitalmente com certificado Authenticode corporativo (`PrysmView Security Solutions`).
- O instalador embute e registra automaticamente o certificado público em `Editores Confiáveis` (`TrustedPublisher`) do Windows através do `certutil.exe`.
- Um manifesto de integridade criptográfica `dist/SHA256SUMS.txt` é gerado para cada release.

### D. Regras de Integridade do Windows Defender nos Instaladores
- O instalador do Host (`prysm-host.iss`), que roda como Administrador, registra automaticamente regras de exclusão no Windows Defender para os processos `prysmview-svc.exe` e `PrysmHost.exe`, impedindo interrupções no PDV.

---

## 2. Scripts Utilitários Disponíveis

### Para o Desenvolvedor / Build Pipeline
```powershell
# Assina todos os binarios em target/release e dist, e gera SHA256SUMS.txt
powershell -ExecutionPolicy Bypass -File .\scripts\sign-binaries.ps1
```

### Para o Técnico / Administrador no Cliente
```powershell
# Instala o certificado nas autoridades da maquina e autoriza no Defender com 1 clique
powershell -ExecutionPolicy Bypass -File .\scripts\trust-prysmview.ps1
```

---

## 3. Como Limpar o SmartScreen Permanentemente na Microsoft (Oficial)

Para qualquer software novo lançado no mercado (mesmo comercial):
1. Acesse o portal oficial da Microsoft: [Microsoft Security Intelligence - File Submission](https://www.microsoft.com/en-us/wdsi/filesubmission).
2. Selecione **Software developer** > **Submit file**.
3. Envie o instalador `PrysmAdmin-Setup-0.1.0.exe` ou `PrysmHost-Setup-0.1.0.exe`.
4. Em "Detection Name", informe: *Incorrectly detected / False positive clean software*.
5. A inteligência em nuvem do Microsoft Defender analisa o binário (geralmente em 1 a 2 horas) e atualiza a reputação na nuvem mundial, liberando o download para todos os usuários do Windows sem nenhum aviso.
