//! prysmview-viewer - Núcleo de estado e tipos do aplicativo do operador

pub mod api_client;
pub mod app;
pub mod canvas;
pub mod session;
pub mod strings;
pub mod theme;

pub use api_client::{
    ApiPdvItem, AuditPayload, LoginRequest, LoginResponse, SessionRequestPayload, SessionResponse,
    ViewerApiClient, ViewerApiError,
};
pub use canvas::{CanvasError, RemoteCanvas, TILE_SIZE};
pub use session::{
    collect_input_events, egui_key_to_abnt2_scancode, normalize_coordinates, SessionMetrics,
};

use serde::{Deserialize, Serialize};

/// Abas de navegação do console administrativo do técnico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminTab {
    Dashboard,
    DirectConnect,
    Stores,
    AuditHistory,
    Settings,
}

/// Filtro de status de conectividade para a lista de PDVs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterStatus {
    All,
    OnlineOnly,
    OfflineOnly,
}

/// Registro histórico de auditoria de sessões executadas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub timestamp: String,
    pub operator_name: String,
    pub pdv_id: String,
    pub store_name: String,
    pub duration: String,
    pub reason: String,
    pub status: String,
    pub bytes_transferred: String,
}

/// Informações cadastrais e de telemetria avançada de um PDV na rede.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdvItem {
    pub id: String,
    pub store_name: String,
    pub is_online: bool,
    pub agent_version: String,
    pub public_key: String,
    pub last_seen: String,
    pub ip_address: String,
    pub os_info: String,
    pub resolution: String,
}

/// Estado global e de interface do operador no Viewer / Prysm Admin.
#[derive(Debug, Clone)]
pub struct ViewerState {
    pub operator_name: String,
    pub auth_token: Option<String>,
    pub server_url: String,
    pub login_operator_id: String,
    pub login_password: String,
    pub is_authenticated: bool,
    pub login_error: Option<String>,
    pub search_query: String,
    pub active_tab: AdminTab,
    pub filter_status: FilterStatus,
    pub selected_store: Option<String>,
    pub pdvs: Vec<PdvItem>,
    pub selected_pdv: Option<PdvItem>,
    pub inspected_pdv: Option<PdvItem>,
    pub session_code_input: String,
    pub direct_pdv_input: String,
    pub direct_code_input: String,
    pub is_connecting: bool,
    pub active_session: bool,
    pub status_message: Option<String>,
    pub audit_records: Vec<AuditRecord>,
    pub cloud_latency_ms: u64,
    pub target_fps: u32,
    pub zstd_level: i32,
    pub remember_user: bool,
    pub show_password: bool,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            operator_name: "Técnico".to_string(),
            auth_token: None,
            server_url: "https://api.prysmview.com".to_string(),
            login_operator_id: "admin".to_string(),
            login_password: String::new(),
            is_authenticated: false,
            login_error: None,
            search_query: String::new(),
            active_tab: AdminTab::Dashboard,
            filter_status: FilterStatus::All,
            selected_store: None,
            pdvs: Vec::new(),
            selected_pdv: None,
            inspected_pdv: None,
            session_code_input: String::new(),
            direct_pdv_input: String::new(),
            direct_code_input: String::new(),
            is_connecting: false,
            active_session: false,
            status_message: None,
            audit_records: Vec::new(),
            cloud_latency_ms: 18,
            target_fps: 30,
            zstd_level: 1,
            remember_user: true,
            show_password: false,
        }
    }
}

