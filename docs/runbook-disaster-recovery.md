# Runbook: Recuperação de Desastre (Disaster Recovery) — PrysmView (O-02)

Este documento descreve o procedimento operacional padrão para restauração completa dos serviços centrais do PrysmView (API de Sinalização e Relay iroh) em caso de falha catastrófica, corrupção de dados ou perda total da VM.

---

## 1. Métricas de Resiliência (Metas Mensuradas)

| Métrica | Definição | Valor Homologado |
|---|---|---|
| **RPO (Recovery Point Objective)** | Perda máxima tolerável de dados transacionais | **24 horas** (backups diários às 03:00 UTC) ou **1 hora** se configurado cron horário |
| **RTO (Recovery Time Objective)** | Tempo total mensurado para retorno à operação | **< 10 minutos** a partir da disponibilização da nova VM |

---

## 2. Dependências para Recuperação

- Nova VM Linux (Ubuntu 24.04 LTS x86_64 ou ARM64).
- Chave privada de descriptografia `age` mantida em cofre seguro fora da VM (`/root/.age/backup.key`).
- Último arquivo de backup `.db.age` obtido do armazenamento remoto (repositório Git privado ou bucket de salvaguarda).
- Binários de release assinados do `prysmview-api` e `prysmview-relay`.

---

## 3. Passo a Passo da Restauração

### Passo 1: Instalação das dependências mínimas
Na nova VM limpa:
```bash
sudo apt-get update -y
sudo apt-get install -y sqlite3 age curl
```

### Passo 2: Criação do usuário e diretórios de serviço
```bash
sudo useradd -r -s /usr/sbin/nologin -d /var/lib/prysmview prysmview || true
sudo mkdir -p /var/lib/prysmview /var/backups/prysmview /etc/prysmview /root/.age
sudo chown -R prysmview:prysmview /var/lib/prysmview
sudo chmod 0700 /var/lib/prysmview /var/backups/prysmview
```

### Passo 3: Posicionamento dos binários de produção
Copie os binários estáticos compilados para `/usr/local/bin/`:
```bash
sudo chmod 0755 /usr/local/bin/prysmview-api /usr/local/bin/prysmview-relay
```

### Passo 4: Instalação dos serviços systemd
```bash
sudo cp deploy/systemd/prysmview-api.service /etc/systemd/system/
sudo cp deploy/systemd/prysmview-relay.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### Passo 5: Restauração e Validação do Banco de Dados
Disponibilize o arquivo de backup `.db.age` e a chave privada do `age`:
```bash
# Executa a descriptografia atômica com validação PRAGMA integrity_check
sudo bash deploy/restore-sqlite.sh \
    /var/backups/prysmview/prysmview_ULTIMO.db.age \
    /root/.age/backup.key \
    /var/lib/prysmview/prysmview.db
```

### Passo 6: Inicialização dos Serviços
```bash
sudo systemctl enable --now prysmview-api prysmview-relay
```

### Passo 7: Verificação de Saúde
```bash
curl -f http://127.0.0.1:8080/health
systemctl status prysmview-api prysmview-relay --no-pager
```

---

## 4. Testes Periódicos de Restauração

Conforme o requisito **O-01**, o procedimento de restauração deve ser validado mensalmente em ambiente isolado via:
```bash
bash deploy/test-backup-restore.sh
```
Resultado esperado: `Sucesso: Backup cifrado e Restauração testados com integridade 100%!`.
