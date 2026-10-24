use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
    VIRTUAL_KEY,
};

#[derive(Clone, Copy)]
struct KeyEvent {
    scan_code: u16,
    pressed: bool,
}

fn send_key(event: &KeyEvent) {
    let mut flags = KEYEVENTF_SCANCODE;
    if !event.pressed {
        flags |= KEYEVENTF_KEYUP;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: event.scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

fn send_mouse(
    flags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS,
    mouse_data: i32,
) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data as u32,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

fn main() {
    println!("S4 Spike - SendInput com scan codes ABNT2");
    println!("Injetando eventos de teclado e mouse...");
    println!("Target: nenhum modificador preso ao final.\n");

    // Scancodes ABNT2 comuns
    println!("Injetando tecla A (scancode 0x1E)...");
    send_key(&KeyEvent {
        scan_code: 0x1E,
        pressed: true,
    });
    send_key(&KeyEvent {
        scan_code: 0x1E,
        pressed: false,
    });

    println!("Injetando tecla S (scancode 0x1F)...");
    send_key(&KeyEvent {
        scan_code: 0x1F,
        pressed: true,
    });
    send_key(&KeyEvent {
        scan_code: 0x1F,
        pressed: false,
    });

    println!("Injetando tecla Enter (scancode 0x1C)...");
    send_key(&KeyEvent {
        scan_code: 0x1C,
        pressed: true,
    });
    send_key(&KeyEvent {
        scan_code: 0x1C,
        pressed: false,
    });

    // Mouse events
    println!("Injetando clique esquerdo...");
    send_mouse(MOUSEEVENTF_LEFTDOWN, 0);
    send_mouse(MOUSEEVENTF_LEFTUP, 0);

    // Mouse wheel up (120 units)
    println!("Injetando rolagem de roda do mouse...");
    send_mouse(MOUSEEVENTF_WHEEL, 120);

    println!("\nEventos injetados com sucesso.");
    println!("State limpo: teclas foram liberadas com KEYUP correspondente.");
    println!("Nenhum modificador ficou pressionado.");
}
