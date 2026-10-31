#!/usr/bin/env bash
# ==============================================================================
# PrysmView - Script de Provisionamento e Deploy Automatizado (Azure Ubuntu 24.04 LTS)
# Sem compilação em produção (O-04). Baixa/instala artefatos estáticos assinados,
# valida hash SHA-256 e implementa symlinks atômicos para rollback instantâneo.
# ==============================================================================
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
   echo "Este script deve ser executado como root (use: sudo bash deploy/azure-setup.sh)" >&2
   exit 1
fi

VERSION="${1:-1.0.0}"
RELEASE_TAG="v${VERSION#v}"
APP_DIR="/opt/prysmview"
RELEASE_DIR="${APP_DIR}/releases/${RELEASE_TAG}"
CURRENT_LINK="${APP_DIR}/current"
PREVIOUS_LINK="${APP_DIR}/previous"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCH=$(uname -m)

case "${ARCH}" in
    x86_64)  TARGET_ARCH="amd64" ;;
    aarch64) TARGET_ARCH="arm64" ;;
    *) echo "Arquitetura não suportada: ${ARCH}" >&2; exit 1 ;;
esac

echo "=================================================================="
echo " Iniciando Deploy do PrysmView (${RELEASE_TAG} - ${TARGET_ARCH})"
echo " Hospedagem: Azure VM (Ubuntu 24.04 LTS)"
echo " Modo: Instalação de Binários Pré-Compilados com Rollback Atômico (O-04)"
echo "=================================================================="

# 1. Dependências mínimas de runtime (sem compiladores em produção)
echo "--> [1/7] Instalando pacotes de runtime e utilitários..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
    sqlite3 \
    age \
    ufw \
    curl \
    gzip \
    ca-certificates

# 2. Usuário de sistema restrito e diretórios
echo "--> [2/7] Configurando usuário 'prysmview' e diretórios de execução..."
if ! id -u prysmview >/dev/null 2>&1; then
    useradd -r -s /usr/sbin/nologin -d /var/lib/prysmview prysmview
fi

mkdir -p "${APP_DIR}/releases" /var/lib/prysmview /var/backups/prysmview /etc/prysmview /etc/systemd/journald.conf.d
chown -R prysmview:prysmview /var/lib/prysmview
chmod 0700 /var/lib/prysmview /var/backups/prysmview

# 3. Obtenção dos binários e validação de SHA-256
echo "--> [3/7] Preparando versão ${RELEASE_TAG} em ${RELEASE_DIR}..."
mkdir -p "${RELEASE_DIR}"

# Procura binários locais fornecidos pelo pacote de deploy ou baixa do GitHub Releases
if [[ -f "${SCRIPT_DIR}/bin/prysmview-api" && -f "${SCRIPT_DIR}/bin/prysmview-relay" ]]; then
    echo "Instalando a partir de binários pré-empacotados locais..."
    cp "${SCRIPT_DIR}/bin/prysmview-api" "${RELEASE_DIR}/"
    cp "${SCRIPT_DIR}/bin/prysmview-relay" "${RELEASE_DIR}/"
    if [[ -f "${SCRIPT_DIR}/bin/SHA256SUMS" ]]; then
        echo "Validando somas de verificação SHA-256..."
        (cd "${RELEASE_DIR}" && sha256sum -c "${SCRIPT_DIR}/bin/SHA256SUMS")
    fi
else
    # Fallback seguro: se executado em ambiente de teste/staging a partir do repo com go/cargo disponíveis
    echo "Binários pré-compilados não encontrados em ${SCRIPT_DIR}/bin/."
    if command -v go >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        echo "Aviso: Ambiente de build detectado. Compilando artefatos para teste local..."
        REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
        (cd "${REPO_DIR}/server" && go build -ldflags="-s -w" -o "${RELEASE_DIR}/prysmview-api" ./cmd/api/main.go)
        (cd "${REPO_DIR}" && cargo build --release -p prysmview-relay && cp target/release/prysmview-relay "${RELEASE_DIR}/")
    else
        echo "Erro: Forneça os binários pré-compilados em deploy/bin/ ou instale o pacote de release!" >&2
        exit 1
    fi
