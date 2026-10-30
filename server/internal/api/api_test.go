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
	"strings"
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

	// Gera chave de assinatura de tickets para os testes
	_, signingKey, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("falha ao gerar chave de assinatura: %v", err)
	}

	router := NewRouter(database, tracker, limiter, signingKey)
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

	var res map[string]interface{}
	if err := json.Unmarshal(w.Body.Bytes(), &res); err != nil {
		t.Fatalf("resposta invalida: %v", err)
	}
	if res["status"] != "ok" {
		t.Errorf("esperava status ok, obteve %v", res["status"])
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

	// Cria operador com hash Argon2id da senha "mock" (S-11)
	passwordHash, _ := auth.HashPasswordArgon2id("mock")
	op := db.Operator{
		ID:           "op-tecnico-1",
		Name:         "Carlos Tecnico",
		PasswordHash: passwordHash,
		CreatedAt:    time.Now(),
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

func TestSessionTicketEmissionAndValidation(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	// Cria operador com hash Argon2id da senha "mock" (S-11)
	passwordHash, _ := auth.HashPasswordArgon2id("mock")
	op := db.Operator{
		ID:           "op-ticket-1",
		Name:         "Tecnico Ticket",
		PasswordHash: passwordHash,
		CreatedAt:    time.Now(),
	}
	_ = database.CreateOperator(op)

	pdvPubKey := "pubkey-pdv-ticket-01"
	_ = database.RegisterPDV(db.PDV{
		PublicKey:    pdvPubKey,
		StoreName:    "Loja Ticket",
		POSNumber:    1,
		AgentVersion: "1.0.0",
		Status:       "online",
		LastSeen:     time.Now(),
	})

	// Concede permissao
	_ = database.GrantPermission("op-ticket-1", pdvPubKey)

	// Login do operador
	loginPayload := []byte(`{"operator_id":"op-ticket-1","password":"mock"}`)
	loginReq := httptest.NewRequest("POST", "/api/v1/operator/login", bytes.NewReader(loginPayload))
	loginW := httptest.NewRecorder()
	router.ServeHTTP(loginW, loginReq)

	if loginW.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK no login, obteve %d: %s", loginW.Code, loginW.Body.String())
	}

	var loginRes struct {
		Token string `json:"token"`
	}
	_ = json.Unmarshal(loginW.Body.Bytes(), &loginRes)
	token := loginRes.Token
	if len(token) != 64 {
		t.Fatalf("token opaco invalido: %s", token)
	}

	// Solicita ticket
	ticketReq := []byte(fmt.Sprintf(`{"pdv_public_key":"%s","machine_id":"pdv-maquina-01","reason":"chamado aberto"}`, pdvPubKey))
	ticketReqHTTP := httptest.NewRequest("POST", "/api/v1/session/ticket", bytes.NewReader(ticketReq))
	ticketReqHTTP.Header.Set("Authorization", "Bearer "+token)
	ticketW := httptest.NewRecorder()
	router.ServeHTTP(ticketW, ticketReqHTTP)

	if ticketW.Code != http.StatusCreated {
		t.Fatalf("esperava 201 Created na emissao de ticket, obteve %d: %s", ticketW.Code, ticketW.Body.String())
	}

	var ticketResp struct {
		Ticket struct {
			TicketID     string    `json:"ticket_id"`
			MachineID    string    `json:"machine_id"`
			TechnicianID string    `json:"technician_id"`
			Groups       []string  `json:"groups"`
			NotBefore    time.Time `json:"not_before"`
			ExpiresAt    time.Time `json:"expires_at"`
			Nonce        string    `json:"nonce"`
			Signature    string    `json:"signature"`
		} `json:"ticket"`
		Message string `json:"message"`
	}
	_ = json.Unmarshal(ticketW.Body.Bytes(), &ticketResp)

	// Valida campos do ticket
	if ticketResp.Ticket.TicketID == "" {
		t.Fatal("ticket_id vazio")
	}
	if ticketResp.Ticket.MachineID != "pdv-maquina-01" {
		t.Errorf("machine_id incorreto: %s", ticketResp.Ticket.MachineID)
	}
	if ticketResp.Ticket.TechnicianID != "op-ticket-1" {
		t.Errorf("technician_id incorreto: %s", ticketResp.Ticket.TechnicianID)
	}
	if ticketResp.Ticket.Signature == "" {
		t.Fatal("assinatura vazia")
	}
	if ticketResp.Ticket.Nonce == "" {
		t.Fatal("nonce vazio")
	}
	// Validade maxima de 30 minutos
	duracao := ticketResp.Ticket.ExpiresAt.Sub(ticketResp.Ticket.NotBefore)
	if duracao > 30*time.Minute {
		t.Errorf("validade do ticket excede 30 minutos: %v", duracao)
	}
	if duracao <= 0 {
		t.Fatal("expires_at deve ser depois de not_before")
	}

	// Valida a assinatura Ed25519 do ticket
	// A validacao completa da assinatura e feita pelo host offline.
	// Aqui apenas verificamos que a assinatura tem 64 bytes (128 hex)
	sigBytes, err := hex.DecodeString(ticketResp.Ticket.Signature)
	if err != nil {
		t.Fatalf("assinatura nao e hex valido: %v", err)
	}
	if len(sigBytes) != 64 {
		t.Errorf("assinatura tem %d bytes, esperado 64", len(sigBytes))
	}

	t.Logf("Ticket emitido com sucesso: id=%s maquina=%s tecnico=%s exp=%v",
		ticketResp.Ticket.TicketID, ticketResp.Ticket.MachineID, ticketResp.Ticket.TechnicianID, ticketResp.Ticket.ExpiresAt)
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

func TestSecurityHeadersAndCookies(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	// 1. Testa headers de seguranca (S-11)
	req := httptest.NewRequest("GET", "/health", nil)
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)

	hsts := w.Header().Get("Strict-Transport-Security")
	if hsts == "" || !strings.Contains(hsts, "max-age=") {
		t.Errorf("cabecalho HSTS ausente ou invalido: %s", hsts)
	}

	nosniff := w.Header().Get("X-Content-Type-Options")
	if nosniff != "nosniff" {
		t.Errorf("cabecalho X-Content-Type-Options incorreto: %s", nosniff)
	}

	frameOpts := w.Header().Get("X-Frame-Options")
	if frameOpts != "DENY" {
		t.Errorf("cabecalho X-Frame-Options incorreto: %s", frameOpts)
	}

	csp := w.Header().Get("Content-Security-Policy")
	if csp == "" || !strings.Contains(csp, "default-src") {
		t.Errorf("cabecalho CSP ausente ou invalido: %s", csp)
	}

	// 2. Testa cookie seguro HttpOnly, Secure e SameSite=Strict no login (S-11)
	passwordHash, _ := auth.HashPasswordArgon2id("mock")
	_ = database.CreateOperator(db.Operator{
		ID:           "op-teste",
		Name:         "Operador Teste",
		PasswordHash: passwordHash,
		CreatedAt:    time.Now(),
	})
	loginPayload := []byte(`{"operator_id":"op-teste","password":"mock"}`)
	loginReq := httptest.NewRequest("POST", "/api/v1/operator/login", bytes.NewReader(loginPayload))
	loginW := httptest.NewRecorder()
	router.ServeHTTP(loginW, loginReq)

	if loginW.Code != http.StatusOK {
		t.Fatalf("esperava 200 no login, obteve %d", loginW.Code)
	}

	cookies := loginW.Result().Cookies()
	var sessionCookie *http.Cookie
	for _, c := range cookies {
		if c.Name == "prysmview_session" {
			sessionCookie = c
			break
		}
	}

	if sessionCookie == nil {
		t.Fatal("cookie de sessao 'prysmview_session' nao foi emitido")
	}
	if !sessionCookie.HttpOnly {
		t.Error("cookie deve ter flag HttpOnly ativada")
	}
	if !sessionCookie.Secure {
		t.Error("cookie deve ter flag Secure ativada")
	}
	if sessionCookie.SameSite != http.SameSiteStrictMode {
		t.Errorf("cookie deve ter SameSite=Strict, obteve %v", sessionCookie.SameSite)
	}
}

func TestOperatorLogin_EntraID(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	header := "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9"
	// Payload com grupo autorizado "prysmview-technicians"
	b64PayloadValid := "eyJzdWIiOiJ0ZWNoLXV1aWQtMTIzNCIsIm5hbWUiOiJNYXJpYSBTdXBvcnRlIiwicHJlZmVycmVkX3VzZXJuYW1lIjoibWFyaWEuc3Vwb3J0ZUBlbXByZXNhLmNvbSIsImVtYWlsIjoibWFyaWEuc3Vwb3J0ZUBlbXByZXNhLmNvbSIsImdyb3VwcyI6WyJwcnlzbXZpZXctdGVjaG5pY2lhbnMiXSwicm9sZXMiOlsiVGVjaG5pY2lhbiJdLCJleHAiOjI1MzQwMjMwMDc5OX0"
	validToken := header + "." + b64PayloadValid + ".mockSignature"

	// 1. Login com token Entra ID valido
	validBody := fmt.Sprintf(`{"entra_token":"%s"}`, validToken)
	req1 := httptest.NewRequest("POST", "/api/v1/operator/login", strings.NewReader(validBody))
	w1 := httptest.NewRecorder()
	router.ServeHTTP(w1, req1)

	if w1.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK para Entra ID valido, obteve %d: %s", w1.Code, w1.Body.String())
	}

	var res LoginResponse
	_ = json.Unmarshal(w1.Body.Bytes(), &res)
	if res.OperatorID != "maria.suporte@empresa.com" {
		t.Errorf("operator_id divergente: %s", res.OperatorID)
	}

	// 2. Login com token Entra ID de usuario FORA do grupo autorizado
	b64PayloadOut := "eyJzdWIiOiJvdXRzaWRlci0xMjM0IiwibmFtZSI6IkpvYW8gRm9yYSIsInByZWZlcnJlZF91c2VybmFtZSI6ImpvYW8uZm9yYUBlbXByZXNhLmNvbSIsImVtYWlsIjoiam9hby5mb3JhQGVtcHJlc2EuY29tIiwiZ3JvdXBzIjpbImZpbmFuY2Vpcm8iXSwicm9sZXMiOlsiVXNlciJdLCJleHAiOjI1MzQwMjMwMDc5OX0"
	invalidToken := header + "." + b64PayloadOut + ".mockSignature"

	invalidBody := fmt.Sprintf(`{"entra_token":"%s"}`, invalidToken)
	req2 := httptest.NewRequest("POST", "/api/v1/operator/login", strings.NewReader(invalidBody))
	w2 := httptest.NewRecorder()
	router.ServeHTTP(w2, req2)

	if w2.Code != http.StatusUnauthorized {
		t.Fatalf("esperava 401 Unauthorized para operador fora do grupo autorizado, obteve %d", w2.Code)
	}
}

