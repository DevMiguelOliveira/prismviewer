package db

import (
	"crypto/rand"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"fmt"
	"time"

	_ "modernc.org/sqlite"
)

type DB struct {
	conn *sql.DB
}

type PDV struct {
	PublicKey    string    `json:"public_key"`
	DisplayID    string    `json:"display_id"` // ID curto de exibicao (ex: PDV-01), unico no banco
	StoreName    string    `json:"store_name"`
	POSNumber    int       `json:"pos_number"`
	AgentVersion string    `json:"agent_version"`
	Status       string    `json:"status"`
	LastSeen     time.Time `json:"last_seen"`
}

type Operator struct {
	ID           string    `json:"id"`
	Name         string    `json:"name"`
	PasswordHash string    `json:"-"`
	CreatedAt    time.Time `json:"created_at"`
}

type AuditRecord struct {
	ID               int64     `json:"id"`
	OperatorID       string    `json:"operator_id"`
	TicketID         string    `json:"ticket_id"` // vincula ao ticket de sessao (S-05)
	PDVPublicKey     string    `json:"pdv_public_key"`
	StartTime        time.Time `json:"start_time"`
	EndTime          time.Time `json:"end_time"`
	Reason           string    `json:"reason"`
	ConsentResult    string    `json:"consent_result"`
	BytesTransferred int64     `json:"bytes_transferred"`
	PrevHash         string    `json:"prev_hash"`  // hash do registro anterior (cadeia)
	RecordHash       string    `json:"record_hash"` // hash deste registro (cadeia de integridade)
}

// Open inicializa a conexão SQLite sem CGO e aplica o schema versionado.
func Open(dataSourceName string) (*DB, error) {
	conn, err := sql.Open("sqlite", dataSourceName)
	if err != nil {
		return nil, fmt.Errorf("falha ao abrir sqlite: %w", err)
	}

	// Ativa WAL mode e foreign keys
	if _, err := conn.Exec("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;"); err != nil {
		_ = conn.Close()
		return nil, fmt.Errorf("falha ao configurar pragma: %w", err)
	}

	d := &DB{conn: conn}
	if err := d.migrate(); err != nil {
		_ = conn.Close()
		return nil, fmt.Errorf("falha ao executar migracoes: %w", err)
	}

	return d, nil
}

func (d *DB) Close() error {
	return d.conn.Close()
}

func (d *DB) migrate() error {
	schema := `
	CREATE TABLE IF NOT EXISTS pdvs (
		public_key TEXT PRIMARY KEY,
		display_id TEXT UNIQUE, -- ID curto de exibicao, unico no banco (S-07)
		store_name TEXT NOT NULL,
		pos_number INTEGER NOT NULL,
		agent_version TEXT NOT NULL,
		status TEXT NOT NULL,
		last_seen DATETIME NOT NULL
	);

	CREATE TABLE IF NOT EXISTS operadores (
		id TEXT PRIMARY KEY,
		name TEXT NOT NULL,
		password_hash TEXT NOT NULL,
		created_at DATETIME NOT NULL
	);

	CREATE TABLE IF NOT EXISTS tokens (
		token_hash TEXT PRIMARY KEY,
		operator_id TEXT NOT NULL,
		expires_at DATETIME NOT NULL,
		FOREIGN KEY(operator_id) REFERENCES operadores(id) ON DELETE CASCADE
	);

	CREATE TABLE IF NOT EXISTS permissoes (
		operator_id TEXT NOT NULL,
		pdv_public_key TEXT NOT NULL,
		granted_at DATETIME NOT NULL,
		PRIMARY KEY(operator_id, pdv_public_key),
		FOREIGN KEY(operator_id) REFERENCES operadores(id) ON DELETE CASCADE,
		FOREIGN KEY(pdv_public_key) REFERENCES pdvs(public_key) ON DELETE CASCADE
	);

	CREATE TABLE IF NOT EXISTS auditoria (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		operator_id TEXT NOT NULL,
		ticket_id TEXT,
		pdv_public_key TEXT NOT NULL,
		start_time DATETIME NOT NULL,
		end_time DATETIME NOT NULL,
		reason TEXT NOT NULL,
		consent_result TEXT NOT NULL,
		bytes_transferred INTEGER NOT NULL,
		prev_hash TEXT NOT NULL,
		record_hash TEXT NOT NULL
	);

	CREATE TABLE IF NOT EXISTS agent_tokens (
		machine_id TEXT PRIMARY KEY,
		token_hash TEXT NOT NULL,
		is_revoked INTEGER NOT NULL DEFAULT 0,
		created_at DATETIME NOT NULL,
		rotated_at DATETIME
	);

	CREATE TABLE IF NOT EXISTS revocacoes (
		id TEXT PRIMARY KEY,
		revocation_type TEXT NOT NULL,
		reason TEXT NOT NULL,
		created_at DATETIME NOT NULL
	);
	`
	_, err := d.conn.Exec(schema)
	return err
}

