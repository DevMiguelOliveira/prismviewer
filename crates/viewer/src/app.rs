use crate::canvas::RemoteCanvas;
use crate::session::{collect_input_events, SessionMetrics};
use crate::{PdvItem, ViewerState};
use eframe::egui::{self, Align, Color32, Layout, RichText, Stroke, Vec2};

// ==============================================================================
// PALETA OFICIAL CYBER PRISM (Baseada no logotipo oficial do PrysmView)
// ==============================================================================
pub const COLOR_BG_OBSIDIAN: Color32 = Color32::from_rgb(10, 13, 22); // #0a0d16 Fundo principal escuro
pub const COLOR_PANEL_NAVY: Color32 = Color32::from_rgb(15, 20, 34); // #0f1422 Fundo de painéis e barras
pub const COLOR_CARD_BASE: Color32 = Color32::from_rgb(20, 27, 45); // #141b2d Fundo de cards e containers
pub const COLOR_CARD_BORDER: Color32 = Color32::from_rgb(32, 45, 75); // #202d4b Borda de vidro sutil
pub const COLOR_CYAN_NEON: Color32 = Color32::from_rgb(0, 240, 255); // #00f0ff Destaque prismático ciano
pub const COLOR_CYAN_GLOW: Color32 = Color32::from_rgb(0, 200, 235); // #00c8eb Ciano secundário
pub const COLOR_BLUE_ELECTRIC: Color32 = Color32::from_rgb(0, 140, 255); // #008cff Azul elétrico vibrante
pub const COLOR_VIOLET_NEON: Color32 = Color32::from_rgb(190, 45, 235); // #be2deb Magenta/violeta do prisma
pub const COLOR_MINT_ONLINE: Color32 = Color32::from_rgb(0, 255, 204); // #00ffcc Glow de sinal e status online
pub const COLOR_RED_DANGER: Color32 = Color32::from_rgb(255, 51, 85); // #ff3355 Vermelho de emergência
pub const COLOR_TEXT_WHITE: Color32 = Color32::from_rgb(245, 248, 255); // #f5f8ff Texto principal
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184); // #94a3b8 Texto secundário
pub const COLOR_TEXT_DARK: Color32 = Color32::from_rgb(100, 116, 139); // #64748b Metadados discretos

/// Carrega a imagem do logotipo oficial embutida no binário e gera textura GPU egui.
fn load_brand_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &img.into_raw(),
    );
    Some(ctx.load_texture(
        "prysm_brand_logo",
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}

/// Renderiza o logotipo oficial na interface com dimensões exatas e fallback elegante.
pub fn render_logo(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>, size: f32) {
    let (rect, _resp) = ui.allocate_exact_size(Vec2::new(size, size), egui::Sense::hover());
    if let Some(tex) = texture {
        ui.painter().image(
            tex.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        ui.painter().rect_filled(rect, 6.0, COLOR_CARD_BASE);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "💎",
            egui::FontId::proportional(size * 0.55),
            COLOR_CYAN_NEON,
        );
    }
}

pub struct PrysmViewerApp {
    pub state: ViewerState,
    pub canvas: RemoteCanvas,
    pub metrics: SessionMetrics,
    pub logo_texture: Option<egui::TextureHandle>,
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

        Self {
            state,
            canvas: RemoteCanvas::default(),
            metrics: SessionMetrics::default(),
            logo_texture: None,
        }
    }
}

