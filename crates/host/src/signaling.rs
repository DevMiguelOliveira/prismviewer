//! Cliente HTTP de Sinalização do Host com autenticação por assinatura Ed25519 e anti-replay.
//!
//! Conforme as Seções 2, 5 e 6.2 do AGENTS.md e Fase 14:
//! - Comunicação HTTPS com a API de sinalização em Go (`prysmview-api`).
//! - Autenticação por assinatura Ed25519 em cada requisição (cabeçalhos X-Pdv-Public-Key, X-Timestamp, X-Nonce, X-Signature).
//! - Mensagem assinada no formato: `"{timestamp}:{nonce}:" + body_bytes`.
//! - Janela anti-replay de timestamp e nonces únicos.

use ed25519_dalek::{Signer, SigningKey};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Converte um slice de bytes em representação hexadecimal minúscula.
pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

/// Constrói a mensagem binária para assinatura digital Ed25519.
///
/// Deve coincidir estritamente com o formato verificado pelo Go em `auth.VerifyHostSignature`:
/// `fmt.Sprintf("%d:%s:", timestamp, nonce) + payload`
pub fn build_signed_message(timestamp: i64, nonce: &str, body_bytes: &[u8]) -> Vec<u8> {
    let prefix = format!("{}:{}:", timestamp, nonce);
    let mut msg = Vec::with_capacity(prefix.len() + body_bytes.len());
    msg.extend_from_slice(prefix.as_bytes());
    msg.extend_from_slice(body_bytes);
    msg
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HostRegisterPayload {
    pub store_name: String,
    pub pos_number: i32,
    pub agent_version: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PdvStatusResponse {
    pub public_key: String,
    pub store_name: String,
    pub pos_number: i32,
    pub agent_version: String,
    pub status: String,
}

#[derive(Debug, Error)]
pub enum SignalingError {
    #[error("Erro de requisição HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Erro de serialização JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Falha na resposta da API ({status}): {body}")]
    ApiError { status: u16, body: String },

    #[error("Erro de cabeçalho HTTP: {0}")]
    InvalidHeader(String),
}

/// Cliente de sinalização utilizado pelo serviço Windows do Host (`prysmview-svc`).
pub struct HostSignalingClient {
    base_url: String,
    signing_key: SigningKey,
    http_client: reqwest::Client,
}

impl HostSignalingClient {
    /// Cria uma nova instância configurada com a URL base da API e o par de chaves Ed25519 do PDV.
    pub fn new(base_url: impl Into<String>, signing_key: SigningKey) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            signing_key,
            http_client: reqwest::Client::new(),
        }
    }

    /// Retorna a chave pública Ed25519 do PDV em formato hexadecimal.
    pub fn public_key_hex(&self) -> String {
        to_hex(self.signing_key.verifying_key().as_bytes())
    }

    /// Constrói os cabeçalhos de autenticação Ed25519 para um corpo de requisição.
    pub fn build_auth_headers(
        &self,
        timestamp: i64,
        nonce: &str,
        body_bytes: &[u8],
    ) -> Result<HeaderMap, SignalingError> {
        let msg_to_sign = build_signed_message(timestamp, nonce, body_bytes);
        let signature = self.signing_key.sign(&msg_to_sign);

        let pub_key_hex = self.public_key_hex();
        let sig_hex = to_hex(&signature.to_bytes());

        let mut headers = HeaderMap::new();

        headers.insert(
            HeaderName::from_static("x-pdv-public-key"),
            HeaderValue::from_str(&pub_key_hex)
                .map_err(|e| SignalingError::InvalidHeader(e.to_string()))?,
        );
        headers.insert(
            HeaderName::from_static("x-timestamp"),
            HeaderValue::from_str(&timestamp.to_string())
                .map_err(|e| SignalingError::InvalidHeader(e.to_string()))?,
        );
        headers.insert(
            HeaderName::from_static("x-nonce"),
            HeaderValue::from_str(nonce)
                .map_err(|e| SignalingError::InvalidHeader(e.to_string()))?,
        );
        headers.insert(
            HeaderName::from_static("x-signature"),
            HeaderValue::from_str(&sig_hex)
                .map_err(|e| SignalingError::InvalidHeader(e.to_string()))?,
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(headers)
    }

    /// Registra o PDV na API de sinalização enviando os metadados assinados via Ed25519.
    pub async fn register(&self, payload: &HostRegisterPayload) -> Result<String, SignalingError> {
        let body_bytes = serde_json::to_vec(payload)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let nonce = format!(
            "{:016x}{:016x}",
            rand::random::<u64>(),
            rand::random::<u64>()
        );

        let headers = self.build_auth_headers(timestamp, &nonce, &body_bytes)?;
        let url = format!("{}/api/v1/host/register", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .body(body_bytes)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(SignalingError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        Ok(body)
    }

    /// Consulta o status de um PDV registrado.
    pub async fn get_status(
        &self,
        public_key_hex: &str,
    ) -> Result<PdvStatusResponse, SignalingError> {
        let url = format!(
            "{}/api/v1/host/status?public_key={}",
            self.base_url, public_key_hex
        );

        let response = self.http_client.get(&url).send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(SignalingError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let resp: PdvStatusResponse = serde_json::from_str(&body)?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Verifier, VerifyingKey};
    use rand::rngs::OsRng;

    #[test]
    fn test_to_hex() {
        let bytes = [0x00, 0x01, 0x0f, 0x10, 0xff];
        assert_eq!(to_hex(&bytes), "00010f10ff");
    }

    #[test]
    fn test_build_signed_message_format() {
        let ts = 1700000000i64;
        let nonce = "a1b2c3d4e5f60718";
        let body = b"{\"status\":\"online\"}";

        let msg = build_signed_message(ts, nonce, body);
        let expected = b"1700000000:a1b2c3d4e5f60718:{\"status\":\"online\"}";
        assert_eq!(msg, expected);
    }

    #[test]
    fn test_signature_compatibility_with_go_verification() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let client = HostSignalingClient::new("http://127.0.0.1:8080", signing_key);

        let ts = 1700000000i64;
        let nonce = "test_nonce_12345";
        let body = b"{\"store_name\":\"Loja 01\",\"pos_number\":1}";

        let headers = client.build_auth_headers(ts, nonce, body).unwrap();

        let pub_key_hex = headers.get("x-pdv-public-key").unwrap().to_str().unwrap();
        let sig_hex = headers.get("x-signature").unwrap().to_str().unwrap();

        assert_eq!(pub_key_hex, client.public_key_hex());
        assert_eq!(sig_hex.len(), 128); // 64 bytes em hex = 128 chars

        // Simula verificação do Go: decodifica hex e valida ed25519
        let pub_key_bytes: Vec<u8> = (0..pub_key_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&pub_key_hex[i..i + 2], 16).unwrap())
            .collect();

        let sig_bytes: Vec<u8> = (0..sig_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&sig_hex[i..i + 2], 16).unwrap())
            .collect();

        let verifying_key = VerifyingKey::from_bytes(&pub_key_bytes.try_into().unwrap()).unwrap();
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes.try_into().unwrap());

        let msg_to_verify = build_signed_message(ts, nonce, body);
        assert!(verifying_key.verify(&msg_to_verify, &signature).is_ok());
    }

    #[test]
    fn test_host_register_payload_json_serialization() {
        let payload = HostRegisterPayload {
            store_name: "Loja Centro".to_string(),
            pos_number: 3,
            agent_version: "0.1.0".to_string(),
            status: "online".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"store_name\":\"Loja Centro\""));
        assert!(json.contains("\"pos_number\":3"));
        assert!(json.contains("\"agent_version\":\"0.1.0\""));
        assert!(json.contains("\"status\":\"online\""));
    }
}
