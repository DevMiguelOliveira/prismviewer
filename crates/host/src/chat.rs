//! Gerenciador de chat de texto simples bidirecional do Host.
//!
//! Em conformidade com as Seções 3 e 8 do AGENTS.md:
//! - Mensagens de chat de texto simples.
//! - Limite estrito de 4 KiB por mensagem (`MAX_CHAT_PAYLOAD`).
//! - Histórico delimitado em memória (máximo 500 mensagens) para evitar estouro em PDVs.
//! - Sem unwrap() ou expect() em caminhos de rede e dados.

use prysmview_core::messages::ChatMessage;
use thiserror::Error;

/// Limite máximo de bytes por texto de chat: 4 KiB
pub const MAX_CHAT_TEXT_LEN: usize = 4 * 1024;

/// Limite máximo de mensagens no histórico em memória
pub const MAX_CHAT_HISTORY: usize = 500;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChatError {
    #[error(
        "Texto da mensagem excede o limite de 4 KiB: recebido {size} bytes, limite é {max} bytes"
    )]
    MessageTooLarge { size: usize, max: usize },

    #[error("Mensagem de chat vazia")]
    EmptyMessage,

    #[error("Remetente inválido ou em branco")]
    InvalidSender,
}

/// Gerenciador de sessão de chat de texto do Host
#[derive(Debug, Clone)]
pub struct ChatManager {
    history: Vec<ChatMessage>,
    next_id: u64,
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatManager {
    /// Cria um novo gerenciador de chat vazio.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            next_id: 1,
        }
    }

    /// Cria e valida uma nova mensagem de chat local para transmissão pela rede.
    pub fn create_outgoing_message(
        &mut self,
        sender: impl Into<String>,
        text: impl Into<String>,
        timestamp_ms: u64,
    ) -> Result<ChatMessage, ChatError> {
        let sender = sender.into();
        let text = text.into();

        if sender.trim().is_empty() {
            return Err(ChatError::InvalidSender);
        }
        if text.is_empty() {
            return Err(ChatError::EmptyMessage);
        }
        if text.len() > MAX_CHAT_TEXT_LEN {
            return Err(ChatError::MessageTooLarge {
                size: text.len(),
                max: MAX_CHAT_TEXT_LEN,
            });
        }

        let id = self.next_id;
        self.next_id += 1;

        let msg = ChatMessage {
            id,
            sender,
            text,
            timestamp_ms,
        };

        self.append_to_history(msg.clone());
        Ok(msg)
    }

    /// Processa e armazena uma mensagem de chat recebida remotamente.
    pub fn handle_incoming_message(&mut self, msg: ChatMessage) -> Result<(), ChatError> {
        if msg.sender.trim().is_empty() {
            return Err(ChatError::InvalidSender);
        }
        if msg.text.is_empty() {
            return Err(ChatError::EmptyMessage);
        }
        if msg.text.len() > MAX_CHAT_TEXT_LEN {
            return Err(ChatError::MessageTooLarge {
                size: msg.text.len(),
                max: MAX_CHAT_TEXT_LEN,
            });
        }

        // Garante que o contador de ID local não colida com IDs recebidos
        if msg.id >= self.next_id {
            self.next_id = msg.id + 1;
        }

        self.append_to_history(msg);
        Ok(())
    }

    /// Adiciona a mensagem ao histórico, garantindo que o limite de capacidade seja respeitado.
    fn append_to_history(&mut self, msg: ChatMessage) {
        if self.history.len() >= MAX_CHAT_HISTORY {
            self.history.remove(0);
        }
        self.history.push(msg);
    }

    /// Retorna a lista de mensagens no histórico.
    pub fn history(&self) -> &[ChatMessage] {
        &self.history
    }

    /// Limpa o histórico de chat da sessão.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_create_outgoing_valid() {
        let mut manager = ChatManager::new();
        let msg = manager
            .create_outgoing_message("Operador Suporte", "Reiniciando aplicação PDV", 1700000000)
            .unwrap();

        assert_eq!(msg.id, 1);
        assert_eq!(msg.sender, "Operador Suporte");
        assert_eq!(msg.text, "Reiniciando aplicação PDV");
        assert_eq!(manager.history().len(), 1);
    }

    #[test]
    fn test_chat_rejects_empty_or_whitespace_sender() {
        let mut manager = ChatManager::new();
        let res1 = manager.create_outgoing_message("", "Olá", 100);
        assert_eq!(res1, Err(ChatError::InvalidSender));

        let res2 = manager.create_outgoing_message("   ", "Olá", 100);
        assert_eq!(res2, Err(ChatError::InvalidSender));
    }

    #[test]
    fn test_chat_rejects_empty_text() {
        let mut manager = ChatManager::new();
        let res = manager.create_outgoing_message("Caixa 01", "", 100);
        assert_eq!(res, Err(ChatError::EmptyMessage));
    }

    #[test]
    fn test_chat_rejects_text_above_4kib() {
        let mut manager = ChatManager::new();
        let oversized = "X".repeat(MAX_CHAT_TEXT_LEN + 1);

        let res = manager.create_outgoing_message("Caixa 01", &oversized, 100);
        assert_eq!(
            res,
            Err(ChatError::MessageTooLarge {
                size: MAX_CHAT_TEXT_LEN + 1,
                max: MAX_CHAT_TEXT_LEN,
            })
        );

        let incoming_err = manager.handle_incoming_message(ChatMessage {
            id: 10,
            sender: "Viewer".to_string(),
            text: oversized,
            timestamp_ms: 100,
        });
        assert_eq!(
            incoming_err,
            Err(ChatError::MessageTooLarge {
                size: MAX_CHAT_TEXT_LEN + 1,
                max: MAX_CHAT_TEXT_LEN,
            })
        );
    }

    #[test]
    fn test_chat_history_bounded_capacity() {
        let mut manager = ChatManager::new();

        for i in 0..(MAX_CHAT_HISTORY + 10) {
            manager
                .handle_incoming_message(ChatMessage {
                    id: i as u64,
                    sender: "User".to_string(),
                    text: format!("Msg {}", i),
                    timestamp_ms: i as u64,
                })
                .unwrap();
        }

        assert_eq!(manager.history().len(), MAX_CHAT_HISTORY);
        // O primeiro elemento deve ser Msg 10 (os 10 primeiros foram descartados)
        assert_eq!(manager.history()[0].text, "Msg 10");
        assert_eq!(manager.history().last().unwrap().text, "Msg 509");
    }

    #[test]
    fn test_chat_handle_incoming_advances_id() {
        let mut manager = ChatManager::new();

        manager
            .handle_incoming_message(ChatMessage {
                id: 42,
                sender: "Operador".to_string(),
                text: "Verificando logs".to_string(),
                timestamp_ms: 500,
            })
            .unwrap();

        let out = manager
            .create_outgoing_message("PDV", "Ok, verifique", 600)
            .unwrap();

        assert_eq!(out.id, 43);
    }
}
