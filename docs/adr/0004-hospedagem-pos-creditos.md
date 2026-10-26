# ADR 0004: Estratégia de Hospedagem e Custos Pós-Vencimento de Créditos Azure (O-06)

- **Status:** Proposto (DECISÃO DO DONO)
- **Data:** 2026-10-25
- **Autores:** Arquiteto de Software & Engenheiro de Segurança
- **Decisores:** Proprietário do Projeto PrysmView

---

## 1. Contexto e Problema

Os serviços de sinalização (`prysmview-api`), proxy reverso TLS (`Caddy`) e relay P2P (`prysmview-relay`) estão atualmente provisionados em uma VM Microsoft Azure (`Standard D2alds_v7`, 2 vCPUs ARM64, 8 GB RAM, Ubuntu 24.04 LTS), mantida com **créditos gratuitos concedidos pela Microsoft com validade improrrogável até 18/11/2026**.

A premissa basilar do PrysmView (Seção 0 do `AGENTS.md`) é **Custo Zero (R$ 0,00)** ou custo estritamente mínimo de subsistência. Em escala operacional de 1.000+ PDVs, o tráfego de dados é composto por:
1. **API de Sinalização e Heartbeats:** Consumo de rede desprezível (~2 a 5 GB/mês no total para 1.000 máquinas enviando heartbeats a cada 30 segundos).
2. **Relay QUIC iroh (Fallback P2P):** Conexões diretas via hole punching resolvem entre 70% e 85% dos acessos. Para os 15% a 30% restantes que atravessam CGNATs simétricos de lojas e shopping centers, os pacotes de vídeo e controle passam pelo relay. Uma sessão de 15 minutos em 1080p consome entre 25 e 60 MB com compressão zstd. Com estimativa de 50 a 100 chamados diários via relay, a saída de dados varia entre **75 GB e 300 GB/mês**, com pico projetado de até **1 TB/mês**.

Se a VM permanecer na Azure na modalidade *Pay-as-you-go* após 18/11/2026, a fatura mensal estimada será de **R$ 250 a R$ 450/mês** (computação D2alds_v7 + disco SSD + tráfego de saída a ~$0.087/GB), violando a diretriz de custo.

---

## 2. Opções Avaliadas

### Opção 1: Oracle Cloud Always Free (Tier Permanente Gratuito — R$ 0,00)
- **Recursos:** Instância Ampere A1 Compute (até 4 OCPUs ARM64, 24 GB RAM, 200 GB SSD em bloco).
- **Tráfego de Saída (Egress):** **10 TB/mês gratuitos** incluídos todos os meses.
- **Custo:** **R$ 0,00/mês vitalício**.
- **Pontos Fortes:**
  - Capacidade computacional muito superior à atual (4 núcleos ARM64 e 24 GB de RAM suportam tranquilamente milhares de conexões simultâneas do relay).
  - 10 TB de tráfego cobrem 10x o pico estimado de 1.000 PDVs.
  - Compatibilidade binária direta: arquitetura ARM64 idêntica à máquina Azure atual.
- **Riscos e Mitigação:** A Oracle possui política de encerramento de instâncias ociosas em contas gratuitas puras. A mitigação recomendada e documentada é adicionar um cartão de crédito e fazer o upgrade gratuito da conta para *Pay-As-You-Go* sem custos, o que garante imunidade contra exclusão mantendo os recursos dentro da cota gratuita Always Free.

### Opção 2: Hetzner Cloud (VPS Mínima Dedicada — ~R$ 25/mês)
- **Recursos:** CAX11 (ARM64, 2 vCPUs, 4 GB RAM, 40 GB NVMe) ou CX22 (x86_64, 2 vCPUs, 4 GB RAM).
- **Tráfego de Saída:** **20 TB/mês inclusos**.
- **Custo:** ~€3,79 a €4,50/mês (~R$ 25,00 a R$ 30,00/mês).
- **Pontos Fortes:** Preço extremamente baixo com estabilidade e SLA europeu reconhecidos; 20 TB de tráfego.
- **Pontos Fracos:** Não é R$ 0,00; servidores localizados na Alemanha/Finlândia (latência de ~140ms para sinalização, embora para tráfego UDP de relay a latência percebida seja aceitável).

### Opção 3: Permanecer na Azure em Pay-As-You-Go
- **Custo:** R$ 250 a R$ 450/mês.
- **Veredito:** **Rejeitado por incompatibilidade orçamentária.**

---

## 3. Alertas de Orçamento na Azure (Ação Imediata — O-06)

Para proteger a infraestrutura antes do vencimento:
1. No Portal Azure > **Cost Management + Billing** > **Cost Alerts**.
2. Criar regra de orçamento (*Budget Alert*) com limite de R$ 0,01 acima do saldo de créditos.
3. Notificação por e-mail quando o consumo atingir 80%, 90% e 100% dos créditos.

---

## 4. Cronograma de Ação Recomendado

1. **Até 30/09/2026:** Provisionar VM espelho na Oracle Cloud Always Free e executar o script idempotente `deploy/azure-setup.sh`.
2. **De 01/10/2026 a 31/10/2026:** Validar tráfego de homologação na Oracle.
3. **Até 10/11/2026:** Virada de DNS do domínio da API e desligamento da VM Azure antes do vencimento dos créditos.

---

## 5. Decisão do Dono (DECISÃO DO DONO)

Qual das opções deve ser adotada para a hospedagem pós-18/11/2026?
- **(Recomendado) Opção 1:** Oracle Cloud Always Free (R$ 0,00 vitalício, 10 TB tráfego/mês, 4 vCPUs ARM64, 24 GB RAM).
- **Opção 2:** Hetzner Cloud (~€3.79/mês, 20 TB tráfego/mês).
- **Opção 3:** Outro provedor definido pelo proprietário.
