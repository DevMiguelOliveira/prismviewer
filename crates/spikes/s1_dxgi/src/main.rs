use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Threading::*;

#[derive(Clone)]
struct FrameStats {
    fps: f64,
    cpu_percent: f64,
}

fn main() {
    println!("S1 DXGI Desktop Duplication Spike");
    println!("Capturing from primary monitor. Press 'q' or wait 30 seconds to exit.");
    println!("Target: 30 fps, CPU < 10%");
    println!();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let stats = Arc::new(Mutex::new(FrameStats {
        fps: 0.0,
        cpu_percent: 0.0,
    }));
    let stats_clone = stats.clone();

    // Thread for keyboard input to quit
    thread::spawn(move || loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if input.trim().to_lowercase() == "q" {
                running_clone.store(false, Ordering::Relaxed);
                break;
            }
        }
        if !running_clone.load(Ordering::Relaxed) {
            break;
        }
    });

    // Capture thread
    let capture_handle = thread::spawn(move || unsafe {
        let _ = run_capture(stats_clone.clone());
    });

    // Monitoring thread
    let monitor_running = running.clone();
    let monitor_handle = thread::spawn(move || {
        monitor_stats(stats.clone(), monitor_running);
    });

    // Auto-exit after 30 seconds
    let start_time = Instant::now();
    while running.load(Ordering::Relaxed) && start_time.elapsed() < Duration::from_secs(30) {
        thread::sleep(Duration::from_millis(100));
    }
    running.store(false, Ordering::Relaxed);

    let _ = capture_handle.join();
    let _ = monitor_handle.join();

    println!("\nCapture stopped. Resources released.");
}

unsafe fn run_capture(stats: Arc<Mutex<FrameStats>>) -> Result<()> {
    // Create DXGI factory
    let dxgi_factory: IDXGIFactory1 = CreateDXGIFactory1()?;

    // Get adapter
    let adapter = dxgi_factory.EnumAdapters1(0)?;

    // Create D3D11 device
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
    let _context = context.unwrap();

    // Get output
    let output = adapter.EnumOutputs(0)?;
    let output1: IDXGIOutput1 = output.cast()?;

    // Create desktop duplication
    let duplication = output1.DuplicateOutput(&device)?;

    let mut frame_count = 0u64;
    let mut fps_timer = Instant::now();

    loop {
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;

        match duplication.AcquireNextFrame(100, &mut frame_info, &mut resource) {
            Ok(_) => {
                frame_count += 1;

                if fps_timer.elapsed() >= Duration::from_secs(1) {
                    let fps = frame_count as f64 / fps_timer.elapsed().as_secs_f64();
                    if let Ok(mut s) = stats.lock() {
                        s.fps = fps;
                    }
                    frame_count = 0;
                    fps_timer = Instant::now();
                }

                // Release frame
                let _ = duplication.ReleaseFrame();
                let _ = resource;
            }
            Err(_) => {
                // Timeout or other error, just continue
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

fn monitor_stats(stats: Arc<Mutex<FrameStats>>, running: Arc<AtomicBool>) {
    let mut cpu_timer = Instant::now();
    let mut last_kernel_time: i64 = 0;
    let mut last_user_time: i64 = 0;
    let mut last_system_time = Instant::now();

    while running.load(Ordering::Relaxed) {
        if cpu_timer.elapsed() >= Duration::from_secs(5) {
            // Measure CPU usage
            unsafe {
                let process_handle = GetCurrentProcess();
                let mut creation_time = FILETIME::default();
                let mut exit_time = FILETIME::default();
                let mut kernel_time = FILETIME::default();
                let mut user_time = FILETIME::default();

                if GetProcessTimes(
                    process_handle,
                    &mut creation_time,
                    &mut exit_time,
                    &mut kernel_time,
                    &mut user_time,
                )
                .is_ok()
                {
                    let kernel_100ns = ((kernel_time.dwHighDateTime as i64) << 32)
                        | (kernel_time.dwLowDateTime as i64);
                    let user_100ns = ((user_time.dwHighDateTime as i64) << 32)
                        | (user_time.dwLowDateTime as i64);
                    let current_total = kernel_100ns + user_100ns;

                    if last_kernel_time > 0 {
                        let delta_total = current_total - last_kernel_time - last_user_time;
                        let delta_time = last_system_time.elapsed().as_secs_f64();
                        if delta_time > 0.0 {
                            let cpu_percent =
                                (delta_total as f64 / 10_000_000.0 / delta_time) * 100.0;
                            if let Ok(mut s) = stats.lock() {
                                s.cpu_percent = cpu_percent.clamp(0.0, 100.0);
                            }
                        }
                    }
                    last_kernel_time = kernel_100ns;
                    last_user_time = user_100ns;
                    last_system_time = Instant::now();
                }
            }

            // Print stats
            if let Ok(s) = stats.lock() {
                let fps_ok = s.fps >= 29.5; // Allow small tolerance
                let cpu_ok = s.cpu_percent < 10.0;
                println!(
                    "[STATS] FPS: {:.1} {} | CPU: {:.1}% {} | Target: 30 fps, CPU < 10%",
                    s.fps,
                    if fps_ok { "✓" } else { "✗" },
                    s.cpu_percent,
                    if cpu_ok { "✓" } else { "✗" }
                );
            }

            cpu_timer = Instant::now();
        }
        thread::sleep(Duration::from_millis(100));
    }
}
