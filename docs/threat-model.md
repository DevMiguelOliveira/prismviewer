# Threat Model — PrysmView (Modelo de Ameaças em PDV)

## 1. Premissas de Confiança e Fronteiras de Segurança
- **Ambiente de Rede do PDV:** Rede local de loja é considerada **hostil**. Pode haver Wi-Fi compartilhado com clientes, roteadores mal configurados ou switches desprotegidos. Qualquer pacote pode ser interceptado, adulterado ou reenviado (replay).
- **Servidor de Sinalização:** Pode ser comprometido externamente. Por isso, a sinalização **nunca recebe** nem descriptografa tráfego de sessão (tela, teclas, clipboard ou arquivos). Ele lida estritamente com metadados de presença e troca de endereços.
- **Relay QUIC (iroh-relay):** O relay é cego: trafega apenas pacotes QUIC cifrados ponta a ponta com TLS 1.3. O relay não possui as chaves de sessão.
- **Credenciais de Operador:** Podem vazar. Por isso a autenticação exige múltiplos fatores: token de API, consentimento explícito na tela do PDV e código de sessão temporário de 8 dígitos.
- **Acesso Físico:** Atacante com acesso físico ao hardware do PDV está fora de escopo.

---

## 2. Ativos Críticos
1. **Chave Privada Ed25519 do PDV:** Identidade criptográfica do caixa.
2. **Dados em Trânsito:** Quadros de vídeo da tela do PDV, digitação do operador, arquivos e área de transferência.
3. **Controle de Entrada do PDV:** Capacidade de injetar cliques de mouse e teclas no sistema do caixa.
4. **Logs de Auditoria:** Histórico imutável de quem acessou, quando e por que motivo.

---

## 3. Ameaças e Vetores de Ataque

| Vetor de Ataque | Risco / Impacto | Mitigação no PrysmView |
| :--- | :--- | :--- |
| **Man-in-the-Middle (MITM)** | Interceptação de dados e injeção de comandos maliciosos | Toda comunicação P2P e relay usa QUIC sobre TLS 1.3 autenticado pela chave pública Ed25519 do peer. |
| **Replay Attack na Sinalização** | Reenvio de solicitações de conexão antigas | Requisições autenticadas por assinatura Ed25519 contendo nonce aleatório de uso único e timestamp de validade curta. |
| **Acesso Não Autorizado ao Caixa** | Operador conecta em caixa de loja sem autorização | Consentimento obrigatório e visível na tela com timeout de 60s + código de sessão de 8 dígitos com HMAC-SHA256 em tempo constante. |
| **Força Bruta no Código de Sessão** | Adivinhação de código de 8 dígitos | Código expira em 120s e é bloqueado permanentemente após 3 tentativas inválidas. |
| **Força Bruta na API de Sinalização** | Tentativas repetidas de autenticação ou requisições forjadas | Bloqueio progressivo de IP: 5 falhas consecutivas bloqueiam o IP de origem por 15 minutos. |
| **Vazamento da Chave do PDV em Disco** | Clonagem de identidade do caixa | Chave privada cifrada via DPAPI do Windows com escopo de máquina (`CryptProtectData`). Chave em memória limpa com `zeroize`. |
| **Exfiltração de Arquivos / Malware** | Envio de binários corrompidos | Todo bloco de arquivo é verificado contra hash SHA-256 no destino. Falha de hash exclui o arquivo temporário imediatamente. |
| **Teclas Presas após Desconexão** | Teclas de atalho maliciosas ficarem pressionadas | Ao término de sessão (normal ou abrupto), o host solta compulsoriamente todas as teclas e modificadores via SendInput. |

---

## 4. Requisitos Inegociáveis de Auditoria
- Os logs de auditoria registram: ID do operador, ID do PDV, data/hora de início e fim, motivo declarado, status do consentimento e total de bytes transferidos.
- **PROIBIDO EM LOGS:** Jamais registrar imagens de tela, texto digitado, senhas, códigos de sessão ou conteúdo de arquivos.

---

## 5. Administrador Local (Cenário do Franqueado — S-10)

### Controles Implementados:
1. **ACL do Diretório e Arquivos:** Permissões NTFS em `C:\Program Files\PrysmView` e `C:\ProgramData\PrysmView` restritas a `SYSTEM` e `Administrators` (Full Control); usuários comuns possuem apenas Read/Execute.
2. **Proteção da Chave Privada do PDV:** A chave privada Ed25519 é protegida por DPAPI no escopo de máquina (`CryptProtectData`) e guardada em arquivo com ACL restrita. A chave em memória é zerada via `ZeroizeOnDrop`.
3. **Recuperação Automática do Serviço:** O serviço `prysmview-svc` é configurado no SCM com recuperação automática em caso de falha (`Restart=on-failure`, com reinicialização em 5s, 10s e 60s).
4. **Detecção Central de Adulteração e Parada:** A API de sinalização monitora a cadência de heartbeats dos hosts. Se uma máquina interromper seus heartbeats por mais de 90 segundos sem encerramento gracioso, o status do PDV passa a `offline` e um alerta de desconexão anômala é emitido para a equipe de suporte.

### Limitações Técnicas Inevitáveis (Fronteira de Segurança):
- **Privilégios de Administrador Local:** Como o franqueado possui direitos de Administrador Local na máquina e o dispositivo não faz parte de um domínio Active Directory gerenciado centralmente, um usuário com privilégios de Administrador pode:
  - Parar o serviço Windows via `sc stop prysmview-svc` ou desativá-lo via `sc config start=disabled`.
  - Bloquear tráfego de rede no Firewall do Windows.
  - Finalizar processos do sistema ou desinstalar o programa.
