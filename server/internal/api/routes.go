package api

import (
	"crypto/ed25519"
	"encoding/hex"
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"prysmview/server/internal/auth"
	"prysmview/server/internal/db"
)

type Server struct {
	db                *db.DB
	tracker           *auth.NonceTracker
	limiter           *auth.IPRateLimiter
	ticketSigningKey  ed25519.PrivateKey
	ticketStore       map[string]bool // ticket_id -> usado (uso unico)
	ticketStoreMu     sync.Mutex
}

func NewRouter(database *db.DB, tracker *auth.NonceTracker, limiter *auth.IPRateLimiter, signingKey ed25519.PrivateKey) http.Handler {
	s := &Server{
		db:             database,
		tracker:        tracker,
		limiter:        limiter,
		ticketSigningKey: signingKey,
		ticketStore:    make(map[string]bool),
	}

	r := chi.NewRouter()

	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Recoverer)
	r.Use(auth.RateLimitMiddleware(limiter))
	r.Use(securityHeadersMiddleware)

	// Health check
	r.Get("/health", s.handleHealth)

	// API v1
	r.Route("/api/v1", func(r chi.Router) {
		// Host endpoints
		r.Post("/host/register", s.handleHostRegister)
		r.Get("/host/status", s.handleHostStatus)

		// Operator login
		r.Post("/operator/login", s.handleOperatorLogin)

		// Rotas protegidas por token de operador
		r.Group(func(r chi.Router) {
			r.Use(s.authMiddleware)

			r.Get("/operator/pdvs", s.handleListPDVs)
			r.Post("/session/request", s.handleSessionRequest)
			r.Post("/session/ticket", s.handleSessionTicket)
			r.Post("/session/audit", s.handleSessionAudit)
		})
	})

	return r
}

func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(map[string]string{
		"status":  "ok",
		"version": "1.0.0",
	})
}

type HostRegisterPayload struct {
	StoreName    string `json:"store_name"`
	POSNumber    int    `json:"pos_number"`
	AgentVersion string `json:"agent_version"`
	Status       string `json:"status"`
}

func (s *Server) handleHostRegister(w http.ResponseWriter, r *http.Request) {
	pubKey := r.Header.Get("X-Pdv-Public-Key")
	tsStr := r.Header.Get("X-Timestamp")
	nonce := r.Header.Get("X-Nonce")
	sig := r.Header.Get("X-Signature")

	if pubKey == "" || tsStr == "" || nonce == "" || sig == "" {
		s.limiter.RecordFailure(r.RemoteAddr)
		http.Error(w, "Cabecalhos de autenticacao ausentes", http.StatusUnauthorized)
		return
	}

	ts, err := strconv.ParseInt(tsStr, 10, 64)
	if err != nil {
		s.limiter.RecordFailure(r.RemoteAddr)
		http.Error(w, "Timestamp invalido", http.StatusBadRequest)
		return
	}

	bodyBytes, err := io.ReadAll(io.LimitReader(r.Body, 64*1024))
	if err != nil {
		http.Error(w, "Falha ao ler corpo da requisicao", http.StatusBadRequest)
		return
	}

	// Valida assinatura Ed25519
	if err := auth.VerifyHostSignature(pubKey, ts, nonce, bodyBytes, sig, s.tracker); err != nil {
		s.limiter.RecordFailure(r.RemoteAddr)
		slog.Warn("Falha na verificacao de assinatura Ed25519", "ip", r.RemoteAddr, "erro", err.Error())
		http.Error(w, "Assinatura invalida: "+err.Error(), http.StatusUnauthorized)
		return
	}

	var payload HostRegisterPayload
	if err := json.Unmarshal(bodyBytes, &payload); err != nil {
		http.Error(w, "JSON invalido", http.StatusBadRequest)
		return
	}

	pdv := db.PDV{
		PublicKey:    pubKey,
		StoreName:    payload.StoreName,
		POSNumber:    payload.POSNumber,
		AgentVersion: payload.AgentVersion,
		Status:       payload.Status,
		LastSeen:     time.Now(),
	}

	if err := s.db.RegisterPDV(pdv); err != nil {
		slog.Error("Falha ao registrar PDV", "erro", err.Error())
		http.Error(w, "Erro interno", http.StatusInternalServerError)
		return
	}

	slog.Info("PDV registrado com sucesso", "pubkey", pubKey, "store", payload.StoreName, "pos", payload.POSNumber)
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(map[string]string{"message": "PDV registrado com sucesso"})
}

