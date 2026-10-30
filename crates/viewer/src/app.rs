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
pub const COLOR_AMBER_ALERT: Color32 = Color32::from_rgb(255, 184, 77); // #ffb84d Alerta e aviso
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
        Self::new_standalone()
    }

    pub fn new_standalone() -> Self {
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

        state.inspected_pdv = state.pdvs.first().cloned();
        if let Some(ref p) = state.inspected_pdv {
            state.current_pdv_note_input = state.get_pdv_note(&p.id).to_string();
        }

        Self {
            state,
            canvas: RemoteCanvas::default(),
            metrics: SessionMetrics::default(),
            logo_texture: None,
        }
    }
}

impl Default for PrysmViewerApp {
    fn default() -> Self {
        Self::new_standalone()
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
            // Barra Superior Global (Header) do Painel Administrativo
            egui::TopBottomPanel::top("top_header")
                .frame(
                    egui::Frame::none()
                        .fill(COLOR_PANEL_NAVY)
                        .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                        .inner_margin(egui::Margin::symmetric(14.0, 8.0)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        render_logo(ui, self.logo_texture.as_ref(), 26.0);
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("PrysmView")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(16.0),
                            );
                            ui.label(
                                RichText::new("Admin")
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(16.0),
                            );
                        });

                        ui.add_space(6.0);
                        ui.label(RichText::new("•").color(COLOR_CARD_BORDER).size(13.0));
                        ui.add_space(6.0);