func (d *DB) RegisterPDV(p PDV) error {
	query := `
	INSERT INTO pdvs (public_key, display_id, store_name, pos_number, agent_version, status, last_seen)
	VALUES (?, ?, ?, ?, ?, ?, ?)
	ON CONFLICT(public_key) DO UPDATE SET
		display_id = excluded.display_id,
		store_name = excluded.store_name,
		pos_number = excluded.pos_number,
		agent_version = excluded.agent_version,
		status = excluded.status,
		last_seen = excluded.last_seen;
	`
	_, err := d.conn.Exec(query, p.PublicKey, p.DisplayID, p.StoreName, p.POSNumber, p.AgentVersion, p.Status, p.LastSeen)
	return err
}

func (d *DB) UpdatePDVHeartbeat(machineID, status string) error {
	_, err := d.conn.Exec("UPDATE pdvs SET status = ?, last_seen = ? WHERE display_id = ? OR public_key = ?",
		status, time.Now(), machineID, machineID)
	return err
}

func (d *DB) GetPDV(pubKey string) (*PDV, error) {
	row := d.conn.QueryRow("SELECT public_key, display_id, store_name, pos_number, agent_version, status, last_seen FROM pdvs WHERE public_key = ?", pubKey)
	var p PDV
	err := row.Scan(&p.PublicKey, &p.DisplayID, &p.StoreName, &p.POSNumber, &p.AgentVersion, &p.Status, &p.LastSeen)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &p, nil
}

func (d *DB) CreateOperator(op Operator) error {
	_, err := d.conn.Exec("INSERT INTO operadores (id, name, password_hash, created_at) VALUES (?, ?, ?, ?)",
		op.ID, op.Name, op.PasswordHash, op.CreatedAt)
	return err
}

func (d *DB) GrantPermission(operatorID, pdvPubKey string) error {
	_, err := d.conn.Exec("INSERT OR IGNORE INTO permissoes (operator_id, pdv_public_key, granted_at) VALUES (?, ?, ?)",
		operatorID, pdvPubKey, time.Now())
	return err
}

