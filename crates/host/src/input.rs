//! prysmview-host - Injeção de entrada (SendInput) por scan code ABNT2 e coordenadas normalizadas.
//!
//! Conforme as seções 8, 9 e 10 do AGENTS.md:
//! - Teclas são injetadas estritamente por SCAN CODE (KEYEVENTF_SCANCODE) para garantir
//!   compatibilidade nativa com o teclado ABNT2 do PDV, independentemente de mapeamentos virtuais.
//! - Coordenadas de mouse recebidas no intervalo normalizado 0..65535 são mapeadas para o monitor ativo.
//! - Rastreamento de teclas ativas para garantir liberação imediata e total de qualquer modificador
//!   ou tecla presa ao encerrar a sessão ou em queda de conexão.

use prysmview_core::messages::{InputMessage, MouseButtonType};
use std::collections::HashSet;
use thiserror::Error;

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Debug, Error)]
pub enum InputInjectionError {
    #[error("Falha ao invocar SendInput do Windows: {0}")]
    SendInputFailed(String),
    #[error("Dimensões de tela inválidas: {0}x{1}")]
    InvalidDimensions(u32, u32),
}

/// Injetor de eventos de mouse e teclado para o PDV.
pub struct InputInjector {
    screen_width: u32,
    screen_height: u32,
    /// Rastreamento de teclas ativas: (scan_code, is_extended)
    pressed_keys: HashSet<(u16, bool)>,
    /// Rastreamento de botões de mouse ativos: bitmask (1=Left, 2=Right, 4=Middle)
    pressed_mouse_buttons: u8,
}

impl InputInjector {
    /// Cria um novo injetor configurado com a resolução do monitor atual.
    pub fn new(screen_width: u32, screen_height: u32) -> Result<Self, InputInjectionError> {
        if screen_width == 0 || screen_height == 0 {
            return Err(InputInjectionError::InvalidDimensions(
                screen_width,
                screen_height,
            ));
        }
        Ok(Self {
            screen_width,
            screen_height,
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: 0,
        })
    }

