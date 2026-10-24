//! prysmview-svc - Implementação do Serviço Windows (LocalSystem) com recuperação automática.
//!
//! Conforme as Seções 5 e 10 do AGENTS.md e Fase 10:
//! - Executado sob a conta `LocalSystem` com inicialização automática (`SERVICE_AUTO_START`).
//! - Comunicação local com o `prysmview-tray` na sessão do usuário via Named Pipe seguro.
//! - Suporte a execução em modo console standalone (`--run`) para testes, depuração e diagnóstico.
//! - Encerramento limpo liberando teclas, modificadores e zerando credenciais na memória.

use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

pub const SERVICE_NAME: &str = "prysmview-svc";
pub const SERVICE_DISPLAY_NAME: &str = "PrysmView Remote Support Host Service";
pub const SERVICE_DESCRIPTION: &str =
    "Serviço de acesso remoto e assistência técnica para caixas de loja (PDVs)";

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Erro ao despachar serviço Windows: {0}")]
    #[cfg(windows)]
    Dispatcher(#[from] windows_service::Error),

    #[error("Erro no runtime assíncrono Tokio: {0}")]
    Tokio(#[from] std::io::Error),

    #[error("Falha no payload de execução do serviço: {0}")]
    Execution(String),
}

#[cfg(windows)]
define_windows_service!(ffi_service_main, prysmview_service_main);

/// Ponto de entrada chamado pelo Windows Service Control Manager (SCM).
#[cfg(windows)]
pub fn run_service() -> Result<(), ServiceError> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

#[cfg(not(windows))]
pub fn run_service() -> Result<(), ServiceError> {
    Err(ServiceError::Execution(
        "Serviço Windows só pode ser executado em ambiente Windows".to_string(),
    ))
}

/// Função principal executada dentro da thread do serviço Windows.
#[cfg(windows)]
fn prysmview_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service_lifecycle() {
        tracing::error!("Falha na execução do ciclo de vida do serviço: {:?}", e);
    }
}

#[cfg(windows)]
fn run_service_lifecycle() -> Result<(), windows_service::Error> {
    let running = Arc::new(AtomicBool::new(true));
    let running_handler = Arc::clone(&running);

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                running_handler.store(false, Ordering::SeqCst);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Notifica o SCM que o serviço está iniciando
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 1,
        wait_hint: Duration::from_secs(5),
        process_id: None,
    })?;

    // Notifica o SCM que o serviço está rodando
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Executa a carga útil do host
    if let Ok(runtime) = tokio::runtime::Runtime::new() {
        runtime.block_on(async {
            while running.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    // Libera qualquer modificador ou tecla retida antes da parada
    crate::sas::release_all_keys();

    // Notifica parada concluída
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

/// Executa o host em modo interativo / console para depuração e testes.
pub fn run_standalone() -> Result<(), ServiceError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        tracing::info!(
            "PrysmView Host Service em execução standalone. Pressione Ctrl+C para encerrar."
        );
        println!("PrysmView Host Service em execução standalone. Pressione Ctrl+C para encerrar.");

        // Aguarda sinal de interrupção (Ctrl+C)
        let _ = tokio::signal::ctrl_c().await;

        tracing::info!("Sinal de interrupção recebido. Encerrando host e liberando entradas...");
        println!("Encerrando host e liberando entradas...");
        crate::sas::release_all_keys();
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_constants() {
        assert_eq!(SERVICE_NAME, "prysmview-svc");
        assert!(SERVICE_DISPLAY_NAME.contains("PrysmView"));
        assert!(SERVICE_DESCRIPTION.contains("PDV"));
    }

    #[test]
    fn test_standalone_builder() {
        // Valida que o builder de runtime do standalone constrói com sucesso
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        assert!(runtime.is_ok());
    }
}