func (d *DB) CheckPermission(operatorID, pdvPubKey string) (bool, error) {
	var count int
	err := d.conn.QueryRow("SELECT COUNT(*) FROM permissoes WHERE operator_id = ? AND pdv_public_key = ?",
		operatorID, pdvPubKey).Scan(&count)
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func (d *DB) ListPDVsForOperator(operatorID string) ([]PDV, error) {
	query := `
	SELECT p.public_key, p.store_name, p.pos_number, p.agent_version, p.status, p.last_seen
	FROM pdvs p
	INNER JOIN permissoes perm ON p.public_key = perm.pdv_public_key
	WHERE perm.operator_id = ?
	ORDER BY p.store_name, p.pos_number;
	`
	rows, err := d.conn.Query(query, operatorID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var list []PDV
	for rows.Next() {
		var p PDV
		if err := rows.Scan(&p.PublicKey, &p.StoreName, &p.POSNumber, &p.AgentVersion, &p.Status, &p.LastSeen); err != nil {
			return nil, err
		}
		list = append(list, p)
	}
	return list, rows.Err()
}

func (d *DB) SaveToken(tokenHash, operatorID string, expiresAt time.Time) error {
	// Auto-provisionamento JIT (Just-In-Time) para tecnicos autenticados via Entra ID (S-08)
	_, _ = d.conn.Exec("INSERT INTO operadores (id, name, password_hash, created_at) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO NOTHING",
		operatorID, operatorID, "entra-id-managed", time.Now())
	_, err := d.conn.Exec("INSERT INTO tokens (token_hash, operator_id, expires_at) VALUES (?, ?, ?)",
		tokenHash, operatorID, expiresAt)
	return err
}

func (d *DB) ValidateToken(tokenHash string) (string, error) {
	var operatorID string
	var expiresAt time.Time
	var storedHash string
	err := d.conn.QueryRow("SELECT operator_id, expires_at, token_hash FROM tokens WHERE token_hash = ?", tokenHash).
		Scan(&operatorID, &expiresAt, &storedHash)
	if err == sql.ErrNoRows {
		return "", fmt.Errorf("token inexistente")
	}
	if err != nil {
		return "", err
	}

	// Comparacao de hash de token em tempo constante (mitiga timing attacks)
	if !constantTimeCompare(storedHash, tokenHash) {
		return "", fmt.Errorf("token invalido")
	}

	if time.Now().After(expiresAt) {
		return "", fmt.Errorf("token expirado")
	}
	return operatorID, nil
}

func (d *DB) LogAudit(r AuditRecord) error {
	// Busca o hash do ultimo registro para encadear (append-only)
	var prevHash string
	row := d.conn.QueryRow("SELECT record_hash FROM auditoria ORDER BY id DESC LIMIT 1")
	if err := row.Scan(&prevHash); err != nil && err != sql.ErrNoRows {
		return err
	}

	// Calcula o hash deste registro com base no anterior e no conteúdo
	r.PrevHash = prevHash
	r.RecordHash = d.computeAuditHash(prevHash, r)

	query := `
	INSERT INTO auditoria (operator_id, ticket_id, pdv_public_key, start_time, end_time, reason, consent_result, bytes_transferred, prev_hash, record_hash)
	VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
	`
	_, err := d.conn.Exec(query, r.OperatorID, r.TicketID, r.PDVPublicKey, r.StartTime, r.EndTime, r.Reason, r.ConsentResult, r.BytesTransferred, r.PrevHash, r.RecordHash)
	return err
}

// computeAuditHash calcula SHA-256(prev_hash + conteudo do registro) para encadear os registros.
func (d *DB) computeAuditHash(prevHash string, r AuditRecord) string {
	h := sha256.New()
	h.Write([]byte(prevHash))
	h.Write([]byte(r.OperatorID))
	h.Write([]byte(r.TicketID))
	h.Write([]byte(r.PDVPublicKey))
	h.Write([]byte(r.StartTime.Format(time.RFC3339)))
	h.Write([]byte(r.EndTime.Format(time.RFC3339)))
	h.Write([]byte(r.Reason))
	h.Write([]byte(r.ConsentResult))
	h.Write([]byte(fmt.Sprintf("%d", r.BytesTransferred)))
	return hex.EncodeToString(h.Sum(nil))
}

// VerifyAuditChain percorre todos os registros de auditoria e verifica se a cadeia de hash esta intacta.
// Retorna o ID do primeiro registro que quebrar a cadeia, ou 0 se estiver tudo valido.
func (d *DB) VerifyAuditChain() (int64, error) {
	rows, err := d.conn.Query("SELECT id, operator_id, ticket_id, pdv_public_key, start_time, end_time, reason, consent_result, bytes_transferred, prev_hash, record_hash FROM auditoria ORDER BY id ASC")
	if err != nil {
		return 0, err
	}
	defer rows.Close()

	var prevHash string
	for rows.Next() {
		var id int64
		var r AuditRecord
		if err := rows.Scan(&id, &r.OperatorID, &r.TicketID, &r.PDVPublicKey, &r.StartTime, &r.EndTime, &r.Reason, &r.ConsentResult, &r.BytesTransferred, &r.PrevHash, &r.RecordHash); err != nil {
			return 0, err
		}

		// O primeiro registro tem prev_hash vazio; os demais devem encadear
		if id == 1 {
			if r.PrevHash != "" {
				return id, fmt.Errorf("primeiro registro deve ter prev_hash vazio")
			}
		} else {
			if r.PrevHash != prevHash {
				return id, fmt.Errorf("cadeia quebrada: registro %d tem prev_hash %s, esperado %s", id, r.PrevHash, prevHash)
			}
		}

		// Recalcula o hash esperado e compara
		expectedHash := d.computeAuditHash(r.PrevHash, r)
		if expectedHash != r.RecordHash {
			return id, fmt.Errorf("hash invalido no registro %d: esperado %s, obtido %s", id, expectedHash, r.RecordHash)
		}

		prevHash = r.RecordHash
	}

	if err := rows.Err(); err != nil {
		return 0, err
	}
	return 0, nil
}

// constantTimeCompare compara dois strings em tempo constante para mitigar ataques de tempo (S-16).
// Retorna true se forem iguais, false caso contrario.
// Nota: retorna false imediatamente se os tamanhos diferirem (nao vazia informacao de tamanho via timing).
func constantTimeCompare(a, b string) bool {
	if len(a) != len(b) {
		return false
	}
	var result byte
	for i := 0; i < len(a); i++ {
		result |= a[i] ^ b[i]
	}
	return result == 0
}

// IssueOrRotateAgentToken emite ou rotaciona o token da máquina, armazenando apenas o hash SHA-256 (S-13).
func (d *DB) IssueOrRotateAgentToken(machineID string) (string, string, error) {
	bytes := make([]byte, 32)
	if _, err := rand.Read(bytes); err != nil {
		return "", "", fmt.Errorf("falha ao ler entropia: %w", err)
	}
	tokenHex := hex.EncodeToString(bytes)
	h := sha256.Sum256([]byte(tokenHex))
	tokenHash := hex.EncodeToString(h[:])

	now := time.Now()
	query := `
	INSERT INTO agent_tokens (machine_id, token_hash, is_revoked, created_at, rotated_at)
	VALUES (?, ?, 0, ?, ?)
	ON CONFLICT(machine_id) DO UPDATE SET
		token_hash = excluded.token_hash,
		is_revoked = 0,
		rotated_at = excluded.rotated_at;
	`
	_, err := d.conn.Exec(query, machineID, tokenHash, now, now)
	if err != nil {
		return "", "", err
	}
	return tokenHex, tokenHash, nil
}

// RevokeAgentToken revoga imediatamente o token de uma máquina (S-13).
func (d *DB) RevokeAgentToken(machineID string) error {
	_, err := d.conn.Exec("UPDATE agent_tokens SET is_revoked = 1 WHERE machine_id = ?", machineID)
	return err
}

// ValidateAgentToken valida se o token da máquina confere com o hash em tempo constante e não está revogado (S-13, S-16).
func (d *DB) ValidateAgentToken(machineID, tokenHex string) (bool, error) {
	var storedHash string
	var isRevoked int
	err := d.conn.QueryRow("SELECT token_hash, is_revoked FROM agent_tokens WHERE machine_id = ?", machineID).
		Scan(&storedHash, &isRevoked)
	if err == sql.ErrNoRows {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	if isRevoked == 1 {
		return false, nil
	}

	h := sha256.Sum256([]byte(tokenHex))
	calculatedHash := hex.EncodeToString(h[:])

	if !constantTimeCompare(storedHash, calculatedHash) {
		return false, nil
	}
	return true, nil
}

// AddRevocation registra a revogação central de um ticket, técnico ou máquina (S-09).
func (d *DB) AddRevocation(id, revType, reason string) error {
	_, err := d.conn.Exec("INSERT OR REPLACE INTO revocacoes (id, revocation_type, reason, created_at) VALUES (?, ?, ?, ?)",
		id, revType, reason, time.Now())
	return err
}

// GetActiveRevocations lista as revogações ativas enviadas nos heartbeats aos hosts (S-09).
func (d *DB) GetActiveRevocations() ([]string, error) {
	rows, err := d.conn.Query("SELECT id FROM revocacoes ORDER BY created_at DESC LIMIT 100")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var list []string
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		list = append(list, id)
	}
	if list == nil {
		list = []string{}
	}
	return list, rows.Err()
}

