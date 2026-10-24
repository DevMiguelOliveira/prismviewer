//! prysmview-host - Módulo de Captura de Tela DXGI Desktop Duplication
//!
//! Executa em thread dedicada, captura o monitor selecionado e despacha quadros
//! brutos via `SyncSender<CapturedFrame>`. Utiliza buffer limitado para descartar
//! quadros intermediários se o canal estiver lento, e recria a duplicação
//! automaticamente em caso de `DXGI_ERROR_ACCESS_LOST` (troca de desktop/resolução).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

#[cfg(windows)]
use windows::core::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;
#[cfg(windows)]
use windows::Win32::Graphics::Direct3D::*;
#[cfg(windows)]
use windows::Win32::Graphics::Direct3D11::*;
#[cfg(windows)]
use windows::Win32::Graphics::Dxgi::*;

#[derive(Debug, Error)]
pub enum DxgiCaptureError {
    #[error("Falha ao inicializar factory DXGI: {0}")]
    Factory(String),
    #[error("Adapter de vídeo não encontrado: {0}")]
    AdapterNotFound(String),
    #[error("Output/Monitor {0} não encontrado")]
    OutputNotFound(u32),
    #[error("Falha ao criar dispositivo Direct3D 11: {0}")]
    D3D11Device(String),
    #[error("Falha ao iniciar Desktop Duplication: {0}")]
    Duplication(String),
    #[error("Acesso à duplicação perdido (DXGI_ERROR_ACCESS_LOST)")]
    AccessLost,
}

/// Quadro capturado da tela (pixels BGRA descompactados).
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Worker de captura que roda em thread dedicada.
pub struct CaptureWorker {
    output_index: u32,
    running: Arc<AtomicBool>,
}

