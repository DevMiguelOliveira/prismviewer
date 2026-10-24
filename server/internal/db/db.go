package db

import (
	"database/sql"
	"fmt"
	"time"

	_ "modernc.org/sqlite"
)

type DB struct {
	conn *sql.DB
}

type PDV struct {
	PublicKey    string    `json:"public_key"`
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
	PDVPublicKey     string    `json:"pdv_public_key"`
	StartTime        time.Time `json:"start_time"`
	EndTime          time.Time `json:"end_time"`
	Reason           string    `json:"reason"`
	ConsentResult    string    `json:"consent_result"`
	BytesTransferred int64     `json:"bytes_transferred"`
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
		pdv_public_key TEXT NOT NULL,
		start_time DATETIME NOT NULL,
		end_time DATETIME NOT NULL,
		reason TEXT NOT NULL,
		consent_result TEXT NOT NULL,
		bytes_transferred INTEGER NOT NULL
	);
	`
	_, err := d.conn.Exec(schema)
	return err
}

func (d *DB) RegisterPDV(p PDV) error {
	query := `
	INSERT INTO pdvs (public_key, store_name, pos_number, agent_version, status, last_seen)
	VALUES (?, ?, ?, ?, ?, ?)
	ON CONFLICT(public_key) DO UPDATE SET
		store_name = excluded.store_name,
		pos_number = excluded.pos_number,
		agent_version = excluded.agent_version,
		status = excluded.status,
		last_seen = excluded.last_seen;
	`
	_, err := d.conn.Exec(query, p.PublicKey, p.StoreName, p.POSNumber, p.AgentVersion, p.Status, p.LastSeen)
	return err
}

func (d *DB) GetPDV(pubKey string) (*PDV, error) {
	row := d.conn.QueryRow("SELECT public_key, store_name, pos_number, agent_version, status, last_seen FROM pdvs WHERE public_key = ?", pubKey)
	var p PDV
	err := row.Scan(&p.PublicKey, &p.StoreName, &p.POSNumber, &p.AgentVersion, &p.Status, &p.LastSeen)
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
	_, err := d.conn.Exec("INSERT INTO tokens (token_hash, operator_id, expires_at) VALUES (?, ?, ?)",
		tokenHash, operatorID, expiresAt)
	return err
}

func (d *DB) ValidateToken(tokenHash string) (string, error) {
	var operatorID string
	var expiresAt time.Time
	err := d.conn.QueryRow("SELECT operator_id, expires_at FROM tokens WHERE token_hash = ?", tokenHash).
		Scan(&operatorID, &expiresAt)
	if err == sql.ErrNoRows {
		return "", fmt.Errorf("token inexistente")
	}
	if err != nil {
		return "", err
	}
	if time.Now().After(expiresAt) {
		return "", fmt.Errorf("token expirado")
	}
	return operatorID, nil
}

func (d *DB) LogAudit(r AuditRecord) error {
	query := `
	INSERT INTO auditoria (operator_id, pdv_public_key, start_time, end_time, reason, consent_result, bytes_transferred)
	VALUES (?, ?, ?, ?, ?, ?, ?);
	`
	_, err := d.conn.Exec(query, r.OperatorID, r.PDVPublicKey, r.StartTime, r.EndTime, r.Reason, r.ConsentResult, r.BytesTransferred)
	return err
}
