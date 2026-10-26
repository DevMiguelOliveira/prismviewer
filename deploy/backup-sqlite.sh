#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Rotina de Backup Cifrado do Banco SQLite (O-01)
# Criptografia moderna com 'age', sem chave privada na VM de produção.
# ==============================================================================
set -euo pipefail

DB_PATH="${DB_PATH:-/var/lib/prysmview/prysmview.db}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/prysmview}"
AGE_PUBKEY_FILE="${AGE_PUBKEY_FILE:-/etc/prysmview/backup.pub}"
RETENTION_DAYS=30
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
TEMP_DB="${BACKUP_DIR}/temp_${TIMESTAMP}.db"
BACKUP_FILE="${BACKUP_DIR}/prysmview_${TIMESTAMP}.db.age"

# 1. Checagem de dependencias
if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "Erro fatal: 'sqlite3' não encontrado. Instale com: apt-get install -y sqlite3" >&2
    exit 1
fi

if ! command -v age >/dev/null 2>&1; then
    echo "Erro fatal: 'age' não encontrado. Instale com: apt-get install -y age" >&2
    exit 1
fi

if [[ ! -f "${DB_PATH}" ]]; then
    echo "Erro: Banco de dados SQLite não encontrado em ${DB_PATH}" >&2
    exit 1
fi

if [[ ! -f "${AGE_PUBKEY_FILE}" ]]; then
    echo "Erro: Chave pública do age não encontrada em ${AGE_PUBKEY_FILE}" >&2
    echo "Gere com: age-keygen -o /root/.age/backup.key && age-keygen -y /root/.age/backup.key > /etc/prysmview/backup.pub" >&2
    exit 1
fi

mkdir -p "${BACKUP_DIR}"
chmod 0700 "${BACKUP_DIR}"

echo "[$(date)] Iniciando backup consistente do SQLite (modo .backup)..."
# O comando .backup do sqlite3 garante consistência mesmo com o modo WAL ativo
sqlite3 "${DB_PATH}" ".backup '${TEMP_DB}'"

echo "[$(date)] Criptografando cópia binária com 'age'..."
age -r "$(cat "${AGE_PUBKEY_FILE}")" -o "${BACKUP_FILE}" "${TEMP_DB}"
rm -f "${TEMP_DB}"
chmod 0600 "${BACKUP_FILE}"

echo "[$(date)] Backup cifrado gerado com sucesso: ${BACKUP_FILE}"

# 2. Envio para fora da VM (se hook de sincronização remota estiver configurado)
if [[ -n "${BACKUP_REMOTE_CMD:-}" ]]; then
    echo "[$(date)] Executando envio para fora da VM..."
    eval "${BACKUP_REMOTE_CMD} '${BACKUP_FILE}'"
fi

# 3. Rotação de 30 dias na VM
echo "[$(date)] Aplicando política de retenção (${RETENTION_DAYS} dias)..."
find "${BACKUP_DIR}" -type f -name "prysmview_*.db.age" -mtime +${RETENTION_DAYS} -delete
echo "[$(date)] Rotação concluída."
