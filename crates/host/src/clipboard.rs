//! Sincronização de área de transferência (clipboard) de texto com limite estrito de 1 MB.
//!
//! Em conformidade com a Seção 3 e 9 do AGENTS.md:
//! - Texto apenas, com limite máximo de 1 MiB (1.048.576 bytes).
//! - Desativável pelo usuário local no PDV (política de consentimento e privacidade).
//! - Rejeição prévia de payloads excessivos sem causar estouro de memória.

use thiserror::Error;

/// Limite máximo de texto para a área de transferência: 1 MiB
pub const MAX_CLIPBOARD_BYTES: usize = 1024 * 1024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClipboardError {
    #[error("Área de transferência desativada pela política local do PDV")]
    DisabledByPosPolicy,

    #[error("Texto excede o limite máximo de 1 MB: recebido {size} bytes, limite é {max} bytes")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Texto inválido para sincronização: {0}")]
    InvalidText(String),
}

/// Gerenciador de sincronização da área de transferência do Host
#[derive(Debug, Clone)]
pub struct ClipboardManager {
    enabled: bool,
    last_synced_text: Option<String>,
}

impl ClipboardManager {
    /// Cria um novo gerenciador com o estado inicial da política do PDV.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_synced_text: None,
        }
    }

    /// Retorna se o compartilhamento de clipboard está ativo no PDV.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Habilita ou desabilita a sincronização de clipboard pelo usuário local.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.last_synced_text = None;
        }
    }

    /// Processa texto recebido do Viewer para aplicação no clipboard local.
    pub fn handle_incoming_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        if !self.enabled {
            return Err(ClipboardError::DisabledByPosPolicy);
        }

        if text.len() > MAX_CLIPBOARD_BYTES {
            return Err(ClipboardError::PayloadTooLarge {
                size: text.len(),
                max: MAX_CLIPBOARD_BYTES,
            });
        }

        self.last_synced_text = Some(text.to_string());
        Ok(())
    }

    /// Prepara texto capturado localmente para envio ao Viewer.
    ///
    /// Retorna `Ok(Some(text))` se o texto for novo e válido, ou `Ok(None)` se
    /// for idêntico ao último texto sincronizado (evitando tráfego repetido).
    pub fn prepare_outgoing_text(&mut self, text: &str) -> Result<Option<String>, ClipboardError> {
        if !self.enabled {
            return Err(ClipboardError::DisabledByPosPolicy);
        }

        if text.len() > MAX_CLIPBOARD_BYTES {
            return Err(ClipboardError::PayloadTooLarge {
                size: text.len(),
                max: MAX_CLIPBOARD_BYTES,
            });
        }

        if let Some(ref last) = self.last_synced_text {
            if last == text {
                return Ok(None);
            }
        }

        self.last_synced_text = Some(text.to_string());
        Ok(Some(text.to_string()))
    }

    /// Retorna o último texto sincronizado com sucesso.
    pub fn last_synced_text(&self) -> Option<&str> {
        self.last_synced_text.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_disabled_by_pos_policy() {
        let mut manager = ClipboardManager::new(false);
        assert!(!manager.is_enabled());

        let res = manager.handle_incoming_text("teste");
        assert_eq!(res, Err(ClipboardError::DisabledByPosPolicy));

        let res_out = manager.prepare_outgoing_text("teste");
        assert_eq!(res_out, Err(ClipboardError::DisabledByPosPolicy));
    }

    #[test]
    fn test_clipboard_accepts_valid_text() {
        let mut manager = ClipboardManager::new(true);
        assert!(manager.is_enabled());

        let res = manager.handle_incoming_text("Chave de Acesso NF-e: 12345");
        assert!(res.is_ok());
        assert_eq!(
            manager.last_synced_text(),
            Some("Chave de Acesso NF-e: 12345")
        );
    }

    #[test]
    fn test_clipboard_rejects_payload_above_1mb() {
        let mut manager = ClipboardManager::new(true);

        // Gera texto com 1 MiB + 1 byte
        let oversized = "a".repeat(MAX_CLIPBOARD_BYTES + 1);

        let err = manager.handle_incoming_text(&oversized);
        assert_eq!(
            err,
            Err(ClipboardError::PayloadTooLarge {
                size: MAX_CLIPBOARD_BYTES + 1,
                max: MAX_CLIPBOARD_BYTES,
            })
        );

        let err_out = manager.prepare_outgoing_text(&oversized);
        assert_eq!(
            err_out,
            Err(ClipboardError::PayloadTooLarge {
                size: MAX_CLIPBOARD_BYTES + 1,
                max: MAX_CLIPBOARD_BYTES,
            })
        );
    }

    #[test]
    fn test_clipboard_deduplicates_outgoing_identical_text() {
        let mut manager = ClipboardManager::new(true);

        let out1 = manager.prepare_outgoing_text("Copiar item");
        assert_eq!(out1.unwrap(), Some("Copiar item".to_string()));

        // Segundo envio com mesmo texto não gera tráfego redundante
        let out2 = manager.prepare_outgoing_text("Copiar item");
        assert_eq!(out2.unwrap(), None);

        // Texto diferente deve gerar tráfego
        let out3 = manager.prepare_outgoing_text("Novo item");
        assert_eq!(out3.unwrap(), Some("Novo item".to_string()));
    }

    #[test]
    fn test_clipboard_toggle_clears_synced_state() {
        let mut manager = ClipboardManager::new(true);
        manager.handle_incoming_text("Texto PDV").unwrap();
        assert_eq!(manager.last_synced_text(), Some("Texto PDV"));

        manager.set_enabled(false);
        assert!(!manager.is_enabled());
        assert_eq!(manager.last_synced_text(), None);
    }
}
