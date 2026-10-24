//! prysmview-host - IPC Local Seguro via Named Pipe
//!
//! Estabelece comunicação segura entre o serviço (prysmview-svc) e o app da bandeja (prysmview-tray).
//! O Named Pipe possui Security Descriptor restrito estritamente a LocalSystem (SYSTEM),
//! Administradores e Usuários Interativos Logados (IU), rejeitando qualquer acesso de rede
//! ou de outros contextos não privilegiados.
//!
//! Todas as mensagens são encapsuladas com o cabeçalho PRSM v1 de 11 bytes e validadas
//! antes de qualquer alocação de memória (limite estrito de 64 KiB).

use prysmview_core::header::{Header, HeaderError, MessageType};
use prysmview_core::limits::{self, LimitError};
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\prysmview-ipc";

/// SDDL: Discretionary ACL Protected:
/// - (A;;GA;;;SY) : Allow Generic All -> LocalSystem
/// - (A;;GA;;;BA) : Allow Generic All -> Builtin Administrators
/// - (A;;GA;;;IU) : Allow Generic All -> Interactive Users (usuário logado na sessão ativa)
pub const PIPE_SECURITY_DESCRIPTOR_SDDL: &str = "D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;IU)";

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Erro de I/O no canal IPC: {0}")]
    Io(#[from] io::Error),
    #[error("Erro no cabeçalho PRSM: {0}")]
    Header(#[from] HeaderError),
    #[error("Limite de tamanho de payload excedido: {0}")]
    LimitExceeded(#[from] LimitError),
    #[error("Erro de serialização postcard: {0}")]
    Serialization(String),
    #[error("Tipo de mensagem incompatível: esperado Control (0), recebido {0:?}")]
    InvalidMessageType(MessageType),
    #[error("Conexão IPC encerrada prematuramente")]
    ConnectionClosed,
    #[error("Falha ao configurar descritor de segurança do Windows: {0}")]
    SecurityDescriptor(String),
}

/// Mensagens trocadas exclusivamente entre prysmview-svc e prysmview-tray.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcMessage {
    /// O serviço solicita consentimento na sessão ativa do usuário
    ConsentRequest {
        operator_name: String,
        reason: String,
        timeout_secs: u32,
    },
    /// A bandeja responde com a decisão do operador local
    ConsentResponse { accepted: bool },
    /// O serviço envia o código de 8 dígitos para ser exibido na bandeja
    SessionCodeDisplay { code: String, expires_in_secs: u32 },
    /// O serviço notifica que a sessão remota está ativa
    SessionActive { operator_name: String },
    /// O usuário solicita o encerramento imediato da sessão ativa
    TerminateSession { reason: String },
    /// O serviço notifica que a sessão foi finalizada
    SessionEnded { reason: String },
    /// Heartbeat ping
    Ping(u64),
    /// Heartbeat pong
    Pong(u64),
}

/// Codifica uma mensagem IPC em um frame PRSM v1 completo (cabeçalho de 11 bytes + payload postcard).
pub fn encode_ipc_frame(msg: &IpcMessage) -> Result<Vec<u8>, IpcError> {
    let payload = postcard::to_allocvec(msg).map_err(|e| IpcError::Serialization(e.to_string()))?;

    let payload_len = payload.len() as u32;
    limits::validate_payload_len(MessageType::Control, payload_len)?;

    let header = Header::new(MessageType::Control, payload_len)?;
    let mut frame = Vec::with_capacity(Header::SIZE + payload.len());
    frame.extend_from_slice(&header.encode());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decodifica um frame PRSM v1 contendo uma IpcMessage.
pub fn decode_ipc_frame(buf: &[u8]) -> Result<(IpcMessage, usize), IpcError> {
    if buf.len() < Header::SIZE {
        return Err(IpcError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Buffer insuficiente para cabeçalho PRSM",
        )));
    }

    let mut header_bytes = [0u8; Header::SIZE];
    header_bytes.copy_from_slice(&buf[0..Header::SIZE]);
    let header = Header::decode(&header_bytes)?;

    if header.msg_type != MessageType::Control {
        return Err(IpcError::InvalidMessageType(header.msg_type));
    }

    limits::validate_payload_len(MessageType::Control, header.payload_len)?;

    let total_len = Header::SIZE + header.payload_len as usize;
    if buf.len() < total_len {
        return Err(IpcError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Buffer insuficiente para payload",
        )));
    }

    let payload_bytes = &buf[Header::SIZE..total_len];
    let msg: IpcMessage =
        postcard::from_bytes(payload_bytes).map_err(|e| IpcError::Serialization(e.to_string()))?;

    Ok((msg, total_len))
}

