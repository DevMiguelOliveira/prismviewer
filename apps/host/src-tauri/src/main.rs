// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
struct SessionCodeResponse {
    code: String,
    expires_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceIdResponse {
    id: String,
    name: String,
    ip: String,
}

struct AppState {
    #[allow(dead_code)]
    secret_key: Vec<u8>,
}

// Comando: Gerar código de sessão de 8 dígitos
#[tauri::command]
async fn generate_session_code(_state: State<'_, AppState>) -> Result<SessionCodeResponse, String> {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(1_0000_0000..10_0000_0000);
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 120; // 2 minutos

    Ok(SessionCodeResponse {
        code: code.to_string(),
        expires_at,
    })
}

// Comando: Validar código de sessão (simulação)
#[tauri::command]
async fn validate_session_code(code: String, expected_code: String) -> Result<bool, String> {
    // Em produção, usar comparação em tempo constante com HMAC
    Ok(code == expected_code)
}

// Comando: Obter ID do dispositivo (simulado - em produção usaria Ed25519)
#[tauri::command]
async fn get_device_id() -> Result<DeviceIdResponse, String> {
    // ID fixo estilo TeamViewer (gerado uma vez e persistido)
    let id = "117 029 2035".to_string();
    let name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Desktop".to_string());

    // Obter IP local (simulado)
    let ip = "192.168.1.100".to_string();

    Ok(DeviceIdResponse { id, name, ip })
}

// Comando: Iniciar captura de tela (stub)
#[tauri::command]
async fn start_screen_capture() -> Result<String, String> {
    // TODO: Implementar captura DXGI/GDI
    Ok("screen_capture_started".to_string())
}

// Comando: Parar captura de tela (stub)
#[tauri::command]
async fn stop_screen_capture() -> Result<String, String> {
    // TODO: Implementar parada de captura
    Ok("screen_capture_stopped".to_string())
}

// Comando: Injetar input (stub)
#[tauri::command]
async fn inject_input(input_type: String, _data: serde_json::Value) -> Result<String, String> {
    // TODO: Implementar SendInput
    Ok(format!("input_injected: {}", input_type))
}

// Comando: Soltar todas as teclas (emergency)
#[tauri::command]
async fn release_all_keys() -> Result<String, String> {
    // TODO: Implementar release_all_keys via SendInput
    Ok("all_keys_released".to_string())
}

fn main() {
    // Gerar chave secreta para HMAC (em produção, persistir de forma segura)
    let secret_key = (0..32).map(|_| rand::random::<u8>()).collect();

    tauri::Builder::default()
        .manage(AppState { secret_key })
        .invoke_handler(tauri::generate_handler![
            generate_session_code,
            validate_session_code,
            get_device_id,
            start_screen_capture,
            stop_screen_capture,
            inject_input,
            release_all_keys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
