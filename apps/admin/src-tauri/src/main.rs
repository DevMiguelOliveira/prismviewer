// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceListResponse {
    devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Device {
    id: String,
    name: String,
    store: String,
    status: String,
    ip: String,
}

// Comando: Login de operador (stub - conectar com API real depois)
#[tauri::command]
async fn login(_email: String, _password: String) -> Result<LoginResponse, String> {
    // Simulação - em produção, chamar API de sinalização
    Ok(LoginResponse {
        success: true,
        token: Some("mock_token_12345".to_string()),
        error: None,
    })
}

// Comando: Listar dispositivos (stub)
#[tauri::command]
async fn list_devices() -> Result<DeviceListResponse, String> {
    // Simulação - em produção, chamar API de sinalização
    Ok(DeviceListResponse {
        devices: vec![
            Device {
                id: "dev-1".to_string(),
                name: "DESKTOP-01".to_string(),
                store: "Loja Centro".to_string(),
                status: "online".to_string(),
                ip: "192.168.1.100".to_string(),
            },
            Device {
                id: "dev-2".to_string(),
                name: "DESKTOP-02".to_string(),
                store: "Loja Norte".to_string(),
                status: "offline".to_string(),
                ip: "192.168.1.101".to_string(),
            },
        ],
    })
}

// Comando: Solicitar sessão (stub)
#[tauri::command]
async fn request_session(device_id: String) -> Result<String, String> {
    // Simulação - em produção, chamar API de sinalização
    Ok(format!("session_requested:{}", device_id))
}

// Comando: Conectar WebRTC (stub)
#[tauri::command]
async fn connect_webrtc(device_id: String, session_code: String) -> Result<String, String> {
    // TODO: Implementar conexão WebRTC real
    Ok(format!("webrtc_connected:{}:{}", device_id, session_code))
}

// Comando: Desconectar (stub)
#[tauri::command]
async fn disconnect() -> Result<String, String> {
    Ok("disconnected".to_string())
}

// Comando: Enviar input para dispositivo remoto (stub)
#[tauri::command]
async fn send_remote_input(input_type: String, _data: serde_json::Value) -> Result<String, String> {
    // TODO: Implementar envio via WebRTC data channel
    Ok(format!("remote_input_sent:{}", input_type))
}

// Comando: Receber frame de vídeo (stub)
#[tauri::command]
async fn receive_video_frame() -> Result<String, String> {
    // TODO: Implementar recebimento via WebRTC data channel
    Ok("video_frame_received".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            login,
            list_devices,
            request_session,
            connect_webrtc,
            disconnect,
            send_remote_input,
            receive_video_frame,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
