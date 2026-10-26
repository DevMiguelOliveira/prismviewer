package auth

import (
	"crypto/ed25519"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"testing"
	"time"
)

func TestVerifyHostSignature_Valid(t *testing.T) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("falha ao gerar chave: %v", err)
	}

	pubHex := hex.EncodeToString(pub)
	now := time.Now().Unix()
	nonce := "test-nonce-12345"
	payload := []byte(`{"pdv": 1, "loja": "Matriz"}`)

	msg := []byte(fmt.Sprintf("%d:%s:%s", now, nonce, payload))
	sig := ed25519.Sign(priv, msg)
	sigHex := hex.EncodeToString(sig)

	tracker := NewNonceTracker()
	err = VerifyHostSignature(pubHex, now, nonce, payload, sigHex, tracker)
	if err != nil {
		t.Fatalf("assinatura valida foi recusada: %v", err)
	}
}

func TestVerifyHostSignature_ClockSkew(t *testing.T) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("falha ao gerar chave: %v", err)
	}

	pubHex := hex.EncodeToString(pub)
	oldTimestamp := time.Now().Add(-120 * time.Second).Unix()
	nonce := "test-nonce-skew"
	payload := []byte("dados")

	msg := []byte(fmt.Sprintf("%d:%s:%s", oldTimestamp, nonce, payload))
	sig := ed25519.Sign(priv, msg)
	sigHex := hex.EncodeToString(sig)

	tracker := NewNonceTracker()
	err = VerifyHostSignature(pubHex, oldTimestamp, nonce, payload, sigHex, tracker)
	if err == nil {
		t.Fatal("esperava erro de clock skew para timestamp de 120s atras, mas passou")
	}
}

func TestVerifyHostSignature_ReplayAttack(t *testing.T) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("falha ao gerar chave: %v", err)
	}

	pubHex := hex.EncodeToString(pub)
	now := time.Now().Unix()
	nonce := "replay-nonce-unique"
	payload := []byte("pagamento-100")

	msg := []byte(fmt.Sprintf("%d:%s:%s", now, nonce, payload))
	sig := ed25519.Sign(priv, msg)
	sigHex := hex.EncodeToString(sig)

	tracker := NewNonceTracker()

	// 1a tentativa: valida
	err = VerifyHostSignature(pubHex, now, nonce, payload, sigHex, tracker)
	if err != nil {
		t.Fatalf("1a requisicao valida falhou: %v", err)
	}

	// 2a tentativa com o mesmo nonce: DEVE falhar (replay)
	err = VerifyHostSignature(pubHex, now, nonce, payload, sigHex, tracker)
	if err == nil {
		t.Fatal("esperava erro de replay attack ao reenviar o mesmo nonce, mas passou")
	}
}

func TestVerifyHostSignature_TamperedPayload(t *testing.T) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("falha ao gerar chave: %v", err)
	}

	pubHex := hex.EncodeToString(pub)
	now := time.Now().Unix()
	nonce := "tamper-nonce"
	payloadOriginal := []byte("origem=10")
	payloadAdulterado := []byte("origem=99")

	msg := []byte(fmt.Sprintf("%d:%s:%s", now, nonce, payloadOriginal))
	sig := ed25519.Sign(priv, msg)
	sigHex := hex.EncodeToString(sig)

	tracker := NewNonceTracker()
	err = VerifyHostSignature(pubHex, now, nonce, payloadAdulterado, sigHex, tracker)
	if err == nil {
		t.Fatal("esperava falha de assinatura em payload adulterado, mas passou")
	}
}

func TestIPRateLimiter_LockoutAfter5Failures(t *testing.T) {
	limiter := NewIPRateLimiter()
	ip := "192.168.1.100:54321"

	// Inicialmente permitido
	allowed, _ := limiter.Allow(ip)
	if !allowed {
		t.Fatal("IP legitimo deveria ser permitido inicialmente")
	}

	// Registra 5 falhas
	for i := 0; i < 5; i++ {
		limiter.RecordFailure(ip)
	}

	// Agora deve estar bloqueado
	allowed, reason := limiter.Allow(ip)
	if allowed {
		t.Fatal("IP com 5 falhas deveria estar bloqueado")
	}
	if reason == "" {
		t.Fatal("esperava motivo de bloqueio retornado")
	}

	// Resetando falhas
	limiter.ResetFailures(ip)
	limiter.locked[ip] = time.Now().Add(-1 * time.Minute) // expira o lock para teste
	allowed, _ = limiter.Allow(ip)
	if !allowed {
		t.Fatal("IP com falhas resetadas deveria ser liberado")
	}
}

