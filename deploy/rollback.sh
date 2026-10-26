#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Script de Rollback Atômico em Produção (O-04)
# Reverte a versão ativa ('current') para a versão anterior ('previous')
# e reinicia os serviços de forma imediata.
# ==============================================================================
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
   echo "Erro: Este script deve ser executado como root (use: sudo bash deploy/rollback.sh)" >&2
   exit 1
fi

APP_DIR="/opt/prysmview"
CURRENT_LINK="${APP_DIR}/current"
PREVIOUS_LINK="${APP_DIR}/previous"

if [[ ! -L "${PREVIOUS_LINK}" || ! -d "${PREVIOUS_LINK}" ]]; then
    echo "Erro: Nenhuma versão anterior encontrada em ${PREVIOUS_LINK} para efetuar rollback." >&2
    exit 1
fi

TARGET_VERSION=$(readlink -f "${PREVIOUS_LINK}")
CURRENT_VERSION=$(readlink -f "${CURRENT_LINK}")

echo "[$(date)] Iniciando rollback da versão atual (${CURRENT_VERSION}) para a anterior (${TARGET_VERSION})..."

# Troca atômica do symlink
ln -sfn "${TARGET_VERSION}" "${CURRENT_LINK}"

echo "[$(date)] Reiniciando serviços systemd..."
systemctl restart prysmview-api
systemctl restart prysmview-relay

echo "[$(date)] Verificando saúde dos serviços pós-rollback..."
sleep 1
if systemctl is-active --quiet prysmview-api && systemctl is-active --quiet prysmview-relay; then
    echo "[$(date)] Rollback concluído com sucesso! Versão ativa agora: ${TARGET_VERSION}"
else
    echo "[$(date)] ALERTA: Um ou mais serviços falharam ao reiniciar pós-rollback!" >&2
    systemctl status prysmview-api --no-pager || true
    systemctl status prysmview-relay --no-pager || true
    exit 2
fi
