use std::time::Instant;

use postcard::to_allocvec;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

const TILE_W: usize = 64;
const TILE_H: usize = 64;
const MAX_FRAMES: u32 = 10;
const ZSTD_LEVEL: i32 = 3;

struct DesktopCapture {
    duplication: IDXGIOutputDuplication,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    width: u32,
    height: u32,
    row_pitch: usize,
    staging: Option<ID3D11Texture2D>,
    initialized: bool,
}

struct FrameMetrics {
    total_tiles: usize,
    changed_tiles: usize,
    compressed_bytes: usize,
    full_frame_bytes: usize,
}

fn main() {
    println!("S2 - Tiles 64x64 + zstd (nivel 3) Spike");
    println!("Capturando frames via DXGI Desktop Duplication...");
    println!("Frames: {}, encerramento automatico.", MAX_FRAMES);
    println!();

    let mut cap = match unsafe { DesktopCapture::new() } {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Falha ao inicializar captura DXGI: {e}");
            std::process::exit(1);
        }
    };

    let mut prev_serialized: Option<Vec<Option<Vec<u8>>>> = None;
    let mut frame_count = 0u32;
    let mut total_compressed = 0usize;
    let mut total_changed = 0usize;
    let start = Instant::now();

    while frame_count < MAX_FRAMES {
        match unsafe { cap.next_frame() } {
            Ok(Some(frame_data)) => {
                if prev_serialized.is_none() {
                    prev_serialized = Some(vec![None; cap.total_tiles()]);
                }
                let metrics = evaluate_frame(
                    &frame_data,
                    prev_serialized.as_mut().unwrap().as_mut_slice(),
                    cap.width as usize,
                    cap.height as usize,
                );
                frame_count += 1;
                total_compressed += metrics.compressed_bytes;
                total_changed += metrics.changed_tiles;

                let full_bits = metrics.full_frame_bytes as f64;
                let comp_bits = metrics.compressed_bytes as f64;
                let reduction = if full_bits > 0.0 {
                    (1.0 - comp_bits / full_bits) * 100.0
                } else {
                    0.0
                };

                println!(
                    "[frame {}] tiles={} alterados={} comprimido={} bytes full_frame={} bytes reducao={:.1}% banda_est={} bytes/s",
                    frame_count,
                    metrics.total_tiles,
                    metrics.changed_tiles,
                    metrics.compressed_bytes,
                    metrics.full_frame_bytes,
                    reduction,
                    metrics.compressed_bytes * 30
                );
            }
            Ok(None) => {
                std::thread::sleep(std::time::Duration::from_millis(5));
                continue;
            }
            Err(e) => {
                eprintln!("Erro ao adquirir frame: {e}");
                break;
            }
        }
    }

    if frame_count > 0 {
        let avg_bytes = total_compressed as f64 / frame_count as f64;
        let avg_changed = total_changed as f64 / frame_count as f64;
        println!();
        println!("=== Resultado apos {} frames ===", frame_count);
        println!("Tiles por tela: {}", cap.total_tiles());
        println!("Media de tiles alterados por frame: {:.1}", avg_changed);
        println!(
            "Media de dados comprimidos por frame: {:.1} bytes",
            avg_bytes
        );
        println!(
            "Largura de banda estimada (media * 30 fps): {:.1} bytes/s = {:.2} kbit/s",
            avg_bytes * 30.0,
            (avg_bytes * 30.0 * 8.0) / 1000.0
        );
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            println!(
                "Tempo total: {:.2} s, fps efetivo: {:.1}",
                elapsed,
                frame_count as f64 / elapsed
            );
        }
    }

    println!("\nSpike S2 finalizado.");
}

fn evaluate_frame(
    data: &[u8],
    prev_serialized: &mut [Option<Vec<u8>>],
    width: usize,
    height: usize,
) -> FrameMetrics {
    let cols = width.div_ceil(TILE_W);
    let rows = height.div_ceil(TILE_H);
    let total_tiles = cols * rows;
    let row_pitch = (width * 4).next_multiple_of(256);
    let full_frame_bytes = width * height * 4;

    let mut changed_tiles = 0usize;
    let mut compressed_bytes = 0usize;

    for ty in 0..rows {
        for tx in 0..cols {
            let idx = ty * cols + tx;
            let x0 = tx * TILE_W;
            let y0 = ty * TILE_H;
            let tw = TILE_W.min(width - x0);
            let th = TILE_H.min(height - y0);

            let mut tile_bytes = Vec::with_capacity(tw * th * 4);
            for y in 0..th {
                let row = (y0 + y) * row_pitch + x0 * 4;
                tile_bytes.extend_from_slice(&data[row..row + tw * 4]);
            }

            let serialized =
                to_allocvec(&tile_bytes).expect("postcard serializacao de tile falhou");

            let changed = match &prev_serialized[idx] {
                Some(prev) => prev != &serialized,
                None => true,
            };

            if changed {
                changed_tiles += 1;
                let compressed = zstd::encode_all(&tile_bytes[..], ZSTD_LEVEL)
                    .expect("zstd compressao de tile falhou");
                compressed_bytes += compressed.len();
            }

            prev_serialized[idx] = Some(serialized);
        }
    }

    FrameMetrics {
        total_tiles,
        changed_tiles,
        compressed_bytes,
        full_frame_bytes,
    }
}

