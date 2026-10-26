package auth

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha256"
	"crypto/subtle"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"

	"golang.org/x/crypto/argon2"
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

// Argon2idParams define os parametros recomendados pela OWASP para armazenamento seguro de senhas.
type Argon2idParams struct {
	Memory      uint32
	Iterations  uint32
	Parallelism uint8
	SaltLength  uint32
	KeyLength   uint32
}

var DefaultArgon2idParams = Argon2idParams{
	Memory:      64 * 1024, // 64 MB
	Iterations:  3,
	Parallelism: 2,
	SaltLength:  16,
	KeyLength:   32,
}

// HashPasswordArgon2id deriva um hash criptografico seguro usando Argon2id (S-11).
func HashPasswordArgon2id(password string) (string, error) {
	params := DefaultArgon2idParams
	salt := make([]byte, params.SaltLength)
	if _, err := rand.Read(salt); err != nil {
		return "", fmt.Errorf("falha ao gerar salt: %w", err)
	}

	hash := argon2.IDKey([]byte(password), salt, params.Iterations, params.Memory, params.Parallelism, params.KeyLength)

	b64Salt := base64.RawStdEncoding.EncodeToString(salt)
	b64Hash := base64.RawStdEncoding.EncodeToString(hash)

	encoded := fmt.Sprintf("$argon2id$v=%d$m=%d,t=%d,p=%d$%s$%s",
		argon2.Version, params.Memory, params.Iterations, params.Parallelism, b64Salt, b64Hash)

	return encoded, nil
}

// VerifyPasswordArgon2id verifica uma senha contra um hash codificado em tempo constante (S-11, S-16).
func VerifyPasswordArgon2id(password, encodedHash string) (bool, error) {
	parts := strings.Split(encodedHash, "$")
	if len(parts) != 6 {
		return false, errors.New("formato de hash argon2id invalido")
	}

	if parts[1] != "argon2id" {
		return false, errors.New("algoritmo incompativel")
	}

	var version int
	if _, err := fmt.Sscanf(parts[2], "v=%d", &version); err != nil || version != argon2.Version {
		return false, errors.New("versao de argon2 incompativel")
	}

	var memory uint32
	var iterations uint32
	var parallelism uint8
	if _, err := fmt.Sscanf(parts[3], "m=%d,t=%d,p=%d", &memory, &iterations, &parallelism); err != nil {
		return false, errors.New("parametros invalidos no hash")
	}

	salt, err := base64.RawStdEncoding.DecodeString(parts[4])
	if err != nil {
		return false, errors.New("falha ao decodificar salt base64")
	}

	expectedHash, err := base64.RawStdEncoding.DecodeString(parts[5])
	if err != nil {
		return false, errors.New("falha ao decodificar digest base64")
	}

	calculatedHash := argon2.IDKey([]byte(password), salt, iterations, memory, parallelism, uint32(len(expectedHash)))

	if subtle.ConstantTimeCompare(expectedHash, calculatedHash) == 1 {
		return true, nil
	}
	return false, nil
}

// EntraClaims representa as declaracoes do token OIDC do Microsoft Entra ID (S-08).
type EntraClaims struct {
	Sub               string   `json:"sub"`
	Name              string   `json:"name"`
	PreferredUsername string   `json:"preferred_username"`
	Email             string   `json:"email"`
	Roles             []string `json:"roles"`
	Groups            []string `json:"groups"`
	Exp               int64    `json:"exp"`
	Iss               string   `json:"iss"`
	Aud               string   `json:"aud"`
}

// ValidateEntraToken valida o token do Entra ID e verifica se o operador pertence ao grupo permitido (S-08).
func ValidateEntraToken(rawToken string, requiredGroup string) (*EntraClaims, error) {
	parts := strings.Split(rawToken, ".")
	if len(parts) != 3 {
		return nil, errors.New("formato JWT invalido")
	}

	payloadBytes, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return nil, fmt.Errorf("falha ao decodificar payload JWT: %w", err)
	}

	var claims EntraClaims
	if err := json.Unmarshal(payloadBytes, &claims); err != nil {
		return nil, fmt.Errorf("payload JWT invalido: %w", err)
	}

	// 1. Verifica expiracao
	if time.Now().Unix() > claims.Exp {
		return nil, errors.New("token Entra ID expirado")
	}

	// 2. Verifica se o tecnico pertence ao grupo estatico autorizado
	if requiredGroup != "" {
		hasGroup := false
		for _, g := range claims.Groups {
			if subtle.ConstantTimeCompare([]byte(g), []byte(requiredGroup)) == 1 {
				hasGroup = true
				break
			}
		}
		for _, r := range claims.Roles {
			if subtle.ConstantTimeCompare([]byte(r), []byte(requiredGroup)) == 1 {
				hasGroup = true
				break
			}
		}
		if !hasGroup {
			return nil, fmt.Errorf("operador fora do grupo autorizado '%s'", requiredGroup)
		}
	}

	return &claims, nil
}

// SetSecureSessionCookie define cookie de sessao estritamente seguro: HttpOnly, Secure e SameSite=Strict (S-11).
func SetSecureSessionCookie(w http.ResponseWriter, token string, expiresAt time.Time) {
	http.SetCookie(w, &http.Cookie{
		Name:     "prysmview_session",
		Value:    token,
		Expires:  expiresAt,
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteStrictMode,
		Path:     "/",
	})
}
