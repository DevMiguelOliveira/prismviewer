//! prysmview-host - Validador de Tickets de Sessão emitidos pelo Servidor (S-05, S-06)
//!
//! Conforme especificação S-05 e S-06:
//! - Ticket assinado com Ed25519 pelo servidor de sinalização.
//! - Contém machine_id, technician_id, groups, not_before, expires_at (máx 30 min), nonce e ticket_id.
//! - Validado offline pelo host.
//! - Uso único com cache de replay persistido localmente em arquivo.
//! - Revogação centralizada (por ticket_id ou technician_id) recebida via heartbeat.
//! - Com ticket válido, a sessão é iniciada imediatamente sem necessidade de consentimento
//!   interativo por clique (S-06), mantendo indicador visível, botão de encerramento e auditoria.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

/// Tolerância de clock skew para not_before (60 segundos).
pub const CLOCK_SKEW_TOLERANCE: Duration = Duration::seconds(60);

/// Janela máxima de validade permitida para um ticket (30 minutos + 5 segundos de folga).
pub const MAX_TICKET_VALIDITY_WINDOW: Duration = Duration::seconds(30 * 60 + 5);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TicketError {
    #[error("Assinatura digital Ed25519 do ticket inválida ou forjada")]
    InvalidSignature,

    #[error("Chave pública do servidor inválida")]
    InvalidServerKey,

    #[error("Ticket emitido para máquina diferente: esperado {expected}, recebido {actual}")]
    MachineMismatch { expected: String, actual: String },

    #[error("Ticket ainda não é válido (not_before {0})")]
    NotYetValid(String),

    #[error("Ticket expirado em {0}")]
    Expired(String),

    #[error("Janela de validade do ticket excede o limite máximo de 30 minutos")]
    ValidityWindowTooLong,

    #[error("Ticket reutilizado (replay detectado): {0}")]
    ReplayDetected(String),

    #[error("Ticket ou técnico revogado: {0}")]
    Revoked(String),

    #[error("Formato de data inválido no ticket: {0}")]
    InvalidDateFormat(String),

    #[error("Erro de E/S no armazenamento de replay: {0}")]
    Io(String),
}

/// Estrutura do Ticket de Sessão emitido pela API de sinalização (Go).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionTicket {
    pub ticket_id: String,
    pub machine_id: String,
    pub technician_id: String,
    pub groups: Vec<String>,
    pub not_before: String,
    pub expires_at: String,
    pub nonce: String,
    #[serde(default)]
    pub signature: String,
}

/// Dados da sessão autenticada após validação bem-sucedida do ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSessionTicket {
    pub ticket_id: String,
    pub machine_id: String,
    pub technician_id: String,
    pub groups: Vec<String>,
}

/// Trait para cache de replay e lista de revogação.
pub trait ReplayCache: Send + Sync {
    fn is_used(&self, ticket_id: &str) -> bool;
    fn mark_used(&mut self, ticket_id: &str, expires_at: OffsetDateTime) -> Result<(), TicketError>;
    fn is_revoked(&self, ticket_id: &str, technician_id: &str) -> bool;
    fn add_revocation(&mut self, id: &str) -> Result<(), TicketError>;
}

/// Implementação em memória para testes e fallback.
#[derive(Debug, Default, Clone)]
pub struct InMemoryReplayCache {
    used_tickets: HashSet<String>,
    revoked_ids: HashSet<String>,
}

impl InMemoryReplayCache {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ReplayCache for InMemoryReplayCache {
    fn is_used(&self, ticket_id: &str) -> bool {
        self.used_tickets.contains(ticket_id)
    }

    fn mark_used(&mut self, ticket_id: &str, _expires_at: OffsetDateTime) -> Result<(), TicketError> {
        self.used_tickets.insert(ticket_id.to_string());
        Ok(())
    }

    fn is_revoked(&self, ticket_id: &str, technician_id: &str) -> bool {
        self.revoked_ids.contains(ticket_id) || self.revoked_ids.contains(technician_id)
    }

