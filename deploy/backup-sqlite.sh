#!/usr/bin/env bash
# Script de backup diário do banco SQLite do PrysmView com rotação de 30 dias
# Conforme Seção 6.3 do AGENTS.md (Custo Zero)
set -euo pipefail

DB_PATH="/var/lib/prysmview/prysmview.db"
BACKUP_DIR="/var/backups/prysmview"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="${BACKUP_DIR}/prysmview_${TIMESTAMP}.sql.gz"
RETENTION_DAYS=30

mkdir -p "${BACKUP_DIR}"

if [[ ! -f "${DB_PATH}" ]]; then
    echo "Erro: Banco de dados SQLite não encontrado em ${DB_PATH}" >&2
    exit 1
fi

echo "[$(date)] Iniciando backup do banco SQLite..."
sqlite3 "${DB_PATH}" ".backup '${BACKUP_DIR}/temp.db'"
gzip -c "${BACKUP_DIR}/temp.db" > "${BACKUP_FILE}"
rm -f "${BACKUP_DIR}/temp.db"
chmod 0600 "${BACKUP_FILE}"

echo "[$(date)] Backup concluído com sucesso: ${BACKUP_FILE}"

# Rotação de 30 dias: exclui backups anteriores a 30 dias
find "${BACKUP_DIR}" -type f -name "prysmview_*.sql.gz" -mtime +${RETENTION_DAYS} -delete
echo "[$(date)] Rotação aplicada: backups com mais de ${RETENTION_DAYS} dias foram removidos."
