package db

import (
	"testing"
	"time"
)

func setupTestDB(t *testing.T) *DB {
	database, err := Open(":memory:")
	if err != nil {
		t.Fatalf("falha ao abrir banco em memoria: %v", err)
	}
	return database
}

func TestPDVRegistrationAndLookup(t *testing.T) {
	database := setupTestDB(t)
	defer database.Close()

	pdv := PDV{
		PublicKey:    "ed25519-dummy-pubkey-001",
		StoreName:    "Loja Central",
		POSNumber:    1,
		AgentVersion: "0.1.0",
		Status:       "online",
		LastSeen:     time.Now().Truncate(time.Second),
	}

	if err := database.RegisterPDV(pdv); err != nil {
		t.Fatalf("erro ao registrar PDV: %v", err)
	}

	got, err := database.GetPDV(pdv.PublicKey)
	if err != nil {
		t.Fatalf("erro ao buscar PDV: %v", err)
	}
	if got == nil || got.StoreName != pdv.StoreName || got.POSNumber != 1 {
		t.Fatalf("dados do PDV divergentes: %+v", got)
	}

	// Atualiza status do PDV existente
	pdv.Status = "in_session"
	if err := database.RegisterPDV(pdv); err != nil {
		t.Fatalf("erro ao atualizar PDV: %v", err)
	}
	gotUpdated, _ := database.GetPDV(pdv.PublicKey)
	if gotUpdated.Status != "in_session" {
		t.Fatalf("status nao atualizado: %s", gotUpdated.Status)
	}
}

func TestOperatorPermissionsAndListing(t *testing.T) {
	database := setupTestDB(t)
	defer database.Close()

	op := Operator{
		ID:           "op-100",
		Name:         "Tecnico Carlos",
		PasswordHash: "dummy-hash",
		CreatedAt:    time.Now(),
	}
	if err := database.CreateOperator(op); err != nil {
		t.Fatalf("erro ao criar operador: %v", err)
	}

	pdv1 := PDV{PublicKey: "key-1", StoreName: "Loja A", POSNumber: 1, AgentVersion: "0.1.0", Status: "online", LastSeen: time.Now()}
	pdv2 := PDV{PublicKey: "key-2", StoreName: "Loja B", POSNumber: 2, AgentVersion: "0.1.0", Status: "online", LastSeen: time.Now()}
	_ = database.RegisterPDV(pdv1)
	_ = database.RegisterPDV(pdv2)

	// Concede acesso apenas ao PDV1
	if err := database.GrantPermission(op.ID, pdv1.PublicKey); err != nil {
		t.Fatalf("erro ao conceder permissao: %v", err)
	}

	hasPerm1, _ := database.CheckPermission(op.ID, pdv1.PublicKey)
	if !hasPerm1 {
		t.Errorf("deveria ter permissao para pdv1")
	}

	hasPerm2, _ := database.CheckPermission(op.ID, pdv2.PublicKey)
	if hasPerm2 {
		t.Errorf("nao deveria ter permissao para pdv2")
	}

	pdvs, err := database.ListPDVsForOperator(op.ID)
	if err != nil {
		t.Fatalf("erro ao listar PDVs do operador: %v", err)
	}
	if len(pdvs) != 1 || pdvs[0].PublicKey != pdv1.PublicKey {
		t.Fatalf("lista de PDVs incorreta: %+v", pdvs)
	}
}

func TestTokenValidationAndExpiry(t *testing.T) {
	database := setupTestDB(t)
	defer database.Close()

	op := Operator{ID: "op-200", Name: "Suporte", PasswordHash: "hash", CreatedAt: time.Now()}
	_ = database.CreateOperator(op)

	// Token ativo válido por 8 horas
	validHash := "valid-token-sha256-hash"
	if err := database.SaveToken(validHash, op.ID, time.Now().Add(8*time.Hour)); err != nil {
		t.Fatalf("erro ao salvar token: %v", err)
	}

	opID, err := database.ValidateToken(validHash)
	if err != nil || opID != op.ID {
		t.Fatalf("token valido foi rejeitado: %v", err)
	}

	// Token já expirado
	expiredHash := "expired-token-sha256-hash"
	if err := database.SaveToken(expiredHash, op.ID, time.Now().Add(-1*time.Minute)); err != nil {
		t.Fatalf("erro ao salvar token expirado: %v", err)
	}

	_, errExp := database.ValidateToken(expiredHash)
	if errExp == nil {
		t.Fatalf("token expirado deveria ser rejeitado")
	}
}

func TestAuditLogPersistence(t *testing.T) {
	database := setupTestDB(t)
	defer database.Close()

	audit := AuditRecord{
		OperatorID:       "op-100",
		PDVPublicKey:     "pdv-pub-01",
		StartTime:        time.Now().Add(-10 * time.Minute),
		EndTime:          time.Now(),
		Reason:           "Verificacao de impressora fiscal",
		ConsentResult:    "aceito",
		BytesTransferred: 1048576,
	}

	if err := database.LogAudit(audit); err != nil {
		t.Fatalf("erro ao gravar auditoria: %v", err)
	}
}