    fn add_revocation(&mut self, id: &str) -> Result<(), TicketError> {
        self.revoked_ids.insert(id.to_string());
        Ok(())
    }
}

/// Registro persistido em arquivo para cada ticket usado.
#[derive(Debug, Serialize, Deserialize)]
struct ReplayStoreData {
    used_tickets: Vec<ReplayRecord>,
    revoked_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ReplayRecord {
    ticket_id: String,
    expires_at_unix: i64,
}

/// Cache de replay persistido localmente em arquivo com expurgo automático de tickets expirados.
pub struct FileReplayCache {
    path: PathBuf,
    data: ReplayStoreData,
}

impl FileReplayCache {
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> Result<Self, TicketError> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            let mut file = File::open(&path).map_err(|e| TicketError::Io(e.to_string()))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| TicketError::Io(e.to_string()))?;
            let data: ReplayStoreData = serde_json::from_str(&content).unwrap_or(ReplayStoreData {
                used_tickets: Vec::new(),
                revoked_ids: Vec::new(),
            });
            let mut cache = Self { path, data };
            cache.prune_expired(OffsetDateTime::now_utc());
            Ok(cache)
        } else {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let data = ReplayStoreData {
                used_tickets: Vec::new(),
                revoked_ids: Vec::new(),
            };
            let cache = Self { path, data };
            cache.persist()?;
            Ok(cache)
        }
    }

    fn persist(&self) -> Result<(), TicketError> {
        let serialized =
            serde_json::to_string_pretty(&self.data).map_err(|e| TicketError::Io(e.to_string()))?;
        let temp_path = self.path.with_extension("tmp");
        {
            let mut file = File::create(&temp_path).map_err(|e| TicketError::Io(e.to_string()))?;
            file.write_all(serialized.as_bytes())
                .map_err(|e| TicketError::Io(e.to_string()))?;
            file.flush().map_err(|e| TicketError::Io(e.to_string()))?;
        }
        fs::rename(&temp_path, &self.path).map_err(|e| TicketError::Io(e.to_string()))?;
        Ok(())
    }

    /// Remove do armazenamento tickets que já expiraram, mantendo o arquivo enxuto.
    pub fn prune_expired(&mut self, now: OffsetDateTime) {
        let now_unix = now.unix_timestamp();
        self.data
            .used_tickets
            .retain(|rec| rec.expires_at_unix > now_unix);
    }
}

impl ReplayCache for FileReplayCache {
    fn is_used(&self, ticket_id: &str) -> bool {
        self.data
            .used_tickets
            .iter()
            .any(|rec| rec.ticket_id == ticket_id)
    }

    fn mark_used(&mut self, ticket_id: &str, expires_at: OffsetDateTime) -> Result<(), TicketError> {
        self.data.used_tickets.push(ReplayRecord {
            ticket_id: ticket_id.to_string(),
            expires_at_unix: expires_at.unix_timestamp(),
        });
        self.persist()
    }

    fn is_revoked(&self, ticket_id: &str, technician_id: &str) -> bool {
        self.data.revoked_ids.iter().any(|id| id == ticket_id || id == technician_id)
    }

    fn add_revocation(&mut self, id: &str) -> Result<(), TicketError> {
        if !self.data.revoked_ids.contains(&id.to_string()) {
            self.data.revoked_ids.push(id.to_string());
            self.persist()?;
        }
        Ok(())
    }
}

/// Validador de Tickets de Sessão.
pub struct SessionTicketValidator {
    server_public_key: [u8; 32],
    local_machine_id: String,
}

impl SessionTicketValidator {
    pub fn new(server_public_key: [u8; 32], local_machine_id: String) -> Self {
        Self {
            server_public_key,
            local_machine_id,
        }
    }

    /// Valida integralmente um ticket de sessão offline.
    pub fn validate_ticket(
        &self,
        ticket: &SessionTicket,
        replay_cache: &mut dyn ReplayCache,
        now: OffsetDateTime,
    ) -> Result<ValidatedSessionTicket, TicketError> {
        // 1. Validação de machine_id (em tempo constante para evitar side-channel)
        let expected_bytes = self.local_machine_id.as_bytes();
        let actual_bytes = ticket.machine_id.as_bytes();
        if expected_bytes.len() != actual_bytes.len()
            || expected_bytes.ct_eq(actual_bytes).unwrap_u8() != 1
        {
            return Err(TicketError::MachineMismatch {
                expected: self.local_machine_id.clone(),
                actual: ticket.machine_id.clone(),
            });
        }

        // 2. Parse das datas RFC3339
        let not_before = OffsetDateTime::parse(&ticket.not_before, &Rfc3339)
            .map_err(|e| TicketError::InvalidDateFormat(format!("not_before: {}", e)))?;
        let expires_at = OffsetDateTime::parse(&ticket.expires_at, &Rfc3339)
            .map_err(|e| TicketError::InvalidDateFormat(format!("expires_at: {}", e)))?;

        // 3. Validação da janela máxima de 30 minutos
        if expires_at < not_before || (expires_at - not_before) > MAX_TICKET_VALIDITY_WINDOW {
            return Err(TicketError::ValidityWindowTooLong);
        }

        // 4. Verificação temporal contra o relógio atual (com tolerância de skew para not_before)
        if now + CLOCK_SKEW_TOLERANCE < not_before {
            return Err(TicketError::NotYetValid(ticket.not_before.clone()));
        }
        if now > expires_at {
            return Err(TicketError::Expired(ticket.expires_at.clone()));
        }

        // 5. Verificação de revogação centralizada
        if replay_cache.is_revoked(&ticket.ticket_id, &ticket.technician_id) {
            return Err(TicketError::Revoked(format!(
                "ticket_id={} ou technician_id={}",
                ticket.ticket_id, ticket.technician_id
            )));
        }

        // 6. Verificação de Replay / Reutilização
        if replay_cache.is_used(&ticket.ticket_id) {
            return Err(TicketError::ReplayDetected(ticket.ticket_id.clone()));
        }

        // 7. Verificação da assinatura Ed25519
        self.verify_signature(ticket)?;

        // 8. Marca o ticket como consumido no cache de uso único
        replay_cache.mark_used(&ticket.ticket_id, expires_at)?;

        Ok(ValidatedSessionTicket {
            ticket_id: ticket.ticket_id.clone(),
            machine_id: ticket.machine_id.clone(),
            technician_id: ticket.technician_id.clone(),
            groups: ticket.groups.clone(),
        })
    }

