//! Cliente HTTP de Sinalização do Viewer para autenticação, listagem e auditoria.
//!
//! Conforme as Seções 2, 5 e 6.2 do AGENTS.md e Fase 14:
//! - Comunicação HTTPS com a API de sinalização em Go (`prysmview-api`).
//! - Autenticação por token Bearer opaco retornado no login do operador.
//! - Listagem de PDVs autorizados para o operador.
//! - Autorização de sessão e registro de auditoria fim a fim.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub operator_id: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiPdvItem {
    pub public_key: String,
    pub store_name: String,
    pub pos_number: i32,
    pub agent_version: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRequestPayload {
    pub pdv_public_key: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionResponse {
    pub authorized: bool,
    pub operator: String,
    pub pdv: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditPayload {
    pub pdv_public_key: String,
    pub start_time: String,
    pub end_time: String,
    pub reason: String,
    pub consent_result: String,
    pub bytes_transferred: i64,
}

#[derive(Debug, Error)]
pub enum ViewerApiError {
    #[error("Erro de requisição HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Erro de serialização JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Falha na resposta da API ({status}): {body}")]
    ApiError { status: u16, body: String },

    #[error("Operador não autenticado (token ausente)")]
    Unauthorized,
}

/// Cliente de API do operador utilizado pelo aplicativo do Viewer (`prysmview-viewer`).
pub struct ViewerApiClient {
    base_url: String,
    token: Option<String>,
    http_client: reqwest::Client,
}

impl ViewerApiClient {
    /// Cria uma nova instância configurada com a URL base da API de sinalização.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: None,
            http_client: reqwest::Client::new(),
        }
    }

    /// Define o token de autenticação Bearer ativo do operador.
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    /// Retorna o token de autenticação atual, se definido.
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Constrói os cabeçalhos de requisição incluindo o token Bearer caso presente.
    pub fn build_headers(&self) -> Result<HeaderMap, ViewerApiError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(token) = &self.token {
            let auth_val = format!("Bearer {}", token);
            if let Ok(val) = HeaderValue::from_str(&auth_val) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        Ok(headers)
    }

    /// Autentica o operador na API e armazena o token retornado na instância.
    pub async fn login(
        &mut self,
        operator_id: &str,
        password: &str,
    ) -> Result<LoginResponse, ViewerApiError> {
        let url = format!("{}/api/v1/operator/login", self.base_url);
        let req_body = LoginRequest {
            operator_id: operator_id.to_string(),
            password: password.to_string(),
        };

        let response = self.http_client.post(&url).json(&req_body).send().await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(ViewerApiError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let resp: LoginResponse = serde_json::from_str(&body)?;
        self.set_token(resp.token.clone());
        Ok(resp)
    }

    /// Lista os PDVs cadastrados aos quais o operador tem permissão de acesso.
    pub async fn list_pdvs(&self) -> Result<Vec<ApiPdvItem>, ViewerApiError> {
        if self.token.is_none() {
            return Err(ViewerApiError::Unauthorized);
        }

        let url = format!("{}/api/v1/operator/pdvs", self.base_url);
        let headers = self.build_headers()?;

        let response = self.http_client.get(&url).headers(headers).send().await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(ViewerApiError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let pdvs: Vec<ApiPdvItem> = serde_json::from_str(&body)?;
        Ok(pdvs)
    }

    /// Solicita autorização para iniciar sessão com um PDV específico.
    pub async fn request_session(
        &self,
        pdv_public_key: &str,
        reason: &str,
    ) -> Result<SessionResponse, ViewerApiError> {
        if self.token.is_none() {
            return Err(ViewerApiError::Unauthorized);
        }

        let url = format!("{}/api/v1/session/request", self.base_url);
        let req_body = SessionRequestPayload {
            pdv_public_key: pdv_public_key.to_string(),
            reason: reason.to_string(),
        };

        let headers = self.build_headers()?;
        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&req_body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(ViewerApiError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        let resp: SessionResponse = serde_json::from_str(&body)?;
        Ok(resp)
    }

    /// Envia o registro de auditoria ao encerrar a sessão remota.
    pub async fn send_audit(&self, payload: &AuditPayload) -> Result<(), ViewerApiError> {
        if self.token.is_none() {
            return Err(ViewerApiError::Unauthorized);
        }

        let url = format!("{}/api/v1/session/audit", self.base_url);
        let headers = self.build_headers()?;

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(payload)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(ViewerApiError::ApiError {
                status: status.as_u16(),
                body,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_payload_serialization() {
        let req = LoginRequest {
            operator_id: "op-101".to_string(),
            password: "secret_password".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"operator_id\":\"op-101\""));
        assert!(json.contains("\"password\":\"secret_password\""));

        let res_json = r#"{"token":"test-token-123","expires_at":"2026-10-24T20:00:00Z"}"#;
        let res: LoginResponse = serde_json::from_str(res_json).unwrap();
        assert_eq!(res.token, "test-token-123");
        assert_eq!(res.expires_at, "2026-10-24T20:00:00Z");
    }

    #[test]
    fn test_token_management_and_auth_headers() {
        let mut client = ViewerApiClient::new("http://127.0.0.1:8080/");
        assert_eq!(client.token(), None);

        // Sem token, header Authorization não é injetado
        let headers = client.build_headers().unwrap();
        assert!(headers.get(AUTHORIZATION).is_none());

        client.set_token("token_abc_123");
        assert_eq!(client.token(), Some("token_abc_123"));

        let headers_with_token = client.build_headers().unwrap();
        assert_eq!(
            headers_with_token
                .get(AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap(),
            "Bearer token_abc_123"
        );
    }

    #[test]
    fn test_session_request_and_response_serialization() {
        let req = SessionRequestPayload {
            pdv_public_key: "abcdef0123456789".to_string(),
            reason: "Suporte caixa travado".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"pdv_public_key\":\"abcdef0123456789\""));
        assert!(json.contains("\"reason\":\"Suporte caixa travado\""));

        let res_json = r#"{"authorized":true,"operator":"op-101","pdv":"abcdef0123456789","message":"Sessao autorizada."}"#;
        let res: SessionResponse = serde_json::from_str(res_json).unwrap();
        assert!(res.authorized);
        assert_eq!(res.operator, "op-101");
        assert_eq!(res.pdv, "abcdef0123456789");
    }

    #[test]
    fn test_audit_payload_serialization() {
        let audit = AuditPayload {
            pdv_public_key: "pubkey123".to_string(),
            start_time: "2026-10-24T18:00:00Z".to_string(),
            end_time: "2026-10-24T18:30:00Z".to_string(),
            reason: "Ajuste de leitor fiscal".to_string(),
            consent_result: "accepted".to_string(),
            bytes_transferred: 1048576,
        };

        let json = serde_json::to_string(&audit).unwrap();
        assert!(json.contains("\"pdv_public_key\":\"pubkey123\""));
        assert!(json.contains("\"bytes_transferred\":1048576"));
        assert!(json.contains("\"consent_result\":\"accepted\""));
    }

    #[tokio::test]
    async fn test_unauthorized_when_calling_without_token() {
        let client = ViewerApiClient::new("http://127.0.0.1:8080");
        let list_res = client.list_pdvs().await;
        assert!(matches!(list_res, Err(ViewerApiError::Unauthorized)));

        let sess_res = client.request_session("pubkey", "motivo").await;
        assert!(matches!(sess_res, Err(ViewerApiError::Unauthorized)));

        let audit = AuditPayload {
            pdv_public_key: "pub".to_string(),
            start_time: "".to_string(),
            end_time: "".to_string(),
            reason: "".to_string(),
            consent_result: "".to_string(),
            bytes_transferred: 0,
        };
        let audit_res = client.send_audit(&audit).await;
        assert!(matches!(audit_res, Err(ViewerApiError::Unauthorized)));
    }
}