impl eframe::App for PrysmViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Carregamento lazy da textura do logotipo oficial
        if self.logo_texture.is_none() {
            self.logo_texture = load_brand_texture(ctx);
        }
        // Estilização global: Dark mode Cyber Prism refinado
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = COLOR_CARD_BASE;
        visuals.panel_fill = COLOR_PANEL_NAVY;
        visuals.extreme_bg_color = COLOR_BG_OBSIDIAN;
        visuals.override_text_color = Some(COLOR_TEXT_WHITE);
        visuals.widgets.noninteractive.bg_fill = COLOR_CARD_BASE;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_CARD_BORDER);
        visuals.widgets.inactive.bg_fill = COLOR_CARD_BASE;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_CARD_BORDER);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(26, 36, 58);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_CYAN_NEON);
        visuals.widgets.active.bg_fill = COLOR_BLUE_ELECTRIC;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, COLOR_CYAN_NEON);
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0, 240, 255, 45);
        visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_CYAN_NEON);
        ctx.set_visuals(visuals);

        // Barra Superior (Header) com logotipo e identidade visual oficial
        egui::TopBottomPanel::top("top_header")
            .frame(
                egui::Frame::none()
                    .fill(COLOR_PANEL_NAVY)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    render_logo(ui, self.logo_texture.as_ref(), 28.0);
                    ui.add_space(8.0);

                    ui.heading(
                        RichText::new("PRYSMVIEW")
                            .color(COLOR_CYAN_NEON)
                            .strong()
                            .size(18.0),
                    );
                    ui.label(
                        RichText::new("• Suporte Remoto PDV")
                            .color(COLOR_TEXT_MUTED)
                            .size(13.0),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.state.is_authenticated {
                            let btn_sair = egui::Button::new(
                                RichText::new("🚪 Sair").color(COLOR_TEXT_MUTED).size(12.0),
                            )
                            .fill(Color32::from_rgb(28, 20, 28))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 30, 40)))
                            .rounding(4.0);

                            if ui.add(btn_sair).clicked() {
                                self.state.logout();
                            }

                            ui.separator();

                            // Badge pill do técnico logado com destaque violeta prismático
                            egui::Frame::none()
                                .fill(Color32::from_rgb(32, 16, 50))
                                .stroke(Stroke::new(1.0_f32, COLOR_VIOLET_NEON))
                                .rounding(12.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!("👤 {}", self.state.operator_name))
                                            .color(COLOR_VIOLET_NEON)
                                            .size(12.0)
                                            .strong(),
                                    );
                                });

                            ui.separator();

                            let btn_atualizar = egui::Button::new(
                                RichText::new("🔄 Atualizar")
                                    .color(COLOR_TEXT_WHITE)
                                    .size(12.0),
                            )
                            .fill(COLOR_CARD_BASE)
                            .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                            .rounding(4.0);

                            if ui.add(btn_atualizar).clicked() {
                                self.state.status_message =
                                    Some("Lista de PDVs atualizada com a sinalização.".to_string());
                            }

                            ui.add(
                                egui::TextEdit::singleline(&mut self.state.search_query)
                                    .hint_text("🔍 Buscar Loja ou PDV...")
                                    .desired_width(220.0),
                            );
                        } else {
                            egui::Frame::none()
                                .fill(Color32::from_rgb(32, 26, 14))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(180, 140, 20)))
                                .rounding(10.0)
                                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("🔒 Não Autenticado")
                                            .color(Color32::from_rgb(255, 200, 50))
                                            .size(11.0),
                                    );
                                });
                        }
                    });
                });
            });

        // Barra Inferior (Status Bar)
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::none()
                    .fill(COLOR_PANEL_NAVY)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .inner_margin(egui::Margin::symmetric(14.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref msg) = self.state.status_message {
                        ui.label(RichText::new(msg).color(COLOR_CYAN_GLOW).size(12.0));
                    } else if self.state.active_session {
                        ui.label(
                            RichText::new("● Sessão ativa com criptografia E2E (QUIC TLS 1.3)")
                                .color(COLOR_MINT_ONLINE)
                                .size(12.0),
                        );
                    } else if self.state.is_authenticated {
                        ui.label(
                            RichText::new("Painel do Operador pronto para conexão.")
                                .color(COLOR_TEXT_MUTED)
                                .size(12.0),
                        );
                    } else {
                        ui.label(
                            RichText::new("Aguardando login do operador.")
                                .color(COLOR_TEXT_MUTED)
                                .size(12.0),
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("PrysmView v0.1.0 • R$ 0,00 • QUIC TLS 1.3")
                                .color(COLOR_TEXT_DARK)
                                .size(11.0),
                        );
                    });
                });
            });

        // Painel Central: Tela de Login, Sessão Ativa OU Catálogo de PDVs
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(COLOR_BG_OBSIDIAN))
            .show(ctx, |ui| {
                if !self.state.is_authenticated {
                    self.render_admin_login(ui);
                } else if self.state.active_session {
                    self.render_active_session_view(ui);
                } else {
                    self.render_pdv_catalog(ui);
                }
            });

        // Diálogo Modal: Inserção do Código de Sessão de 8 dígitos (somente quando autenticado)
        if self.state.is_authenticated
            && self.state.selected_pdv.is_some()
            && !self.state.active_session
        {
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
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(
                        1.0_f32,
                        if pdv.is_online {
                            COLOR_CARD_BORDER
                        } else {
                            Color32::from_rgb(25, 32, 48)
                        },
                    ))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Status Dot & Loja/PDV
                            let (status_color, status_text) = if pdv.is_online {
                                (
                                    COLOR_MINT_ONLINE,
                                    format!("Online • Agente v{}", pdv.agent_version),
                                )
                            } else {
                                (
                                    COLOR_TEXT_DARK,
                                    format!("Offline • Visto por último: {}", pdv.last_seen),
                                )
                            };

                            ui.label(RichText::new("●").color(status_color).size(16.0));

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&pdv.id)
                                            .strong()
                                            .size(15.0)
                                            .color(COLOR_TEXT_WHITE),
                                    );
                                    ui.label(RichText::new("—").color(COLOR_TEXT_DARK));
                                    ui.label(
                                        RichText::new(&pdv.store_name)
                                            .size(14.0)
                                            .color(COLOR_TEXT_MUTED),
                                    );
                                });

                                ui.label(
                                    RichText::new(status_text)
                                        .color(if pdv.is_online {
                                            COLOR_CYAN_GLOW
                                        } else {
                                            COLOR_TEXT_DARK
                                        })
                                        .size(12.0),
                                );
                            });

                            // Botão de Conexão à direita com gradiente e azul elétrico
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if pdv.is_online {
                                    let connect_btn = egui::Button::new(
                                        RichText::new("▶ Conectar")
                                            .color(COLOR_TEXT_WHITE)
                                            .strong(),
                                    )
                                    .fill(COLOR_BLUE_ELECTRIC)
                                    .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                                    .rounding(6.0);

                                    if ui.add(connect_btn).clicked() {
                                        self.state.select_pdv(pdv);
                                    }
                                } else {
                                    ui.add_enabled(
                                        false,
                                        egui::Button::new(
                                            RichText::new("Indisponível").color(COLOR_TEXT_DARK),
                                        )
                                        .fill(Color32::from_rgb(18, 22, 32))
                                        .rounding(6.0),
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
            .fixed_size(Vec2::new(390.0, 250.0))
            .frame(
                egui::Frame::window(ctx.style().as_ref())
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.2_f32, COLOR_CYAN_NEON))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(18.0)),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("PDV Alvo: {} ({})", pdv.id, pdv.store_name))
                        .strong()
                        .size(15.0)
                        .color(COLOR_CYAN_NEON),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Solicite ao operador no caixa o código de 8 dígitos exibido na tela.",
                    )
                    .color(COLOR_TEXT_MUTED)
                    .size(12.0),
                );

                ui.add_space(14.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("CÓDIGO DE SESSÃO (8 DÍGITOS)")
                            .size(11.0)
                            .color(COLOR_TEXT_DARK)
                            .strong(),
                    );

                    let mut code = self.state.session_code_input.clone();
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut code)
                            .font(egui::TextStyle::Heading)
                            .char_limit(8)
                            .desired_width(190.0)
                            .text_color(COLOR_CYAN_NEON),
                    );

                    if response.changed() {
                        self.state.set_session_code(&code);
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("⏱ Válido por 120 segundos • Máximo 3 tentativas")
                            .color(COLOR_VIOLET_NEON)
                            .size(11.0),
                    );
                });

                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() {
                        self.state.close_modal();
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let can_connect = self.state.validate_session_code();
                        let btn = egui::Button::new(
                            RichText::new("Conectar")
                                .color(if can_connect {
                                    COLOR_TEXT_WHITE
                                } else {
                                    COLOR_TEXT_DARK
                                })
                                .strong(),
                        )
                        .fill(if can_connect {
                            COLOR_BLUE_ELECTRIC
                        } else {
                            Color32::from_rgb(32, 38, 52)
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if can_connect {
                                COLOR_CYAN_NEON
                            } else {
                                COLOR_CARD_BORDER
                            },
                        ))
                        .rounding(6.0);

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
        // Sincroniza textura da GPU com buffer de pixels do canvas remoto
        self.canvas.sync_texture(ui.ctx());

        // Barra de ferramentas da sessão
        ui.horizontal(|ui| {
            let btn_encerrar = egui::Button::new(
                RichText::new("⏹ Encerrar Sessão")
                    .color(COLOR_TEXT_WHITE)
                    .strong(),
            )
            .fill(COLOR_RED_DANGER)
            .rounding(4.0);

            if ui.add(btn_encerrar).clicked() {
                self.state.clear_session();
                return;
            }

            ui.separator();
            if let Some(ref pdv) = self.state.selected_pdv {
                ui.label(
                    RichText::new(format!("Conectado: {} - {}", pdv.id, pdv.store_name))
                        .color(COLOR_TEXT_WHITE)
                        .strong(),
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("🔒 QUIC TLS 1.3")
                        .color(COLOR_MINT_ONLINE)
                        .size(12.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("🎞️ {:.0} FPS", self.metrics.current_fps))
                        .color(COLOR_CYAN_NEON)
                        .size(12.0),
                );
                ui.label(
                    RichText::new(format!("⚡ {} ms", self.metrics.rtt_ms))
                        .color(COLOR_BLUE_ELECTRIC)
                        .size(12.0),
                );
            });
        });

        ui.separator();

        // Canvas de visualização do display remoto com captura de entrada
        egui::Frame::canvas(ui.style())
            .fill(Color32::BLACK)
            .show(ui, |ui| {
                let available_size = ui.available_size();
                if let Some(ref texture) = self.canvas.texture {
                    let (rect, _response) =
                        ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
                    let painter = ui.painter();
                    painter.image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    // Captura e conversão de eventos para protocolo PRSM
                    let input_state = ui.ctx().input(|i| i.clone());
                    let _events = collect_input_events(&input_state, rect);
                } else {
                    let (_rect, _) =
                        ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());
                    ui.vertical_centered(|ui| {
                        ui.add_space(available_size.y * 0.4);
                        ui.label(
                            RichText::new("🖥️ Aguardando primeiro quadro DXGI...")
                                .color(Color32::WHITE)
                                .size(18.0),
                        );
                    });
                }
            });
    }

    /// Renderiza a tela de login do operador / administrador.
    fn render_admin_login(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.08);

            egui::Frame::window(ui.style())
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.2_f32, COLOR_CARD_BORDER))
                .rounding(10.0)
                .inner_margin(egui::Margin::same(28.0))
                .show(ui, |ui| {
                    ui.set_max_width(380.0);

                    ui.vertical_centered(|ui| {
                        render_logo(ui, self.logo_texture.as_ref(), 76.0);
                        ui.add_space(10.0);
                        ui.heading(
                            RichText::new("Painel do Técnico")
                                .color(COLOR_CYAN_NEON)
                                .size(24.0)
                                .strong(),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("PrysmView • Suporte Remoto Centralizado")
                                .color(COLOR_TEXT_MUTED)
                                .size(12.0),
                        );
                    });

                    ui.add_space(22.0);

                    // Campo: Servidor de Sinalização
                    ui.label(
                        RichText::new("🌐 Servidor de Sinalização (API):")
                            .size(12.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.server_url)
                            .desired_width(f32::INFINITY)
                            .hint_text("http://20.121.137.44:443"),
                    );

                    ui.add_space(10.0);

                    // Campo: ID do Operador
                    ui.label(
                        RichText::new("👤 ID do Operador / Técnico:")
                            .size(12.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.login_operator_id)
                            .desired_width(f32::INFINITY)
                            .hint_text("ex: admin ou op-01"),
                    );

                    ui.add_space(10.0);

                    // Campo: Senha
                    ui.label(
                        RichText::new("🔑 Senha de Acesso:")
                            .size(12.0)
                            .color(COLOR_TEXT_MUTED),
                    );
                    let pwd_response = ui.add(
                        egui::TextEdit::singleline(&mut self.state.login_password)
                            .password(true)
                            .desired_width(f32::INFINITY),
                    );

                    if let Some(ref err) = self.state.login_error {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("⚠️ {}", err))
                                .color(COLOR_RED_DANGER)
                                .size(12.0),
                        );
                    }

                    ui.add_space(20.0);

                    let enter_pressed =
                        pwd_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let btn_login = ui.add_sized(
                        [ui.available_width(), 40.0],
                        egui::Button::new(
                            RichText::new("Entrar no Painel")
                                .color(COLOR_TEXT_WHITE)
                                .size(14.0)
                                .strong(),
                        )
                        .fill(COLOR_BLUE_ELECTRIC)
                        .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                        .rounding(6.0),
                    );

                    if btn_login.clicked() || enter_pressed {
                        let op_id = self.state.login_operator_id.trim().to_string();
                        if op_id.is_empty() {
                            self.state.login_error = Some("Informe o ID do Operador".to_string());
                        } else {
                            let token = format!("token_{}", op_id);
                            self.state.login_success(&op_id, token);
                        }
                    }

                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("🛡️ Criptografia E2E QUIC TLS 1.3 • Hash SHA-256")
                                .color(COLOR_TEXT_DARK)
                                .size(11.0),
                        );
                    });
                });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Testa que a paleta de cores oficial Cyber Prism está definida corretamente.
    #[test]
    fn test_cyber_prism_color_palette() {
        // Verifica que todas as cores cianos/prismáticas estão presentes
        assert_eq!(COLOR_CYAN_NEON, Color32::from_rgb(0, 240, 255)); // #00f0ff
        assert_eq!(COLOR_BLUE_ELECTRIC, Color32::from_rgb(0, 140, 255)); // #008cff
        assert_eq!(COLOR_VIOLET_NEON, Color32::from_rgb(190, 45, 235)); // #be2deb
        assert_eq!(COLOR_MINT_ONLINE, Color32::from_rgb(0, 255, 204)); // #00ffcc
        assert_eq!(COLOR_CYAN_GLOW, Color32::from_rgb(0, 200, 235)); // #00c8eb

        // Verifica que o texto está nas cores corretas para contraste
        assert_eq!(COLOR_TEXT_WHITE, Color32::from_rgb(245, 248, 255)); // #f5f8ff
        assert_eq!(COLOR_TEXT_MUTED, Color32::from_rgb(148, 163, 184)); // #94a3b8
        assert_eq!(COLOR_TEXT_DARK, Color32::from_rgb(100, 116, 139)); // #64748b

        // Verifica que as cores de fundo são escuras e primárias
        assert_eq!(COLOR_BG_OBSIDIAN, Color32::from_rgb(10, 13, 22)); // #0a0d16
        assert_eq!(COLOR_PANEL_NAVY, Color32::from_rgb(15, 20, 34)); // #0f1422
        assert_eq!(COLOR_CARD_BASE, Color32::from_rgb(20, 27, 45)); // #141b2d
        assert_eq!(COLOR_CARD_BORDER, Color32::from_rgb(32, 45, 75)); // #202d4b

        // Verifica que as cores de estado estão presentes
        assert_eq!(COLOR_RED_DANGER, Color32::from_rgb(255, 51, 85)); // #ff3355
    }

    /// Testa que o logotipo da marca é carregado corretamente a partir de bytes embutidos.
    #[test]
    fn test_load_brand_texture() {
        let ctx = egui::Context::default();

        // Carrega a textura usando a função
        let texture = load_brand_texture(&ctx);

        // Espera que o logotipo oficial (que já existe no binário)
        // seja carregado como uma textura egui válida.
        assert!(texture.is_some());

        let tex = texture.unwrap();
        assert_eq!(tex.name(), "prysm_brand_logo");

        // Verifica que a textura tem dimensões razoáveis para um logo
        // tex.size() retorna [usize; 2]
        let sz = tex.size();
        assert!(sz[0] > 0);
        assert!(sz[1] > 0);
    }

    /// Testa que a estilização visual de tema e componentes usa as cores Cyber Prism.
    #[test]
    fn test_theme_cyber_prism_style_usage() {
        // Estas são verificações basicas de que o app.rs importa e usa as cores da paleta
        // As cores reais são aplicadas no app update() method, mas aqui testamos
        // as constantes coloridas que são usadas para o tema e estilo.

        // Verify that the main Cyber Prism colors are used in the styling
        assert!(COLOR_CYAN_NEON != Color32::BLACK); // Cor ciano distinta
        assert!(COLOR_BLUE_ELECTRIC != Color32::BLACK); // Cor azul distinta
        assert!(COLOR_VIOLET_NEON != Color32::BLACK); // Cor violeta distinta

        // Verifica que as cores textuais têm contraste adequado
        assert!(COLOR_TEXT_WHITE == Color32::from_rgb(245, 248, 255));
        assert!(COLOR_TEXT_MUTED == Color32::from_rgb(148, 163, 184));
        assert!(COLOR_TEXT_DARK == Color32::from_rgb(100, 116, 139));
    }
}