- **O que o software garante:** O software garante que **nenhuma chave privada é exfiltrada** (ela é protegida por DPAPI), que **nenhuma sessão é estabelecida sem ticket assinado válido** e que qualquer adulteração ou tentativa de parada é **imediatamente detectada e auditada pelo servidor de sinalização** através da ausência do heartbeat assinado.

---

## 6. Metadados do Relay e Proteção de Tráfego

- **Isolamento de Criptografia:** O `prysmview-relay` (iroh-relay) opera em camada de transporte cega (QUIC / TLS 1.3). Ele apenas comuta datagramas criptografados ponto a ponta com as chaves efêmeras derivadas no handshake QUIC entre Host e Viewer.
- **Vazamento de Metadados:** Um atacante com controle sobre o relay poderia observar:
  - IPs de origem e destino;
  - Horários e volume agregado de bytes transferidos.
- **Mitigação:** 
  1. O relay não armazena logs de payload nem inspecta cabeçalhos internos.
  2. A telemetria expõe apenas contadores atômicos de bytes e conexões ativas (`RelayMetrics`) para monitoramento de custos, sem associar endereços IP a identidades de lojas nos logs persistidos.
  3. Tentativas de injeção de pacotes forjados são descartadas pela camada de autenticação criptográfica do QUIC/TLS 1.3.

---

## 7. Canal de Atualização e Proteção Anti-Rollback (S-01, S-02, S-03)

- **Ameaça:** Atacante intercepta o tráfego de atualização para injetar um executável malicioso ou rebaixar a versão do host para uma versão anterior vulnerável (ataque de rollback).
- **Mitigações Implementadas:**
  1. **Manifesto Assinado Digitalmente:** O manifesto de atualização (`version`, `channel`, `min_version`, `sha256`, `valid_until`) é assinado inteiramente com chave Ed25519 privada mantida offline.
  2. **Validação Estrita:** O host rejeita manifesto com data expirada (`valid_until`), canal divergente (ex: tentar aplicar build de teste em canal estável) ou assinatura inválida.
  3. **Guarda Anti-Rollback Local:** O host persiste localmente em `min_version.dat` a versão mínima aceita, protegida por ACL. O manifesto é rejeitado se tentar instalar versão inferior à versão em execução.
  4. **Rollback Automático:** Caso a nova versão sofra crash ou falhe na inicialização, o guardião reverte automaticamente para a versão anterior preservada.

---

## 8. Higiene de Segredos em CI/CD e Armazenamento (C-01, C-02, S-17)

- **Ameaça:** Chaves de API, credenciais de banco ou chaves privadas vazarem em repositórios Git públicos ou pastas sincronizadas com nuvem (ex: Google Drive).
- **Mitigações Implementadas:**
  1. **Varredura Automatizada:** Execução obrigatória do `gitleaks` em modo pre-commit e em todos os workflows de CI do GitHub Actions.
  2. **Bloqueio no `.gitignore`:** Exclusão sistemática de extensões sensíveis: `*.db`, `*.sqlite*`, `*.pem`, `*.key`, `.env*`, `secrets/` e `*.bak`.
  3. **Isolamento de Diretórios de Sincronização:** Pastas de build (`target/`, `dist/`), pastas de controle (`.git/`, `.dual-agent/`) e ambientes virtuais são estritamente proibidos em diretórios de sincronização compartilhada.
  4. **GitHub Environments Protegidos:** As chaves de assinatura de release ficam armazenadas exclusivamente em segredos de ambientes protegidos do GitHub que exigem aprovação manual de engenheiro sênior antes da publicação.

---

## 9. Integração com Microsoft Entra ID e Governança de Técnicos (S-08, S-09)

- **Ameaça:** Técnico desligado da empresa ou com credencial vazada tentar acessar PDVs da rede de franquias fora do horário ou sem autorização.
- **Mitigações Implementadas:**
  1. **Autenticação OIDC + PKCE:** O técnico autentica-se com suas credenciais corporativas no Microsoft 365, com MFA compulsório gerenciado pelas políticas de Acesso Condicional do tenant.
  2. **Restrição por Grupos de Segurança:** Apenas membros do grupo corporativo `prysmview-technicians` conseguem emitir tokens de sinalização na API.
  3. **Revogação Instantânea:** Ao remover o colaborador do grupo no Entra ID ou desativar sua conta, os tokens expiram e o servidor recusa a emissão de novos tickets imediatamente.
  4. **Auditoria com UPN:** O log de auditoria vincula cada sessão ao e-mail institucional exato do técnico, garantindo não-repúdio.

---

## 10. Cenário de Franquias e Limites de Confiança entre Lojas

- **Ameaça:** Uma franquia vizinha tentar acessar os caixas de outra loja ou visualizar dados confidenciais de faturamento e vendas.
- **Mitigações Implementadas:**
  1. **Isolamento Lógico Multitenancy:** O banco de dados vincula cada máquina à sua loja (`store_id`) e franquia correspondente.
  2. **Restrição de Acesso por Ticket:** O técnico só consegue conectar a um PDV específico se possuir um ticket assinado emitido com a chave pública daquele PDV exato.
  3. **Impossibilidade de Conexão Indiscriminada:** Nenhuma máquina aceita comandos de controle ou vídeo sem sessão previamente autenticada com ticket válido.
  4. **Conformidade LGPD:** As transmissões de tela e teclado ocorrem em canal direto peer-to-peer criptografado; nenhum dado de venda, cartão de crédito ou transação bancária é gravado ou armazenado na infraestrutura central.