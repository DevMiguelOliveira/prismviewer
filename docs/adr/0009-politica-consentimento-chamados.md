# ADR 0009: Política de Consentimento Central por Ticket e Operação de Caixa

- **Status:** Aprovado
- **Data:** 2026-10-25
- **Autores:** Arquiteto de Software & Engenheiro de Segurança PrysmView
- **Contexto:** S-05, S-06, Q-06 do Roadmap Master

---

## 1. Contexto e Problema

No ambiente de frente de caixa de varejo (supermercados, farmácias, franquias), o operador de caixa está atendendo clientes continuamente:
- **Fricção operacional:** Janelas modais solicitando "Aceitar ou Recusar Conexão" paralisam a venda, geram filas no caixa e frequentemente são ignoradas pelo operador por falta de tempo.
- **Risco de senhas estáticas desatendidas:** O modelo antigo do TeamViewer com senhas fixas desatendidas facilitava acessos indevidos sem chamado aberto e sem rastreabilidade.
- **Necessidade de equilíbrio:** O acesso deve ser ágil para suporte durante incidentes, mas sem permitir que qualquer técnico acesse qualquer caixa quando quiser sem autorização do cliente.

---

## 2. Decisão Arquitetural

### 2.1 Consentimento Vinculado a Chamado de Suporte (Ticket de Sessão) — S-05
- O consentimento para acesso remoto deixa de ser um clique síncrono no caixa e passa a ser **concedido pela abertura de chamado formal** (Service Desk / Helpdesk da franquia).
- Quando o chamado é aberto, a API central emite um **Ticket de Sessão Criptográfico** contendo:
  - `ticket_id`: Identificador único do chamado (UUIDv4);
  - `machine_id`: Chave pública Ed25519 do PDV alvo;
  - `technician_id`: Identidade Entra ID do técnico designado;
  - `not_before` e `expires_at`: Janela estrita de validade (máximo 30 minutos);
  - `nonce`: Valor aleatório de 16 bytes para mitigar replay;
  - `signature`: Assinatura digital Ed25519 do servidor.

### 2.2 Conexão sem Interrupção do Caixa com Ticket Válido — S-06
- Quando o `PrysmHost` recebe um ticket válido com assinatura íntegra do servidor central e dentro da janela de validade:
  1. A sessão é iniciada **sem exibir janela modal de consentimento** no PDV.
  2. O ticket é registrado no cache local anti-replay (persistido em disco protegido) para impedir reutilização do mesmo ticket.
  3. O indicador visual persistente de sessão é ativado imediatamente na tela do PDV.

### 2.3 Garantias Inegociáveis de Transparência no PDV
Mesmo sem clique de aceite no início da sessão:
1. **Indicador Visual Obrigatório:** O aplicativo `PrysmHost` mantém um banner ou janela flutuante no topo da tela informando: *"Sessão Remota em Andamento — Técnico: [Nome] (Chamado #[Ticket])"*. O viewer **não tem capacidade técnica** de ocultar esse indicador.
2. **Botão de Encerramento de Emergência:** O operador do caixa possui um botão visível e de destaque em vermelho na tela para **encerrar a sessão imediatamente** com 1 clique caso necessite.
3. **Soltura Completa de Teclas:** Ao encerrar (pelo caixa, pelo técnico ou por queda), o host libera todas as teclas e descarta as chaves de sessão.
4. **Política Configurável por Loja (Exceção):** Caso uma franquia específica exija aceite manual em 100% dos acessos mesmo com chamado, uma política local pode ser ativada na loja para restaurar a janela modal com tempo limite de 60 segundos.

---

## 3. Consequências

### Positivas
- **Agilidade no suporte:** O técnico conecta instantaneamente para corrigir travamentos de TEF, impressora fiscal ou leitor sem depender da atenção do operador do caixa.
- **Rastreabilidade total:** Toda sessão está obrigatoriamente vinculada a um chamado do Helpdesk.
- **Prevenção de espionagem:** Técnicos não conseguem abrir sessões aleatórias em caixas fora do horário de atendimento do chamado.

### Auditoria e Conformidade
- O evento de auditoria registra: `ticket_id`, `technician_id`, `pdv_id`, `inicio`, `fim`, `motivo` e `bytes_transferidos`, com encadeamento em hash SHA-256.
