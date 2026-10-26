#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Monitor de Queda Abrupta de Heartbeats (O-03)
# Compara a contagem atual de hosts ativos com a amostra anterior e alerta se
# a queda for superior ao limite percentual (padrão: 50%).
# ==============================================================================
set -euo pipefail

HEALTH_URL="${HEALTH_URL:-http://127.0.0.1:8080/health}"
STATE_FILE="${STATE_FILE:-/tmp/prysmview_last_heartbeat_count.txt}"
DROP_THRESHOLD_PCT="${DROP_THRESHOLD_PCT:-50}"
ALERT_WEBHOOK_URL="${ALERT_WEBHOOK_URL:-}"

# 1. Consulta o endpoint de saude
RESPONSE=$(curl -sf "${HEALTH_URL}" || echo "")
if [[ -z "${RESPONSE}" ]]; then
    echo "[$(date)] ALERTA CRÍTICO: Falha ao consultar endpoint de saúde em ${HEALTH_URL}" >&2
    exit 2
fi

# Extrai o total de hosts ativos nos ultimos 5 minutos
CURRENT_ACTIVE=$(echo "${RESPONSE}" | grep -o '"active_hosts_5m":[0-9]*' | cut -d':' -f2 || echo "0")
if [[ -z "${CURRENT_ACTIVE}" ]]; then
    CURRENT_ACTIVE=0
fi

echo "[$(date)] Total atual de hosts com heartbeat recente: ${CURRENT_ACTIVE}"

# 2. Se houver histórico anterior, avalia se houve queda abrupta
if [[ -f "${STATE_FILE}" ]]; then
    PREV_ACTIVE=$(cat "${STATE_FILE}")
    if [[ "${PREV_ACTIVE}" -gt 10 ]]; then
        DROP_DIFF=$(( PREV_ACTIVE - CURRENT_ACTIVE ))
        if [[ "${DROP_DIFF}" -gt 0 ]]; then
            DROP_PCT=$(( (DROP_DIFF * 100) / PREV_ACTIVE ))
            if [[ "${DROP_PCT}" -ge "${DROP_THRESHOLD_PCT}" ]]; then
                MSG="ALERTA: Queda abrupta de heartbeats detectada no PrysmView! Anterior: ${PREV_ACTIVE}, Atual: ${CURRENT_ACTIVE} (Queda de ${DROP_PCT}% >= ${DROP_THRESHOLD_PCT}%)"
                echo "[$(date)] ${MSG}" >&2
                
                # Se houver webhook de alerta configurado (Slack/Discord/Teams/Pager)
                if [[ -n "${ALERT_WEBHOOK_URL}" ]]; then
                    curl -s -X POST -H "Content-Type: application/json" \
                        -d "{\"text\":\"${MSG}\"}" "${ALERT_WEBHOOK_URL}" || true
                fi
                echo "${CURRENT_ACTIVE}" > "${STATE_FILE}"
                exit 1
            fi
        fi
    fi
fi

# Atualiza a amostra anterior
echo "${CURRENT_ACTIVE}" > "${STATE_FILE}"
echo "[$(date)] Monitoramento de heartbeats normal."