impl ViewerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Autentica o operador com sucesso e transiciona para o painel de PDVs.
    pub fn login_success(&mut self, operator_id: &str, token: String) {
        self.operator_name = operator_id.to_string();
        self.auth_token = Some(token);
        self.is_authenticated = true;
        self.login_error = None;
        self.login_password.clear();
        self.status_message = Some(format!("Conectado como {}", operator_id));
    }

    /// Encerra a sessão do operador e retorna para a tela de login.
    pub fn logout(&mut self) {
        self.auth_token = None;
        self.is_authenticated = false;
        self.selected_pdv = None;
        self.active_session = false;
        self.status_message = Some("Sessão encerrada com sucesso.".to_string());
    }

    /// Retorna os PDVs filtrados por busca textual, status (online/offline) e loja selecionada.
    pub fn filter_pdvs(&self) -> Vec<&PdvItem> {
        let query = self.search_query.trim().to_lowercase();
        self.pdvs
            .iter()
            .filter(|p| {
                // Filtro de status
                let matches_status = match self.filter_status {
                    FilterStatus::All => true,
                    FilterStatus::OnlineOnly => p.is_online,
                    FilterStatus::OfflineOnly => !p.is_online,
                };
                if !matches_status {
                    return false;
                }

                // Filtro de loja
                if let Some(ref store) = self.selected_store {
                    if !p.store_name.eq_ignore_ascii_case(store) {
                        return false;
                    }
                }

                // Filtro de busca textual
                if query.is_empty() {
                    true
                } else {
                    p.id.to_lowercase().contains(&query)
                        || p.store_name.to_lowercase().contains(&query)
                        || p.ip_address.contains(&query)
                }
            })
            .collect()
    }

    /// Retorna o número de caixas online cadastrados.
    pub fn online_count(&self) -> usize {
        self.pdvs.iter().filter(|p| p.is_online).count()
    }

    /// Retorna o número de caixas offline cadastrados.
    pub fn offline_count(&self) -> usize {
        self.pdvs.iter().filter(|p| !p.is_online).count()
    }

    /// Retorna a lista de nomes únicos de lojas monitoradas.
    pub fn unique_stores(&self) -> Vec<String> {
        let mut stores: Vec<String> = self.pdvs.iter().map(|p| p.store_name.clone()).collect();
        stores.sort();
        stores.dedup();
        stores
    }

    /// Seleciona um PDV para inspeção detalhada de hardware / telemetria.
    pub fn inspect_pdv(&mut self, pdv: PdvItem) {
        self.inspected_pdv = Some(pdv);
    }

    /// Limpa o PDV em inspeção.
    pub fn close_inspector(&mut self) {
        self.inspected_pdv = None;
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
                ip_address: "192.168.1.101".to_string(),
                os_info: "Windows 11 Pro 23H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
            },
            PdvItem {
                id: "PDV-02".to_string(),
                store_name: "Loja Shopping 202".to_string(),
                is_online: false,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pub_key_2".to_string(),
                last_seen: "10 min atrás".to_string(),
                ip_address: "192.168.2.102".to_string(),
                os_info: "Windows 10 Enterprise 22H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
            },
            PdvItem {
                id: "PDV-03".to_string(),
                store_name: "Loja Centro 101".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pub_key_3".to_string(),
                last_seen: "Agora".to_string(),
                ip_address: "192.168.1.103".to_string(),
                os_info: "Windows 11 Pro 23H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
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

    #[test]
    fn test_login_success_and_logout() {
        let mut state = ViewerState::new();
        assert!(!state.is_authenticated);
        assert_eq!(state.auth_token, None);

        state.login_success("admin_master", "token_xyz".to_string());
        assert!(state.is_authenticated);
        assert_eq!(state.operator_name, "admin_master");
        assert_eq!(state.auth_token.as_deref(), Some("token_xyz"));
        assert!(state
            .status_message
            .as_ref()
            .unwrap()
            .contains("admin_master"));

        state.logout();
        assert!(!state.is_authenticated);
        assert_eq!(state.auth_token, None);
        assert_eq!(state.selected_pdv, None);
    }

    #[test]
    fn test_login_fields_default_and_toggle() {
        let mut state = ViewerState::new();
        assert!(state.remember_user);
        assert!(!state.show_password);

        state.show_password = true;
        assert!(state.show_password);

        state.remember_user = false;
        assert!(!state.remember_user);
    }
}