    /// Reconstrói o payload JSON canônico e verifica a assinatura Ed25519.
    fn verify_signature(&self, ticket: &SessionTicket) -> Result<(), TicketError> {
        let sig_bytes = decode_hex_signature(&ticket.signature)?;

        // BTreeMap garante ordenação alfabética estrita das chaves, idêntica ao json.Marshal em Go.
        let mut map = BTreeMap::new();
        map.insert("expires_at", serde_json::Value::String(ticket.expires_at.clone()));
        map.insert(
            "groups",
            serde_json::to_value(&ticket.groups).map_err(|e| TicketError::Io(e.to_string()))?,
        );
        map.insert("machine_id", serde_json::Value::String(ticket.machine_id.clone()));
        map.insert("nonce", serde_json::Value::String(ticket.nonce.clone()));
        map.insert("not_before", serde_json::Value::String(ticket.not_before.clone()));
        map.insert("technician_id", serde_json::Value::String(ticket.technician_id.clone()));
        map.insert("ticket_id", serde_json::Value::String(ticket.ticket_id.clone()));

        let canonical_bytes =
            serde_json::to_vec(&map).map_err(|e| TicketError::Io(e.to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&self.server_public_key)
            .map_err(|_| TicketError::InvalidServerKey)?;

        let signature = Signature::from_bytes(&sig_bytes);

        verifying_key
            .verify(&canonical_bytes, &signature)
            .map_err(|_| TicketError::InvalidSignature)?;

        Ok(())
    }
}

/// Decodifica uma string hexadecimal de 128 caracteres em 64 bytes de assinatura Ed25519.
fn decode_hex_signature(hex_str: &str) -> Result<[u8; 64], TicketError> {
    if hex_str.len() != 128 {
        return Err(TicketError::InvalidSignature);
    }
    let mut sig = [0u8; 64];
    for i in 0..64 {
        sig[i] = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
            .map_err(|_| TicketError::InvalidSignature)?;
    }
    Ok(sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn generate_test_ticket(
        signing_key: &SigningKey,
        machine_id: &str,
        technician_id: &str,
        not_before: OffsetDateTime,
        expires_at: OffsetDateTime,
    ) -> SessionTicket {
        let not_before_str = not_before.format(&Rfc3339).unwrap();
        let expires_at_str = expires_at.format(&Rfc3339).unwrap();
        let ticket_id = format!("tkt_{}", rand::random::<u32>());
        let nonce = format!("nonce_{}", rand::random::<u32>());
        let groups = vec!["suporte".to_string(), "ti".to_string()];

        let mut map = BTreeMap::new();
        map.insert("expires_at", serde_json::Value::String(expires_at_str.clone()));
        map.insert("groups", serde_json::to_value(&groups).unwrap());
        map.insert("machine_id", serde_json::Value::String(machine_id.to_string()));
        map.insert("nonce", serde_json::Value::String(nonce.clone()));
        map.insert("not_before", serde_json::Value::String(not_before_str.clone()));
        map.insert("technician_id", serde_json::Value::String(technician_id.to_string()));
        map.insert("ticket_id", serde_json::Value::String(ticket_id.clone()));

        let canonical_bytes = serde_json::to_vec(&map).unwrap();
        let signature = signing_key.sign(&canonical_bytes);
        let signature_hex = hex_encode(signature.to_bytes().as_ref());

        SessionTicket {
            ticket_id,
            machine_id: machine_id.to_string(),
            technician_id: technician_id.to_string(),
            groups,
            not_before: not_before_str,
            expires_at: expires_at_str,
            nonce,
            signature: signature_hex,
        }
    }

    fn hex_encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            use std::fmt::Write;
            write!(&mut s, "{:02x}", b).unwrap();
        }
        s
    }

