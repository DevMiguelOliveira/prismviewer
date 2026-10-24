# Mecanismo de Atualização Assinada e Rollback — PrysmView

Em conformidade com a **Fase 11** e as Seções 6.4, 9 (Requisito 13) e 10 do `AGENTS.md`.

---

## 1. Modelo de Segurança e Confiança

Em ambientes de PDV varejista, uma atualização corrompida ou maliciosa pode interromper a operação comercial de uma loja ou permitir execução remota de código não autorizado. Por este motivo:

1. **Assinatura Ed25519 Obrigatória**: Toda atualização distribuída exige assinatura criptográfica Ed25519 válida gerada pelo time oficial do projeto. O Host recusa e descarta imediatamente pacotes com assinatura ausente, adulterada ou inválida.
2. **Chave Pública Embutida**: A chave pública mestre de verificação Ed25519 é embutida estaticamente no código do `prysmview-svc`.
3. **Validação de Hash SHA-256**: O digest SHA-256 de 32 bytes do arquivo baixado é verificado com comparação em tempo constante (`subtle::ConstantTimeEq`) para mitigar ataques de temporização (*timing attacks*).
4. **Sem Execução como Administrador Não-Assistida**: A substituição de binários é efetuada exclusivamente pelo serviço `prysmview-svc` sob `LocalSystem`, sem expor caixas de diálogo no PDV que possam interromper o operador de caixa.

---

## 2. Canais de Distribuição

O PrysmView suporta dois canais de distribuição:

- **`stable`**: Canal padrão de produção. Atualizações só são disponibilizadas após aprovação completa e testes em ambiente real.
- **`beta`**: Canal de testes preliminares para homologação antecipada em lojas piloto.

---

## 3. Estrutura do Manifesto (`UpdateManifest`)

O manifesto de atualização é disponibilizado em formato binário ou JSON assinado:

```rust
pub struct UpdateManifest {
    pub version: String,
    pub channel: UpdateChannel,
    pub package_url: String,
    pub package_sha256: [u8; 32],
    pub signature_ed25519: Vec<u8>,
}
```

A assinatura digital Ed25519 de 64 bytes é calculada exatamente sobre o digest `package_sha256` de 32 bytes.

---

## 4. Algoritmo de Rollback Automático (`UpdateBackupGuard`)

Para garantir disponibilidade contínua mesmo se um novo binário falhar ao inicializar (por exemplo, incompatibilidade de DLL ou corrupção de sistema):

1. **Backup Pré-Substituição**:
   O executável atual (`prysmview-svc.exe`) é atômica e seguramente renomeado para `prysmview-svc.exe.bak`.
2. **Gravação da Nova Versão**:
   O novo binário verificado criptograficamente é gravado no caminho principal.
3. **Healthcheck e Commit**:
   O `UpdateBackupGuard` monitora o ciclo de vida da nova versão.
   - **Sucesso**: O método `guard.commit()` é invocado e o arquivo de backup `.bak` é removido.
   - **Falha / Crash**: Caso o serviço sofra *panic*, encerramento prematuro ou não atinja o estado `ServiceState::Running`, o guardião executa o `drop()`, que automaticamente exclui o binário corrompido e restaura o executável original `.bak`.

---

## 5. Procedimento de Lançamento de Release

1. Criar tag Git correspondente à versão:
   ```powershell
   git tag v0.2.0
   git push origin v0.2.0
   ```
2. O workflow do GitHub Actions `.github/workflows/release.yml` compilará os binários em modo release, gerará o instalador via Inno Setup 6, publicará os hashes `SHA256SUMS.txt` e publicará o release no GitHub.
