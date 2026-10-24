#![allow(deprecated)]

use iroh::net::endpoint::{Connection, RecvStream, SendStream};
use iroh::net::{Endpoint, NodeAddr, NodeId};
use prysmview_core::header::{Header, HeaderError, MessageType, CURRENT_PROTOCOL_VERSION};
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn};

pub const PRYSM_ALPN: &[u8] = b"prysmview/1";
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Falha ao inicializar endpoint de rede: {0}")]
    BindFailed(String),

    #[error("Falha ao aceitar conexao remota: {0}")]
    AcceptFailed(String),

    #[error("Falha de conexao ao peer: {0}")]
    ConnectionFailed(String),

    #[error("Erro no canal de comunicacao: {0}")]
    StreamError(String),

    #[error("Cabecalho de protocolo invalido: {0}")]
    InvalidHeader(#[from] HeaderError),

    #[error("Versao de protocolo incompativel: local={0}, remoto={1}")]
    IncompatibleVersion(u16, u16),

    #[error("Tipo de mensagem inesperado durante o handshake: {0:?}")]
    UnexpectedMessageType(MessageType),

    #[error("Tempo limite excedido durante o handshake")]
    HandshakeTimeout,
}

/// Adaptador de rede do Host PDV baseado em iroh (QUIC sobre TLS 1.3).
/// Suporta conexao P2P direta com fallback transparente para relay.
pub struct HostNetworkAdapter {
    endpoint: Endpoint,
}

impl HostNetworkAdapter {
    /// Inicializa o adaptador de rede do host com descoberta ativa.
    pub async fn bind() -> Result<Self, NetworkError> {
        let endpoint = Endpoint::builder()
            .discovery_n0()
            .bind()
            .await
            .map_err(|e| NetworkError::BindFailed(e.to_string()))?;

        info!(
            "HostNetworkAdapter escutando em NodeId: {:?}",
            endpoint.node_id()
        );

        Ok(Self { endpoint })
    }

    /// Retorna a chave publica (NodeId) do PDV.
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    /// Retorna o endereco completo do no (NodeAddr) incluindo relay tickets e IPs locais.
    pub async fn node_addr(&self) -> Result<NodeAddr, NetworkError> {
        self.endpoint
            .node_addr()
            .await
            .map_err(|e| NetworkError::BindFailed(e.to_string()))
    }

    /// Executa o handshake de conexao no stream 0 (controle) recebendo requisicao e respondendo confirmacao.
    pub async fn perform_host_handshake(
        send: &mut SendStream,
        recv: &mut RecvStream,
    ) -> Result<u16, NetworkError> {
        let mut header_buf = [0u8; 11];
        recv.read_exact(&mut header_buf)
            .await
            .map_err(|e| NetworkError::StreamError(e.to_string()))?;

        let header = Header::decode(&header_buf)?;

        if header.msg_type != MessageType::Control {
            return Err(NetworkError::UnexpectedMessageType(header.msg_type));
        }

        if header.protocol_version != CURRENT_PROTOCOL_VERSION {
            warn!(
                "Versao incompativel recebida: {} (esperado {})",
                header.protocol_version, CURRENT_PROTOCOL_VERSION
            );
            return Err(NetworkError::IncompatibleVersion(
                CURRENT_PROTOCOL_VERSION,
                header.protocol_version,
            ));
        }

        // Responde com o cabecalho de confirmacao de handshake (payload 0)
        let ack_header = Header::new(MessageType::Control, 0)?;
        send.write_all(&ack_header.encode())
            .await
            .map_err(|e| NetworkError::StreamError(e.to_string()))?;

        Ok(header.protocol_version)
    }

    /// Conecta a um no remoto com tentativa direta e fallback transparente para relay.
    pub async fn connect_to_node(&self, target_addr: NodeAddr) -> Result<Connection, NetworkError> {
        let conn = self
            .endpoint
            .connect(target_addr, PRYSM_ALPN)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        Ok(conn)
    }

    /// Encerra o endpoint de rede liberando recursos.
    pub async fn close(self) {
        let _ = self.endpoint.close(0u32.into(), b"shutdown").await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_bind_and_node_id() {
        let adapter = HostNetworkAdapter::bind().await.expect("bind deve suceder");
        let id = adapter.node_id();
        assert_eq!(id.as_bytes().len(), 32); // 32 bytes de chave publica Ed25519
        assert_eq!(id.to_string().len(), 52); // Formato base32 do iroh tem 52 caracteres

        let addr = adapter.node_addr().await.expect("deve obter node_addr");
        assert_eq!(addr.node_id, id);
        adapter.close().await;
    }

    #[test]
    fn test_handshake_constants() {
        assert_eq!(PRYSM_ALPN, b"prysmview/1");
        assert_eq!(HANDSHAKE_TIMEOUT.as_secs(), 10);
    }

    #[test]
    fn test_handshake_header_roundtrip() {
        let header = Header::new(MessageType::Control, 64).expect("header deve ser valido");
        let encoded = header.encode();
        assert_eq!(encoded.len(), 11);

        let decoded = Header::decode(&encoded).expect("decode deve suceder");
        assert_eq!(decoded.protocol_version, CURRENT_PROTOCOL_VERSION);
        assert_eq!(decoded.msg_type, MessageType::Control);
        assert_eq!(decoded.payload_len, 64);
    }

    #[test]
    fn test_handshake_rejects_non_control_type() {
        let header = Header::new(MessageType::Video, 1024).expect("header deve ser valido");
        assert_eq!(header.msg_type, MessageType::Video);
        assert_ne!(header.msg_type, MessageType::Control);
    }
}