func (s *Server) handleHostStatus(w http.ResponseWriter, r *http.Request) {
	pubKey := r.URL.Query().Get("public_key")
	if pubKey == "" {
		http.Error(w, "public_key obrigatorio", http.StatusBadRequest)
		return
	}

	pdv, err := s.db.GetPDV(pubKey)
	if err != nil {
		http.Error(w, "Erro interno", http.StatusInternalServerError)
		return
	}
	if pdv == nil {
		http.Error(w, "PDV nao encontrado", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(pdv)
}

type LoginRequest struct {
	OperatorID string `json:"operator_id"`
	Password   string `json:"password"`
}

type LoginResponse struct {
	Token     string `json:"token"`
	ExpiresAt string `json:"expires_at"`
}

func (s *Server) handleOperatorLogin(w http.ResponseWriter, r *http.Request) {
	var req LoginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Requisicao invalida", http.StatusBadRequest)
		return
	}

	if req.OperatorID == "" {
		s.limiter.RecordFailure(r.RemoteAddr)
		http.Error(w, "operator_id obrigatorio", http.StatusBadRequest)
		return
	}

	tokenHex, tokenHash, err := auth.GenerateOpaqueToken()
	if err != nil {
		http.Error(w, "Falha ao gerar credencial", http.StatusInternalServerError)
		return
	}

	expiresAt := time.Now().Add(auth.TokenTTL)
	if err := s.db.SaveToken(tokenHash, req.OperatorID, expiresAt); err != nil {
		http.Error(w, "Falha ao salvar sessao", http.StatusInternalServerError)
		return
	}

	slog.Info("Operador autenticado", "operator_id", req.OperatorID)
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(LoginResponse{
		Token:     tokenHex,
		ExpiresAt: expiresAt.Format(time.RFC3339),
	})
}

type contextKey string

const operatorIDKey contextKey = "operator_id"

func (s *Server) authMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		if !strings.HasPrefix(authHeader, "Bearer ") {
			s.limiter.RecordFailure(r.RemoteAddr)
			http.Error(w, "Token ausente ou formato invalido", http.StatusUnauthorized)
			return
		}

		token := strings.TrimPrefix(authHeader, "Bearer ")
		tokenHash := auth.HashSHA256(token)

		operatorID, err := s.db.ValidateToken(tokenHash)
		if err != nil {
			s.limiter.RecordFailure(r.RemoteAddr)
			slog.Warn("Falha de autenticacao de operador", "erro", err.Error())
			http.Error(w, "Acesso nao autorizado: "+err.Error(), http.StatusUnauthorized)
			return
		}

		r.Header.Set("X-Operator-ID", operatorID)
		next.ServeHTTP(w, r)
	})
}

func (s *Server) handleListPDVs(w http.ResponseWriter, r *http.Request) {
	operatorID := r.Header.Get("X-Operator-ID")
	pdvs, err := s.db.ListPDVsForOperator(operatorID)
	if err != nil {
		http.Error(w, "Erro ao listar PDVs", http.StatusInternalServerError)
		return
	}

	if pdvs == nil {
		pdvs = []db.PDV{}
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(pdvs)
}

type SessionRequestPayload struct {
	PDVPublicKey string `json:"pdv_public_key"`
	Reason       string `json:"reason"`
}

/// Ticket de sessão assinado com Ed25519, emitido pelo servidor de sinalização.
/// Contém machine_id, technician_id, grupos, not_before, expires_at (max 30 min),
/// nonce e ticket_id. Validado offline pelo host.
type SessionTicket struct {
	TicketID       string    `json:"ticket_id"`
	MachineID      string    `json:"machine_id"`
	TechnicianID   string    `json:"technician_id"`
	Groups         []string  `json:"groups"`
	NotBefore      time.Time `json:"not_before"`
	ExpiresAt      time.Time `json:"expires_at"`
	Nonce          string    `json:"nonce"`
	Signature      string    `json:"signature"` // hex Ed25519 sobre os campos acima
}

/// Payload de solicitação de ticket (chamado pelo host/operador).
type SessionTicketRequest struct {
	PDVPublicKey string `json:"pdv_public_key"`
	MachineID    string `json:"machine_id"`
	Reason       string `json:"reason"`
}

/// Resposta de emissão de ticket.
type SessionTicketResponse struct {
	Ticket SessionTicket `json:"ticket"`
	Message string       `json:"message"`
}

func (s *Server) handleSessionRequest(w http.ResponseWriter, r *http.Request) {
	operatorID := r.Header.Get("X-Operator-ID")

	var req SessionRequestPayload
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "JSON invalido", http.StatusBadRequest)
		return
	}

	allowed, err := s.db.CheckPermission(operatorID, req.PDVPublicKey)
	if err != nil {
		http.Error(w, "Erro ao consultar permissoes", http.StatusInternalServerError)
		return
	}

	if !allowed {
		slog.Warn("Operador sem permissao de acesso ao PDV", "operator", operatorID, "pdv", req.PDVPublicKey)
		http.Error(w, "Permissao negada para este PDV", http.StatusForbidden)
		return
	}

	slog.Info("Sessao autorizada para operador", "operator", operatorID, "pdv", req.PDVPublicKey)
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]interface{}{
		"authorized": true,
		"operator":   operatorID,
		"pdv":        req.PDVPublicKey,
		"message":    "Sessao autorizada. Solicite um ticket via /api/v1/session/ticket antes de conectar.",
	})
}

