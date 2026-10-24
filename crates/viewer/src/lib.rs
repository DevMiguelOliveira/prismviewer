//! prysmview-viewer - Núcleo de estado e tipos do aplicativo do operador

pub mod app;
pub mod canvas;
pub mod session;

pub use canvas::{CanvasError, RemoteCanvas, TILE_SIZE};
pub use session::{
    collect_input_events, egui_key_to_abnt2_scancode, normalize_coordinates, SessionMetrics,
};

use serde::{Deserialize, Serialize};

/// Informações cadastrais e de telemetria de um PDV na rede.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdvItem {
    pub id: String,
    pub store_name: String,
    pub is_online: bool,
    pub agent_version: String,
    pub public_key: String,
    pub last_seen: String,
}

/// Estado global e de interface do operador no Viewer.
#[derive(Debug, Clone)]
pub struct ViewerState {
    pub operator_name: String,
    pub auth_token: Option<String>,
    pub search_query: String,
    pub pdvs: Vec<PdvItem>,
    pub selected_pdv: Option<PdvItem>,
    pub session_code_input: String,
    pub is_connecting: bool,
    pub active_session: bool,
    pub status_message: Option<String>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            operator_name: "Operador Suporte".to_string(),
            auth_token: None,
            search_query: String::new(),
            pdvs: Vec::new(),
            selected_pdv: None,
            session_code_input: String::new(),
            is_connecting: false,
            active_session: false,
            status_message: None,
        }
    }
}

impl ViewerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retorna os PDVs filtrados pelo termo de busca (nome da loja ou ID do PDV).
    pub fn filter_pdvs(&self) -> Vec<&PdvItem> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            self.pdvs.iter().collect()
        } else {
            self.pdvs
                .iter()
                .filter(|p| {
                    p.id.to_lowercase().contains(&query)
                        || p.store_name.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    /// Seleciona um PDV para iniciar fluxo de conexão e abre o modal de código.
    pub fn select_pdv(&mut self, pdv: PdvItem) {
        self.selected_pdv = Some(pdv);
        self.session_code_input.clear();
        self.status_message = None;
    }

    /// Fecha o diálogo modal de conexão.
    pub fn close_modal(&mut self) {
        self.selected_pdv = None;
        self.session_code_input.clear();
        self.is_connecting = false;
    }

    /// Define e sanitiza o código de sessão de 8 dígitos.
    pub fn set_session_code(&mut self, code: &str) {
        // Mantém apenas dígitos e trunca em 8 caracteres
        self.session_code_input = code
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(8)
            .collect();
    }

    /// Valida se o código de sessão digitado possui exatamente 8 dígitos numéricos.
    pub fn validate_session_code(&self) -> bool {
        self.session_code_input.len() == 8
            && self.session_code_input.chars().all(|c| c.is_ascii_digit())
    }

    /// Encerra a sessão ativa e redefine o estado.
    pub fn clear_session(&mut self) {
        self.active_session = false;
        self.is_connecting = false;
        self.selected_pdv = None;
        self.session_code_input.clear();
        self.status_message = Some("Sessão finalizada.".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pdvs() -> Vec<PdvItem> {
        vec![
            PdvItem {
                id: "PDV-01".to_string(),
                store_name: "Loja Centro 101".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pub_key_1".to_string(),
                last_seen: "Agora".to_string(),
            },
            PdvItem {
                id: "PDV-02".to_string(),
                store_name: "Loja Shopping 202".to_string(),
                is_online: false,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pub_key_2".to_string(),
                last_seen: "10 min atrás".to_string(),
            },
            PdvItem {
                id: "PDV-03".to_string(),
                store_name: "Loja Centro 101".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pub_key_3".to_string(),
                last_seen: "Agora".to_string(),
            },
        ]
    }

    #[test]
    fn test_filter_pdvs_empty_query() {
        let mut state = ViewerState::new();
        state.pdvs = sample_pdvs();
        assert_eq!(state.filter_pdvs().len(), 3);
    }

    #[test]
    fn test_filter_pdvs_by_store_name() {
        let mut state = ViewerState::new();
        state.pdvs = sample_pdvs();
        state.search_query = "Shopping".to_string();
        let filtered = state.filter_pdvs();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "PDV-02");
    }

    #[test]
    fn test_filter_pdvs_by_id() {
        let mut state = ViewerState::new();
        state.pdvs = sample_pdvs();
        state.search_query = "pdv-03".to_string();
        let filtered = state.filter_pdvs();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "PDV-03");
    }

    #[test]
    fn test_session_code_validation_and_sanitization() {
        let mut state = ViewerState::new();

        // Código parcial
        state.set_session_code("12345");
        assert_eq!(state.session_code_input, "12345");
        assert!(!state.validate_session_code());

        // Código com caracteres não numéricos deve filtrar
        state.set_session_code("12-34-56-78");
        assert_eq!(state.session_code_input, "12345678");
        assert!(state.validate_session_code());

        // Código excedendo 8 dígitos deve truncar em 8
        state.set_session_code("123456789999");
        assert_eq!(state.session_code_input, "12345678");
        assert!(state.validate_session_code());
    }

    #[test]
    fn test_select_and_clear_session() {
        let mut state = ViewerState::new();
        let pdv = sample_pdvs().remove(0);

        state.select_pdv(pdv.clone());
        assert_eq!(state.selected_pdv, Some(pdv));

        state.clear_session();
        assert!(!state.active_session);
        assert_eq!(state.selected_pdv, None);
        assert_eq!(state.status_message.as_deref(), Some("Sessão finalizada."));
    }
}
