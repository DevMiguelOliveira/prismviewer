# Plano de Piloto Controlado em Lojas Franqueadas — PrysmView (D-07)

Este documento define os critérios, escopo e metas de saída do programa de piloto controlado para implantação do PrysmView em lojas franqueadas com administradores locais, em substituição ao TeamViewer.

---

## 1. Contexto e Desafio Operacional

- **Perfil dos Terminais:** 1.000+ caixas de loja rodando Windows 10 e Windows 11.
- **Ambiente de Segurança:** As máquinas das franquias estão fora do Active Directory corporativo, e os operadores locais/franqueados possuem privilégios de Administrador Local.
- **Regra de Ouro da Frente de Caixa:** A operação de suporte remoto nunca pode interferir na emissão de cupons fiscais (NFC-e/SAT) nem travar o terminal durante o horário de pico de vendas.

---

## 2. Estratégia de Implantação em Anéis (Rings)

```
[Anel 0: Lab Homologação] ────> 3 PDVs em bancada interna de testes de estresse
                                     │ (Aprovado em 24h)
                                     ▼
[Anel 1: Canary (D-01)]  ────> 5 PDVs em 2 lojas franqueadas voluntárias (48h)
                                     │ (Critérios de saída 100% atingidos)
                                     ▼
[Anel 2: Piloto Ampliado] ────> 50 PDVs em 15 lojas com perfis mistos (7 dias)
                                     │ (Validação de estabilidade em fim de semana)
                                     ▼
[Anel 3: Rollout Geral]   ────> Restante da frota (1.000+ PDVs em lotes de 100)
```

---

## 3. Critérios Obrigatórios de Saída do Piloto (Exit Criteria)

O avanço do Anel 1 para os anéis seguintes está condicionado ao cumprimento de 100% dos seguintes critérios mensuráveis:

| Critério | Meta Estrita | Método de Verificação |
|---|---|---|
| **Disponibilidade do Serviço (`prysmview-svc`)** | **100% sem crashes** | Consulta à telemetria de heartbeats na API e logs do journald |
| **Integridade da Operação Fiscal** | **Zero interferências** | Declaração assinada pelos gerentes das lojas piloto |
| **Taxa de Conexão Bem-Sucedida** | **>= 98.5%** | Auditoria central de sessões na API SQLite |
| **Liberação de Teclas e Modificadores** | **100% (Zero teclas presas)** | Testes de queda forçada de Wi-Fi durante digitação |
| **Latência de Entrada Percebida** | **< 150 ms** | Medição automática HUD do Viewer com RTT de 20 ms |
| **Cadeia de Auditoria Criptográfica** | **100% íntegra** | Execução de `VerifyAuditChain` sem alertas de quebra |
| **Rollback Automático** | **Testado e operacional** | Simulação de falha de inicialização com reversão < 2s |

---

## 4. Plano de Rollback e Contingência de Emergência

Caso ocorra qualquer instabilidade crítica em um terminal piloto durante a operação da loja:
1. O caixa pode encerrar a sessão imediatamente pelo botão vermelho persistente do `PrysmHost`.
2. O script `Uninstall-PrysmHost.ps1` é disparado remotamente, removendo o serviço em menos de 10 segundos e restaurando o estado original sem necessidade de reinicialização do Windows.
