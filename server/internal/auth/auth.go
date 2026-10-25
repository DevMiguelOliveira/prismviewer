package auth

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"net/http"
	"sync"
	"time"

	"golang.org/x/time/rate"
)

const (
	MaxClockSkew  = 60 * time.Second
	TokenTTL      = 8 * time.Hour
	LockoutLimit  = 5
	LockoutPeriod = 15 * time.Minute
)

// NonceTracker rastreia nonces utilizados para evitar replay attacks.
type NonceTracker struct {
	mu     sync.Mutex
	nonces map[string]time.Time
}

func NewNonceTracker() *NonceTracker {
	nt := &NonceTracker{
		nonces: make(map[string]time.Time),
	}
	// Limpeza periódica de nonces expirados
	go nt.cleanupLoop()
	return nt
}

func (nt *NonceTracker) CheckAndMark(nonce string) bool {
	nt.mu.Lock()
	defer nt.mu.Unlock()

	if _, exists := nt.nonces[nonce]; exists {
		return false // Replay detectado!
	}
	nt.nonces[nonce] = time.Now()
	return true
}

func (nt *NonceTracker) cleanupLoop() {
	ticker := time.NewTicker(2 * time.Minute)
	for range ticker.C {
		nt.mu.Lock()
		cutoff := time.Now().Add(-2 * MaxClockSkew)
		for nonce, ts := range nt.nonces {
			if ts.Before(cutoff) {
				delete(nt.nonces, nonce)
			}
		}
		nt.mu.Unlock()
	}
}

// RateLimiter controla requisições por IP e bloqueio de IPs com falhas sucessivas.
type IPRateLimiter struct {
	mu       sync.Mutex
	limiters map[string]*rate.Limiter
	failures map[string]int
	locked   map[string]time.Time
}

func NewIPRateLimiter() *IPRateLimiter {
	return &IPRateLimiter{
		limiters: make(map[string]*rate.Limiter),
		failures: make(map[string]int),
		locked:   make(map[string]time.Time),
	}
}

func (rl *IPRateLimiter) Allow(ip string) (bool, string) {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	// Verifica se IP está em bloqueio de 15 minutos
	if lockUntil, ok := rl.locked[ip]; ok {
		if time.Now().Before(lockUntil) {
			return false, "IP temporariamente bloqueado por excesso de falhas"
		}
		delete(rl.locked, ip)
		delete(rl.failures, ip)
	}

	limiter, exists := rl.limiters[ip]
	if !exists {
		// 10 requisições por segundo com burst de 20
		limiter = rate.NewLimiter(rate.Limit(10), 20)
		rl.limiters[ip] = limiter
	}

	if !limiter.Allow() {
		return false, "Limite de taxa excedido (Rate limit)"
	}
	return true, ""
}

func (rl *IPRateLimiter) RecordFailure(ip string) {
	rl.mu.Lock()
	defer rl.mu.Unlock()

	rl.failures[ip]++
	if rl.failures[ip] >= LockoutLimit {
		rl.locked[ip] = time.Now().Add(LockoutPeriod)
	}
}

func (rl *IPRateLimiter) ResetFailures(ip string) {
	rl.mu.Lock()
	defer rl.mu.Unlock()
	delete(rl.failures, ip)
}

// GenerateOpaqueToken gera um token opaco criptograficamente seguro de 32 bytes e retorna (tokenHex, hashSHA256).
func GenerateOpaqueToken() (string, string, error) {
	bytes := make([]byte, 32)
	if _, err := rand.Read(bytes); err != nil {
		return "", "", fmt.Errorf("falha ao ler entropia: %w", err)
	}
	tokenHex := hex.EncodeToString(bytes)
	tokenHash := HashSHA256(tokenHex)
	return tokenHex, tokenHash, nil
}

// GenerateOpaqueTokenNonce gera um nonce aleatório de 32 bytes em hex (64 caracteres),
// usado em tickets de sessão para prevenir replay.
func GenerateOpaqueTokenNonce() string {
	bytes := make([]byte, 32)
	if _, err := rand.Read(bytes); err != nil {
		// Em caso de falha crítica de entropia, retorna um valor baseado em tempo (fallback fraco, mas não deve acontecer)
		return fmt.Sprintf("%x", time.Now().UnixNano())
	}
	return hex.EncodeToString(bytes)
}

// HashSHA256 calcula o hash SHA-256 de uma string e retorna em formato hex.
func HashSHA256(data string) string {
	h := sha256.Sum256([]byte(data))
	return hex.EncodeToString(h[:])
}

// VerifyHostSignature valida uma requisição assinada pelo PDV via Ed25519.
func VerifyHostSignature(
	pubKeyHex string,
	timestamp int64,
	nonce string,
	payload []byte,
	signatureHex string,
	tracker *NonceTracker,
) error {
	// 1. Valida janela de tempo contra clock skew
	reqTime := time.Unix(timestamp, 0)
	now := time.Now()
	diff := now.Sub(reqTime)
	if diff < 0 {
		diff = -diff
	}
	if diff > MaxClockSkew {
		return fmt.Errorf("timestamp fora da janela aceitavel (diff=%v)", diff)
	}

	// 2. Valida se o nonce já foi utilizado (anti-replay)
	if tracker != nil && !tracker.CheckAndMark(nonce) {
		return fmt.Errorf("nonce duplicado - replay attack detectado")
	}

	// 3. Decodifica chave pública e assinatura
	pubKeyBytes, err := hex.DecodeString(pubKeyHex)
	if err != nil || len(pubKeyBytes) != ed25519.PublicKeySize {
		return fmt.Errorf("chave publica Ed25519 invalida")
	}

	sigBytes, err := hex.DecodeString(signatureHex)
	if err != nil || len(sigBytes) != ed25519.SignatureSize {
		return fmt.Errorf("assinatura Ed25519 invalida")
	}

	// 4. Monta mensagem assinada: timestamp + nonce + payload
	msgToVerify := make([]byte, 0, 8+len(nonce)+len(payload))
	msgToVerify = append(msgToVerify, []byte(fmt.Sprintf("%d:%s:", timestamp, nonce))...)
	msgToVerify = append(msgToVerify, payload...)

	if !ed25519.Verify(pubKeyBytes, msgToVerify, sigBytes) {
		return fmt.Errorf("assinatura Ed25519 nao confere com a chave publica")
	}

	return nil
}

// Middleware de rate limit por IP.
func RateLimitMiddleware(rl *IPRateLimiter) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip := r.RemoteAddr
			allowed, reason := rl.Allow(ip)
			if !allowed {
				http.Error(w, reason, http.StatusTooManyRequests)
				return
			}
			next.ServeHTTP(w, r)
		})
	}
}