/// Envia de forma assíncrona uma mensagem IPC em qualquer stream gravável.
pub async fn send_ipc_message<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &IpcMessage,
) -> Result<(), IpcError> {
    let frame = encode_ipc_frame(msg)?;
    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

/// Recebe de forma assíncrona uma mensagem IPC com validação estrita antes da alocação de memória.
pub async fn recv_ipc_message<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<IpcMessage, IpcError> {
    let mut header_buf = [0u8; Header::SIZE];
    match reader.read_exact(&mut header_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            return Err(IpcError::ConnectionClosed)
        }
        Err(e) => return Err(IpcError::Io(e)),
    }

    let header = Header::decode(&header_buf)?;
    if header.msg_type != MessageType::Control {
        return Err(IpcError::InvalidMessageType(header.msg_type));
    }

    // Regra inegociável: Validação de limite ANTES de alocar memória
    limits::validate_payload_len(MessageType::Control, header.payload_len)?;

    let mut payload = vec![0u8; header.payload_len as usize];
    reader.read_exact(&mut payload).await?;

    let msg: IpcMessage =
        postcard::from_bytes(&payload).map_err(|e| IpcError::Serialization(e.to_string()))?;

    Ok(msg)
}

#[cfg(windows)]
mod windows_pipe {
    use super::*;
    use std::ffi::c_void;
    use tokio::net::windows::named_pipe::{
        ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
    };
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::LocalFree;
    use windows::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};

    /// Cria uma instância de NamedPipeServer com Security Descriptor estrito (SYSTEM + Administradores + Usuários Interativos).
    pub fn create_secure_pipe_server(pipe_name: &str) -> Result<NamedPipeServer, IpcError> {
        let mut server_options = ServerOptions::new();
        server_options.first_pipe_instance(false);
        server_options.max_instances(4);
        server_options.in_buffer_size(65536);
        server_options.out_buffer_size(65536);
        server_options.reject_remote_clients(true);

        let wide_sddl: Vec<u16> = PIPE_SECURITY_DESCRIPTOR_SDDL
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut p_sd = PSECURITY_DESCRIPTOR::default();
        let mut sd_size = 0u32;

        // SAFETY: ConvertStringSecurityDescriptorToSecurityDescriptorW lê um PCWSTR null-terminated válido
        // e escreve em ponteiros válidos locais (p_sd e sd_size).
        let conv_res = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(wide_sddl.as_ptr()),
                1, // SDDL_REVISION_1
                &mut p_sd,
                Some(&mut sd_size),
            )
        };

        if conv_res.is_err() {
            return Err(IpcError::SecurityDescriptor(
                "Falha ao converter SDDL para Security Descriptor".into(),
            ));
        }

        let mut sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: p_sd.0,
            bInheritHandle: windows::Win32::Foundation::BOOL(0),
        };

        // SAFETY: sa aponta para estrutura SECURITY_ATTRIBUTES devidamente inicializada com
        // o descritor de segurança gerado pela API do Windows.
        let server = unsafe {
            server_options
                .create_with_security_attributes_raw(pipe_name, &mut sa as *mut _ as *mut c_void)
        };

        // SAFETY: Libera a memória alocada por ConvertStringSecurityDescriptorToSecurityDescriptorW
        unsafe {
            let _ = LocalFree(windows::Win32::Foundation::HLOCAL(p_sd.0));
        }

        server.map_err(IpcError::Io)
    }

    /// Conecta como cliente ao named pipe local.
    pub async fn connect_pipe_client(pipe_name: &str) -> Result<NamedPipeClient, IpcError> {
        ClientOptions::new().open(pipe_name).map_err(IpcError::Io)
    }
}

#[cfg(windows)]
pub use windows_pipe::{connect_pipe_client, create_secure_pipe_server};

#[cfg(test)]
mod tests {
    use super::*;
    use prysmview_core::header::{CURRENT_PROTOCOL_VERSION, PROTOCOL_MAGIC};

