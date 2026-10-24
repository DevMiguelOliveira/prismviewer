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