impl DesktopCapture {
    unsafe fn new() -> Result<Self> {
        let dxgi_factory: IDXGIFactory1 = CreateDXGIFactory1()?;
        let adapter = dxgi_factory.EnumAdapters1(0)?;

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
        )?;

        let device = device.unwrap();
        let context = context.unwrap();

        let output = adapter.EnumOutputs(0)?;
        let output1: IDXGIOutput1 = output.cast()?;
        let duplication = output1.DuplicateOutput(&device)?;

        Ok(Self {
            duplication,
            device,
            context,
            width: 0,
            height: 0,
            row_pitch: 0,
            staging: None,
            initialized: false,
        })
    }

    fn total_tiles(&self) -> usize {
        let cols = (self.width as usize).div_ceil(TILE_W);
        let rows = (self.height as usize).div_ceil(TILE_H);
        cols * rows
    }

    unsafe fn next_frame(&mut self) -> Result<Option<Vec<u8>>> {
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;

        match self
            .duplication
            .AcquireNextFrame(100, &mut frame_info, &mut resource)
        {
            Ok(_) => {}
            Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(None),
            Err(e) => return Err(e),
        }

        let resource = resource.ok_or_else(|| Error::from(HRESULT(0x80004003u32.cast_signed())))?;
        let texture: ID3D11Texture2D = resource.cast()?;
        let mut tex_desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut tex_desc);

        if !self.initialized {
            self.width = tex_desc.Width;
            self.height = tex_desc.Height;
            self.row_pitch = (tex_desc.Width as usize * 4).next_multiple_of(256);

            let staging_desc = D3D11_TEXTURE2D_DESC {
                Width: tex_desc.Width,
                Height: tex_desc.Height,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_STAGING,
                BindFlags: 0,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };
            let mut staging: Option<ID3D11Texture2D> = None;
            self.device
                .CreateTexture2D(&staging_desc, None, Some(&mut staging))?;
            self.staging = staging;
            self.initialized = true;
        }

        let staging = self.staging.as_ref().unwrap();
        self.context.CopyResource(staging, &texture);

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        self.context
            .Map(staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;

        let data = std::slice::from_raw_parts(
            mapped.pData as *const u8,
            self.row_pitch * self.height as usize,
        );

        let frame = data.to_vec();

        self.context.Unmap(staging, 0);
        let _ = self.duplication.ReleaseFrame();

        Ok(Some(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_grid_covers_1080p() {
        let width = 1920usize;
        let height = 1080usize;
        let cols = width.div_ceil(TILE_W);
        let rows = height.div_ceil(TILE_H);
        assert_eq!(cols, 30);
        assert_eq!(rows, 17);
        assert_eq!(cols * rows, 510);
    }

    #[test]
    fn detecta_tile_alterado() {
        let width = 128usize;
        let height = 64usize;
        let row_pitch = (width * 4).next_multiple_of(256);
        let mut prev: Vec<Option<Vec<u8>>> = vec![None; 2];
        let data = vec![0u8; row_pitch * height];

        let m0 = evaluate_frame(&data, &mut prev, width, height);
        assert_eq!(m0.changed_tiles, 2);
        assert!(m0.compressed_bytes > 0);

        let m1 = evaluate_frame(&data, &mut prev, width, height);
        assert_eq!(m1.changed_tiles, 0);
        assert_eq!(m1.compressed_bytes, 0);

        let mut data2 = data.clone();
        data2[100] = 1;
        let m2 = evaluate_frame(&data2, &mut prev, width, height);
        assert_eq!(m2.changed_tiles, 1);

        use sha2::{Digest, Sha256};
        let h1 = Sha256::digest(&data);
        let h2 = Sha256::digest(&data);
        assert_eq!(h1, h2);
        let mut data3 = data.clone();
        data3[0] ^= 1;
        assert_ne!(Sha256::digest(&data3), h1);
    }
}