func TestAgentTokenAndHeartbeatFlow(t *testing.T) {
	database, router := setupTestServer(t)
	defer database.Close()

	machineID := "PDV-TEST-01"

	// 1. Emissao de token de agente (S-13)
	issueReqBody := fmt.Sprintf(`{"machine_id":"%s"}`, machineID)
	issueReq := httptest.NewRequest("POST", "/api/v1/host/token", strings.NewReader(issueReqBody))
	issueW := httptest.NewRecorder()
	router.ServeHTTP(issueW, issueReq)

	if issueW.Code != http.StatusCreated {
		t.Fatalf("esperava 201 Created na emissao de token de agente, obteve %d", issueW.Code)
	}

	var issueRes IssueTokenResponse
	_ = json.Unmarshal(issueW.Body.Bytes(), &issueRes)
	token := issueRes.Token
	if len(token) != 64 {
		t.Fatalf("token de agente tem tamanho incorreto: %s", token)
	}

	// 2. Adiciona revogacao centralizada para testar distribuicao via heartbeat (S-09)
	_ = database.AddRevocation("tkt-comprometido-001", "ticket", "teste de revogacao")

	// 3. Heartbeat valido com o token
	hbReq := httptest.NewRequest("POST", "/api/v1/host/heartbeat", nil)
	hbReq.Header.Set("X-Machine-ID", machineID)
	hbReq.Header.Set("Authorization", "Bearer "+token)
	hbW := httptest.NewRecorder()
	router.ServeHTTP(hbW, hbReq)

	if hbW.Code != http.StatusOK {
		t.Fatalf("esperava 200 OK no heartbeat valido, obteve %d: %s", hbW.Code, hbW.Body.String())
	}

	var hbRes HeartbeatResponse
	_ = json.Unmarshal(hbW.Body.Bytes(), &hbRes)
	if hbRes.Status != "ok" {
		t.Errorf("status inesperado no heartbeat: %s", hbRes.Status)
	}
	if len(hbRes.Revocations) == 0 || hbRes.Revocations[0] != "tkt-comprometido-001" {
		t.Errorf("lista de revogacoes nao distribuida no heartbeat: %v", hbRes.Revocations)
	}

	// 4. Revoga o token do agente e tenta enviar novo heartbeat -> 401 Unauthorized (S-13)
	if err := database.RevokeAgentToken(machineID); err != nil {
		t.Fatalf("falha ao revogar token: %v", err)
	}

	hbReqRevoked := httptest.NewRequest("POST", "/api/v1/host/heartbeat", nil)
	hbReqRevoked.Header.Set("X-Machine-ID", machineID)
	hbReqRevoked.Header.Set("Authorization", "Bearer "+token)
	hbWRevoked := httptest.NewRecorder()
	router.ServeHTTP(hbWRevoked, hbReqRevoked)

	if hbWRevoked.Code != http.StatusUnauthorized {
		t.Fatalf("esperava 401 Unauthorized para token de agente revogado, obteve %d", hbWRevoked.Code)
	}
}
