extern crate component;
extern crate eframe;

use eframe::egui;

use component::gui::AtlantixApp;

fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "Atlantix EDA - Component Library Generator",
        options,
        Box::new(|cc| {
            // Configure egui style
            configure_egui_style(&cc.egui_ctx);
            Ok(Box::new(AtlantixApp::new(cc)))
        }),
    )
}

fn configure_egui_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // PROPER DARK THEME FOR ENGINEERS
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
    style.visuals.window_fill = egui::Color32::from_rgb(32, 32, 32);
    style.visuals.panel_fill = egui::Color32::from_rgb(40, 40, 40);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(24, 24, 24);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(16, 16, 16);
    style.visuals.code_bg_color = egui::Color32::from_rgb(64, 64, 64);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 120, 215);
    style.visuals.selection.stroke.color = egui::Color32::from_rgb(0, 150, 255);
    
    // Widget colors
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(48, 48, 48);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(56, 56, 56);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(72, 72, 72);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 120, 215);
    
    // Button styling
    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(180, 180, 180);
    style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_rgb(200, 200, 200);
    style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);
    style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(255, 255, 255);
    
    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    
    ctx.set_style(style);
}