    #[test]
    fn test_valid_ticket_accepted() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        let ticket = generate_test_ticket(
            &signing_key,
            "PDV-01",
            "tech-miguel",
            now - Duration::seconds(10),
            now + Duration::minutes(25),
        );

        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert!(res.is_ok());
        let session = res.unwrap();
        assert_eq!(session.machine_id, "PDV-01");
        assert_eq!(session.technician_id, "tech-miguel");
    }

    #[test]
    fn test_ticket_replay_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        let ticket = generate_test_ticket(
            &signing_key,
            "PDV-01",
            "tech-miguel",
            now - Duration::seconds(5),
            now + Duration::minutes(20),
        );

        // Primeiro uso: sucesso
        assert!(validator.validate_ticket(&ticket, &mut cache, now).is_ok());

        // Segundo uso com o mesmo ticket_id: rejeitado por ReplayDetected
        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert_eq!(
            res,
            Err(TicketError::ReplayDetected(ticket.ticket_id.clone()))
        );
    }

    #[test]
    fn test_ticket_expired_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        let ticket = generate_test_ticket(
            &signing_key,
            "PDV-01",
            "tech-miguel",
            now - Duration::minutes(35),
            now - Duration::minutes(5),
        );

        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert_eq!(res, Err(TicketError::Expired(ticket.expires_at)));
    }

    #[test]
    fn test_ticket_machine_mismatch_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        let ticket = generate_test_ticket(
            &signing_key,
            "PDV-02", // Outra máquina
            "tech-miguel",
            now - Duration::seconds(5),
            now + Duration::minutes(20),
        );

        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert_eq!(
            res,
            Err(TicketError::MachineMismatch {
                expected: "PDV-01".to_string(),
                actual: "PDV-02".to_string(),
            })
        );
    }

    #[test]
    fn test_ticket_tampered_signature_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        let mut ticket = generate_test_ticket(
            &signing_key,
            "PDV-01",
            "tech-miguel",
            now - Duration::seconds(5),
            now + Duration::minutes(20),
        );

        // Adulteração de dados do ticket (elevação de grupo)
        ticket.groups.push("superadmin".to_string());

        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert_eq!(res, Err(TicketError::InvalidSignature));
    }

    #[test]
    fn test_ticket_revocation_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        let ticket = generate_test_ticket(
            &signing_key,
            "PDV-01",
            "tech-demitido",
            now - Duration::seconds(5),
            now + Duration::minutes(20),
        );

        // Simula revogação centralizada recebida no heartbeat
        cache.add_revocation("tech-demitido").unwrap();

        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert!(matches!(res, Err(TicketError::Revoked(_))));
    }

    #[test]
    fn test_ticket_validity_window_too_long_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let server_pub = signing_key.verifying_key().to_bytes();
        let validator = SessionTicketValidator::new(server_pub, "PDV-01".to_string());
        let mut cache = InMemoryReplayCache::new();

        let now = OffsetDateTime::now_utc();
        // Ticket tentando ter 2 horas de validade (máximo permitido é 30 minutos)
        let ticket = generate_test_ticket(
            &signing_key,
            "PDV-01",
            "tech-miguel",
            now - Duration::seconds(5),
            now + Duration::hours(2),
        );

        let res = validator.validate_ticket(&ticket, &mut cache, now);
        assert_eq!(res, Err(TicketError::ValidityWindowTooLong));
    }

    #[test]
    fn test_file_replay_cache_persistence() {
        let temp_dir = std::env::temp_dir().join(format!("replay_test_{}", rand::random::<u32>()));
        let file_path = temp_dir.join("replay_cache.json");

        {
            let mut cache = FileReplayCache::open_or_create(&file_path).unwrap();
            let now = OffsetDateTime::now_utc();
            cache
                .mark_used("tkt_alpha", now + Duration::minutes(15))
                .unwrap();
            cache.add_revocation("revoked_tech_1").unwrap();
            assert!(cache.is_used("tkt_alpha"));
            assert!(cache.is_revoked("unknown", "revoked_tech_1"));
        }

        // Reabre do disco e verifica persistência
        {
            let cache = FileReplayCache::open_or_create(&file_path).unwrap();
            assert!(cache.is_used("tkt_alpha"));
            assert!(!cache.is_used("tkt_beta"));
            assert!(cache.is_revoked("unknown", "revoked_tech_1"));
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
