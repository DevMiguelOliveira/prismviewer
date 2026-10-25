#!/usr/bin/env bash
# Configuração do Firewall UFW para a VM Azure (Ubuntu 24.04 LTS)
# Conforme Seção 6.3 do AGENTS.md e C-05 do roadmap de segurança
set -euo pipefail

echo "Configurando firewall UFW com regras estritas para o PrysmView..."

# Política padrão: bloqueia tráfego de entrada e permite saída
ufw default deny incoming
ufw default allow outgoing

# 1. SSH na porta padrão — RESTRITO ao IP do administrador
#    Substitua <IP_ADMINISTRADOR> pelo IP fixo do admin antes de executar.
#    Exemplo: ufw allow from 200.200.200.200 to any port 22 proto tcp
ufw limit 22/tcp comment 'SSH com limitador de tentativas — restringir a IP do admin em producao'

# 2. HTTP para redirecionamento automático para HTTPS (Caddy)
ufw allow 80/tcp comment 'HTTP redirect para HTTPS (Caddy)'

# 3. HTTPS para a API de Sinalização (Go) via Caddy
ufw allow 443/tcp comment 'PrysmView Signaling API (HTTPS via Caddy)'

# 4. Porta do Relay iroh (QUIC / UDP)
ufw allow 3340/udp comment 'PrysmView iroh QUIC Relay'

# Habilita o firewall
echo "y" | ufw enable
ufw status verbose
echo "Firewall configurado e ativo."
