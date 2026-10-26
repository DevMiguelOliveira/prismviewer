//! prysmview-viewer - Tokens de Design System e Tema Cyber Prism (U-01)
//!
//! Conforme as regras da Fase 4 (U-01 e U-13):
//! - Tokens centralizados de cores semânticas, tipografia e espaçamento.
//! - Contraste mínimo de 4,5:1 para conformidade com acessibilidade WCAG AA.
//! - Corpo de texto padrão com no mínimo 14 px para legibilidade em monitores de caixa.

use eframe::egui::{Color32, Stroke, Visuals};

// ==============================================================================
// 1. CORES SEMÂNTICAS (Cyber Prism Palette)
// ==============================================================================
pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(0, 240, 255); // #00F0FF Neon Cyan
pub const COLOR_PRIMARY_HOVER: Color32 = Color32::from_rgb(0, 200, 235); // #00C8EB
pub const COLOR_SECONDARY: Color32 = Color32::from_rgb(0, 140, 255); // #008CFF Electric Blue
pub const COLOR_ACCENT: Color32 = Color32::from_rgb(190, 45, 235); // #BE2DEB Prism Violet

pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(0, 255, 204); // #00FFCC Signal Mint
pub const COLOR_WARNING: Color32 = Color32::from_rgb(255, 184, 0); // #FFB800 Amber Alert
pub const COLOR_DANGER: Color32 = Color32::from_rgb(255, 51, 85); // #FF3355 Emergency Crimson

pub const COLOR_BG_OBSIDIAN: Color32 = Color32::from_rgb(10, 13, 22); // #0A0D16 Fundo principal escuro
pub const COLOR_PANEL_NAVY: Color32 = Color32::from_rgb(15, 20, 34); // #0F1422 Painéis laterais e header
pub const COLOR_CARD_BASE: Color32 = Color32::from_rgb(20, 27, 45); // #141B2D Fundo de cards
pub const COLOR_CARD_BORDER: Color32 = Color32::from_rgb(32, 45, 75); // #202D4B Borda sutil

pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255); // #FFFFFF Branco puro (contraste > 10:1)
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 174, 192); // #A0AEC0 Cinza claro (contraste > 4.5:1)
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(113, 128, 150); // #718096 Cinza neutro

// ==============================================================================
// 2. ESCALA TIPOGRÁFICA (Tamanhos em pixels)
// ==============================================================================
pub const TEXT_SIZE_SM: f32 = 12.0;
pub const TEXT_SIZE_BODY: f32 = 14.0; // Padrão mínimo de acessibilidade (U-13)
pub const TEXT_SIZE_MD: f32 = 16.0;
pub const TEXT_SIZE_LG: f32 = 20.0;
pub const TEXT_SIZE_XL: f32 = 24.0;

// ==============================================================================
// 3. ESPAÇAMENTO (Design Spacing Tokens)
// ==============================================================================
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

// ==============================================================================
// 4. CONFIGURAÇÃO DE VISUAL DO EGUI
// ==============================================================================
pub fn build_cyber_prism_visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    visuals.window_fill = COLOR_CARD_BASE;
    visuals.panel_fill = COLOR_PANEL_NAVY;
    visuals.extreme_bg_color = COLOR_BG_OBSIDIAN;
    visuals.override_text_color = Some(COLOR_TEXT_PRIMARY);

    visuals.widgets.noninteractive.bg_fill = COLOR_CARD_BASE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_CARD_BORDER);

    visuals.widgets.inactive.bg_fill = COLOR_CARD_BASE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_CARD_BORDER);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(26, 36, 58);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_PRIMARY);

    visuals.widgets.active.bg_fill = COLOR_SECONDARY;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, COLOR_PRIMARY);

    visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0, 240, 255, 45);
    visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_PRIMARY);

    visuals
}

/// Calcula a luminância relativa de uma cor RGB (sRGB) conforme a fórmula WCAG 2.1.
pub fn relative_luminance(c: Color32) -> f64 {
    fn channel(v: u8) -> f64 {
        let s = v as f64 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
}

/// Calcula o ratio de contraste entre duas cores conforme WCAG 2.1 (mínimo 4.5:1 para texto normal).
pub fn contrast_ratio(foreground: Color32, background: Color32) -> f64 {
    let l1 = relative_luminance(foreground);
    let l2 = relative_luminance(background);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_contrast_ratios_meet_wcag_aa() {
        // Texto primário (#FFFFFF) sobre fundo escuro (#0A0D16)
        let cr_primary = contrast_ratio(COLOR_TEXT_PRIMARY, COLOR_BG_OBSIDIAN);
        assert!(
            cr_primary >= 4.5,
            "Contraste do texto primário deve ser >= 4.5:1, obteve {:.2}",
            cr_primary
        );

        // Texto secundário (#A0AEC0) sobre fundo escuro (#0A0D16)
        let cr_secondary = contrast_ratio(COLOR_TEXT_SECONDARY, COLOR_BG_OBSIDIAN);
        assert!(
            cr_secondary >= 4.5,
            "Contraste do texto secundário deve ser >= 4.5:1, obteve {:.2}",
            cr_secondary
        );

        // Texto primário sobre fundo do card (#141B2D)
        let cr_card = contrast_ratio(COLOR_TEXT_PRIMARY, COLOR_CARD_BASE);
        assert!(
            cr_card >= 4.5,
            "Contraste sobre fundo do card deve ser >= 4.5:1, obteve {:.2}",
            cr_card
        );
    }

    #[test]
    fn test_typography_scale_hierarchy() {
        assert!(TEXT_SIZE_SM < TEXT_SIZE_BODY);
        assert!(TEXT_SIZE_BODY < TEXT_SIZE_MD);
        assert!(TEXT_SIZE_MD < TEXT_SIZE_LG);
        assert!(TEXT_SIZE_LG < TEXT_SIZE_XL);
        assert!(TEXT_SIZE_BODY >= 14.0, "Corpo de texto deve ser >= 14px para acessibilidade");
    }
}
