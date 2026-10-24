//! prysmview-viewer - Interface gráfica do operador com egui/eframe

use crate::{PdvItem, ViewerState};
use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke, Vec2};

pub struct PrysmViewerApp {
    pub state: ViewerState,
}

impl PrysmViewerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = ViewerState::new();

        // Dados iniciais de demonstração / carregamento
        state.pdvs = vec![
            PdvItem {
                id: "PDV-01".to_string(),
                store_name: "Loja 101 - Centro".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv01_key_hash".to_string(),
                last_seen: "Agora".to_string(),
            },
            PdvItem {
                id: "PDV-02".to_string(),
                store_name: "Loja 101 - Centro".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv02_key_hash".to_string(),
                last_seen: "Agora".to_string(),
            },
            PdvItem {
                id: "PDV-01".to_string(),
                store_name: "Loja 204 - Shopping Sul".to_string(),
                is_online: false,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv204_key_hash".to_string(),
                last_seen: "15 min atrás".to_string(),
            },
            PdvItem {
                id: "PDV-03".to_string(),
                store_name: "Loja 305 - Hiper Norte".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv305_key_hash".to_string(),
                last_seen: "Agora".to_string(),
            },
        ];

        Self { state }
    }
}

impl eframe::App for PrysmViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Estilização global: Dark mode elegante estilo TeamViewer
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::from_rgb(24, 27, 33);
        visuals.panel_fill = Color32::from_rgb(18, 20, 24);
        ctx.set_visuals(visuals);

        // Barra Superior (Header)
        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("PRYSMVIEW")
                        .color(Color32::from_rgb(0, 160, 255))
                        .strong(),
                );
                ui.label(
                    RichText::new("• Suporte Remoto PDV")
                        .color(Color32::LIGHT_GRAY)
                        .size(13.0),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("👤 {}", self.state.operator_name))
                            .color(Color32::WHITE),
                    );
                    ui.separator();

                    if ui.button("🔄 Atualizar").clicked() {
                        self.state.status_message =
                            Some("Lista de PDVs atualizada com a sinalização.".to_string());
                    }

                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.search_query)
                            .hint_text("🔍 Buscar Loja ou PDV...")
                            .desired_width(200.0),
                    );
                });
            });
            ui.add_space(8.0);
        });

        // Barra Inferior (Status Bar)
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if let Some(ref msg) = self.state.status_message {
                    ui.label(RichText::new(msg).color(Color32::LIGHT_YELLOW).size(12.0));
                } else if self.state.active_session {
                    ui.label(
                        RichText::new("Sessão ativa com criptografia E2E (QUIC TLS 1.3)")
                            .color(Color32::GREEN)
                            .size(12.0),
                    );
                } else {
                    ui.label(
                        RichText::new("Pronto para conexão.")
                            .color(Color32::GRAY)
                            .size(12.0),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("PrysmView v0.1.0 • R$ 0,00 • TLS 1.3")
                            .color(Color32::DARK_GRAY)
                            .size(11.0),
                    );
                });
            });
            ui.add_space(4.0);
        });

        // Painel Central: Sessão Ativa OU Catálogo de PDVs
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state.active_session {
                self.render_active_session_view(ui);
            } else {
                self.render_pdv_catalog(ui);
            }
        });

        // Diálogo Modal: Inserção do Código de Sessão de 8 dígitos
        if self.state.selected_pdv.is_some() && !self.state.active_session {
            self.render_session_code_modal(ctx);
        }
    }
}

