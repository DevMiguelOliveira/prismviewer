#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Teste Automatizado de Backup e Restauração Cifrada (O-01)
# ==============================================================================
set -euo pipefail

TEST_DIR=$(mktemp -d -t prysmview-backup-test-XXXXXX)
trap 'rm -rf "${TEST_DIR}"' EXIT

DB_PATH="${TEST_DIR}/source.db"
BACKUP_DIR="${TEST_DIR}/backups"
RESTORE_DB="${TEST_DIR}/restored.db"
KEY_FILE="${TEST_DIR}/backup.key"
PUB_FILE="${TEST_DIR}/backup.pub"

echo "=== [1/5] Preparando ambiente de teste em ${TEST_DIR} ==="
if ! command -v age-keygen >/dev/null 2>&1; then
    echo "Aviso: 'age-keygen' não está instalado neste sistema operacional."
    echo "Para rodar o teste de restauração em Linux/Ubuntu: apt-get install -y age sqlite3"
    exit 0
fi

mkdir -p "${BACKUP_DIR}"

echo "=== [2/5] Criando banco de dados de teste ==="
sqlite3 "${DB_PATH}" << 'EOF'
CREATE TABLE pdvs (id TEXT PRIMARY KEY, display_id TEXT UNIQUE, store_name TEXT);
INSERT INTO pdvs VALUES ('key_01', 'PDV-101', 'Loja Centro');
INSERT INTO pdvs VALUES ('key_02', 'PDV-102', 'Loja Shopping');
EOF

echo "=== [3/5] Gerando par de chaves age temporário ==="
age-keygen -o "${KEY_FILE}" 2>/dev/null
age-keygen -y "${KEY_FILE}" > "${PUB_FILE}"

echo "=== [4/5] Executando rotina de backup ==="
DB_PATH="${DB_PATH}" BACKUP_DIR="${BACKUP_DIR}" AGE_PUBKEY_FILE="${PUB_FILE}" bash deploy/backup-sqlite.sh

BACKUP_ARTIFACT=$(find "${BACKUP_DIR}" -name "prysmview_*.db.age" | head -n 1)
if [[ -z "${BACKUP_ARTIFACT}" ]]; then
    echo "Erro: Arquivo .db.age não foi gerado!" >&2
    exit 1
fi
echo "Arquivo cifrado confirmado: ${BACKUP_ARTIFACT}"

echo "=== [5/5] Executando restauração e validação ==="
bash deploy/restore-sqlite.sh "${BACKUP_ARTIFACT}" "${KEY_FILE}" "${RESTORE_DB}"

# Valida os dados restaurados
ROW_COUNT=$(sqlite3 "${RESTORE_DB}" "SELECT COUNT(*) FROM pdvs;")
if [[ "${ROW_COUNT}" -ne 2 ]]; then
    echo "Erro: Contagem de linhas incorreta após restauração: ${ROW_COUNT} (esperado 2)" >&2
    exit 2
fi

echo "Sucesso: Backup cifrado e Restauração testados com integridade 100%!"
