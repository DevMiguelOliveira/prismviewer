//! prysmview-viewer - Centralização de Textos em Português do Brasil (U-12)
//!
//! Conforme a regra U-12 do roadmap:
//! - Centralizar todos os textos em um arquivo de strings.
//! - Utilizar linguagem clara e acessível para operadores técnicos e caixas de loja.
//! - Evitar jargões técnicos excessivos nas telas do caixa (ex: "Arquivos verificados" em vez de "SHA-256").

// ==============================================================================
// 1. TÍTULOS E CABEÇALHOS
// ==============================================================================
pub const APP_TITLE: &str = "PrysmView - Suporte Remoto PDV";
pub const APP_ADMIN_HEADING: &str = "PRYSM ADMIN";
pub const APP_ADMIN_SUBHEADING: &str = "Central de Suporte & Gestão de PDVs";
pub const HOST_TITLE: &str = "PrysmHost - Suporte Remoto do Caixa";

// ==============================================================================
// 2. AUTENTICAÇÃO E LOGIN (U-05 / S-08)
// ==============================================================================
pub const LOGIN_TITLE: &str = "Acesso Técnico PrysmAdmin";
pub const LOGIN_SUBTITLE: &str = "Identifique-se com sua credencial corporativa autorizada";
pub const LOGIN_ENTRA_BTN: &str = "Entrar com Microsoft 365 (Entra ID)";
pub const LOGIN_MANUAL_TAB: &str = "Acesso de Emergência";
pub const LOGIN_USER_LABEL: &str = "Identificador / E-mail do Técnico:";
pub const LOGIN_PASS_LABEL: &str = "Senha com Segundo Fator (MFA):";
pub const LOGIN_SUBMIT_BTN: &str = "Entrar no Sistema";
pub const LOGIN_LOGOUT_BTN: &str = "Encerrar Sessão do Técnico";
pub const LOGIN_LOCKED_MSG: &str = "Acesso bloqueado por 15 minutos após tentativas inválidas repetidas.";

// ==============================================================================
// 3. ESTADOS E STATUS DE SESSÃO
// ==============================================================================
pub const STATUS_IDLE: &str = "Aguardando solicitação de suporte...";
pub const STATUS_CONNECTING: &str = "Estabelecendo conexão segura E2E...";
pub const STATUS_ACTIVE: &str = "Sessão remota ativa";
pub const STATUS_TERMINATED: &str = "Sessão finalizada com sucesso";
pub const STATUS_SECURE_BADGE: &str = "Criptografia E2E Ativa (TLS 1.3)";

// ==============================================================================
// 4. PERMISSÕES E CONSENTIMENTO (S-06)
// ==============================================================================
pub const PERM_INPUT: &str = "Permitir controle de mouse e teclado";
pub const PERM_FILES: &str = "Permitir envio de arquivos (verificados)";
pub const PERM_CLIPBOARD: &str = "Permitir compartilhamento de textos copiados";

// ==============================================================================
// 5. AÇÕES E BOTÕES
// ==============================================================================
pub const BTN_CONNECT: &str = "Iniciar Acesso";
pub const BTN_TERMINATE: &str = "Encerrar Sessão Imediatamente";
pub const BTN_COPY_ID: &str = "Copiar Identificador";
pub const BTN_REFRESH: &str = "Atualizar Lista";
pub const BTN_AUDIT: &str = "Ver Histórico de Auditoria";

// ==============================================================================
// 6. MENSAGENS DE ERRO E SEGURANÇA
// ==============================================================================
pub const ERR_TICKET_EXPIRED: &str = "O ticket do chamado expirou. Solicite a reabertura no sistema.";
pub const ERR_TICKET_REVOKED: &str = "Acesso cancelado centralmente pela equipe de segurança.";
pub const ERR_UNAUTHORIZED: &str = "Você não possui permissão para acessar este terminal de caixa.";
pub const ERR_NETWORK_FAIL: &str = "Falha de comunicação com o PDV. Verifique a conexão de rede da loja.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strings_are_non_empty_and_in_pt_br() {
        assert!(!APP_TITLE.is_empty());
        assert!(LOGIN_ENTRA_BTN.contains("Microsoft 365"));
        assert!(PERM_FILES.contains("verificados"));
        assert!(!BTN_TERMINATE.is_empty());
    }
}
