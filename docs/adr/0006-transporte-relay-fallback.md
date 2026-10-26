# ADR 0006: Transporte P2P, Relay iroh e Fallback em Redes com UDP Bloqueado (P-06 / P-07)

- **Status:** Aprovado
- **Data:** 2026-10-25
- **Autores:** Arquiteto de Software & Engenheiro de Segurança
- **Decisores:** Proprietário do Projeto & Time de Engenharia PrysmView

---

## 1. Contexto e Desafio de Conectividade

O PrysmView conecta operadores a caixas de lojas (PDVs) situados em redes corporativas variadas, incluindo:
- Redes locais com roteadores comuns (UPnP/NAT simples).
- Redes de shopping centers e grandes varejistas com **CGNAT simétrico duplo**.
- Redes corporativas e bancárias restritas com políticas estritas de firewall que **bloqueiam 100% de tráfego UDP de saída**, permitindo apenas conexões TCP nas portas 80 e 443.

O objetivo do PrysmView é garantir **100% de taxa de sucesso de conexão** sem comprometer a segurança E2E (criptografia fim a fim TLS 1.3), o custo (R$ 0,00) ou a confidencialidade do tráfego.

---

## 2. Arquitetura de Transporte em Três Níveis

A pilha de transporte do PrysmView opera em modelo de cascata automática:

```
[Início da Sessão]
       │
       ▼
 1. Conexão Direta P2P (QUIC / UDP)
    - Atravessamento via STUN nativo do iroh
    - Sucesso esperado: 70% a 85% dos casos
    - Latência: RTT direto (sem intermediários)
       │
       ▼ (Falhou ou CGNAT simétrico)
 2. Relay iroh Primário (QUIC / UDP 3340)
    - Servidor relay na VM Azure / Oracle
    - Pacotes QUIC tunelados sem decifragem
    - Latência: RTT via servidor (< 20 ms em território nacional)
       │
       ▼ (UDP bloqueado pelo firewall da loja)
 3. Fallback Corporativo (HTTPS / WebSocket sobre TCP 443)
    - Conexão tunelada TLS via proxy reverso Caddy na porta 443 TCP
    - Passa por qualquer firewall ou proxy corporativo que permita navegação web
    - Garante que nenhuma loja fique inacessível
```

---

## 3. Telemetria e Monitoramento de Saída do Relay (P-07)

1. **Privacidade Absoluta:** O relay opera em nível de transporte QUIC TLS 1.3. O relay **nunca** possui as chaves de sessão e não pode decifrar tela, áudio, teclas ou arquivos.
2. **Contador de Bytes Atômico (`RelayMetrics`):**
   - O binário `prysmview-relay` mantém contadores atômicos em memória (`AtomicU64`) registrando o total de bytes transferidos e a quantidade de sessões ativas.
   - Um threshold de consumo configurável dispara alertas de saturação ou risco de estouro de cota da nuvem, conforme implementado em [`crates/relay/src/main.rs`](../../crates/relay/src/main.rs).
3. **Registro de Relação P2P vs Relay:**
   - A API de sinalização e os logs de auditoria registram a proporção de conexões diretas versus via relay, permitindo avaliar a saúde da rede de lojas.

---

## 4. Decisão

1. **Adotar a arquitetura de cascata tripla:** P2P direto -> Relay UDP 3340 -> Fallback TCP 443.
2. **Configuração nos Scripts de Deploy:** O script `deploy/azure-setup.sh` e o firewall `setup-ufw.sh` mantêm abertas a porta 3340 (UDP) para o relay e 443 (TCP) para a API e fallback HTTP.
3. **Monitoramento:** Métricas de bytes repassados expostas e monitoradas para cumprimento do orçamento.