func (s *Server) handleSessionTicket(w http.ResponseWriter, r *http.Request) {
	operatorID := r.Header.Get("X-Operator-ID")

	var req SessionTicketRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "JSON invalido", http.StatusBadRequest)
		return
	}

	if req.PDVPublicKey == "" || req.MachineID == "" {
		http.Error(w, "pdv_public_key e machine_id obrigatorios", http.StatusBadRequest)
		return
	}

	allowed, err := s.db.CheckPermission(operatorID, req.PDVPublicKey)
	if err != nil {
		http.Error(w, "Erro ao consultar permissoes", http.StatusInternalServerError)
		return
	}

	if !allowed {
		slog.Warn("Operador sem permissao para solicitar ticket", "operator", operatorID, "pdv", req.PDVPublicKey)
		http.Error(w, "Permissao negada para este PDV", http.StatusForbidden)
		return
	}

	// Gera ticket com validade maxima de 30 minutos
	now := time.Now()
	expiresAt := now.Add(30 * time.Minute)
	nonce := auth.GenerateOpaqueTokenNonce()
	ticketID := auth.GenerateOpaqueTokenNonce()

	ticket := SessionTicket{
		TicketID:    ticketID,
		MachineID:   req.MachineID,
		TechnicianID: operatorID,
		Groups:      []string{"suporte"},
		NotBefore:   now,
		ExpiresAt:   expiresAt,
		Nonce:       nonce,
	}

	// Assina o ticket (sem o campo signature)
	toSign := map[string]interface{}{
		"ticket_id":   ticket.TicketID,
		"machine_id":  ticket.MachineID,
		"technician_id": ticket.TechnicianID,
		"groups":      ticket.Groups,
		"not_before":  ticket.NotBefore.UTC().Format(time.RFC3339),
		"expires_at":  ticket.ExpiresAt.UTC().Format(time.RFC3339),
		"nonce":       ticket.Nonce,
	}
	payloadBytes, err := json.Marshal(toSign)
	if err != nil {
		http.Error(w, "Erro ao montar ticket", http.StatusInternalServerError)
		return
	}
	sig := ed25519.Sign(s.ticketSigningKey, payloadBytes)
	ticket.Signature = hex.EncodeToString(sig)

	// Registra ticket para uso unico
	s.ticketStoreMu.Lock()
	s.ticketStore[ticketID] = false
	s.ticketStoreMu.Unlock()

	slog.Info("Ticket de sessao emitido", "operator", operatorID, "machine", req.MachineID, "ticket_id", ticketID, "expires_at", expiresAt)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	_ = json.NewEncoder(w).Encode(SessionTicketResponse{
		Ticket:  ticket,
		Message: "Ticket emitido. Valido por 30 minutos e de uso unico.",
	})
}

/// Marca um ticket como usado (consulta pelo host após validação offline).
func (s *Server) markTicketUsed(ticketID string) bool {
	s.ticketStoreMu.Lock()
	defer s.ticketStoreMu.Unlock()
	if used, ok := s.ticketStore[ticketID]; ok && !used {
		s.ticketStore[ticketID] = true
		return true
	}
	return false
}

/// Verifica se um ticket ainda não foi usado (consulta de revogação/uso único).
func (s *Server) isTicketValid(ticketID string) bool {
	s.ticketStoreMu.Lock()
	defer s.ticketStoreMu.Unlock()
	if used, ok := s.ticketStore[ticketID]; ok {
		return !used
	}
	return false // ticket desconhecido é inválido
}

type AuditPayload struct {
	PDVPublicKey     string    `json:"pdv_public_key"`
	StartTime        time.Time `json:"start_time"`
	EndTime          time.Time `json:"end_time"`
	Reason           string    `json:"reason"`
	ConsentResult    string    `json:"consent_result"`
	BytesTransferred int64     `json:"bytes_transferred"`
}

func (s *Server) handleSessionAudit(w http.ResponseWriter, r *http.Request) {
	operatorID := r.Header.Get("X-Operator-ID")

	var req AuditPayload
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "JSON invalido", http.StatusBadRequest)
		return
	}

	record := db.AuditRecord{
		OperatorID:       operatorID,
		PDVPublicKey:     req.PDVPublicKey,
		StartTime:        req.StartTime,
		EndTime:          req.EndTime,
		Reason:           req.Reason,
		ConsentResult:    req.ConsentResult,
		BytesTransferred: req.BytesTransferred,
	}

	if err := s.db.LogAudit(record); err != nil {
		http.Error(w, "Erro ao gravar auditoria", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusCreated)
	_ = json.NewEncoder(w).Encode(map[string]string{"status": "auditoria gravada com sucesso"})
}

// securityHeadersMiddleware adiciona cabecalhos de seguranca a todas as respostas (S-11).
func securityHeadersMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload")
		w.Header().Set("X-Content-Type-Options", "nosniff")
		w.Header().Set("X-Frame-Options", "DENY")
		w.Header().Set("Content-Security-Policy", "default-src 'self'; connect-src 'self'; img-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'")
		w.Header().Set("Referrer-Policy", "strict-origin-when-cross-origin")
		w.Header().Set("Permissions-Policy", "geolocation=(), microphone=(), camera=()")
		next.ServeHTTP(w, r)
	})
}
