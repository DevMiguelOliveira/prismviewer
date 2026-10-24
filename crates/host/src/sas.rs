//! prysmview-host - Secure Attention Sequence (Ctrl+Alt+Del) e reinicialização com reconexão automática.
//!
//! Conforme as Seções 3, 9 e 11 do AGENTS.md:
//! - SAS (Ctrl+Alt+Del) exige privilégios de serviço SYSTEM e direcionamento para Winlogon.
//! - Utiliza a API `SendSAS` da biblioteca do Windows `sas.dll`.
//! - Liberação compulsória de todas as teclas e modificadores ao encerrar sessão ou invocar SAS.
//! - Persistência do estado de reconexão para restauração automática após reinício do PDV.

use std::path::Path;
use thiserror::Error;

/// Erros decorrentes da invocação de SAS ou gerenciamento de reinício do PDV.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SasError {
    #[error("Biblioteca sas.dll do Windows não encontrada ou símbolo SendSAS indisponível")]
    DllNotFound,

    #[error("Chamada SendSAS falhou com código de erro do Windows: {0}")]
    SendSasFailed(u32),

    #[error("Erro de E/S: {0}")]
    Io(String),

    #[error("Erro de serialização/desserialização com postcard: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for SasError {
    fn from(err: std::io::Error) -> Self {
        SasError::Io(err.to_string())
    }
}

/// Estado persistido para permitir reconexão automática do Viewer após reinicialização do PDV.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReconnectState {
    pub peer_address: String,
    pub session_code: Option<String>,
    pub was_graceful: bool,
    pub timestamp_ms: u64,
}

impl ReconnectState {
    pub fn new(peer_address: impl Into<String>, session_code: Option<String>) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            peer_address: peer_address.into(),
            session_code,
            was_graceful: false,
            timestamp_ms,
        }
    }
}

/// Caminho padrão para persistência do estado de reconexão no PDV
pub const DEFAULT_RECONNECT_STATE_PATH: &str = r"C:\ProgramData\PrysmView\reconnect_state.bin";

#[cfg(windows)]
extern "system" {
    fn LoadLibraryW(lp_lib_file_name: *const u16) -> *mut std::ffi::c_void;
    fn GetProcAddress(
        h_module: *mut std::ffi::c_void,
        lp_proc_name: *const u8,
    ) -> *mut std::ffi::c_void;
    fn FreeLibrary(h_module: *mut std::ffi::c_void) -> i32;
    fn GetLastError() -> u32;
}

/// Dispara a sequência de atenção segura (Ctrl+Alt+Del) no desktop Winlogon.
///
/// Exige execução como LocalSystem (`prysmview-svc`). Em sistemas onde a política
/// de software SAS está ativa, simula a sequência sem intervenção do teclado.
#[cfg(windows)]
pub fn send_sas() -> Result<(), SasError> {
    unsafe {
        let dll_name: Vec<u16> = "sas.dll\0".encode_utf16().collect();
        let hmodule = LoadLibraryW(dll_name.as_ptr());
        if hmodule.is_null() {
            return Err(SasError::DllNotFound);
        }

        let proc_name = b"SendSAS\0";
        let proc = GetProcAddress(hmodule, proc_name.as_ptr());
        if proc.is_null() {
            FreeLibrary(hmodule);
            return Err(SasError::DllNotFound);
        }

        type SendSasFn = unsafe extern "system" fn(as_user: i32) -> i32;
        let send_sas_fn: SendSasFn = std::mem::transmute(proc);

        // SAFETY: as_user = 0 (FALSE) indica que o disparo é realizado pelo serviço SYSTEM
        let success = send_sas_fn(0);
        let err = if success == 0 { GetLastError() } else { 0 };

        FreeLibrary(hmodule);

        if success == 0 {
            return Err(SasError::SendSasFailed(err));
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn send_sas() -> Result<(), SasError> {
    Err(SasError::DllNotFound)
}

/// Libera todas as teclas e modificadores retidos (Ctrl, Alt, Shift, Win).
pub fn release_all_keys() {
    #[cfg(windows)]
    {
        if let Ok(mut injector) = crate::input::InputInjector::new(1920, 1080) {
            injector.release_all();
        }
    }
}

/// Salva o estado de reconexão em disco utilizando postcard para serialização binária compacta.
pub fn save_reconnect_state<P: AsRef<Path>>(
    path: P,
    state: &ReconnectState,
) -> Result<(), SasError> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let encoded =
        postcard::to_allocvec(state).map_err(|e| SasError::Serialization(e.to_string()))?;

    std::fs::write(p, encoded)?;
    Ok(())
}

/// Lê o estado de reconexão persistido em disco. Retorna `Ok(None)` se o arquivo não existir.
pub fn load_reconnect_state<P: AsRef<Path>>(path: P) -> Result<Option<ReconnectState>, SasError> {
    let p = path.as_ref();
    if !p.exists() {
        return Ok(None);
    }

    let bytes = std::fs::read(p)?;
    let state = postcard::from_bytes(&bytes).map_err(|e| SasError::Serialization(e.to_string()))?;

    Ok(Some(state))
}

/// Remove o estado de reconexão após ser consumido ou expirado.
pub fn clear_reconnect_state<P: AsRef<Path>>(path: P) -> Result<(), SasError> {
    let p = path.as_ref();
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

/// Agenda reinicialização controlada do PDV gravando o estado de reconexão antes de emitir o comando.
pub fn request_system_reboot<P: AsRef<Path>>(
    state_path: P,
    state: &ReconnectState,
) -> Result<(), SasError> {
    save_reconnect_state(state_path, state)?;

    #[cfg(windows)]
    {
        let _ = std::process::Command::new("shutdown.exe")
            .args(["/r", "/t", "5", "/c", "PrysmView PDV Reboot"])
            .spawn();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_state_creation() {
        let state = ReconnectState::new("node-ed25519-abc123", Some("87654321".to_string()));
        assert_eq!(state.peer_address, "node-ed25519-abc123");
        assert_eq!(state.session_code, Some("87654321".to_string()));
        assert!(!state.was_graceful);
        assert!(state.timestamp_ms > 0);
    }

    #[test]
    fn test_save_and_load_reconnect_state_roundtrip() {
        let temp_dir =
            std::env::temp_dir().join(format!("prysm_sas_test_{}", rand::random::<u32>()));
        let state_path = temp_dir.join("reconnect.bin");

        let state = ReconnectState::new("pdv-store-01-caixa-03", Some("11223344".to_string()));

        // Salvar estado
        save_reconnect_state(&state_path, &state).unwrap();
        assert!(state_path.exists());

        // Carregar estado
        let loaded = load_reconnect_state(&state_path).unwrap();
        assert_eq!(loaded, Some(state));

        // Limpar estado
        clear_reconnect_state(&state_path).unwrap();
        assert!(!state_path.exists());

        let after_clear = load_reconnect_state(&state_path).unwrap();
        assert_eq!(after_clear, None);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_non_existent_state_returns_none() {
        let temp_dir =
            std::env::temp_dir().join(format!("prysm_sas_nonexistent_{}", rand::random::<u32>()));
        let state_path = temp_dir.join("does_not_exist.bin");

        let loaded = load_reconnect_state(&state_path).unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn test_release_all_keys_does_not_panic() {
        release_all_keys();
    }
}
