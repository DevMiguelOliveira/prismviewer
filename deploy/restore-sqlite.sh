#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Script de Restauração de Backup SQLite (O-01 / O-02)
# Descriptografa arquivo .db.age com age e valida integridade com PRAGMA integrity_check
# ==============================================================================
set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Uso: $0 <caminho_do_arquivo.db.age> <caminho_chave_privada_age> [caminho_destino_db]"
    echo "Exemplo: $0 /var/backups/prysmview/prysmview_20261025.db.age /root/.age/backup.key /var/lib/prysmview/prysmview.db"
    exit 1
fi

BACKUP_FILE="$1"
KEY_FILE="$2"
TARGET_DB="${3:-/var/lib/prysmview/prysmview.db}"

if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "Erro: 'sqlite3' não instalado." >&2
    exit 1
fi

if ! command -v age >/dev/null 2>&1; then
    echo "Erro: 'age' não instalado." >&2
    exit 1
fi

if [[ ! -f "${BACKUP_FILE}" ]]; then
    echo "Erro: Arquivo de backup não encontrado: ${BACKUP_FILE}" >&2
    exit 1
fi

if [[ ! -f "${KEY_FILE}" ]]; then
    echo "Erro: Chave privada não encontrada: ${KEY_FILE}" >&2
    exit 1
fi

TEMP_RESTORE="${TARGET_DB}.restoring.$$"

echo "[$(date)] Descriptografando ${BACKUP_FILE}..."
age -d -i "${KEY_FILE}" -o "${TEMP_RESTORE}" "${BACKUP_FILE}"

echo "[$(date)] Validando integridade física do SQLite..."
CHECK_RESULT=$(sqlite3 "${TEMP_RESTORE}" "PRAGMA integrity_check;")
if [[ "${CHECK_RESULT}" != "ok" ]]; then
    echo "Erro fatal: Banco de dados corrompido! Resultado do integrity_check: ${CHECK_RESULT}" >&2
    rm -f "${TEMP_RESTORE}"
    exit 2
fi

echo "[$(date)] Integridade aprovada. Aplicando substituição atômica no banco oficial..."
TARGET_DIR=$(dirname "${TARGET_DB}")
mkdir -p "${TARGET_DIR}"

# Backup de salvaguarda do atual caso exista
if [[ -f "${TARGET_DB}" ]]; then
    cp "${TARGET_DB}" "${TARGET_DB}.pre-restore.bak"
fi

mv "${TEMP_RESTORE}" "${TARGET_DB}"
chmod 0600 "${TARGET_DB}"
if id -u prysmview >/dev/null 2>&1; then
    chown prysmview:prysmview "${TARGET_DB}"
fi

echo "[$(date)] Restauração concluída com sucesso em ${TARGET_DB}!"