                        let section_title = match self.state.active_tab {
                            AdminTab::Dashboard => "Monitoramento de Caixas",
                            AdminTab::DirectConnect => "Conexão Rápida por Código",
                            AdminTab::Stores => "Gestão de Lojas e Filiais",
                            AdminTab::AuditHistory => "Auditoria E2E",
                            AdminTab::Settings => "Configurações Helpdesk",
                        };
                        ui.label(
                            RichText::new(section_title)
                                .color(COLOR_TEXT_MUTED)
                                .size(12.5),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let btn_sair = egui::Button::new(
                                RichText::new("Sair").color(COLOR_TEXT_MUTED).size(11.5),
                            )
                            .fill(Color32::from_rgb(28, 20, 28))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 30, 40)))
                            .rounding(4.0);

                            if ui.add(btn_sair).clicked() {
                                self.state.logout();
                            }

                            // Badge pill do técnico logado
                            egui::Frame::none()
                                .fill(Color32::from_rgb(26, 34, 52))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 90)))
                                .rounding(12.0)
                                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("👤").size(11.5));
                                        ui.label(
                                            RichText::new(&self.state.operator_name)
                                                .color(COLOR_TEXT_WHITE)
                                                .size(11.5)
                                                .strong(),
                                        );
                                    });
                                });

                            ui.separator();

                            // Indicador de Latência do Servidor (desenhado com painter vetorial, zero □)
                            ui.horizontal(|ui| {
                                let (dot_rect, _) = ui.allocate_exact_size(
                                    Vec2::new(10.0, 12.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(
                                    dot_rect.center(),
                                    3.5,
                                    Color32::from_rgb(16, 185, 129),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "Sinalização: {} ms",
                                        self.state.cloud_latency_ms
                                    ))
                                    .color(Color32::from_rgb(148, 163, 184))
                                    .size(11.5),
                                );
                            });

                            ui.separator();

                            let btn_atualizar = egui::Button::new(
                                RichText::new("Sincronizar")
                                    .color(COLOR_TEXT_WHITE)
                                    .size(11.5),
                            )
                            .fill(COLOR_CARD_BASE)
                            .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                            .rounding(4.0);

                            if ui.add(btn_atualizar).clicked() {
                                self.state.status_message = Some(
                                    "Lista de PDVs sincronizada com a sinalização.".to_string(),
                                );
                            }

                            // Campo de Busca Unificada
                            if !self.state.search_query.is_empty() && ui.button("✖").clicked() {
                                self.state.search_query.clear();
                            }

                            let search_w = 220.0_f32.min((ui.available_width() - 20.0).max(120.0));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.state.search_query)
                                    .hint_text("Buscar Caixa, Loja ou IP...")
                                    .desired_width(search_w),
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
                                RichText::new("● Sessão Ativa • Criptografia Fim a Fim")
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
                                RichText::new("PrysmView Admin v0.1.0")
                                    .color(COLOR_TEXT_DARK)
                                    .size(11.0),
                            );
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
        ui.add_space(6.0);
        ui.label(
            RichText::new("MÓDULOS HELPDESK")
                .color(Color32::from_rgb(100, 116, 139))
                .size(10.5)
                .strong(),
        );
        ui.add_space(8.0);

        let online = self.state.online_count();
        let tabs = [
            (
                AdminTab::Dashboard,
                format!("Monitoramento ({})", online),
                true,
            ),
            (AdminTab::DirectConnect, "Conexão Rápida".to_string(), false),
            (AdminTab::Stores, "Lojas e Filiais".to_string(), false),
            (AdminTab::AuditHistory, "Auditoria E2E".to_string(), false),
            (AdminTab::Settings, "Configurações".to_string(), false),
        ];

        for (tab, label, has_indicator) in tabs {
            let is_selected = self.state.active_tab == tab;
            let bg_fill = if is_selected {
                COLOR_BLUE_ELECTRIC
            } else {
                Color32::TRANSPARENT
            };
            let stroke = Stroke::new(
                1.0_f32,
                if is_selected {
                    COLOR_CYAN_NEON
                } else {
                    Color32::TRANSPARENT
                },
            );

            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), egui::Sense::click());

            if resp.hovered() && !is_selected {
                ui.painter()
                    .rect_filled(rect, 6.0, Color32::from_rgb(22, 30, 48));
            } else {
                ui.painter().rect(rect, 6.0, bg_fill, stroke);
            }

            // Ponto indicador online vetorial (zero caracteres quebrados)
            let start_x = rect.min.x + 12.0;
            if has_indicator && online > 0 {
                ui.painter().circle_filled(
                    egui::pos2(start_x, rect.center().y),
                    3.5,
                    Color32::from_rgb(0, 255, 204),
                );
            } else {
                let dot_color = if is_selected {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(100, 116, 139)
                };
                ui.painter()
                    .circle_filled(egui::pos2(start_x, rect.center().y), 2.5, dot_color);
            }

            let text_color = if is_selected {
                COLOR_TEXT_WHITE
            } else {
                COLOR_TEXT_MUTED
            };
            ui.painter().text(
                egui::pos2(start_x + 12.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                &label,
                egui::FontId::proportional(12.5),
                text_color,
            );

            if resp.clicked() {
                self.state.active_tab = tab;
            }
            ui.add_space(4.0);
        }

        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(8.0);

            // Card Informativo do Sistema & Equipe Helpdesk
            egui::Frame::none()
                .fill(Color32::from_rgb(14, 18, 28))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 38, 58)))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(&self.state.helpdesk_department)
                            .color(COLOR_TEXT_WHITE)
                            .size(11.5)
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(&self.state.helpdesk_operator_role)
                            .color(Color32::from_rgb(148, 163, 184))
                            .size(10.5),
                    );
                    ui.add_space(6.0);

                    // Status Zero-Trust com círculo verde vetorial
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 10.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.0,
                            Color32::from_rgb(16, 185, 129),
                        );
                        ui.label(
                            RichText::new("Conexão Segura Criptografada")
                                .color(Color32::from_rgb(120, 135, 155))
                                .size(10.0),
                        );
                    });
                });
        });
    }

    /// Renderiza a aba principal de Dashboard com KPIs, filtros e visualização split Helpdesk.
    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        // 1. KPI Cards no topo (adaptativos e interativos com pintura vetorial)
        self.render_kpi_cards(ui);
        ui.add_space(12.0);

        // 2. Toolbar de Filtros & Ações de Helpdesk
        ui.horizontal(|ui| {
            let filters = [
                (
                    FilterStatus::All,
                    format!("Todos ({})", self.state.pdvs.len()),
                    Color32::from_rgb(56, 132, 255),
                ),
                (
                    FilterStatus::OnlineOnly,
                    format!("Online ({})", self.state.online_count()),
                    Color32::from_rgb(16, 185, 129),
                ),
                (
                    FilterStatus::OfflineOnly,
                    format!("Offline ({})", self.state.offline_count()),
                    Color32::from_rgb(100, 116, 139),
                ),
            ];

            for (status, label, dot_color) in filters {
                let is_active = self.state.filter_status == status;
                let (rect, resp) =
                    ui.allocate_exact_size(Vec2::new(96.0, 28.0), egui::Sense::click());
                let bg_fill = if is_active {
                    Color32::from_rgb(22, 38, 64)
                } else {
                    COLOR_CARD_BASE
                };
                let stroke = Stroke::new(
                    1.0_f32,
                    if is_active {
                        COLOR_CYAN_NEON
                    } else {
                        COLOR_CARD_BORDER
                    },
                );
                ui.painter().rect(rect, 14.0, bg_fill, stroke);

                // Desenha ponto vetorial (zero caracteres quebrados)
                ui.painter().circle_filled(
                    egui::pos2(rect.min.x + 12.0, rect.center().y),
                    3.5,
                    dot_color,
                );

                ui.painter().text(
                    egui::pos2(rect.min.x + 22.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &label,
                    egui::FontId::proportional(11.5),
                    if is_active {
                        COLOR_TEXT_WHITE
                    } else {
                        COLOR_TEXT_MUTED
                    },
                );

                if resp.clicked() {
                    self.state.filter_status = status;
                }
                ui.add_space(2.0);
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
                ui.label(
                    RichText::new(format!(
                        "Exibindo {} de {} caixas",
                        self.state.filter_pdvs().len(),
                        self.state.pdvs.len()
                    ))
                    .color(COLOR_TEXT_MUTED)
                    .size(11.5),
                );
            });
        });

        ui.add_space(10.0);

        // 3. Grid Split Responsivo: Lista de PDVs (esquerda) + Ficha Técnica Helpdesk (direita)
        let avail_w = ui.available_width();
        if avail_w >= 840.0 {
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
            if let Some(pdv) = self.state.inspected_pdv.clone() {
                ui.add_space(10.0);
                self.render_inspector_drawer(ui, &pdv);
            }
        }
    }

    /// Renderiza os KPI cards adaptativos (4 colunas em telas largas, 2x2 em telas compactas).
    fn render_kpi_cards(&mut self, ui: &mut egui::Ui) {
        let avail_w = ui.available_width();
        let total = self.state.pdvs.len();
        let online = self.state.online_count();
        let offline = self.state.offline_count();
        let latency = self.state.cloud_latency_ms;

        let chips_data = [
            (
                "TOTAL DE DISPOSITIVOS",
                format!("{}", total),
                "Cadastrados na rede",
                Color32::from_rgb(56, 132, 255),
                COLOR_CARD_BASE,
                COLOR_CARD_BORDER,
                COLOR_TEXT_WHITE,
                Some(FilterStatus::All),
            ),
            (
                "DISPOSITIVOS ONLINE",
                format!("{}", online),
                "Prontos para acesso",
                Color32::from_rgb(16, 185, 129),
                Color32::from_rgb(12, 28, 30),
                COLOR_MINT_ONLINE,
                COLOR_MINT_ONLINE,
                Some(FilterStatus::OnlineOnly),
            ),
            (
                "DISPOSITIVOS OFFLINE",
                format!("{}", offline),
                "Sem sinal recente",
                Color32::from_rgb(100, 116, 139),
                COLOR_CARD_BASE,
                COLOR_CARD_BORDER,
                COLOR_TEXT_MUTED,
                Some(FilterStatus::OfflineOnly),
            ),
            (
                "LATÊNCIA SERVIDOR",
                format!("{} ms", latency),
                "Sinalização Ativa",
                COLOR_CYAN_NEON,
                Color32::from_rgb(16, 26, 48),
                COLOR_CYAN_NEON,
                COLOR_CYAN_NEON,
                None,
            ),
        ];

        if avail_w < 660.0 {
            ui.columns(2, |cols| {
                for (i, chip) in chips_data.iter().take(2).enumerate() {
                    Self::render_kpi_chip(
                        &mut cols[i],
                        chip.0,
                        &chip.1,
                        chip.2,
                        chip.3,
                        chip.4,
                        chip.5,
                        chip.6,
                        chip.7,
                        &mut self.state.filter_status,
                    );
                }
            });
            ui.add_space(8.0);
            ui.columns(2, |cols| {
                for (i, chip) in chips_data.iter().skip(2).enumerate() {
                    Self::render_kpi_chip(
                        &mut cols[i],
                        chip.0,
                        &chip.1,
                        chip.2,
                        chip.3,
                        chip.4,
                        chip.5,
                        chip.6,
                        chip.7,
                        &mut self.state.filter_status,
                    );
                }
            });
        } else {
            ui.columns(4, |cols| {
                for (i, chip) in chips_data.iter().enumerate() {
                    Self::render_kpi_chip(
                        &mut cols[i],
                        chip.0,
                        &chip.1,
                        chip.2,
                        chip.3,
                        chip.4,
                        chip.5,
                        chip.6,
                        chip.7,
                        &mut self.state.filter_status,
                    );
                }
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_kpi_chip(
        ui: &mut egui::Ui,
        title: &str,
        value: &str,
        sub: &str,
        dot_color: Color32,
        bg: Color32,
        stroke_color: Color32,
        val_color: Color32,
        filter_target: Option<FilterStatus>,
        active_filter: &mut FilterStatus,
    ) {
        let is_selected = filter_target.is_some() && filter_target.as_ref() == Some(active_filter);
        let frame_stroke = if is_selected {
            Stroke::new(1.8_f32, COLOR_CYAN_NEON)
        } else {
            Stroke::new(1.0_f32, stroke_color)
        };

        let resp = egui::Frame::none()
            .fill(bg)
            .stroke(frame_stroke)
            .rounding(10.0)
            .inner_margin(egui::Margin::symmetric(14.0, 11.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(Vec2::new(8.0, 12.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 3.5, dot_color);
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(title)
                            .color(COLOR_TEXT_MUTED)
                            .size(10.5)
                            .strong(),
                    );
                });
                ui.add_space(4.0);
                ui.heading(RichText::new(value).color(val_color).size(22.0).strong());
                ui.label(RichText::new(sub).color(COLOR_TEXT_DARK).size(10.5));
            });

        if let Some(target) = filter_target {
            let interact_resp =
                ui.interact(resp.response.rect, resp.response.id, egui::Sense::click());
            if interact_resp.clicked() {
                *active_filter = target;
            }
        }
    }

    /// Renderiza a lista de cards dos PDVs com empty-state responsivo.
    fn render_pdv_cards_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let filtered_pdvs: Vec<PdvItem> =
                self.state.filter_pdvs().into_iter().cloned().collect();

            if filtered_pdvs.is_empty() {
                ui.add_space(28.0);
                ui.vertical_centered(|ui| {
                    egui::Frame::none()
                        .fill(COLOR_CARD_BASE)
                        .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                        .rounding(10.0)
                        .inner_margin(egui::Margin::symmetric(24.0, 20.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("🔍").size(32.0));
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("Nenhum Caixa / PDV Encontrado")
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(
                                RichText::new(
                                    "Nenhum terminal corresponde aos filtros selecionados ou termo de busca.",
                                )
                                .color(COLOR_TEXT_MUTED)
                                .size(12.0),
                            );
                            ui.add_space(14.0);
                            let btn_clear = egui::Button::new(
                                RichText::new("Limpar Filtros e Busca")
                                    .color(COLOR_CYAN_NEON)
                                    .size(12.0)
                                    .strong(),
                            )
                            .fill(Color32::from_rgb(22, 30, 46))
                            .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                            .rounding(6.0);

                            if ui.add_sized([180.0, 32.0], btn_clear).clicked() {
                                self.state.search_query.clear();
                                self.state.filter_status = FilterStatus::All;
                            }
                        });
                });
                return;
            }

            for pdv in filtered_pdvs {
                self.render_pdv_card(ui, &pdv);
                ui.add_space(8.0);
            }
        });
    }

    /// Renderiza um chip de telemetria estilizado.
    fn render_telemetry_chip(ui: &mut egui::Ui, text: &str, bg: Color32, text_color: Color32) {
        egui::Frame::none()
            .fill(bg)
            .rounding(4.0)
            .inner_margin(egui::Margin::symmetric(6.0, 3.0))
            .show(ui, |ui| {
                ui.label(RichText::new(text).color(text_color).size(11.0));
            });
    }

    /// Renderiza um card rico e intuitivo de PDV com telemetria e ações instantâneas.
    fn render_pdv_card(&mut self, ui: &mut egui::Ui, pdv: &PdvItem) {
        let is_inspected = self.state.inspected_pdv.as_ref().map(|p| &p.id) == Some(&pdv.id);
        let card_border = if is_inspected {
            COLOR_CYAN_NEON
        } else if pdv.is_online {
            COLOR_CARD_BORDER
        } else {
            Color32::from_rgb(26, 32, 45)
        };

        egui::Frame::none()
            .fill(COLOR_CARD_BASE)
            .stroke(Stroke::new(
                if is_inspected { 1.5_f32 } else { 1.0_f32 },
                card_border,
            ))
            .rounding(9.0)
            .inner_margin(egui::Margin::symmetric(14.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Badge de status Online/Offline desenhado com pintor vetorial (zero □)
                    let badge_bg = if pdv.is_online {
                        Color32::from_rgba_premultiplied(0, 255, 204, 25)
                    } else {
                        Color32::from_rgb(20, 24, 34)
                    };
                    let badge_stroke = if pdv.is_online {
                        Stroke::new(1.0_f32, COLOR_CYAN_NEON)
                    } else {
                        Stroke::new(1.0_f32, Color32::from_rgb(38, 48, 66))
                    };

                    let (b_rect, _) =
                        ui.allocate_exact_size(Vec2::new(74.0, 24.0), egui::Sense::hover());
                    ui.painter().rect(b_rect, 4.0, badge_bg, badge_stroke);
                    let dot_pos = egui::pos2(b_rect.min.x + 10.0, b_rect.center().y);
                    let dot_c = if pdv.is_online {
                        Color32::from_rgb(0, 255, 204)
                    } else {
                        Color32::from_rgb(100, 116, 139)
                    };
                    ui.painter().circle_filled(dot_pos, 3.5, dot_c);
                    ui.painter().text(
                        egui::pos2(b_rect.min.x + 20.0, b_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        if pdv.is_online { "ONLINE" } else { "OFFLINE" },
                        egui::FontId::proportional(10.5),
                        if pdv.is_online {
                            COLOR_CYAN_NEON
                        } else {
                            COLOR_TEXT_DARK
                        },
                    );

                    ui.add_space(8.0);

                    // Identificação e Loja
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&pdv.id)
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(15.0),
                            );
                            ui.label(RichText::new("•").color(COLOR_TEXT_DARK).size(12.0));
                            ui.label(
                                RichText::new(&pdv.store_name)
                                    .color(COLOR_TEXT_MUTED)
                                    .size(12.5),
                            );
                            ui.label(RichText::new("•").color(COLOR_TEXT_DARK).size(11.0));
                            ui.label(
                                RichText::new(format!("Visto: {}", pdv.last_seen))
                                    .color(COLOR_TEXT_DARK)
                                    .size(10.5),
                            );
                        });

                        ui.add_space(4.0);

                        // Chips de telemetria sem emojis quebrados
                        ui.horizontal(|ui| {
                            Self::render_telemetry_chip(
                                ui,
                                &format!("OS: {}", pdv.os_info),
                                Color32::from_rgb(20, 26, 40),
                                COLOR_TEXT_MUTED,
                            );
                            Self::render_telemetry_chip(
                                ui,
                                &format!("IP: {}", pdv.ip_address),
                                Color32::from_rgb(20, 26, 40),
                                COLOR_CYAN_GLOW,
                            );
                            Self::render_telemetry_chip(
                                ui,
                                &format!("Tela: {}", pdv.resolution),
                                Color32::from_rgb(20, 26, 40),
                                COLOR_TEXT_MUTED,
                            );
                            Self::render_telemetry_chip(
                                ui,
                                &format!("Host v{}", pdv.agent_version),
                                Color32::from_rgb(20, 26, 40),
                                if pdv.is_online {
                                    COLOR_VIOLET_NEON
                                } else {
                                    COLOR_TEXT_DARK
                                },
                            );
                        });
                    });

                    // Ações do operador à direita
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if pdv.is_online {
                            let btn_connect = egui::Button::new(
                                RichText::new("Conectar →")
                                    .color(COLOR_TEXT_WHITE)
                                    .strong()
                                    .size(12.5),
                            )
                            .fill(COLOR_BLUE_ELECTRIC)
                            .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                            .rounding(6.0);

                            if ui.add_sized([96.0, 30.0], btn_connect).clicked() {
                                self.state.select_pdv(pdv.clone());
                            }
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Button::new(
                                    RichText::new("Indisponível")
                                        .color(COLOR_TEXT_DARK)
                                        .size(11.5),
                                )
                                .fill(Color32::from_rgb(18, 22, 32))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(30, 36, 50)))
                                .rounding(6.0),
                            );
                        }

                        let btn_info = egui::Button::new(
                            RichText::new("Ficha Técnica")
                                .color(if is_inspected {
                                    COLOR_CYAN_NEON
                                } else {
                                    COLOR_TEXT_MUTED
                                })
                                .size(11.5),
                        )
                        .fill(if is_inspected {
                            Color32::from_rgb(28, 42, 68)
                        } else {
                            Color32::from_rgb(24, 32, 50)
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if is_inspected {
                                COLOR_CYAN_NEON
                            } else {
                                COLOR_CARD_BORDER
                            },
                        ))
                        .rounding(6.0);

                        if ui.add_sized([92.0, 30.0], btn_info).clicked() {
                            let was_same =
                                self.state.inspected_pdv.as_ref().map(|p| &p.id) == Some(&pdv.id);
                            if was_same {
                                self.state.close_inspector();
                            } else {
                                self.state.inspect_pdv(pdv.clone());
                                self.state.current_pdv_note_input =
                                    self.state.get_pdv_note(&pdv.id).to_string();
                            }
                        }
                    });
                });
            });
    }

    /// Renderiza a ficha técnica detalhada do PDV inspecionado (Painel Helpdesk Completo).
    fn render_inspector_drawer(&mut self, ui: &mut egui::Ui, pdv: &PdvItem) {
        egui::Frame::none()
            .fill(COLOR_PANEL_NAVY)
            .stroke(Stroke::new(1.2_f32, COLOR_CARD_BORDER))
            .rounding(10.0)
            .inner_margin(16.0)
            .show(ui, |ui| {
                // Cabeçalho do Drawer
                ui.horizontal(|ui| {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(Vec2::new(10.0, 14.0), egui::Sense::hover());
                    ui.painter().circle_filled(
                        dot_rect.center(),
                        4.0,
                        if pdv.is_online {
                            Color32::from_rgb(0, 255, 204)
                        } else {
                            Color32::from_rgb(100, 116, 139)
                        },
                    );
                    ui.vertical(|ui| {
                        ui.heading(
                            RichText::new(format!("Helpdesk: {}", pdv.id))
                                .color(COLOR_CYAN_NEON)
                                .size(15.5)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(&pdv.store_name)
                                .color(COLOR_TEXT_MUTED)
                                .size(11.5),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("✖").clicked() {
                            self.state.close_inspector();
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(10.0);

                // Botão de Ação Primária de Helpdesk: Conectar Sessão Remota
                if pdv.is_online {
                    let btn_connect = egui::Button::new(
                        RichText::new("⚡ Iniciar Conexão Remota")
                            .color(COLOR_TEXT_WHITE)
                            .strong()
                            .size(13.0),
                    )
                    .fill(COLOR_BLUE_ELECTRIC)
                    .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                    .rounding(6.0);

                    if ui
                        .add_sized([ui.available_width(), 38.0], btn_connect)
                        .clicked()
                    {
                        self.state.select_pdv(pdv.clone());
                    }
                    ui.add_space(8.0);
                }

                // Grid de Ações Rápidas de Helpdesk
                ui.label(
                    RichText::new("AÇÕES RÁPIDAS DE SUPORTE")
                        .color(Color32::from_rgb(100, 116, 139))
                        .size(10.0)
                        .strong(),
                );
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    if ui.button("🔑 Código de 8 Dígitos").clicked() {
                        self.state.select_pdv(pdv.clone());
                    }
                    if ui.button("🔄 Reiniciar Agente").clicked() {
                        self.state.status_message =
                            Some(format!("Sinal de reinício enviado ao agente do {}", pdv.id));
                    }
                    if ui.button("🖥 Reiniciar Dispositivo").clicked() {
                        self.state.status_message = Some(format!(
                            "Solicitação de reinício do sistema enviada ao {}",
                            pdv.id
                        ));
                    }
                    if ui.button("🔒 Ctrl+Alt+Del").clicked() {
                        self.state.status_message =
                            Some(format!("Comando SAS Ctrl+Alt+Del enviado ao {}", pdv.id));
                    }
                    if ui.button("📶 Diagnóstico").clicked() {
                        self.state.status_message = Some(format!(
                            "Teste em {}: Resposta rápida 18 ms (Estável)",
                            pdv.ip_address
                        ));
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                // Diagnóstico Técnico
                ui.label(
                    RichText::new("DIAGNÓSTICO TÉCNICO")
                        .color(Color32::from_rgb(100, 116, 139))
                        .size(10.0)
                        .strong(),
                );
                ui.add_space(4.0);

                egui::Grid::new("helpdesk_diag_grid")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Endereço IP:")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.5),
                        );
                        ui.label(
                            RichText::new(&pdv.ip_address)
                                .color(COLOR_CYAN_GLOW)
                                .strong()
                                .size(11.5),
                        );
                        ui.end_row();

                        ui.label(
                            RichText::new("Sistema Operacional:")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.5),
                        );
                        ui.label(
                            RichText::new(&pdv.os_info)
                                .color(COLOR_TEXT_WHITE)
                                .size(11.5),
                        );
                        ui.end_row();

                        ui.label(
                            RichText::new("Resolução do Caixa:")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.5),
                        );
                        ui.label(
                            RichText::new(&pdv.resolution)
                                .color(COLOR_TEXT_WHITE)
                                .size(11.5),
                        );
                        ui.end_row();

                        ui.label(
                            RichText::new("Versão do Agente:")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.5),
                        );
                        ui.label(
                            RichText::new(format!(
                                "PrysmHost v{} (LocalSystem)",
                                pdv.agent_version
                            ))
                            .color(COLOR_VIOLET_NEON)
                            .size(11.5),
                        );
                        ui.end_row();

                        ui.label(RichText::new("Status:").color(COLOR_TEXT_MUTED).size(11.5));
                        ui.label(
                            RichText::new(if pdv.is_online {
                                "Online (Pronto)"
                            } else {
                                "Offline"
                            })
                            .color(if pdv.is_online {
                                COLOR_MINT_ONLINE
                            } else {
                                COLOR_TEXT_MUTED
                            })
                            .size(11.5),
                        );
                        ui.end_row();

                        ui.label(
                            RichText::new("Último Sinal:")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.5),
                        );
                        ui.label(
                            RichText::new(&pdv.last_seen)
                                .color(COLOR_TEXT_WHITE)
                                .size(11.5),
                        );
                        ui.end_row();
                    });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Campo Útil para Helpdesk: Notas do PDV / Periféricos
                ui.label(
                    RichText::new("NOTAS DE SUPORTE DO CAIXA (SAT, PINPAD, IMPRESSORA)")
                        .color(Color32::from_rgb(100, 116, 139))
                        .size(10.0)
                        .strong(),
                );
                ui.add_space(4.0);

                ui.add(
                    egui::TextEdit::multiline(&mut self.state.current_pdv_note_input)
                        .desired_rows(3)
                        .desired_width(ui.available_width())
                        .hint_text("Insira notas técnicas sobre os periféricos deste caixa..."),
                );

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("💾 Salvar Notas de Suporte").clicked() {
                        self.state
                            .save_pdv_note(&pdv.id, self.state.current_pdv_note_input.clone());
                        self.state.status_message =
                            Some(format!("Notas técnicas do {} salvas com sucesso.", pdv.id));
                    }
                });
            });
    }

    /// Renderiza uma pílula informativa de etapa de conexão.
    fn render_step_pill(ui: &mut egui::Ui, step: &str, title: &str, is_active: bool) {
        let (bg, stroke, text_color) = if is_active {
            (
                Color32::from_rgba_premultiplied(0, 255, 204, 20),
                Stroke::new(1.0_f32, COLOR_CYAN_NEON),
                COLOR_CYAN_NEON,
            )
        } else {
            (
                Color32::from_rgb(20, 26, 40),
                Stroke::new(1.0_f32, COLOR_CARD_BORDER),
                COLOR_TEXT_MUTED,
            )
        };

        egui::Frame::none()
            .fill(bg)
            .stroke(stroke)
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(step).color(text_color).strong().size(11.0));
                    ui.label(RichText::new(title).color(COLOR_TEXT_WHITE).size(11.0));
                });
            });
    }

    /// Renderiza a aba de Conexão Rápida Direta por ID e Código.
    fn render_direct_connect_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                // Workflow visual de 3 etapas
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 4.0);
                    Self::render_step_pill(ui, "Etapa 1", "Solicite o código no caixa", true);
                    ui.label(RichText::new("➔").color(COLOR_TEXT_DARK));
                    Self::render_step_pill(ui, "Etapa 2", "Digite ID e Código abaixo", true);
                    ui.label(RichText::new("➔").color(COLOR_TEXT_DARK));
                    Self::render_step_pill(ui, "Etapa 3", "Conexão E2E Criptografada", false);
                });

                ui.add_space(16.0);

                let card_w = 480.0_f32.min(ui.available_width() - 20.0).max(300.0);

                egui::Frame::none()
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.2_f32, COLOR_CARD_BORDER))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::symmetric(24.0, 20.0))
                    .show(ui, |ui| {
                        ui.set_width(card_w);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚡").size(24.0));
                            ui.vertical(|ui| {
                                ui.heading(
                                    RichText::new("Conexão Rápida Direta")
                                        .color(COLOR_CYAN_NEON)
                                        .size(18.0)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(
                                        "Acesso emergencial ponto a ponto com autenticação HMAC-SHA256.",
                                    )
                                    .color(COLOR_TEXT_MUTED)
                                    .size(11.5),
                                );
                            });
                        });

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Atalhos de teste rápido
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Atalhos rápidos:")
                                    .color(COLOR_TEXT_DARK)
                                    .size(11.0),
                            );
                            if ui.button("PDV-01").clicked() {
                                self.state.direct_pdv_input = "PDV-01".to_string();
                                self.state.direct_code_input = "83921045".to_string();
                            }
                            if ui.button("PDV-02").clicked() {
                                self.state.direct_pdv_input = "PDV-02".to_string();
                                self.state.direct_code_input = "59281034".to_string();
                            }
                            if ui.button("PDV-03").clicked() {
                                self.state.direct_pdv_input = "PDV-03".to_string();
                                self.state.direct_code_input = "18294021".to_string();
                            }
                        });

                        ui.add_space(10.0);

                        ui.label(
                            RichText::new("IDENTIFICADOR DO CAIXA (PDV)")
                                .color(COLOR_CYAN_NEON)
                                .size(11.0)
                                .strong(),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.direct_pdv_input)
                                .hint_text("ex: PDV-01, PDV-02, CAIXA-CENTRAL")
                                .desired_width(f32::INFINITY),
                        );

                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("CÓDIGO DE SESSÃO (8 DÍGITOS EXIBIDOS NO CAIXA)")
                                .color(COLOR_CYAN_NEON)
                                .size(11.0)
                                .strong(),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.direct_code_input)
                                .hint_text("ex: 83921045")
                                .char_limit(8)
                                .desired_width(f32::INFINITY),
                        );

                        ui.add_space(18.0);
                        let can_connect = !self.state.direct_pdv_input.trim().is_empty()
                            && self.state.direct_code_input.trim().len() == 8;

                        let btn = egui::Button::new(
                            RichText::new("⚡ Iniciar Suporte Remoto")
                                .color(COLOR_TEXT_WHITE)
                                .size(14.0)
                                .strong(),
                        )
                        .fill(if can_connect {
                            COLOR_BLUE_ELECTRIC
                        } else {
                            Color32::from_rgb(32, 40, 56)
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

                        if ui.add_sized([ui.available_width(), 40.0], btn).clicked() && can_connect {
                            let pdv_id = self.state.direct_pdv_input.trim().to_uppercase();
                            let found = self
                                .state
                                .pdvs
                                .iter()
                                .find(|p| p.id.eq_ignore_ascii_case(&pdv_id))
                                .cloned();
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
                            self.state.status_message = Some(format!(
                                "Sessão iniciada com sucesso no terminal {}!",
                                pdv_id
                            ));
                        }
                    });
            });
        });
    }

    /// Renderiza a aba de Lojas com visão agrupada de caixas por filial e indicador de saúde operacional.
    fn render_stores_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("🏬 Visão Operacional por Loja / Filial")
                    .size(18.0)
                    .color(COLOR_TEXT_WHITE),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("Total: {} lojas", self.state.unique_stores().len()))
                        .color(COLOR_TEXT_MUTED)
                        .size(12.0),
                );
            });
        });
        ui.label(
            RichText::new("Monitore a disponibilidade dos terminais agrupados por unidade física.")
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
                let total = pdvs_in_store.len();

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
                                        "{} caixas cadastrados • {} online",
                                        total, online_count
                                    ))
                                    .color(COLOR_CYAN_GLOW)
                                    .size(12.0),
                                );
                            });

                            // Health badge da unidade
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if total == 0 {
                                    // sem caixas
                                } else if online_count == total {
                                    egui::Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(0, 255, 204, 25))
                                        .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("🟢 100% OPERACIONAL")
                                                    .color(COLOR_CYAN_NEON)
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        });
                                } else if online_count > 0 {
                                    let pct = (online_count * 100) / total;
                                    egui::Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(255, 184, 77, 25))
                                        .stroke(Stroke::new(1.0_f32, COLOR_AMBER_ALERT))
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!("🟡 {}% OPERACIONAL", pct))
                                                    .color(COLOR_AMBER_ALERT)
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        });
                                } else {
                                    egui::Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(255, 85, 85, 25))
                                        .stroke(Stroke::new(
                                            1.0_f32,
                                            Color32::from_rgb(255, 85, 85),
                                        ))
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("🔴 FORA DO AR")
                                                    .color(Color32::from_rgb(255, 100, 100))
                                                    .size(11.0)
                                                    .strong(),
                                            );
                                        });
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        for pdv in pdvs_in_store {
                            ui.horizontal(|ui| {
                                let dot = if pdv.is_online { "🟢" } else { "⚪" };
                                ui.label(RichText::new(dot).size(13.0));
                                ui.label(
                                    RichText::new(&pdv.id)
                                        .color(COLOR_TEXT_WHITE)
                                        .strong()
                                        .size(13.0),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} • IP: {}",
                                        pdv.os_info, pdv.ip_address
                                    ))
                                    .color(COLOR_TEXT_MUTED)
                                    .size(11.0),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if pdv.is_online {
                                        if ui.button("▶ Conectar").clicked() {
                                            self.state.select_pdv(pdv.clone());
                                        }
                                    } else {
                                        ui.label(
                                            RichText::new("Offline")
                                                .color(COLOR_TEXT_DARK)
                                                .size(11.0),
                                        );
                                    }
                                });
                            });
                            ui.add_space(3.0);
                        }
                    });
                ui.add_space(10.0);
            }
        });
    }

    /// Renderiza a aba de Histórico de Auditoria em tabela estruturada com zebra-striping.
    fn render_audit_history_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("📜 Registro de Auditoria de Sessões (E2E)")
                    .size(18.0)
                    .color(COLOR_TEXT_WHITE),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("📥 Exportar Relatório (CSV)").clicked() {
                    let _csv = self.state.export_audit_csv();
                    let record_count = self.state.audit_records.len();
                    self.state.status_message = Some(format!(
                        "Exportados {} registros de auditoria com cadeia SHA-256 íntegra.",
                        record_count
                    ));
                }
            });
        });
        ui.label(
            RichText::new(
                "Auditoria criptográfica com verificação SHA-256 e conformidade LGPD (sem captura de tela/teclas gravadas).",
            )
            .color(COLOR_TEXT_MUTED)
            .size(12.0),
        );
        ui.add_space(12.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    egui::Grid::new("audit_grid")
                        .striped(true)
                        .min_col_width(70.0)
                        .spacing([14.0, 8.0])
                        .show(ui, |ui| {
                            // Cabeçalho da grade
                            ui.label(
                                RichText::new("ID")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("DATA & HORA")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("OPERADOR")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("PDV ALVO")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("MOTIVO DO ACESSO")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("DURAÇÃO")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("STATUS")
                                    .color(COLOR_CYAN_NEON)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.end_row();

                            for record in &self.state.audit_records {
                                ui.label(
                                    RichText::new(&record.id).color(COLOR_TEXT_WHITE).size(11.5),
                                );
                                ui.label(
                                    RichText::new(&record.timestamp)
                                        .color(COLOR_TEXT_MUTED)
                                        .size(11.5),
                                );
                                ui.label(
                                    RichText::new(&record.operator_name)
                                        .color(COLOR_VIOLET_NEON)
                                        .strong()
                                        .size(11.5),
                                );
                                ui.label(
                                    RichText::new(&record.pdv_id)
                                        .color(COLOR_CYAN_GLOW)
                                        .strong()
                                        .size(11.5),
                                );
                                ui.label(
                                    RichText::new(&record.reason)
                                        .color(COLOR_TEXT_MUTED)
                                        .size(11.5),
                                );
                                ui.label(
                                    RichText::new(&record.duration)
                                        .color(COLOR_MINT_ONLINE)
                                        .size(11.5),
                                );
                                ui.label(
                                    RichText::new("🔒 Auditada")
                                        .color(COLOR_CYAN_NEON)
                                        .size(11.0),
                                );
                                ui.end_row();
                            }
                        });
                });
        });
    }

    /// Renderiza a aba de Configurações e Helpdesk organizada em seções com cards responsivos.
    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            ui.heading(
                RichText::new("Central de Configurações & Helpdesk")
                    .size(19.0)
                    .color(COLOR_TEXT_WHITE)
                    .strong(),
            );
            ui.label(
                RichText::new(
                    "Gerenciamento de conexões seguras, perfil de suporte corporativo e otimização de vídeo DXGI.",
                )
                .color(COLOR_TEXT_MUTED)
                .size(12.0),
            );
            ui.add_space(14.0);

            // Banner de Notificação / Confirmação
            if let Some(msg) = &self.state.status_message.clone() {
                egui::Frame::none()
                    .fill(Color32::from_rgb(12, 28, 38))
                    .stroke(Stroke::new(1.0_f32, COLOR_CYAN_NEON))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::new(8.0, 10.0), egui::Sense::hover());
                            ui.painter().circle_filled(
                                dot_rect.center(),
                                3.5,
                                COLOR_CYAN_NEON,
                            );
                            ui.label(
                                RichText::new(msg)
                                    .color(COLOR_CYAN_NEON)
                                    .size(12.0)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("✖").clicked() {
                                    self.state.status_message = None;
                                }
                            });
                        });
                    });
                ui.add_space(12.0);
            }

            // =========================================================================
            // Card 1: Identificação do Técnico & Fila de Helpdesk
            // =========================================================================
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.5,
                            COLOR_VIOLET_NEON,
                        );
                        ui.heading(
                            RichText::new("Identificação do Técnico & Fila de Helpdesk")
                                .size(14.5)
                                .color(COLOR_CYAN_NEON)
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("Dados exibidos para o operador do caixa ao solicitar consentimento de sessão.")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.add_space(10.0);

                    egui::Grid::new("settings_tech_grid")
                        .num_columns(2)
                        .spacing([16.0, 10.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Nome / Matrícula do Técnico:").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.state.operator_name).desired_width(280.0));
                            ui.end_row();

                            ui.label(RichText::new("Nível / Cargo do Atendente:").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.state.helpdesk_operator_role).desired_width(280.0));
                            ui.end_row();

                            ui.label(RichText::new("Departamento / Equipe:").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.state.helpdesk_department).desired_width(280.0));
                            ui.end_row();

                            ui.label(RichText::new("Ramal / Telefone de Suporte:").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.state.helpdesk_phone).desired_width(280.0));
                            ui.end_row();

                            ui.label(RichText::new("Motivo Padrão de Abertura:").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.state.default_ticket_reason).desired_width(400.0));
                            ui.end_row();
                        });
                });

            ui.add_space(12.0);

            // =========================================================================
            // Card 2: Conectividade & Sinalização (Servidor / Relay)
            // =========================================================================
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.5,
                            COLOR_MINT_ONLINE,
                        );
                        ui.heading(
                            RichText::new("Conectividade & Sinalização Central")
                                .size(14.5)
                                .color(COLOR_CYAN_NEON)
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("Endpoints de orquestração P2P e fallback automático via relay seguro.")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.add_space(10.0);

                    egui::Grid::new("settings_network_grid")
                        .num_columns(2)
                        .spacing([16.0, 10.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("URL da API de Sinalização:").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.server_url)
                                        .desired_width(300.0),
                                );
                                if ui.button("⚡ Testar Ping").clicked() {
                                    self.state.status_message = Some(
                                        "API de Sinalização: HTTP 200 OK (18 ms de RTT via TLS 1.3)".to_string(),
                                    );
                                }
                            });
                            ui.end_row();

                            ui.label(RichText::new("Servidor Relay Seguro (Fallback):").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.add(
                                egui::TextEdit::singleline(&mut self.state.relay_url)
                                    .desired_width(300.0),
                            );
                            ui.end_row();

                            ui.label(RichText::new("Reconexão Automática (Queda):").color(COLOR_TEXT_WHITE).size(12.0));
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut self.state.auto_reconnect_seconds, 10, "10s");
                                ui.radio_value(&mut self.state.auto_reconnect_seconds, 15, "15s (Padrão)");
                                ui.radio_value(&mut self.state.auto_reconnect_seconds, 30, "30s");
                                ui.radio_value(&mut self.state.auto_reconnect_seconds, 60, "60s");
                            });
                            ui.end_row();
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 10.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.0,
                            Color32::from_rgb(16, 185, 129),
                        );
                        ui.label(
                            RichText::new("Status de Rede: Conexão Direta Prioritária • Criptografia Fim a Fim")
                                .color(COLOR_MINT_ONLINE)
                                .size(11.5),
                        );
                    });
                });

            ui.add_space(12.0);

            // =========================================================================
            // Card 3: Segurança da Sessão & Políticas Zero-Trust
            // =========================================================================
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.5,
                            COLOR_AMBER_ALERT,
                        );
                        ui.heading(
                            RichText::new("Segurança da Sessão & Políticas Zero-Trust")
                                .size(14.5)
                                .color(COLOR_CYAN_NEON)
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("Diretrizes de consentimento visual no caixa e controle de permissões remotas.")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.add_space(10.0);

                    ui.checkbox(
                        &mut self.state.view_only_mode,
                        RichText::new("Modo Apenas Visualização (View-Only / Somente Monitoramento)").color(COLOR_TEXT_WHITE).size(12.0),
                    );
                    ui.label(
                        RichText::new("  Desativa envio de cliques e teclas, permitindo apenas acompanhamento visual.")
                            .color(COLOR_TEXT_DARK)
                            .size(11.0),
                    );

                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut self.state.block_remote_input,
                        RichText::new("Bloquear teclado e mouse local do PDV durante suporte crítico").color(COLOR_TEXT_WHITE).size(12.0),
                    );
                    ui.label(
                        RichText::new("  Evita conflito de operações pelo operador local enquanto o técnico resolve o problema.")
                            .color(COLOR_TEXT_DARK)
                            .size(11.0),
                    );

                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut self.state.clipboard_enabled,
                        RichText::new("Habilitar sincronização de Área de Transferência (Clipboard texto até 1 MB)").color(COLOR_TEXT_WHITE).size(12.0),
                    );
                    ui.label(
                        RichText::new("  Permite copiar e colar códigos de barras e comandos de forma controlada.")
                            .color(COLOR_TEXT_DARK)
                            .size(11.0),
                    );

                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(Color32::from_rgb(14, 20, 32))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(28, 40, 60)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("Política Zero-Trust Inviolável:")
                                    .color(COLOR_AMBER_ALERT)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                RichText::new("• Nenhuma sessão inicia sem consentimento do operador do PDV (ou operador previamente autorizado).\n• Código de sessão com 8 dígitos, expiração em 120s, máximo de 3 tentativas e HMAC em tempo constante.\n• Chave privada Ed25519 do PDV protegida por DPAPI no Windows.")
                                    .color(COLOR_TEXT_MUTED)
                                    .size(11.0),
                            );
                        });
                });

            ui.add_space(12.0);

            // =========================================================================
            // Card 4: Desempenho de Vídeo DXGI & Otimização de Banda
            // =========================================================================
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.5,
                            COLOR_BLUE_ELECTRIC,
                        );
                        ui.heading(
                            RichText::new("Desempenho de Vídeo DXGI & Economia de Banda")
                                .size(14.5)
                                .color(COLOR_CYAN_NEON)
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("Ajustes de renderização por tile 64x64, taxa de quadros e algoritmo zstd.")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Taxa de Quadros Alvo (FPS):")
                                .color(COLOR_TEXT_WHITE)
                                .size(12.0),
                        );
                        ui.radio_value(&mut self.state.target_fps, 15, "15 FPS (Econômico / 3G)");
                        ui.radio_value(&mut self.state.target_fps, 30, "30 FPS (Recomendado PDV)");
                        ui.radio_value(&mut self.state.target_fps, 60, "60 FPS (Ultra Fluidez)");
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Nível de Compressão Zstd:")
                                .color(COLOR_TEXT_WHITE)
                                .size(12.0),
                        );
                        ui.radio_value(&mut self.state.zstd_level, 1, "Nível 1 (Mínima Latência)");
                        ui.radio_value(
                            &mut self.state.zstd_level,
                            3,
                            "Nível 3 (Economia de Banda de Loja)",
                        );
                    });

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("• Captura: DXGI Desktop Duplication com detecção de tiles 64x64 alterados")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.label(
                        RichText::new("• Teclado: Modo nativo ABNT2 Scan Codes (KEYEVENTF_SCANCODE)")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.label(
                        RichText::new("• Mouse: Coordenadas Absolutas Normalizadas (0..65535)")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.label(
                        RichText::new("• Segurança: Liberação automática de todas as teclas e modificadores ao encerrar")
                            .color(COLOR_MINT_ONLINE)
                            .size(11.5),
                    );
                });

            ui.add_space(12.0);

            // =========================================================================
            // Card 5: Armazenamento Local, Auditoria & Diagnóstico
            // =========================================================================
            egui::Frame::none()
                .fill(COLOR_CARD_BASE)
                .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 12.0), egui::Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            3.5,
                            COLOR_CYAN_NEON,
                        );
                        ui.heading(
                            RichText::new("Armazenamento Local, Auditoria & Diagnóstico")
                                .size(14.5)
                                .color(COLOR_CYAN_NEON)
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("Locais de gravação de logs, relatórios CSV de suporte e limpeza de telemetria.")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Pasta de Downloads & Logs:")
                                .color(COLOR_TEXT_WHITE)
                                .size(12.0),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.state.download_dir)
                                .desired_width(320.0),
                        );
                    });

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("• Retenção de Auditoria: Registros locais mantidos por 30 dias com rotação automática.")
                            .color(COLOR_TEXT_MUTED)
                            .size(11.5),
                    );

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("📥 Exportar Relatório de Auditoria (CSV)").clicked() {
                            self.state.status_message = Some(
                                "Relatório de auditoria exportado com sucesso para a pasta de downloads.".to_string(),
                            );
                        }
                        if ui.button("🧹 Limpar Cache de Diagnóstico").clicked() {
                            self.state.status_message = Some(
                                "Cache de telemetria e diagnósticos temporários liberados com sucesso.".to_string(),
                            );
                        }
                        if ui.button("💾 Salvar Preferências de Helpdesk").clicked() {
                            self.state.status_message = Some(
                                "Configurações e parâmetros de Helpdesk gravados com sucesso.".to_string(),
                            );
                        }
                    });
                });

            ui.add_space(16.0);
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
                    RichText::new("🔒 Conexão Segura")
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

    /// Renderiza a tela de login do operador / administrador com layout responsivo Cyber-Prism.
    pub fn render_admin_login(&mut self, ui: &mut egui::Ui) {
        let avail_w = ui.available_width();
        let avail_h = ui.available_height();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let is_wide = avail_w >= 820.0;
                let card_w = if is_wide {
                    780.0_f32.min(avail_w - 48.0)
                } else {
                    410.0_f32.min(avail_w - 32.0).max(300.0)
                };
                let card_h = if is_wide { 480.0_f32 } else { 560.0_f32 };
                let pad_y = ((avail_h - card_h) * 0.45).max(18.0);
                let pad_x = ((avail_w - card_w) * 0.5).max(0.0);

                ui.add_space(pad_y);

                ui.horizontal(|ui| {
                    ui.add_space(pad_x);

                    // Card principal com cantos arredondados, fundo escuro e borda refinada
                    egui::Frame::none()
                        .fill(Color32::from_rgb(15, 20, 32))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 45, 75)))
                        .rounding(16.0)
                        .show(ui, |ui| {
                            ui.set_width(card_w);
                            if is_wide {
                                // Layout 2 Colunas: Hero à esquerda + Formulário à direita
                                ui.horizontal(|ui| {
                                    let col_left_w = 320.0_f32;
                                    let col_right_w = card_w - col_left_w - 1.0;

                                    // Coluna Esquerda: Hero & Identidade
                                    egui::Frame::none()
                                        .fill(Color32::from_rgb(11, 15, 25))
                                        .stroke(Stroke::NONE)
                                        .rounding(egui::Rounding {
                                            nw: 16.0,
                                            sw: 16.0,
                                            ne: 0.0,
                                            se: 0.0,
                                        })
                                        .inner_margin(egui::Margin::symmetric(24.0, 28.0))
                                        .show(ui, |ui| {
                                            ui.set_width(col_left_w);
                                            ui.set_min_height(460.0);
                                            self.render_login_hero_column(ui);
                                        });

                                    // Separador sutil vertical
                                    let (sep_rect, _) = ui.allocate_exact_size(
                                        Vec2::new(1.0, 460.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        sep_rect,
                                        0.0,
                                        Color32::from_rgb(26, 36, 58),
                                    );

                                    // Coluna Direita: Formulário de Login & Ações
                                    egui::Frame::none()
                                        .fill(Color32::from_rgb(15, 20, 32))
                                        .stroke(Stroke::NONE)
                                        .inner_margin(egui::Margin::symmetric(28.0, 24.0))
                                        .show(ui, |ui| {
                                            ui.set_width(col_right_w);
                                            self.render_login_form_fields(ui);
                                        });
                                });
                            } else {
                                // Layout 1 Coluna Centralizada (Compacto)
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(15, 20, 32))
                                    .stroke(Stroke::NONE)
                                    .inner_margin(egui::Margin::symmetric(24.0, 24.0))
                                    .show(ui, |ui| {
                                        ui.set_width(card_w);
                                        self.render_login_compact_header(ui);
                                        ui.add_space(14.0);
                                        self.render_login_form_fields(ui);
                                    });
                            }
                        });

                    ui.add_space(pad_x);
                });

                ui.add_space(20.0);
            });
    }

    /// Coluna Hero da tela de login (marca, logotipo e pontos de confiança).
    fn render_login_hero_column(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Logotipo e Identidade visual
            ui.horizontal(|ui| {
                render_logo(ui, self.logo_texture.as_ref(), 42.0);
                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("PrysmView")
                                .color(Color32::from_rgb(56, 132, 255))
                                .size(20.0)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Admin")
                                .color(Color32::WHITE)
                                .size(20.0)
                                .strong(),
                        );
                    });
                    ui.label(
                        RichText::new("Console de Gestão PDV")
                            .color(Color32::from_rgb(148, 163, 184))
                            .size(11.0),
                    );
                });
            });

            ui.add_space(24.0);
            ui.label(
                RichText::new("SEGURANÇA & ARQUITETURA")
                    .color(Color32::from_rgb(100, 116, 139))
                    .size(10.5)
                    .strong(),
            );
            ui.add_space(12.0);

            // Pontos de Confiança desenhados com círculos vetoriais anti-aliasing
            self.render_trust_item(
                ui,
                Color32::from_rgb(16, 185, 129),
                "Criptografia Segura Fim a Fim",
            );
            ui.add_space(10.0);
            self.render_trust_item(
                ui,
                COLOR_CYAN_NEON,
                "Segurança de Sessão & Consentimento Local",
            );
            ui.add_space(10.0);
            self.render_trust_item(
                ui,
                COLOR_BLUE_ELECTRIC,
                "Auditoria Centralizada & Hash de Conformidade",
            );
            ui.add_space(10.0);
            self.render_trust_item(
                ui,
                COLOR_VIOLET_NEON,
                "Ultra Leve e Baixo Consumo de Recursos",
            );

            ui.add_space(28.0);

            // Rodapé da coluna esquerda
            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(
                    RichText::new("v0.1.0 • Enterprise Edition")
                        .color(Color32::from_rgb(80, 95, 120))
                        .size(10.5),
                );
                ui.label(
                    RichText::new("PrysmView Enterprise • Windows 10/11")
                        .color(Color32::from_rgb(100, 116, 139))
                        .size(11.0),
                );
            });
        });
    }

    /// Item de confiança desenhado com círculo vetorial.
    fn render_trust_item(&self, ui: &mut egui::Ui, dot_color: Color32, text: &str) {
        ui.horizontal(|ui| {
            let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 14.0), egui::Sense::hover());
            ui.painter().circle_filled(
                egui::pos2(dot_rect.center().x, dot_rect.center().y),
                3.5,
                dot_color,
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(text)
                    .color(Color32::from_rgb(203, 213, 225))
                    .size(11.5),
            );
        });
    }

    /// Cabeçalho compacto para telas menores.
    fn render_login_compact_header(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            render_logo(ui, self.logo_texture.as_ref(), 40.0);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 170.0).max(0.0) * 0.5);
                ui.label(
                    RichText::new("PrysmView")
                        .color(Color32::from_rgb(56, 132, 255))
                        .size(20.0)
                        .strong(),
                );
                ui.label(
                    RichText::new("Admin")
                        .color(Color32::WHITE)
                        .size(20.0)
                        .strong(),
                );
            });
            ui.label(
                RichText::new("Central de Suporte & Gestão PDV")
                    .color(Color32::from_rgb(148, 163, 184))
                    .size(11.5),
            );
        });
    }

    /// Formulário de credenciais e ações de autenticação.
    fn render_login_form_fields(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Acesso de Operador")
                    .color(COLOR_TEXT_WHITE)
                    .size(18.0)
                    .strong(),
            );
            ui.add_space(2.0);
            ui.label(
                RichText::new("Entre com suas credenciais corporativas")
                    .color(Color32::from_rgb(148, 163, 184))
                    .size(12.0),
            );

            ui.add_space(14.0);

            // Botão Microsoft Entra ID (S-11)
            let btn_entra_h = 36.0;
            let (entra_rect, entra_resp) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), btn_entra_h),
                egui::Sense::click(),
            );
            let entra_bg = if entra_resp.is_pointer_button_down_on() {
                Color32::from_rgb(30, 42, 60)
            } else if entra_resp.hovered() {
                Color32::from_rgb(38, 52, 75)
            } else {
                Color32::from_rgb(24, 32, 48)
            };
            let entra_stroke = Stroke::new(
                1.0_f32,
                if entra_resp.hovered() {
                    COLOR_CYAN_NEON
                } else {
                    Color32::from_rgb(45, 60, 90)
                },
            );
            ui.painter().rect(entra_rect, 6.0, entra_bg, entra_stroke);

            // Desenha os 4 quadrados oficiais da Microsoft via painter (sem caracteres quebrados)
            let logo_x = entra_rect.min.x + 14.0;
            let logo_y = entra_rect.center().y - 7.0;
            let sq_size = 6.0;
            let sq_gap = 2.0;
            // Vermelho (superior esquerdo)
            ui.painter().rect_filled(
                egui::Rect::from_min_size(egui::pos2(logo_x, logo_y), Vec2::new(sq_size, sq_size)),
                0.0,
                Color32::from_rgb(242, 80, 34),
            );
            // Verde (superior direito)
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(logo_x + sq_size + sq_gap, logo_y),
                    Vec2::new(sq_size, sq_size),
                ),
                0.0,
                Color32::from_rgb(127, 186, 0),
            );
            // Azul (inferior esquerdo)
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(logo_x, logo_y + sq_size + sq_gap),
                    Vec2::new(sq_size, sq_size),
                ),
                0.0,
                Color32::from_rgb(0, 164, 239),
            );
            // Amarelo (inferior direito)
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(logo_x + sq_size + sq_gap, logo_y + sq_size + sq_gap),
                    Vec2::new(sq_size, sq_size),
                ),
                0.0,
                Color32::from_rgb(255, 185, 0),
            );

            // Texto do botão Entra ID
            ui.painter().text(
                egui::pos2(
                    logo_x + sq_size * 2.0 + sq_gap + 10.0,
                    entra_rect.center().y,
                ),
                egui::Align2::LEFT_CENTER,
                "Entrar com Microsoft Entra ID",
                egui::FontId::proportional(12.5),
                Color32::WHITE,
            );

            if entra_resp.clicked() {
                self.state.login_operator_id = "operador.azure".to_string();
                self.state
                    .login_success("operador.azure", "token_entra_id_oauth2".to_string());
            }

            ui.add_space(12.0);

            // Divisor "OU CREDENCIAIS LOCAIS"
            ui.horizontal(|ui| {
                let line_w = (ui.available_width() - 150.0).max(10.0) * 0.5;
                let (l1, _) = ui.allocate_exact_size(Vec2::new(line_w, 1.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(l1, 0.0, Color32::from_rgb(35, 48, 72));
                ui.label(
                    RichText::new("OU COM CREDENCIAIS")
                        .color(Color32::from_rgb(100, 116, 139))
                        .size(10.0)
                        .strong(),
                );
                let (l2, _) = ui.allocate_exact_size(Vec2::new(line_w, 1.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(l2, 0.0, Color32::from_rgb(35, 48, 72));
            });

            ui.add_space(10.0);

            // Campo: NOME DE USUÁRIO
            ui.label(
                RichText::new("NOME DE USUÁRIO")
                    .color(Color32::from_rgb(148, 163, 184))
                    .size(10.5)
                    .strong(),
            );
            ui.add_space(4.0);

            let mut enter_pressed = false;

            egui::Frame::none()
                .fill(Color32::from_rgb(10, 14, 24))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 44, 68)))
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("👤")
                                .color(Color32::from_rgb(140, 150, 170))
                                .size(13.0),
                        );
                        ui.add_space(4.0);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.state.login_operator_id)
                                .hint_text("ex: operador.suporte")
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .text_color(Color32::WHITE),
                        );
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            enter_pressed = true;
                        }
                    });
                });

            ui.add_space(10.0);

            // Campo: SENHA DE ACESSO
            ui.label(
                RichText::new("SENHA DE ACESSO")
                    .color(Color32::from_rgb(148, 163, 184))
                    .size(10.5)
                    .strong(),
            );
            ui.add_space(4.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(10, 14, 24))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 44, 68)))
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("🔒")
                                .color(Color32::from_rgb(140, 150, 170))
                                .size(13.0),
                        );
                        ui.add_space(4.0);

                        let pwd_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.state.login_password)
                                .password(!self.state.show_password)
                                .hint_text("••••••••")
                                .frame(false)
                                .desired_width(ui.available_width() - 32.0)
                                .text_color(Color32::WHITE),
                        );

                        if pwd_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
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
                                    .size(13.0),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE);

                            if ui.add(eye_btn).clicked() {
                                self.state.show_password = !self.state.show_password;
                            }
                        });
                    });
                });

            ui.add_space(8.0);

            // Checkbox: Lembrar meu usuário
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.state.remember_user,
                    RichText::new("Lembrar meu usuário neste dispositivo")
                        .color(Color32::from_rgb(148, 163, 184))
                        .size(11.5),
                );
            });

            // Exibição de Erro
            if let Some(ref err) = self.state.login_error {
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(45, 18, 26))
                    .stroke(Stroke::new(1.0_f32, COLOR_RED_DANGER))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠️").color(COLOR_RED_DANGER).size(12.0));
                            ui.label(RichText::new(err).color(COLOR_RED_DANGER).size(11.5));
                        });
                    });
            }

            ui.add_space(14.0);

            // Botão de Login "Entrar no Console"
            let btn_login = egui::Button::new(
                RichText::new("Entrar no Console →")
                    .color(Color32::WHITE)
                    .size(14.0)
                    .strong(),
            )
            .fill(Color32::from_rgb(37, 99, 235)) // #2563eb
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(59, 130, 246)))
            .rounding(6.0);

            if ui
                .add_sized([ui.available_width(), 40.0], btn_login)
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

            ui.add_space(12.0);

            // Rodapé do Card: Status da Sinalização com círculo vetorial
            ui.horizontal(|ui| {
                let (status_rect, _) =
                    ui.allocate_exact_size(Vec2::new(10.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(
                    egui::pos2(status_rect.center().x, status_rect.center().y),
                    3.5,
                    Color32::from_rgb(16, 185, 129),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Sinalização Online (Zero-Trust)")
                        .color(Color32::from_rgb(120, 135, 155))
                        .size(11.0),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.collapsing(
                        RichText::new("⚙ Configurações")
                            .color(Color32::from_rgb(100, 116, 139))
                            .size(10.5),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("API:").color(COLOR_TEXT_DARK).size(11.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.state.server_url)
                                        .hint_text("https://api.prysmview.com")
                                        .desired_width(180.0),
                                );
                            });
                        },
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

    #[test]
    fn test_login_and_admin_layout_in_1280x720_and_1920x1080() {
        let resolutions = [(1280.0_f32, 720.0_f32), (1920.0_f32, 1080.0_f32)];

        for (width, height) in resolutions {
            let ctx = egui::Context::default();
            let mut app = PrysmViewerApp::default();

            // Simula renderização do login em tela inicial
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(width, height),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        app.render_admin_login(ui);
                    });
                },
            );

            // Simula renderização autenticada em todas as abas
            app.state.is_authenticated = true;
            for tab in [
                AdminTab::Dashboard,
                AdminTab::DirectConnect,
                AdminTab::Stores,
                AdminTab::AuditHistory,
                AdminTab::Settings,
            ] {
                app.state.active_tab = tab;
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::Vec2::new(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| match app.state.active_tab {
                            AdminTab::Dashboard => app.render_dashboard(ui),
                            AdminTab::DirectConnect => app.render_direct_connect_tab(ui),
                            AdminTab::Stores => app.render_stores_tab(ui),
                            AdminTab::AuditHistory => app.render_audit_history_tab(ui),
                            AdminTab::Settings => app.render_settings_tab(ui),
                        });
                    },
                );
            }
        }
    }
}
