#!/usr/bin/env bash
# Configuração do Firewall UFW para a VM Oracle Cloud Always Free (Ubuntu 24.04 LTS)
# Conforme Seção 6.3 do AGENTS.md
set -euo pipefail

echo "Configurando firewall UFW com regras estritas para o PrysmView..."

# Política padrão: bloqueia tráfego de entrada e permite saída
ufw default deny incoming
ufw default allow outgoing

# 1. SSH na porta padrão limitado contra ataques de força bruta
ufw limit 22/tcp comment 'SSH com limitador de tentativas'

# 2. HTTPS para a API de Sinalização (Go)
ufw allow 443/tcp comment 'PrysmView Signaling API (HTTPS)'

# 3. Porta do Relay iroh (QUIC / UDP)
ufw allow 3340/udp comment 'PrysmView iroh QUIC Relay'

# Habilita o firewall
echo "y" | ufw enable
ufw status verbose
echo "Firewall configurado e ativo."