impl CaptureWorker {
    /// Cria uma nova instância do CaptureWorker para o monitor selecionado.
    pub fn new(output_index: u32) -> Self {
        Self {
            output_index,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Retorna se o worker está ativo.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Para a captura de tela e encerra a thread de captura.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Inicia a thread dedicada de captura.
    /// Despacha `CapturedFrame`s pelo canal bounded `sender`.
    pub fn start(&self, sender: SyncSender<CapturedFrame>) -> JoinHandle<()> {
        self.running.store(true, Ordering::Relaxed);
        let running = Arc::clone(&self.running);
        let output_index = self.output_index;

        thread::spawn(move || {
            #[cfg(windows)]
            {
                run_dxgi_capture_loop(output_index, running, sender);
            }
            #[cfg(not(windows))]
            {
                let _ = (output_index, running, sender);
                tracing::warn!("DXGI Desktop Duplication só é suportado em Windows");
            }
        })
    }
}

#[cfg(windows)]
fn run_dxgi_capture_loop(
    output_index: u32,
    running: Arc<AtomicBool>,
    sender: SyncSender<CapturedFrame>,
) {
    while running.load(Ordering::Relaxed) {
        // Tenta inicializar DXGI Desktop Duplication
        match init_duplication(output_index) {
            Ok((duplication, device, context, width, height)) => {
                let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();

                while running.load(Ordering::Relaxed) {
                    let mut resource: Option<IDXGIResource> = None;

                    // Timeout de 100ms para aguardar o próximo quadro
                    match unsafe {
                        duplication.AcquireNextFrame(100, &mut frame_info, &mut resource)
                    } {
                        Ok(()) => {
                            if frame_info.AccumulatedFrames > 0 {
                                if let Some(res) = resource {
                                    if let Ok(texture) = res.cast::<ID3D11Texture2D>() {
                                        if let Ok(frame_data) = read_texture_data(
                                            &device, &context, &texture, width, height,
                                        ) {
                                            let frame = CapturedFrame {
                                                width,
                                                height,
                                                data: frame_data,
                                            };
                                            // try_send descarta quadros intermediários se o encoder estiver ocupado
                                            let _ = sender.try_send(frame);
                                        }
                                    }
                                }
                            }
                            unsafe {
                                let _ = duplication.ReleaseFrame();
                            }
                        }
                        Err(e) => {
                            // Se perdeu o acesso (troca de desktop, UAC, logon, mudança de resolução), recria
                            if e.code() == DXGI_ERROR_ACCESS_LOST
                                || e.code() == DXGI_ERROR_INVALID_CALL
                            {
                                tracing::warn!(
                                    "DXGI Desktop Duplication perdeu acesso ({:?}), recriando...",
                                    e
                                );
                                break;
                            }
                            // Timeout de 100ms é normal em tela estática, continua normalmente
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Erro ao inicializar duplicação DXGI: {:?}. Tentando novamente em 500ms...",
                    e
                );
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

#[cfg(windows)]
fn init_duplication(
    output_index: u32,
) -> std::result::Result<
    (
        IDXGIOutputDuplication,
        ID3D11Device,
        ID3D11DeviceContext,
        u32,
        u32,
    ),
    DxgiCaptureError,
> {
    unsafe {
        let factory: IDXGIFactory1 =
            CreateDXGIFactory1().map_err(|e| DxgiCaptureError::Factory(e.to_string()))?;

        let adapter = factory
            .EnumAdapters1(0)
            .map_err(|e| DxgiCaptureError::AdapterNotFound(e.to_string()))?;

        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let mut feature_level = D3D_FEATURE_LEVEL_11_0;

        D3D11CreateDevice(
            &adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&[D3D_FEATURE_LEVEL_11_0]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut feature_level),
            Some(&mut context),
        )
        .map_err(|e| DxgiCaptureError::D3D11Device(e.to_string()))?;

        let device =
            device.ok_or_else(|| DxgiCaptureError::D3D11Device("Dispositivo D3D11 nulo".into()))?;
        let context =
            context.ok_or_else(|| DxgiCaptureError::D3D11Device("Contexto D3D11 nulo".into()))?;

        let output = adapter
            .EnumOutputs(output_index)
            .map_err(|_| DxgiCaptureError::OutputNotFound(output_index))?;

        let output1: IDXGIOutput1 = output
            .cast()
            .map_err(|e| DxgiCaptureError::Duplication(e.to_string()))?;

        let duplication = output1
            .DuplicateOutput(&device)
            .map_err(|e| DxgiCaptureError::Duplication(e.to_string()))?;

        let desc = duplication.GetDesc();
        let width = desc.ModeDesc.Width;
        let height = desc.ModeDesc.Height;

        Ok((duplication, device, context, width, height))
    }
}

#[cfg(windows)]
fn read_texture_data(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    texture: &ID3D11Texture2D,
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    unsafe {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);

        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;

        let mut staging: Option<ID3D11Texture2D> = None;
        device.CreateTexture2D(&desc, None, Some(&mut staging))?;
        let staging = staging.ok_or_else(|| Error::from(E_FAIL))?;

        context.CopyResource(&staging, texture);

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        context.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;

        let row_pitch = mapped.RowPitch as usize;
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let src = mapped.pData as *const u8;

        for y in 0..(height as usize) {
            let src_row = src.add(y * row_pitch);
            let dst_row = &mut buffer[y * (width as usize * 4)..(y + 1) * (width as usize * 4)];
            std::ptr::copy_nonoverlapping(src_row, dst_row.as_mut_ptr(), width as usize * 4);
        }

        context.Unmap(&staging, 0);
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;

    #[test]
    fn test_worker_lifecycle() {
        let (sender, _receiver) = sync_channel::<CapturedFrame>(2);
        let worker = CaptureWorker::new(0);
        assert!(!worker.is_running());

        let handle = worker.start(sender);
        assert!(worker.is_running());

        worker.stop();
        assert!(!worker.is_running());

        let _ = handle.join();
    }
}