    /// Atualiza as dimensões do monitor (ex: troca de resolução).
    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_width = width;
            self.screen_height = height;
        }
    }

    pub fn active_keys_count(&self) -> usize {
        self.pressed_keys.len()
    }

    pub fn has_active_mouse_buttons(&self) -> bool {
        self.pressed_mouse_buttons != 0
    }

    /// Processa e injeta um evento de entrada recebido do viewer.
    pub fn handle_input(&mut self, msg: &InputMessage) -> Result<(), InputInjectionError> {
        match *msg {
            InputMessage::MouseMove { x, y } => self.inject_mouse_move(x, y),
            InputMessage::MouseButton { button, pressed } => {
                self.inject_mouse_button(button, pressed)
            }
            InputMessage::MouseWheel { delta_v, delta_h } => {
                self.inject_mouse_wheel(delta_v, delta_h)
            }
            InputMessage::Key {
                scan_code,
                extended,
                pressed,
            } => self.inject_key(scan_code, extended, pressed),
        }
    }

    /// Mapeia coordenadas normalizadas (0..65535) e move o cursor.
    pub fn inject_mouse_move(
        &mut self,
        norm_x: u16,
        norm_y: u16,
    ) -> Result<(), InputInjectionError> {
        #[cfg(windows)]
        unsafe {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: norm_x as i32,
                        dy: norm_y as i32,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            if SendInput(&[input], std::mem::size_of::<INPUT>() as i32) == 0 {
                return Err(InputInjectionError::SendInputFailed(
                    "SendInput mouse move retornou 0".into(),
                ));
            }
        }
        #[cfg(not(windows))]
        let _ = (norm_x, norm_y);

        Ok(())
    }

    /// Injeta clique/soltura de botão de mouse e atualiza o estado interno.
    pub fn inject_mouse_button(
        &mut self,
        button: MouseButtonType,
        pressed: bool,
    ) -> Result<(), InputInjectionError> {
        #[cfg(windows)]
        unsafe {
            let flag = match (button, pressed) {
                (MouseButtonType::Left, true) => {
                    self.pressed_mouse_buttons |= 1;
                    MOUSEEVENTF_LEFTDOWN
                }
                (MouseButtonType::Left, false) => {
                    self.pressed_mouse_buttons &= !1;
                    MOUSEEVENTF_LEFTUP
                }
                (MouseButtonType::Right, true) => {
                    self.pressed_mouse_buttons |= 2;
                    MOUSEEVENTF_RIGHTDOWN
                }
                (MouseButtonType::Right, false) => {
                    self.pressed_mouse_buttons &= !2;
                    MOUSEEVENTF_RIGHTUP
                }
                (MouseButtonType::Middle, true) => {
                    self.pressed_mouse_buttons |= 4;
                    MOUSEEVENTF_MIDDLEDOWN
                }
                (MouseButtonType::Middle, false) => {
                    self.pressed_mouse_buttons &= !4;
                    MOUSEEVENTF_MIDDLEUP
                }
                (MouseButtonType::Other(_), _) => return Ok(()),
            };

            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: 0,
                        dy: 0,
                        mouseData: 0,
                        dwFlags: flag,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }
        #[cfg(not(windows))]
        {
            match (button, pressed) {
                (MouseButtonType::Left, true) => self.pressed_mouse_buttons |= 1,
                (MouseButtonType::Left, false) => self.pressed_mouse_buttons &= !1,
                (MouseButtonType::Right, true) => self.pressed_mouse_buttons |= 2,
                (MouseButtonType::Right, false) => self.pressed_mouse_buttons &= !2,
                (MouseButtonType::Middle, true) => self.pressed_mouse_buttons |= 4,
                (MouseButtonType::Middle, false) => self.pressed_mouse_buttons &= !4,
                _ => {}
            }
        }

        Ok(())
    }

    /// Injeta scroll de roda do mouse.
    pub fn inject_mouse_wheel(
        &mut self,
        delta_v: i16,
        delta_h: i16,
    ) -> Result<(), InputInjectionError> {
        #[cfg(windows)]
        unsafe {
            if delta_v != 0 {
                let input_v = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: delta_v as u32,
                            dwFlags: MOUSEEVENTF_WHEEL,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input_v], std::mem::size_of::<INPUT>() as i32);
            }
            if delta_h != 0 {
                let input_h = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: delta_h as u32,
                            dwFlags: MOUSEEVENTF_HWHEEL,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input_h], std::mem::size_of::<INPUT>() as i32);
            }
        }
        #[cfg(not(windows))]
        let _ = (delta_v, delta_h);

        Ok(())
    }

    /// Injeta evento de teclado usando estritamente SCAN CODE com KEYEVENTF_SCANCODE.
    pub fn inject_key(
        &mut self,
        scan_code: u16,
        extended: bool,
        pressed: bool,
    ) -> Result<(), InputInjectionError> {
        #[cfg(windows)]
        unsafe {
            let mut flags = KEYEVENTF_SCANCODE;
            if extended {
                flags |= KEYEVENTF_EXTENDEDKEY;
            }
            if !pressed {
                flags |= KEYEVENTF_KEYUP;
            }

            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: scan_code,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        }

        if pressed {
            self.pressed_keys.insert((scan_code, extended));
        } else {
            self.pressed_keys.remove(&(scan_code, extended));
        }

        Ok(())
    }

    /// Libera IMEDIATAMENTE todas as teclas, modificadores e botões de mouse que foram pressionados.
    /// Chamado obrigatoriamente ao encerrar a sessão, na perda de conexão QUIC ou queda do viewer.
    pub fn release_all(&mut self) {
        #[cfg(windows)]
        unsafe {
            // Solta todos os botões de mouse que estejam pressionados
            if self.pressed_mouse_buttons & 1 != 0 {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            if self.pressed_mouse_buttons & 2 != 0 {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_RIGHTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
            if self.pressed_mouse_buttons & 4 != 0 {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_MIDDLEUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }

            // Solta todas as teclas registradas como ativas
            for &(scan_code, extended) in &self.pressed_keys {
                let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                if extended {
                    flags |= KEYEVENTF_EXTENDEDKEY;
                }
                let input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: scan_code,
                            dwFlags: flags,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }

            // Segurança adicional: solta explicitamente os modificadores principais
            // (Ctrl: 0x1D, Alt: 0x38, Shift: 0x2A/0x36, Win: 0x5B)
            let common_modifiers = [
                (0x1D, false), // Left Ctrl
                (0x1D, true),  // Right Ctrl
                (0x38, false), // Left Alt
                (0x38, true),  // AltGr / Right Alt
                (0x2A, false), // Left Shift
                (0x36, false), // Right Shift
                (0x5B, true),  // Left Windows Key
            ];
            for &(scan_code, extended) in &common_modifiers {
                let mut flags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
                if extended {
                    flags |= KEYEVENTF_EXTENDEDKEY;
                }
                let input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: scan_code,
                            dwFlags: flags,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }

        self.pressed_keys.clear();
        self.pressed_mouse_buttons = 0;
    }
}

impl Drop for InputInjector {
    fn drop(&mut self) {
        self.release_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_tracking_and_release() {
        let mut injector = InputInjector::new(1920, 1080).expect("injector criado");
        assert_eq!(injector.active_keys_count(), 0);
        assert!(!injector.has_active_mouse_buttons());

        // Pressiona tecla 'A' (scan code 0x1E no ABNT2)
        injector
            .handle_input(&InputMessage::Key {
                scan_code: 0x1E,
                extended: false,
                pressed: true,
            })
            .expect("key press");
        assert_eq!(injector.active_keys_count(), 1);

        // Pressiona Shift esquerdo (scan code 0x2A)
        injector
            .handle_input(&InputMessage::Key {
                scan_code: 0x2A,
                extended: false,
                pressed: true,
            })
            .expect("shift press");
        assert_eq!(injector.active_keys_count(), 2);

        // Pressiona botão esquerdo do mouse
        injector
            .handle_input(&InputMessage::MouseButton {
                button: MouseButtonType::Left,
                pressed: true,
            })
            .expect("mouse press");
        assert!(injector.has_active_mouse_buttons());

        // Simula queda de sessão / encerramento
        injector.release_all();

        // Confirma que o estado foi completamente limpo e zerado
        assert_eq!(injector.active_keys_count(), 0);
        assert!(!injector.has_active_mouse_buttons());
    }

    #[test]
    fn test_key_press_and_release_flow() {
        let mut injector = InputInjector::new(1920, 1080).unwrap();

        // Pressiona 'S'
        injector.inject_key(0x1F, false, true).expect("key S down");
        assert_eq!(injector.active_keys_count(), 1);

        // Solta 'S'
        injector.inject_key(0x1F, false, false).expect("key S up");
        assert_eq!(injector.active_keys_count(), 0);
    }
}