func TestGenerateOpaqueToken(t *testing.T) {
	tokenHex, tokenHash, err := GenerateOpaqueToken()
	if err != nil {
		t.Fatalf("falha ao gerar token: %v", err)
	}

	// 32 bytes em hex = 64 caracteres
	if len(tokenHex) != 64 {
		t.Errorf("token com tamanho incorreto: esperado 64, obtido %d", len(tokenHex))
	}

	// Hash SHA-256 = 64 caracteres hex
	if len(tokenHash) != 64 {
		t.Errorf("token hash com tamanho incorreto: esperado 64, obtido %d", len(tokenHash))
	}

	expectedHash := HashSHA256(tokenHex)
	if tokenHash != expectedHash {
		t.Errorf("hash nao confere: esperado %s, obtido %s", expectedHash, tokenHash)
	}
}

func TestArgon2id_HashAndVerify(t *testing.T) {
	password := "SenhaSegura!123"
	hash, err := HashPasswordArgon2id(password)
	if err != nil {
		t.Fatalf("falha ao gerar hash argon2id: %v", err)
	}

	valid, err := VerifyPasswordArgon2id(password, hash)
	if err != nil || !valid {
		t.Fatalf("senha correta nao foi validada pelo argon2id: %v", err)
	}
}

func TestArgon2id_WrongPasswordRejected(t *testing.T) {
	hash, err := HashPasswordArgon2id("SenhaOriginal123")
	if err != nil {
		t.Fatalf("falha ao gerar hash: %v", err)
	}

	valid, err := VerifyPasswordArgon2id("SenhaErrada456", hash)
	if err != nil {
		t.Fatalf("erro inesperado ao verificar senha errada: %v", err)
	}
	if valid {
		t.Fatal("senha errada foi aceita pelo argon2id")
	}
}

func TestEntraIDToken_ValidWithGroup(t *testing.T) {
	// Cria mock de JWT header.payload.sig
	header := "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9"
	claimsJSON := fmt.Sprintf(`{
		"sub": "tech-uuid-1234",
		"name": "Maria Suporte",
		"preferred_username": "maria.suporte@empresa.com",
		"email": "maria.suporte@empresa.com",
		"groups": ["prysmview-technicians", "ti-suporte"],
		"roles": ["Technician"],
		"exp": %d
	}`, time.Now().Add(1*time.Hour).Unix())

	b64Payload := "eyJzdWIiOiJ0ZWNoLXV1aWQtMTIzNCIsIm5hbWUiOiJNYXJpYSBTdXBvcnRlIiwicHJlZmVycmVkX3VzZXJuYW1lIjoibWFyaWEuc3Vwb3J0ZUBlbXByZXNhLmNvbSIsImVtYWlsIjoibWFyaWEuc3Vwb3J0ZUBlbXByZXNhLmNvbSIsImdyb3VwcyI6WyJwcnlzbXZpZXctdGVjaG5pY2lhbnMiLCJ0aS1zdXBvcnRlIl0sInJvbGVzIjpbIlRlY2huaWNpYW4iXSwiZXhwIjoyNTM0MDIzMDA3OTl9"
	_ = claimsJSON
	rawToken := header + "." + b64Payload + ".mockSignature"

	claims, err := ValidateEntraToken(rawToken, "prysmview-technicians")
	if err != nil {
		t.Fatalf("token valido do Entra ID foi rejeitado: %v", err)
	}
	if claims.PreferredUsername != "maria.suporte@empresa.com" {
		t.Errorf("username incorreto: %s", claims.PreferredUsername)
	}
}

func TestEntraIDToken_OutsideGroupRejected(t *testing.T) {
	header := "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9"
	// Payload sem o grupo "prysmview-technicians"
	b64Payload := "eyJzdWIiOiJvdXRzaWRlci0xMjM0IiwibmFtZSI6IkpvYW8gRm9yYSIsInByZWZlcnJlZF91c2VybmFtZSI6ImpvYW8uZm9yYUBlbXByZXNhLmNvbSIsImVtYWlsIjoiam9hby5mb3JhQGVtcHJlc2EuY29tIiwiZ3JvdXBzIjpbImZpbmFuY2Vpcm8iXSwicm9sZXMiOlsiVXNlciJdLCJleHAiOjI1MzQwMjMwMDc5OX0"
	rawToken := header + "." + b64Payload + ".mockSignature"

	_, err := ValidateEntraToken(rawToken, "prysmview-technicians")
	if err == nil {
		t.Fatal("token de operador fora do grupo autorizado deveria ser rejeitado")
	}
}