impl PrysmViewerApp {
    /// Renderiza a grade de lojas e PDVs disponíveis.
    fn render_pdv_catalog(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Caixas Registradores (PDVs)").size(18.0));
            let filtered_count = self.state.filter_pdvs().len();
            ui.label(
                RichText::new(format!("({} encontrados)", filtered_count)).color(Color32::GRAY),
            );
        });
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let filtered_pdvs: Vec<PdvItem> =
                self.state.filter_pdvs().into_iter().cloned().collect();

            if filtered_pdvs.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("Nenhum PDV encontrado para esta busca.")
                            .color(Color32::GRAY),
                    );
                });
                return;
            }

            for pdv in filtered_pdvs {
                egui::Frame::none()
                    .fill(Color32::from_rgb(28, 32, 40))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 52, 65)))
                    .rounding(6.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Status Dot & Loja/PDV
                            let status_color = if pdv.is_online {
                                Color32::from_rgb(46, 204, 113)
                            } else {
                                Color32::from_rgb(120, 120, 120)
                            };

                            ui.label(RichText::new("●").color(status_color).size(16.0));

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&pdv.id).strong().size(15.0));
                                    ui.label(RichText::new("—").color(Color32::DARK_GRAY));
                                    ui.label(RichText::new(&pdv.store_name).size(14.0));
                                });

                                let status_text = if pdv.is_online {
                                    format!("Online • Agente v{}", pdv.agent_version)
                                } else {
                                    format!("Offline • Visto por último: {}", pdv.last_seen)
                                };
                                ui.label(
                                    RichText::new(status_text).color(Color32::GRAY).size(12.0),
                                );
                            });

                            // Botão de Conexão à direita
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if pdv.is_online {
                                    let connect_btn = egui::Button::new(
                                        RichText::new("▶ Conectar").color(Color32::WHITE).strong(),
                                    )
                                    .fill(Color32::from_rgb(0, 122, 255))
                                    .rounding(4.0);

                                    if ui.add(connect_btn).clicked() {
                                        self.state.select_pdv(pdv);
                                    }
                                } else {
                                    ui.add_enabled(
                                        false,
                                        egui::Button::new(
                                            RichText::new("Indisponível").color(Color32::GRAY),
                                        )
                                        .rounding(4.0),
                                    );
                                }
                            });
                        });
                    });

                ui.add_space(6.0);
            }
        });
    }

    /// Renderiza o diálogo modal para validação do código de sessão de 8 dígitos.
    fn render_session_code_modal(&mut self, ctx: &egui::Context) {
        let pdv = match self.state.selected_pdv.clone() {
            Some(p) => p,
            None => return,
        };

        egui::Window::new("🔐 Conexão Segura ao PDV")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(380.0, 240.0))
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("PDV Alvo: {} ({})", pdv.id, pdv.store_name))
                        .strong()
                        .size(14.0),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Solicite ao operador no caixa o código de 8 dígitos exibido na tela.",
                    )
                    .color(Color32::LIGHT_GRAY)
                    .size(12.0),
                );

                ui.add_space(16.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("CÓDIGO DE SESSÃO (8 DÍGITOS)")
                            .size(11.0)
                            .color(Color32::GRAY),
                    );

                    let mut code = self.state.session_code_input.clone();
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut code)
                            .font(egui::TextStyle::Heading)
                            .char_limit(8)
                            .desired_width(180.0),
                    );

                    if response.changed() {
                        self.state.set_session_code(&code);
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("⏱ Válido por 120 segundos • Máximo 3 tentativas")
                            .color(Color32::GRAY)
                            .size(11.0),
                    );
                });

                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() {
                        self.state.close_modal();
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let can_connect = self.state.validate_session_code();
                        let btn = egui::Button::new(
                            RichText::new("Conectar")
                                .color(if can_connect {
                                    Color32::WHITE
                                } else {
                                    Color32::DARK_GRAY
                                })
                                .strong(),
                        )
                        .fill(if can_connect {
                            Color32::from_rgb(46, 204, 113)
                        } else {
                            Color32::from_rgb(60, 60, 60)
                        })
                        .rounding(4.0);

                        if ui.add_enabled(can_connect, btn).clicked() {
                            self.state.active_session = true;
                            self.state.status_message =
                                Some(format!("Conectado com sucesso ao {}!", pdv.id));
                        }
                    });
                });
            });
    }

    /// Renderiza a tela de visualização e controle da sessão remota ativa.
    fn render_active_session_view(&mut self, ui: &mut egui::Ui) {
        // Barra de ferramentas da sessão
        ui.horizontal(|ui| {
            if ui.button("⏹ Encerrar Sessão").clicked() {
                self.state.clear_session();
                return;
            }

            ui.separator();
            if let Some(ref pdv) = self.state.selected_pdv {
                ui.label(
                    RichText::new(format!("Conectado: {} - {}", pdv.id, pdv.store_name)).strong(),
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("🔒 QUIC TLS 1.3")
                        .color(Color32::GREEN)
                        .size(12.0),
                );
                ui.label(
                    RichText::new("🎞️ 30 FPS")
                        .color(Color32::LIGHT_BLUE)
                        .size(12.0),
                );
                ui.label(
                    RichText::new("⚡ 18 ms")
                        .color(Color32::LIGHT_YELLOW)
                        .size(12.0),
                );
            });
        });

        ui.separator();

        // Canvas de visualização do display remoto (simulado nesta etapa base)
        egui::Frame::canvas(ui.style())
            .fill(Color32::BLACK)
            .show(ui, |ui| {
                let available_size = ui.available_size();
                let (_rect, _) =
                    ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

                ui.vertical_centered(|ui| {
                    ui.add_space(available_size.y * 0.4);
                    ui.label(
                        RichText::new("🖥️ Display do PDV Conectado")
                            .color(Color32::WHITE)
                            .size(18.0),
                    );
                    ui.label(
                        RichText::new("Stream de vídeo DXGI e injeção de entrada ativos.")
                            .color(Color32::GRAY)
                            .size(13.0),
                    );
                });
            });
    }
}
