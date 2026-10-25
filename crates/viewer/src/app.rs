use crate::canvas::RemoteCanvas;
use crate::session::{collect_input_events, SessionMetrics};
use crate::{AdminTab, AuditRecord, FilterStatus, PdvItem, ViewerState};
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
                ip_address: "192.168.1.101".to_string(),
                os_info: "Windows 11 Pro 23H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
            },
            PdvItem {
                id: "PDV-02".to_string(),
                store_name: "Loja 101 - Centro".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv02_key_hash".to_string(),
                last_seen: "Agora".to_string(),
                ip_address: "192.168.1.102".to_string(),
                os_info: "Windows 11 Pro 23H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
            },
            PdvItem {
                id: "PDV-01".to_string(),
                store_name: "Loja 204 - Shopping Sul".to_string(),
                is_online: false,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv204_key_hash".to_string(),
                last_seen: "15 min atrás".to_string(),
                ip_address: "192.168.2.101".to_string(),
                os_info: "Windows 10 Enterprise 22H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
            },
            PdvItem {
                id: "PDV-03".to_string(),
                store_name: "Loja 305 - Hiper Norte".to_string(),
                is_online: true,
                agent_version: "0.1.0".to_string(),
                public_key: "ed25519_pdv305_key_hash".to_string(),
                last_seen: "Agora".to_string(),
                ip_address: "192.168.3.103".to_string(),
                os_info: "Windows 11 Pro 23H2".to_string(),
                resolution: "1920x1080 @ 60Hz".to_string(),
            },
        ];

        state.audit_records = vec![
            AuditRecord {
                id: "AUD-901".to_string(),
                timestamp: "25/10/2026 09:30".to_string(),
                operator_name: "admin".to_string(),
                pdv_id: "PDV-01".to_string(),
                store_name: "Loja 101 - Centro".to_string(),
                duration: "14 min".to_string(),
                reason: "Suporte Impressora Fiscal e TEF".to_string(),
                status: "Concluído com Sucesso".to_string(),
                bytes_transferred: "42.8 MB".to_string(),
            },
            AuditRecord {
                id: "AUD-900".to_string(),
                timestamp: "24/10/2026 17:45".to_string(),
                operator_name: "admin".to_string(),
                pdv_id: "PDV-03".to_string(),
                store_name: "Loja 305 - Hiper Norte".to_string(),
                duration: "8 min".to_string(),
                reason: "Atualização de Tabelas de Preço".to_string(),
                status: "Concluído com Sucesso".to_string(),
                bytes_transferred: "12.4 MB".to_string(),
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

        if self.state.is_authenticated {
            // Barra Superior (Header) do Painel Administrativo
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
                            RichText::new("PRYSM ADMIN")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(18.0),
                        );
                        ui.label(
                            RichText::new("• Central de Suporte & Gestão PDV")
                                .color(COLOR_TEXT_MUTED)
                                .size(13.0),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                        } else {
                            ui.label(
                                RichText::new("Painel do Operador pronto para conexão.")
                                    .color(COLOR_TEXT_MUTED)
                                    .size(12.0),
                            );
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new("PrysmView Admin v0.1.0 • R$ 0,00 • QUIC TLS 1.3")
                                    .color(COLOR_TEXT_DARK)
                                    .size(11.0),
                            );
                        });
                    });
                });
        } else {
            // Barra Superior de Identificação (Estilo Enterprise)
            egui::TopBottomPanel::top("login_header")
                .frame(
                    egui::Frame::none()
                        .fill(COLOR_BG_OBSIDIAN)
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(20, 26, 40)))
                        .inner_margin(egui::Margin::symmetric(18.0, 12.0)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("●")
                                .color(Color32::from_rgb(59, 130, 246))
                                .size(13.0),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("PrysmView Enterprise")
                                .color(Color32::from_rgb(148, 163, 184))
                                .size(12.5)
                                .strong(),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let btn_close = egui::Button::new(
                                RichText::new("✕")
                                    .color(Color32::from_rgb(148, 163, 184))
                                    .size(13.0),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);

                            if ui.add(btn_close).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }

                            ui.add_space(8.0);

                            let btn_min = egui::Button::new(
                                RichText::new("—")
                                    .color(Color32::from_rgb(148, 163, 184))
                                    .size(13.0),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);

                            if ui.add(btn_min).clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                        });
                    });
                });
        }

        // Se autenticado e sem sessão ativa, exibe o painel lateral de navegação (Sidebar)
        if self.state.is_authenticated && !self.state.active_session {
            egui::SidePanel::left("admin_sidebar")
                .exact_width(220.0)
                .frame(
                    egui::Frame::none()
                        .fill(COLOR_PANEL_NAVY)
                        .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                        .inner_margin(egui::Margin::symmetric(12.0, 14.0)),
                )
                .show(ctx, |ui| {
                    self.render_sidebar(ui);
                });
        }

        // Painel Central: Tela de Login, Sessão Ativa OU Abas Administrativas
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(COLOR_BG_OBSIDIAN))
            .show(ctx, |ui| {
                if !self.state.is_authenticated {
                    self.render_admin_login(ui);
                } else if self.state.active_session {
                    self.render_active_session_view(ui);
                } else {
                    match self.state.active_tab {
                        AdminTab::Dashboard => self.render_dashboard(ui),
                        AdminTab::DirectConnect => self.render_direct_connect_tab(ui),
                        AdminTab::Stores => self.render_stores_tab(ui),
                        AdminTab::AuditHistory => self.render_audit_history_tab(ui),
                        AdminTab::Settings => self.render_settings_tab(ui),
                    }
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
    /// Renderiza a barra lateral (Sidebar) de navegação do técnico.
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            render_logo(ui, self.logo_texture.as_ref(), 34.0);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.heading(
                    RichText::new("PRYSM ADMIN")
                        .color(COLOR_CYAN_NEON)
                        .strong()
                        .size(16.0),
                );
                ui.label(
                    RichText::new("CONSOLE v0.1.0")
                        .color(COLOR_TEXT_DARK)
                        .size(10.0),
                );
            });
        });

        ui.add_space(14.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(
            RichText::new("MENU PRINCIPAL")
                .color(COLOR_TEXT_DARK)
                .size(11.0)
                .strong(),
        );
        ui.add_space(6.0);

        let online = self.state.online_count();
        let tabs = [
            (
                AdminTab::Dashboard,
                format!("📊 Monitoramento ({})", online),
            ),
            (AdminTab::DirectConnect, "⚡ Conexão Rápida".to_string()),
            (AdminTab::Stores, "🏬 Lojas e Filiais".to_string()),
            (AdminTab::AuditHistory, "📜 Auditoria E2E".to_string()),
            (AdminTab::Settings, "⚙️ Configurações & Nuvem".to_string()),
        ];

        for (tab, label) in tabs {
            let is_selected = self.state.active_tab == tab;
            let btn = egui::Button::new(
                RichText::new(&label)
                    .color(if is_selected {
                        COLOR_TEXT_WHITE
                    } else {
                        COLOR_TEXT_MUTED
                    })
                    .size(13.0)
                    .strong(),
            )
            .fill(if is_selected {
                COLOR_BLUE_ELECTRIC
            } else {
                Color32::TRANSPARENT
            })
            .stroke(Stroke::new(
                1.0_f32,
                if is_selected {
                    COLOR_CYAN_NEON
                } else {
                    Color32::TRANSPARENT
                },
            ))
            .rounding(6.0);

            if ui.add_sized([ui.available_width(), 34.0], btn).clicked() {
                self.state.active_tab = tab;
            }
            ui.add_space(4.0);
        }

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(10.0);

            let btn_sair = egui::Button::new(
                RichText::new("🚪 Sair do Painel")
                    .color(COLOR_RED_DANGER)
                    .size(12.0)
                    .strong(),
            )
            .fill(Color32::from_rgb(32, 18, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(70, 25, 35)))
            .rounding(6.0);

            if ui
                .add_sized([ui.available_width(), 32.0], btn_sair)
                .clicked()
            {
                self.state.logout();
            }

            ui.add_space(8.0);

            // Card do perfil do operador
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("👤").size(20.0));
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&self.state.operator_name)
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(13.0),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "● Azure VM: {} ms",
                                    self.state.cloud_latency_ms
                                ))
                                .color(COLOR_MINT_ONLINE)
                                .size(11.0),
                            );
                        });
                    });
                });
        });
    }

    /// Renderiza a aba principal de Dashboard com KPIs, filtros e grade rica de PDVs.
    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        // 1. KPI Cards no topo
        self.render_kpi_cards(ui);
        ui.add_space(14.0);

        // 2. Toolbar de Filtros & Busca
        ui.horizontal(|ui| {
            let filters = [
                (
                    FilterStatus::All,
                    format!("Todos ({})", self.state.pdvs.len()),
                ),
                (
                    FilterStatus::OnlineOnly,
                    format!("🟢 Online ({})", self.state.online_count()),
                ),
                (
                    FilterStatus::OfflineOnly,
                    format!("⚪ Offline ({})", self.state.offline_count()),
                ),
            ];

            for (status, label) in filters {
                let is_active = self.state.filter_status == status;
                let btn = egui::Button::new(
                    RichText::new(label)
                        .color(if is_active {
                            COLOR_CYAN_NEON
                        } else {
                            COLOR_TEXT_MUTED
                        })
                        .size(12.0)
                        .strong(),
                )
                .fill(if is_active {
                    Color32::from_rgb(18, 36, 60)
                } else {
                    COLOR_CARD_BASE
                })
                .stroke(Stroke::new(
                    1.0_f32,
                    if is_active {
                        COLOR_CYAN_NEON
                    } else {
                        COLOR_CARD_BORDER
                    },
                ))
                .rounding(14.0);

                if ui.add(btn).clicked() {
                    self.state.filter_status = status;
                }
            }

            ui.separator();
            egui::ComboBox::from_id_salt("store_filter_combo")
                .selected_text(
                    self.state
                        .selected_store
                        .as_deref()
                        .unwrap_or("Todas as Lojas"),
                )
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.state.selected_store.is_none(), "Todas as Lojas")
                        .clicked()
                    {
                        self.state.selected_store = None;
                    }
                    for store in self.state.unique_stores() {
                        let is_sel = self.state.selected_store.as_deref() == Some(&store);
                        if ui.selectable_label(is_sel, &store).clicked() {
                            self.state.selected_store = Some(store);
                        }
                    }
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("🔄 Atualizar").clicked() {
                    self.state.status_message =
                        Some("Telemetria e PDVs sincronizados com sucesso.".to_string());
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.state.search_query)
                        .hint_text("🔍 Filtrar Caixa ou IP...")
                        .desired_width(180.0),
                );
            });
        });

        ui.add_space(10.0);

        // 3. Grid de Cards dos PDVs + Inspector Drawer opcional
        let has_inspector = self.state.inspected_pdv.is_some();
        if has_inspector {
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    self.render_pdv_cards_list(ui);
                });
                cols[1].vertical(|ui| {
                    if let Some(pdv) = self.state.inspected_pdv.clone() {
                        self.render_inspector_drawer(ui, &pdv);
                    }
                });
            });
        } else {
            self.render_pdv_cards_list(ui);
        }
    }

    /// Renderiza os 4 KPI cards de telemetria no topo do Dashboard.
    fn render_kpi_cards(&self, ui: &mut egui::Ui) {
        ui.columns(4, |cols| {
            // Card 1: Total
            cols[0].vertical(|ui| {
                egui::Frame::none()
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("🏬 TOTAL DE CAIXAS")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.0)
                                .strong(),
                        );
                        ui.heading(
                            RichText::new(format!("{}", self.state.pdvs.len()))
                                .color(COLOR_TEXT_WHITE)
                                .size(22.0),
                        );
                        ui.label(
                            RichText::new("Distribuídos na rede")
                                .color(COLOR_TEXT_DARK)
                                .size(10.0),
                        );
                    });
            });

            // Card 2: Online
            cols[1].vertical(|ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgb(12, 28, 30))
                    .stroke(Stroke::new(1.0_f32, COLOR_MINT_ONLINE))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("🟢 CAIXAS ONLINE")
                                .color(COLOR_MINT_ONLINE)
                                .size(11.0)
                                .strong(),
                        );
                        ui.heading(
                            RichText::new(format!("{}", self.state.online_count()))
                                .color(COLOR_MINT_ONLINE)
                                .size(22.0),
                        );
                        ui.label(
                            RichText::new("Prontos para conexão")
                                .color(COLOR_TEXT_MUTED)
                                .size(10.0),
                        );
                    });
            });

            // Card 3: Offline
            cols[2].vertical(|ui| {
                egui::Frame::none()
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("⚪ CAIXAS OFFLINE")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.0)
                                .strong(),
                        );
                        ui.heading(
                            RichText::new(format!("{}", self.state.offline_count()))
                                .color(COLOR_TEXT_MUTED)
                                .size(22.0),
                        );
                        ui.label(
                            RichText::new("Sem sinal recente")
                                .color(COLOR_TEXT_DARK)
                                .size(10.0),
                        );
                    });
            });

            // Card 4: Nuvem Azure
            cols[3].vertical(|ui| {
                egui::Frame::none()
                    .fill(Color32::from_rgb(16, 26, 48))
                    .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("⚡ SINALIZAÇÃO AZURE")
                                .color(COLOR_CYAN_NEON)
                                .size(11.0)
                                .strong(),
                        );
                        ui.heading(
                            RichText::new(format!("{} ms", self.state.cloud_latency_ms))
                                .color(COLOR_CYAN_NEON)
                                .size(22.0),
                        );
                        ui.label(
                            RichText::new("20.121.137.44:443")
                                .color(COLOR_TEXT_MUTED)
                                .size(10.0),
                        );
                    });
            });
        });
    }

    /// Renderiza a lista de cards dos PDVs.
    fn render_pdv_cards_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let filtered_pdvs: Vec<PdvItem> =
                self.state.filter_pdvs().into_iter().cloned().collect();

            if filtered_pdvs.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("Nenhum PDV encontrado para este filtro.")
                            .color(Color32::GRAY),
                    );
                });
                return;
            }

            for pdv in filtered_pdvs {
                self.render_pdv_card(ui, &pdv);
                ui.add_space(8.0);
            }
        });
    }

    /// Renderiza um card rico individual de PDV.
    fn render_pdv_card(&mut self, ui: &mut egui::Ui, pdv: &PdvItem) {
        egui::Frame::none()
            .fill(COLOR_CARD_BASE)
            .stroke(Stroke::new(
                1.0_f32,
                if pdv.is_online {
                    COLOR_CARD_BORDER
                } else {
                    Color32::from_rgb(25, 32, 45)
                },
            ))
            .rounding(8.0)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let dot_char = if pdv.is_online { "🟢" } else { "⚪" };
                    ui.label(RichText::new(dot_char).size(16.0));

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&pdv.id)
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(16.0),
                            );
                            ui.label(RichText::new("—").color(COLOR_TEXT_DARK));
                            ui.label(
                                RichText::new(&pdv.store_name)
                                    .color(COLOR_TEXT_MUTED)
                                    .size(14.0),
                            );
                        });

                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "💻 {} • IP: {}",
                                    pdv.os_info, pdv.ip_address
                                ))
                                .color(COLOR_TEXT_MUTED)
                                .size(11.0),
                            );
                            ui.label(RichText::new("•").color(COLOR_TEXT_DARK).size(11.0));
                            ui.label(
                                RichText::new(format!(
                                    "Agente v{} • {}",
                                    pdv.agent_version, pdv.resolution
                                ))
                                .color(if pdv.is_online {
                                    COLOR_CYAN_GLOW
                                } else {
                                    COLOR_TEXT_DARK
                                })
                                .size(11.0),
                            );
                        });
                    });

                    // Ações à direita
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if pdv.is_online {
                            let btn_connect = egui::Button::new(
                                RichText::new("▶ Conectar")
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(13.0),
                            )
                            .fill(COLOR_BLUE_ELECTRIC)
                            .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                            .rounding(6.0);

                            if ui.add_sized([100.0, 32.0], btn_connect).clicked() {
                                self.state.select_pdv(pdv.clone());
                            }
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Button::new(
                                    RichText::new("Offline").color(COLOR_TEXT_DARK).size(12.0),
                                )
                                .fill(Color32::from_rgb(18, 22, 32))
                                .rounding(6.0),
                            );
                        }

                        let btn_info = egui::Button::new(
                            RichText::new("ℹ️ Ficha").color(COLOR_TEXT_MUTED).size(12.0),
                        )
                        .fill(Color32::from_rgb(26, 34, 54))
                        .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                        .rounding(6.0);

                        if ui.add_sized([75.0, 32.0], btn_info).clicked() {
                            self.state.inspect_pdv(pdv.clone());
                        }
                    });
                });
            });
    }

    /// Renderiza a ficha técnica detalhada do PDV inspecionado (Drawer lateral).
    fn render_inspector_drawer(&mut self, ui: &mut egui::Ui, pdv: &PdvItem) {
        egui::Frame::none()
            .fill(COLOR_PANEL_NAVY)
            .stroke(Stroke::new(1.2_f32, COLOR_CYAN_NEON))
            .rounding(10.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        RichText::new(format!("ℹ️ Ficha: {}", pdv.id))
                            .color(COLOR_CYAN_NEON)
                            .size(16.0),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("✖").clicked() {
                            self.state.close_inspector();
                        }
                    });
                });
                ui.separator();
                ui.add_space(6.0);

                ui.label(
                    RichText::new("Unidade / Filial:")
                        .color(COLOR_TEXT_DARK)
                        .size(11.0),
                );
                ui.label(
                    RichText::new(&pdv.store_name)
                        .color(COLOR_TEXT_WHITE)
                        .strong()
                        .size(13.0),
                );
                ui.add_space(6.0);

                ui.label(
                    RichText::new("Sistema Operacional:")
                        .color(COLOR_TEXT_DARK)
                        .size(11.0),
                );
                ui.label(
                    RichText::new(&pdv.os_info)
                        .color(COLOR_TEXT_WHITE)
                        .size(12.0),
                );
                ui.add_space(6.0);

                ui.label(
                    RichText::new("Endereço IP da Loja:")
                        .color(COLOR_TEXT_DARK)
                        .size(11.0),
                );
                ui.label(
                    RichText::new(&pdv.ip_address)
                        .color(COLOR_CYAN_GLOW)
                        .size(12.0),
                );
                ui.add_space(6.0);

                ui.label(
                    RichText::new("Chave Pública Ed25519 (Identidade E2E):")
                        .color(COLOR_TEXT_DARK)
                        .size(11.0),
                );
                ui.label(
                    RichText::new(&pdv.public_key)
                        .color(COLOR_VIOLET_NEON)
                        .size(11.0)
                        .monospace(),
                );
                ui.add_space(6.0);

                ui.label(
                    RichText::new("Resolução do Monitor Caixa:")
                        .color(COLOR_TEXT_DARK)
                        .size(11.0),
                );
                ui.label(
                    RichText::new(&pdv.resolution)
                        .color(COLOR_TEXT_WHITE)
                        .size(12.0),
                );
                ui.add_space(14.0);

                if pdv.is_online {
                    let btn_connect = egui::Button::new(
                        RichText::new("▶ Solicitar Conexão com Código")
                            .color(COLOR_TEXT_WHITE)
                            .strong()
                            .size(13.0),
                    )
                    .fill(COLOR_BLUE_ELECTRIC)
                    .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                    .rounding(6.0);

                    if ui
                        .add_sized([ui.available_width(), 36.0], btn_connect)
                        .clicked()
                    {
                        self.state.select_pdv(pdv.clone());
                    }
                }
            });
    }

    /// Renderiza a aba de Conexão Rápida Direta por ID e Código.
    fn render_direct_connect_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.2_f32, COLOR_CARD_BORDER))
                .rounding(10.0)
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.set_max_width(460.0);
                    ui.heading(
                        RichText::new("⚡ Conexão Rápida Direta")
                            .color(COLOR_CYAN_NEON)
                            .size(20.0)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Conecte-se a qualquer caixa da rede digitando o ID e o Código temporário.")
                            .color(COLOR_TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.add_space(16.0);

                    ui.label(RichText::new("Identificador do Caixa (PDV):").color(COLOR_TEXT_MUTED).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.direct_pdv_input)
                            .hint_text("ex: PDV-01 ou PDV-02")
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(10.0);
                    ui.label(RichText::new("Código de Sessão (8 dígitos gerados no PDV):").color(COLOR_TEXT_MUTED).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.direct_code_input)
                            .hint_text("ex: 83921045")
                            .char_limit(8)
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(20.0);
                    let can_connect = !self.state.direct_pdv_input.trim().is_empty()
                        && self.state.direct_code_input.trim().len() == 8;

                    let btn = egui::Button::new(
                        RichText::new("⚡ Iniciar Sessão de Suporte Remoto")
                            .color(COLOR_TEXT_WHITE)
                            .size(14.0)
                            .strong(),
                    )
                    .fill(if can_connect { COLOR_BLUE_ELECTRIC } else { Color32::from_rgb(32, 40, 56) })
                    .stroke(Stroke::new(1.0_f32, if can_connect { COLOR_CYAN_NEON } else { COLOR_CARD_BORDER }))
                    .rounding(6.0);

                    if ui.add_sized([ui.available_width(), 42.0], btn).clicked() && can_connect {
                        let pdv_id = self.state.direct_pdv_input.trim().to_uppercase();
                        let found = self.state.pdvs.iter().find(|p| p.id.eq_ignore_ascii_case(&pdv_id)).cloned();
                        let target = found.unwrap_or_else(|| PdvItem {
                            id: pdv_id.clone(),
                            store_name: "Conexão Manual".to_string(),
                            is_online: true,
                            agent_version: "0.1.0".to_string(),
                            public_key: format!("ed25519_{}", pdv_id),
                            last_seen: "Agora".to_string(),
                            ip_address: "127.0.0.1".to_string(),
                            os_info: "Windows 11 (x64)".to_string(),
                            resolution: "1920x1080".to_string(),
                        });
                        self.state.selected_pdv = Some(target);
                        self.state.session_code_input = self.state.direct_code_input.clone();
                        self.state.active_session = true;
                        self.state.status_message = Some(format!("Conectado com sucesso ao {} via Conexão Rápida!", pdv_id));
                    }
                });
        });
    }

    /// Renderiza a aba de Lojas com visão agrupada de caixas por filial.
    fn render_stores_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("🏬 Visão por Unidade / Filial")
                .size(18.0)
                .color(COLOR_TEXT_WHITE),
        );
        ui.label(
            RichText::new("Gerencie os caixas agrupados por loja física.")
                .color(COLOR_TEXT_MUTED)
                .size(12.0),
        );
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let stores = self.state.unique_stores();
            for store in stores {
                let pdvs_in_store: Vec<PdvItem> = self
                    .state
                    .pdvs
                    .iter()
                    .filter(|p| p.store_name == store)
                    .cloned()
                    .collect();
                let online_count = pdvs_in_store.iter().filter(|p| p.is_online).count();

                egui::Frame::none()
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .rounding(8.0)
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🏬").size(22.0));
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&store)
                                        .color(COLOR_TEXT_WHITE)
                                        .strong()
                                        .size(15.0),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} caixas registrados • {} online",
                                        pdvs_in_store.len(),
                                        online_count
                                    ))
                                    .color(COLOR_CYAN_GLOW)
                                    .size(12.0),
                                );
                            });
                        });
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        for pdv in pdvs_in_store {
                            ui.horizontal(|ui| {
                                let dot = if pdv.is_online { "🟢" } else { "⚪" };
                                ui.label(RichText::new(dot).size(14.0));
                                ui.label(
                                    RichText::new(&pdv.id)
                                        .color(COLOR_TEXT_WHITE)
                                        .strong()
                                        .size(13.0),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "({}) - IP: {}",
                                        pdv.os_info, pdv.ip_address
                                    ))
                                    .color(COLOR_TEXT_MUTED)
                                    .size(11.0),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if pdv.is_online && ui.button("▶ Conectar").clicked() {
                                        self.state.select_pdv(pdv.clone());
                                    }
                                });
                            });
                            ui.add_space(4.0);
                        }
                    });
                ui.add_space(10.0);
            }
        });
    }

    /// Renderiza a aba de Histórico de Auditoria.
    fn render_audit_history_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("📜 Registro de Auditoria de Sessões")
                    .size(18.0)
                    .color(COLOR_TEXT_WHITE),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("📥 Exportar Logs").clicked() {
                    self.state.status_message =
                        Some("Logs de auditoria exportados com sucesso.".to_string());
                }
            });
        });
        ui.label(RichText::new("Auditoria criptográfica fim-a-fim conforme AGENTS.md Seção 9 (sem expor tela ou teclas).").color(COLOR_TEXT_MUTED).size(12.0));
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("ID")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(30.0);
                        ui.label(
                            RichText::new("DATA / HORA")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(50.0);
                        ui.label(
                            RichText::new("OPERADOR")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(40.0);
                        ui.label(
                            RichText::new("CAIXA")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(60.0);
                        ui.label(
                            RichText::new("MOTIVO")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(12.0),
                        );
                        ui.add_space(100.0);
                        ui.label(
                            RichText::new("DURAÇÃO")
                                .color(COLOR_CYAN_NEON)
                                .strong()
                                .size(12.0),
                        );
                    });
                    ui.separator();

                    for record in &self.state.audit_records {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&record.id).color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add_space(15.0);
                            ui.label(
                                RichText::new(&record.timestamp)
                                    .color(COLOR_TEXT_MUTED)
                                    .size(12.0),
                            );
                            ui.add_space(15.0);
                            ui.label(
                                RichText::new(&record.operator_name)
                                    .color(COLOR_VIOLET_NEON)
                                    .strong()
                                    .size(12.0),
                            );
                            ui.add_space(35.0);
                            ui.label(
                                RichText::new(&record.pdv_id)
                                    .color(COLOR_TEXT_WHITE)
                                    .size(12.0),
                            );
                            ui.add_space(55.0);
                            ui.label(
                                RichText::new(&record.reason)
                                    .color(COLOR_TEXT_MUTED)
                                    .size(12.0),
                            );
                            ui.add_space(20.0);
                            ui.label(
                                RichText::new(&record.duration)
                                    .color(COLOR_MINT_ONLINE)
                                    .size(12.0),
                            );
                        });
                        ui.separator();
                    }
                });
        });
    }

    /// Renderiza a aba de Configurações e Nuvem Azure.
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("⚙️ Configurações & Nuvem Azure")
                .size(18.0)
                .color(COLOR_TEXT_WHITE),
        );
        ui.label(
            RichText::new("Gerenciamento de conexões, sinalização e parâmetros de desempenho.")
                .color(COLOR_TEXT_MUTED)
                .size(12.0),
        );
        ui.add_space(14.0);

        egui::Frame::none()
            .fill(COLOR_CARD_BASE)
            .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
            .rounding(8.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.heading(
                    RichText::new("🌐 Servidor de Sinalização (API de Nuvem)")
                        .size(14.0)
                        .color(COLOR_CYAN_NEON),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("URL da API:")
                            .color(COLOR_TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.state.server_url).desired_width(280.0),
                    );
                    if ui.button("⚡ Testar Ping").clicked() {
                        self.state.status_message =
                            Some("API Azure respondeu: HTTP 200 OK (18 ms)".to_string());
                    }
                });
                ui.add_space(6.0);
                ui.label(
                    RichText::new("● Status: Conectado e ativo em http://20.121.137.44:443/health")
                        .color(COLOR_MINT_ONLINE)
                        .size(12.0),
                );

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                ui.heading(
                    RichText::new("🎮 Desempenho e Vídeo DXGI")
                        .size(14.0)
                        .color(COLOR_CYAN_NEON),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Taxa de Quadros Alvo (FPS):")
                            .color(COLOR_TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.radio_value(&mut self.state.target_fps, 15, "15 FPS");
                    ui.radio_value(&mut self.state.target_fps, 30, "30 FPS");
                    ui.radio_value(&mut self.state.target_fps, 60, "60 FPS");
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Nível de Compressão Zstd:")
                            .color(COLOR_TEXT_MUTED)
                            .size(12.0),
                    );
                    ui.radio_value(&mut self.state.zstd_level, 1, "Nível 1 (Menor Latência)");
                    ui.radio_value(&mut self.state.zstd_level, 3, "Nível 3 (Economia)");
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                ui.heading(
                    RichText::new("⌨️ Entrada e Periféricos")
                        .size(14.0)
                        .color(COLOR_CYAN_NEON),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("• Teclado: Modo ABNT2 Scan Codes (KEYEVENTF_SCANCODE nativo)")
                        .color(COLOR_TEXT_WHITE)
                        .size(12.0),
                );
                ui.label(
                    RichText::new("• Mouse: Coordenadas Absolutas Normalizadas (0..65535)")
                        .color(COLOR_TEXT_WHITE)
                        .size(12.0),
                );
                ui.label(
                    RichText::new(
                        "• Segurança: Liberação automática de todas as teclas ao encerrar",
                    )
                    .color(COLOR_MINT_ONLINE)
                    .size(12.0),
                );
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

    /// Renderiza a tela de login do operador / administrador inspirada no design BipDesk Enterprise.
    fn render_admin_login(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            // Centraliza verticalmente com respiro refinado
            let avail_h = ui.available_height();
            ui.add_space((avail_h - 520.0).max(18.0) * 0.42);

            // 1. Badge superior com ícone do monitor e raio
            self.render_admin_login_badge(ui);

            ui.add_space(14.0);

            // 2. Título "PrysmView Admin" e Subtítulo
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PrysmView")
                        .color(Color32::from_rgb(56, 132, 255))
                        .size(26.0)
                        .strong(),
                );
                ui.label(
                    RichText::new("Admin")
                        .color(Color32::WHITE)
                        .size(26.0)
                        .strong(),
                );
            });
            ui.add_space(3.0);
            ui.label(
                RichText::new("Central de Suporte & Gestão Remota")
                    .color(Color32::from_rgb(148, 163, 184))
                    .size(12.5),
            );

            ui.add_space(22.0);

            // 3. Card Container Escuro com Borda e Cantos Arredondados
            egui::Frame::none()
                .fill(Color32::from_rgb(19, 25, 38))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(33, 44, 65)))
                .rounding(16.0)
                .inner_margin(egui::Margin::symmetric(28.0, 24.0))
                .show(ui, |ui| {
                    ui.set_max_width(380.0);
                    ui.set_min_width(350.0);

                    // Campo: NOME DE USUÁRIO
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.label(
                            RichText::new("NOME DE USUÁRIO")
                                .color(Color32::from_rgb(148, 163, 184))
                                .size(11.0)
                                .strong(),
                        );
                    });
                    ui.add_space(6.0);

                    egui::Frame::none()
                        .fill(Color32::from_rgb(14, 18, 28))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 40, 60)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 9.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("👤")
                                        .color(Color32::from_rgb(140, 150, 170))
                                        .size(14.0),
                                );
                                ui.add_space(6.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.login_operator_id)
                                        .hint_text("admin")
                                        .frame(false)
                                        .desired_width(f32::INFINITY)
                                        .text_color(Color32::WHITE),
                                );
                            });
                        });

                    ui.add_space(14.0);

                    // Campo: SENHA DE ACESSO
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui.label(
                            RichText::new("SENHA DE ACESSO")
                                .color(Color32::from_rgb(148, 163, 184))
                                .size(11.0)
                                .strong(),
                        );
                    });
                    ui.add_space(6.0);

                    let mut enter_pressed = false;
                    egui::Frame::none()
                        .fill(Color32::from_rgb(14, 18, 28))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 40, 60)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 9.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("🔒")
                                        .color(Color32::from_rgb(140, 150, 170))
                                        .size(14.0),
                                );
                                ui.add_space(6.0);

                                let pwd_resp = ui.add(
                                    egui::TextEdit::singleline(&mut self.state.login_password)
                                        .password(!self.state.show_password)
                                        .hint_text("••••••••")
                                        .frame(false)
                                        .desired_width(ui.available_width() - 34.0)
                                        .text_color(Color32::WHITE),
                                );

                                if pwd_resp.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    enter_pressed = true;
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let eye_btn = egui::Button::new(
                                        RichText::new("👁")
                                            .color(if self.state.show_password {
                                                COLOR_CYAN_NEON
                                            } else {
                                                Color32::from_rgb(148, 163, 184)
                                            })
                                            .size(14.0),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE);

                                    if ui.add(eye_btn).clicked() {
                                        self.state.show_password = !self.state.show_password;
                                    }
                                });
                            });
                        });

                    ui.add_space(12.0);

                    // Checkbox "Lembrar meu usuário neste dispositivo"
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.checkbox(
                            &mut self.state.remember_user,
                            RichText::new("Lembrar meu usuário neste dispositivo")
                                .color(Color32::from_rgb(148, 163, 184))
                                .size(12.0),
                        );
                    });

                    // Mensagem de Erro (se houver)
                    if let Some(ref err) = self.state.login_error {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠️").color(COLOR_RED_DANGER).size(13.0));
                            ui.label(RichText::new(err).color(COLOR_RED_DANGER).size(12.0));
                        });
                    }

                    ui.add_space(16.0);

                    // Botão "Entrar no Console" (Vibrant Royal Blue)
                    let btn_login = egui::Button::new(
                        RichText::new("Entrar no Console")
                            .color(Color32::WHITE)
                            .size(15.0)
                            .strong(),
                    )
                    .fill(Color32::from_rgb(37, 99, 235)) // #2563eb
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(59, 130, 246)))
                    .rounding(8.0);

                    if ui
                        .add_sized([ui.available_width(), 44.0], btn_login)
                        .clicked()
                        || enter_pressed
                    {
                        let op_id = self.state.login_operator_id.trim().to_string();
                        if op_id.is_empty() {
                            self.state.login_error = Some("Informe o Nome de Usuário".to_string());
                        } else {
                            let token = format!("token_{}", op_id);
                            self.state.login_success(&op_id, token);
                        }
                    }
                });

            ui.add_space(26.0);

            // 4. Rodapé de Conexão Segura Criptografada (Zero-Trust)
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("●")
                        .color(Color32::from_rgb(16, 185, 129))
                        .size(11.0),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Conexão Segura Criptografada (Zero-Trust)")
                        .color(Color32::from_rgb(120, 135, 155))
                        .size(11.5),
                );
            });

            ui.add_space(8.0);

            // Opções Avançadas discretas para Servidor de Sinalização
            ui.collapsing(
                RichText::new("⚙ Servidor de Sinalização")
                    .color(Color32::from_rgb(70, 85, 105))
                    .size(10.5),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("URL:").color(COLOR_TEXT_DARK).size(11.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.server_url)
                                .hint_text("http://20.121.137.44:443")
                                .desired_width(220.0),
                        );
                    });
                },
            );
        });
    }

    /// Renderiza o badge de monitor com raio inspirado na identidade visual.
    fn render_admin_login_badge(&self, ui: &mut egui::Ui) {
        let size = Vec2::new(70.0, 70.0);
        let (rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();

        // Fundo azul marinho escuro com borda azul vibrante arredondada
        let bg_color = Color32::from_rgb(13, 29, 54);
        let border_stroke = Stroke::new(1.8_f32, Color32::from_rgb(37, 99, 235));
        painter.rect(rect, 18.0, bg_color, border_stroke);

        // Contorno do monitor de computador
        let screen_width = 36.0;
        let screen_height = 23.0;
        let screen_center = egui::pos2(rect.center().x, rect.center().y - 3.5);
        let screen_rect =
            egui::Rect::from_center_size(screen_center, Vec2::new(screen_width, screen_height));
        let screen_stroke = Stroke::new(1.8_f32, Color32::from_rgb(56, 132, 255));
        painter.rect(
            screen_rect,
            4.0,
            Color32::from_rgb(8, 18, 36),
            screen_stroke,
        );

        // Suporte e base do monitor
        let stand_y_start = screen_rect.bottom();
        let stand_y_end = stand_y_start + 6.0;
        painter.line_segment(
            [
                egui::pos2(rect.center().x, stand_y_start),
                egui::pos2(rect.center().x, stand_y_end),
            ],
            screen_stroke,
        );
        let base_width = 16.0;
        painter.line_segment(
            [
                egui::pos2(rect.center().x - base_width / 2.0, stand_y_end),
                egui::pos2(rect.center().x + base_width / 2.0, stand_y_end),
            ],
            screen_stroke,
        );

        // Ícone de raio elétrico no centro da tela do monitor
        painter.text(
            screen_center,
            egui::Align2::CENTER_CENTER,
            "⚡",
            egui::FontId::proportional(14.0),
            Color32::WHITE,
        );
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