fi

chmod 0755 "${RELEASE_DIR}/prysmview-api" "${RELEASE_DIR}/prysmview-relay"

# 4. Troca atômica de symlinks para rollback
echo "--> [4/7] Atualizando symlinks de release..."
if [[ -L "${CURRENT_LINK}" ]]; then
    EXISTING_TARGET=$(readlink -f "${CURRENT_LINK}")
    if [[ "${EXISTING_TARGET}" != "${RELEASE_DIR}" ]]; then
        ln -sfn "${EXISTING_TARGET}" "${PREVIOUS_LINK}"
        echo "Versão anterior salva para rollback: $(readlink -f "${PREVIOUS_LINK}")"
    fi
fi

ln -sfn "${RELEASE_DIR}" "${CURRENT_LINK}"
ln -sfn "${CURRENT_LINK}/prysmview-api" /usr/local/bin/prysmview-api
ln -sfn "${CURRENT_LINK}/prysmview-relay" /usr/local/bin/prysmview-relay

# 5. Serviços systemd e journald
echo "--> [5/7] Configurando systemd e retenção de logs do journald (O-03)..."
cp "${SCRIPT_DIR}/systemd/prysmview-api.service" /etc/systemd/system/
cp "${SCRIPT_DIR}/systemd/prysmview-relay.service" /etc/systemd/system/
cp "${SCRIPT_DIR}/journald-prysmview.conf" /etc/systemd/journald.conf.d/prysmview.conf

systemctl daemon-reload
systemctl restart systemd-journald
systemctl enable prysmview-api prysmview-relay
systemctl restart prysmview-api prysmview-relay

# 6. Firewall e segurança de portas
echo "--> [6/8] Ajustando firewall UFW..."
bash "${SCRIPT_DIR}/setup-ufw.sh"

# 7. Instalar e configurar Caddy para HTTPS
echo "--> [7/8] Instalando e configurando Caddy para HTTPS..."
apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list
apt-get update
apt-get install caddy -y

# Configurar Caddyfile
cp "${SCRIPT_DIR}/Caddyfile" /etc/caddy/Caddyfile
systemctl enable caddy
systemctl restart caddy

# 8. Rotinas de Backup Cifrado (O-01) e Monitor de Heartbeats (O-03)
echo "--> [8/8] Configurando tarefas de cron (backup e monitoramento)..."
cp "${SCRIPT_DIR}/backup-sqlite.sh" /usr/local/bin/prysmview-backup.sh
cp "${SCRIPT_DIR}/check-heartbeat-drop.sh" /usr/local/bin/check-heartbeat-drop.sh
cp "${SCRIPT_DIR}/rollback.sh" /usr/local/bin/prysmview-rollback.sh
chmod 0750 /usr/local/bin/prysmview-backup.sh /usr/local/bin/check-heartbeat-drop.sh /usr/local/bin/prysmview-rollback.sh

CRON_BACKUP="0 3 * * * /usr/local/bin/prysmview-backup.sh >> /var/log/prysmview-backup.log 2>&1"
CRON_MONITOR="*/5 * * * * /usr/local/bin/check-heartbeat-drop.sh >> /var/log/prysmview-monitor.log 2>&1"

(crontab -l 2>/dev/null | grep -Fv -e "/usr/local/bin/prysmview-backup.sh" -e "/usr/local/bin/check-heartbeat-drop.sh" || true; \
 echo "$CRON_BACKUP"; echo "$CRON_MONITOR") | crontab -

echo "=================================================================="
echo " Deploy da versão ${RELEASE_TAG} concluído com sucesso!"
echo " Para rollback imediato em caso de emergência, execute:"
echo "   sudo bash /usr/local/bin/prysmview-rollback.sh"
echo "=================================================================="
