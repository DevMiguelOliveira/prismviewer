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
| **Vazamento da Chave do PDV em Disco** | Clonagem de identidade do caixa | Chave privada cifrada via DPAPI do Windows com escopo de máquina (`CryptProtectData`). Chave em memória limpa com `zeroize`. |
| **Exfiltração de Arquivos / Malware** | Envio de binários corrompidos | Todo bloco de arquivo é verificado contra hash SHA-256 no destino. Falha de hash exclui o arquivo temporário imediatamente. |
| **Teclas Presas após Desconexão** | Teclas de atalho maliciosas ficarem pressionadas | Ao término de sessão (normal ou abrupto), o host solta compulsoriamente todas as teclas e modificadores via SendInput. |

---

## 4. Requisitos Inegociáveis de Auditoria
- Os logs de auditoria registram: ID do operador, ID do PDV, data/hora de início e fim, motivo declarado, status do consentimento e total de bytes transferidos.
- **PROIBIDO EM LOGS:** Jamais registrar imagens de tela, texto digitado, senhas, códigos de sessão ou conteúdo de arquivos.