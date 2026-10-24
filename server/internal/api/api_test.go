package api

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"prysmview/server/internal/auth"
	"prysmview/server/internal/db"
)

func setupTestServer(t *testing.T) (*db.DB, http.Handler) {
	database, err := db.Open(":memory:")
	if err != nil {
		t.Fatalf("falha ao abrir banco em memoria: %v", err)
	}

	tracker := auth.NewNonceTracker()
	limiter := auth.NewIPRateLimiter()

	router := NewRouter(database, tracker, limiter)
	return database, router
}

func TestHealthEndpoint(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	req := httptest.NewRequest("GET", "/health", nil)
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK, obteve %d", w.Code)
	}

	var res map[string]string
	if err := json.Unmarshal(w.Body.Bytes(), &res); err != nil {
		t.Fatalf("resposta invalida: %v", err)
	}
	if res["status"] != "ok" {
		t.Errorf("esperava status ok, obteve %s", res["status"])
	}
}

func TestHostRegister_ValidAndInvalidSignature(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("erro ao gerar par ed25519: %v", err)
	}

	pubHex := hex.EncodeToString(pub)
	payload := []byte(`{"store_name":"Loja Centro","pos_number":1,"agent_version":"1.0.0","status":"online"}`)
	now := time.Now().Unix()
	nonce := "nonce-reg-1"

	msg := []byte(fmt.Sprintf("%d:%s:%s", now, nonce, payload))
	sig := ed25519.Sign(priv, msg)
	sigHex := hex.EncodeToString(sig)

	// 1. Requisicao valida
	req := httptest.NewRequest("POST", "/api/v1/host/register", bytes.NewReader(payload))
	req.Header.Set("X-Pdv-Public-Key", pubHex)
	req.Header.Set("X-Timestamp", fmt.Sprintf("%d", now))
	req.Header.Set("X-Nonce", nonce)
	req.Header.Set("X-Signature", sigHex)

	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK para registro valido, obteve %d: %s", w.Code, w.Body.String())
	}

	// 2. Consulta de status
	statusReq := httptest.NewRequest("GET", "/api/v1/host/status?public_key="+pubHex, nil)
	statusW := httptest.NewRecorder()
	router.ServeHTTP(statusW, statusReq)

	if statusW.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK ao consultar status, obteve %d", statusW.Code)
	}

	// 3. Requisicao com assinatura forjada/invalida
	fakeSig := hex.EncodeToString(make([]byte, 64))
	fakeReq := httptest.NewRequest("POST", "/api/v1/host/register", bytes.NewReader(payload))
	fakeReq.Header.Set("X-Pdv-Public-Key", pubHex)
	fakeReq.Header.Set("X-Timestamp", fmt.Sprintf("%d", now))
	fakeReq.Header.Set("X-Nonce", "nonce-reg-2")
	fakeReq.Header.Set("X-Signature", fakeSig)

	fakeW := httptest.NewRecorder()
	router.ServeHTTP(fakeW, fakeReq)

	if fakeW.Code != http.StatusUnauthorized {
		t.Fatalf("esperava 401 Unauthorized para assinatura forjada, obteve %d", fakeW.Code)
	}
}