    #[test]
    fn test_ipc_message_encoding_roundtrip() {
        let msg = IpcMessage::ConsentRequest {
            operator_name: "Suporte Central".to_string(),
            reason: "Verificação de emissão fiscal".to_string(),
            timeout_secs: 60,
        };

        let frame = encode_ipc_frame(&msg).expect("codificação bem-sucedida");
        assert!(frame.len() > Header::SIZE);

        let (decoded, consumed) = decode_ipc_frame(&frame).expect("decodificação bem-sucedida");
        assert_eq!(consumed, frame.len());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_ipc_all_variants_roundtrip() {
        let variants = vec![
            IpcMessage::ConsentResponse { accepted: true },
            IpcMessage::ConsentResponse { accepted: false },
            IpcMessage::SessionCodeDisplay {
                code: "98765432".to_string(),
                expires_in_secs: 120,
            },
            IpcMessage::SessionActive {
                operator_name: "Técnico Maria".to_string(),
            },
            IpcMessage::TerminateSession {
                reason: "Cancelado pelo operador da loja".to_string(),
            },
            IpcMessage::SessionEnded {
                reason: "Tempo limite esgotado".to_string(),
            },
            IpcMessage::Ping(42),
            IpcMessage::Pong(42),
        ];

        for var in variants {
            let frame = encode_ipc_frame(&var).expect("encode");
            let (decoded, _) = decode_ipc_frame(&frame).expect("decode");
            assert_eq!(decoded, var);
        }
    }

    #[test]
    fn test_ipc_rejects_payload_above_limit_before_allocating() {
        // Gera um cabeçalho malicioso simulando payload maior que 64 KiB
        let mut malicious_frame = [0u8; Header::SIZE + 10];
        malicious_frame[0..4].copy_from_slice(&PROTOCOL_MAGIC);
        malicious_frame[4..6].copy_from_slice(&CURRENT_PROTOCOL_VERSION.to_be_bytes());
        malicious_frame[6] = MessageType::Control as u8;
        // 65 KiB (excede CONTROL_MAX de 64 KiB)
        let oversized = 65 * 1024u32;
        malicious_frame[7..11].copy_from_slice(&oversized.to_be_bytes());

        let res = decode_ipc_frame(&malicious_frame);
        assert!(res.is_err(), "Payload oversized deve ser rejeitado");
    }

    #[tokio::test]
    async fn test_async_ipc_send_recv_stream() {
        let (mut client_stream, mut server_stream) = tokio::io::duplex(1024);

        let msg_to_send = IpcMessage::SessionCodeDisplay {
            code: "12345678".to_string(),
            expires_in_secs: 120,
        };

        // Envia do servidor para o cliente
        send_ipc_message(&mut server_stream, &msg_to_send)
            .await
            .expect("envio bem-sucedido");

        // Recebe no cliente
        let received_msg = recv_ipc_message(&mut client_stream)
            .await
            .expect("recebimento bem-sucedido");

        assert_eq!(received_msg, msg_to_send);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_windows_named_pipe_creation_and_communication() {
        use rand::Rng;
        let test_pipe_name = format!(
            r"\\.\pipe\prysmview-test-pipe-{}",
            rand::thread_rng().gen::<u32>()
        );

        let server = create_secure_pipe_server(&test_pipe_name).expect("criação do server pipe");

        let client_fut = async {
            // Pequeno delay para garantir que o server está pronto
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            let mut client = connect_pipe_client(&test_pipe_name)
                .await
                .expect("conexão do cliente");

            let msg = recv_ipc_message(&mut client)
                .await
                .expect("recebe no cliente");
            assert_eq!(
                msg,
                IpcMessage::ConsentRequest {
                    operator_name: "Operador Teste".into(),
                    reason: "Auditoria".into(),
                    timeout_secs: 60,
                }
            );

            send_ipc_message(&mut client, &IpcMessage::ConsentResponse { accepted: true })
                .await
                .expect("responde no cliente");
        };

        let server_fut = async {
            server.connect().await.expect("server aceita conexão");
            let mut server = server;

            send_ipc_message(
                &mut server,
                &IpcMessage::ConsentRequest {
                    operator_name: "Operador Teste".into(),
                    reason: "Auditoria".into(),
                    timeout_secs: 60,
                },
            )
            .await
            .expect("envia pelo server");

            let resp = recv_ipc_message(&mut server)
                .await
                .expect("recebe resposta no server");
            assert_eq!(resp, IpcMessage::ConsentResponse { accepted: true });
        };

        tokio::join!(client_fut, server_fut);
    }
}