func TestOperatorLoginAndPermissionFlow(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	// Cria operador e PDV
	op := db.Operator{
		ID:        "op-tecnico-1",
		Name:      "Carlos Tecnico",
		CreatedAt: time.Now(),
	}
	_ = database.CreateOperator(op)

	pdvPubKey := "pubkey-pdv-matriz-01"
	_ = database.RegisterPDV(db.PDV{
		PublicKey:    pdvPubKey,
		StoreName:    "Matriz",
		POSNumber:    1,
		AgentVersion: "1.0.0",
		Status:       "online",
		LastSeen:     time.Now(),
	})

	// 1. Login de operador
	loginPayload := []byte(`{"operator_id":"op-tecnico-1","password":"mock"}`)
	loginReq := httptest.NewRequest("POST", "/api/v1/operator/login", bytes.NewReader(loginPayload))
	loginW := httptest.NewRecorder()
	router.ServeHTTP(loginW, loginReq)

	if loginW.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK no login, obteve %d: %s", loginW.Code, loginW.Body.String())
	}

	var loginRes LoginResponse
	_ = json.Unmarshal(loginW.Body.Bytes(), &loginRes)
	token := loginRes.Token
	if len(token) != 64 {
		t.Fatalf("token opaco invalido retornado: %s", token)
	}

	// 2. Tentar acessar PDV antes de permissão concedida -> 403 Forbidden
	sessionReqPayload := []byte(fmt.Sprintf(`{"pdv_public_key":"%s","reason":"suporte emergencial"}`, pdvPubKey))
	sessReq := httptest.NewRequest("POST", "/api/v1/session/request", bytes.NewReader(sessionReqPayload))
	sessReq.Header.Set("Authorization", "Bearer "+token)
	sessW := httptest.NewRecorder()
	router.ServeHTTP(sessW, sessReq)

	if sessW.Code != http.StatusForbidden {
		t.Fatalf("esperava 403 Forbidden antes de conceder permissao, obteve %d", sessW.Code)
	}

	// 3. Concede permissao e tenta novamente -> 200 OK
	_ = database.GrantPermission("op-tecnico-1", pdvPubKey)

	sessReq2 := httptest.NewRequest("POST", "/api/v1/session/request", bytes.NewReader(sessionReqPayload))
	sessReq2.Header.Set("Authorization", "Bearer "+token)
	sessW2 := httptest.NewRecorder()
	router.ServeHTTP(sessW2, sessReq2)

	if sessW2.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK apos permissao concedida, obteve %d", sessW2.Code)
	}

	// 4. Lista PDVs permitidos
	listReq := httptest.NewRequest("GET", "/api/v1/operator/pdvs", nil)
	listReq.Header.Set("Authorization", "Bearer "+token)
	listW := httptest.NewRecorder()
	router.ServeHTTP(listW, listReq)

	if listW.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK ao listar PDVs, obteve %d", listW.Code)
	}

	var pdvs []db.PDV
	_ = json.Unmarshal(listW.Body.Bytes(), &pdvs)
	if len(pdvs) != 1 || pdvs[0].PublicKey != pdvPubKey {
		t.Fatalf("esperava 1 PDV listado com chave %s, obteve %v", pdvPubKey, pdvs)
	}

	// 5. Grava auditoria
	auditPayload := []byte(fmt.Sprintf(`{
		"pdv_public_key":"%s",
		"start_time":"%s",
		"end_time":"%s",
		"reason":"Ajuste de impressora fiscal",
		"consent_result":"accepted",
		"bytes_transferred": 1048576
	}`, pdvPubKey, time.Now().Format(time.RFC3339), time.Now().Add(5*time.Minute).Format(time.RFC3339)))

	auditReq := httptest.NewRequest("POST", "/api/v1/session/audit", bytes.NewReader(auditPayload))
	auditReq.Header.Set("Authorization", "Bearer "+token)
	auditW := httptest.NewRecorder()
	router.ServeHTTP(auditW, auditReq)

	if auditW.Code != http.StatusCreated {
		t.Fatalf("esperava 201 Created na auditoria, obteve %d: %s", auditW.Code, auditW.Body.String())
	}
}

func TestExpiredTokenRejection(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	// Insere token ja expirado diretamente no banco
	tokenHex, tokenHash, _ := auth.GenerateOpaqueToken()
	expiredAt := time.Now().Add(-10 * time.Minute)
	_ = database.SaveToken(tokenHash, "op-expirado", expiredAt)

	req := httptest.NewRequest("GET", "/api/v1/operator/pdvs", nil)
	req.Header.Set("Authorization", "Bearer "+tokenHex)
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	if w.Code != http.StatusUnauthorized {
		t.Fatalf("esperava 401 Unauthorized para token expirado, obteve %d", w.Code)
	}
}
