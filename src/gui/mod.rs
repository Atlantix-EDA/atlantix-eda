use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use egui_file_dialog::FileDialog;
// use egui_logger::LoggerUi;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::ecs::{components::*, resources::*, systems};
use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};
use log::info;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub output_formats: HashSet<String>,
    pub packages: HashSet<String>,
    pub manufacturers: HashSet<String>,
    pub e_series: u32,
    pub symbol_style: String,
    pub output_directory: String,
    pub kicad_target_lib: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut output_formats = HashSet::new();
        output_formats.insert("KiCad".to_string());
        
        let mut packages = HashSet::new();
        packages.insert("0603".to_string());
        packages.insert("0805".to_string());
        
        let mut manufacturers = HashSet::new();
        manufacturers.insert("Vishay".to_string());
        
        Self {
            output_formats,
            packages,
            manufacturers,
            e_series: 96,
            symbol_style: "European".to_string(),
            output_directory: "outputs".to_string(),
            kicad_target_lib: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GenerationStatus {
    Idle,
    Running { progress: f32, message: String },
    Completed { component_count: usize, duration: std::time::Duration },
    Error(String),
}

#[derive(Clone, PartialEq)]
pub enum TabType {
    Configuration,
    Generation,
    Preview,
    Logs,
}

pub struct AtlantixTab {
    pub tab_type: TabType,
    pub title: String,
}

impl AtlantixTab {
    pub fn new(tab_type: TabType) -> Self {
        let title = match tab_type {
            TabType::Configuration => "⚙️ Configuration".to_string(),
            TabType::Generation => "🚀 Generation".to_string(), 
            TabType::Preview => "👁️ Preview".to_string(),
            TabType::Logs => "📋 Logs".to_string(),
        };
        
        Self { tab_type, title }
    }
}

pub struct AtlantixApp {
    dock_state: DockState<AtlantixTab>,
    config: AppConfig,
    generation_status: Arc<Mutex<GenerationStatus>>,
    log_messages: Arc<Mutex<Vec<String>>>,
    // logger_ui: LoggerUi,
    file_dialog: FileDialog,
    preview_component_count: usize,
    show_about: bool,
}

impl AtlantixApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize logging
        info!("Atlantix EDA GUI starting up");
        
        // Create dock state with tabs
        let mut dock_state = DockState::new(vec![AtlantixTab::new(TabType::Configuration)]);
        
        // Add other tabs
        let [config_tab, _gen_tab] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.6,
            vec![AtlantixTab::new(TabType::Generation)]
        );
        
        let [_prev_tab, _logs_tab] = dock_state.main_surface_mut().split_below(
            config_tab,
            0.7,
            vec![AtlantixTab::new(TabType::Preview)]
        );
        
        dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.7,
            vec![AtlantixTab::new(TabType::Logs)]
        );
        
        Self {
            dock_state,
            config: AppConfig::default(),
            generation_status: Arc::new(Mutex::new(GenerationStatus::Idle)),
            log_messages: Arc::new(Mutex::new(Vec::new())),
            // logger_ui: LoggerUi,
            file_dialog: FileDialog::new(),
            preview_component_count: 0,
            show_about: false,
        }
    }

    fn calculate_component_count(&self) -> usize {
        let decades = 6; // 1Ω to 976KΩ
        let values_per_decade = match self.config.e_series {
            24 => 24,
            48 => 48,
            96 => 96,
            192 => 192,
            _ => 96,
        };
        let packages = self.config.packages.len();
        let manufacturers = self.config.manufacturers.len();
        
        values_per_decade * decades * packages * manufacturers
    }

    fn start_generation(&mut self) {
        let config = self.config.clone();
        let status = Arc::clone(&self.generation_status);
        let _log_messages = Arc::clone(&self.log_messages);
        
        info!("Starting component generation with {:?} packages", config.packages);
        
        // Reset status
        *status.lock().unwrap() = GenerationStatus::Running { 
            progress: 0.0, 
            message: "Initializing generation...".to_string() 
        };
        
        thread::spawn(move || {
            let start_time = std::time::Instant::now();
            
            // Create ECS world for generation
            let mut world = World::new();
            world.insert_resource(GeneratorConfig {
                output_formats: config.output_formats.iter()
                    .filter_map(|f| match f.as_str() {
                        "KiCad" => Some(OutputFormat::KicadSymbols),
                        "Altium" => Some(OutputFormat::Altium),
                        _ => None,
                    })
                    .collect(),
                manufacturers: config.manufacturers.iter().cloned().collect(),
                decades: vec![1, 10, 100, 1000, 10000, 100000],
            });
            world.insert_resource(ESeriesCache::default());
            
            // Update progress
            {
                let mut status_guard = status.lock().unwrap();
                *status_guard = GenerationStatus::Running { 
                    progress: 0.2, 
                    message: "Setting up component templates...".to_string() 
                };
            }
            
            // Spawn package templates
            for package_name in &config.packages {
                world.spawn((
                    ESeries(config.e_series as usize),
                    Package {
                        name: package_name.clone(),
                        imperial: package_name.clone(),
                        metric: get_metric_name(package_name),
                    },
                ));
                
                info!("Added package template: {}", package_name);
            }
            
            // Update progress
            {
                let mut status_guard = status.lock().unwrap();
                *status_guard = GenerationStatus::Running { 
                    progress: 0.6, 
                    message: "Generating resistor values...".to_string() 
                };
            }
            
            // Run generation systems
            let mut schedule = Schedule::default();
            schedule.add_systems((
                systems::generate_eseries_values,
                systems::assign_package_attributes,
                systems::generate_manufacturer_parts,
            ));
            
            schedule.run(&mut world);
            
            // Run post-generation systems
            let mut post_schedule = Schedule::default();
            post_schedule.add_systems((
                systems::assign_package_attributes,
                systems::generate_manufacturer_parts,
            ));
            post_schedule.run(&mut world);
            
            // Count generated components
            let component_count = world.query::<&ResistorValue>().iter(&world).count();
            
            info!("Generated {} resistor components", component_count);
            
            let duration = start_time.elapsed();
            
            // Final success status
            {
                let mut status_guard = status.lock().unwrap();
                *status_guard = GenerationStatus::Completed { component_count, duration };
            }
            
            info!("Generation completed in {:.2}s", duration.as_secs_f64());
        });
    }
}

impl AtlantixApp {
    fn show_configuration_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("🏭 Component Library Configuration");
            ui.add_space(10.0);
            
            // Output Format
            ui.group(|ui| {
                ui.label("📤 Output Format:");
                ui.horizontal(|ui| {
                    let mut kicad = self.config.output_formats.contains("KiCad");
                    let mut altium = self.config.output_formats.contains("Altium");
                    
                    if ui.checkbox(&mut kicad, "KiCad").clicked() {
                        if kicad {
                            self.config.output_formats.insert("KiCad".to_string());
                        } else {
                            self.config.output_formats.remove("KiCad");
                        }
                    }
                    
                    if ui.checkbox(&mut altium, "Altium").clicked() {
                        if altium {
                            self.config.output_formats.insert("Altium".to_string());
                        } else {
                            self.config.output_formats.remove("Altium");
                        }
                    }
                });
            });
            
            ui.add_space(10.0);
            
            // E-Series
            ui.group(|ui| {
                ui.label("📐 E-Series:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("E-{}", self.config.e_series))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.e_series, 24, "E-24 (5% tolerance)");
                        ui.selectable_value(&mut self.config.e_series, 48, "E-48 (2% tolerance)");
                        ui.selectable_value(&mut self.config.e_series, 96, "E-96 (1% tolerance)");
                        ui.selectable_value(&mut self.config.e_series, 192, "E-192 (0.5% tolerance)");
                    });
            });
            
            ui.add_space(10.0);
            
            // Packages
            ui.group(|ui| {
                ui.label("📦 Packages:");
                ui.horizontal_wrapped(|ui| {
                    let packages = ["0402", "0603", "0805", "1206", "1210", "2512"];
                    for package in &packages {
                        let mut selected = self.config.packages.contains(*package);
                        if ui.checkbox(&mut selected, *package).clicked() {
                            if selected {
                                self.config.packages.insert(package.to_string());
                            } else {
                                self.config.packages.remove(*package);
                            }
                        }
                    }
                });
            });
            
            ui.add_space(10.0);
            
            // Manufacturers
            ui.group(|ui| {
                ui.label("🏭 Manufacturers:");
                let mut vishay = self.config.manufacturers.contains("Vishay");
                if ui.checkbox(&mut vishay, "Vishay").clicked() {
                    if vishay {
                        self.config.manufacturers.insert("Vishay".to_string());
                    } else {
                        self.config.manufacturers.remove("Vishay");
                    }
                }
                
                ui.add_enabled(false, egui::Checkbox::new(&mut false, "Yageo (Coming Soon)"));
                ui.add_enabled(false, egui::Checkbox::new(&mut false, "KOA (Coming Soon)"));
            });
            
            ui.add_space(10.0);
            
            // KiCad Symbol Style
            if self.config.output_formats.contains("KiCad") {
                ui.group(|ui| {
                    ui.label("🎨 Symbol Style:");
                    egui::ComboBox::from_label("")
                        .selected_text(&self.config.symbol_style)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.symbol_style, "European".to_string(), "European (Rectangle)");
                            ui.selectable_value(&mut self.config.symbol_style, "American".to_string(), "American (Zigzag)");
                        });
                });
                ui.add_space(10.0);
            }
            
            // Directories
            ui.group(|ui| {
                ui.label("📁 Output Directory:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.config.output_directory);
                    if ui.add_enabled(false, egui::Button::new("📂 Browse")).clicked() {
                        // File dialog temporarily disabled
                    }
                });
                
                if !self.config.kicad_target_lib.is_empty() || self.config.output_formats.contains("KiCad") {
                    ui.label("🎯 KiCad Target Library:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.config.kicad_target_lib);
                        if ui.add_enabled(false, egui::Button::new("📂 Browse")).clicked() {
                            // File dialog temporarily disabled
                        }
                    });
                }
            });
        });
        
        // Handle file dialog (simplified)
        // TODO: Implement proper file dialog when compatible version is available
    }
    
    fn show_generation_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Component Generation");
        ui.add_space(10.0);
        
        // Component count preview
        self.preview_component_count = self.calculate_component_count();
        ui.label(format!("📊 Will generate {} components", self.preview_component_count));
        ui.add_space(10.0);
        
        let status = self.generation_status.lock().unwrap().clone();
        
        match status {
            GenerationStatus::Idle => {
                let can_generate = !self.config.packages.is_empty() 
                    && !self.config.output_formats.is_empty()
                    && !self.config.manufacturers.is_empty();
                
                if ui.add_enabled(can_generate, egui::Button::new("🚀 Generate Libraries").min_size(egui::vec2(200.0, 50.0))).clicked() {
                    self.start_generation();
                }
                
                if !can_generate {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "⚠️ Please select at least one package, format, and manufacturer");
                }
            }
            GenerationStatus::Running { progress, message } => {
                ui.add(egui::ProgressBar::new(progress).text(&message).desired_width(300.0));
                ui.colored_label(egui::Color32::from_rgb(0, 255, 127), &message);
            }
            GenerationStatus::Completed { component_count, duration } => {
                ui.colored_label(egui::Color32::from_rgb(0, 255, 127), 
                    format!("✅ Completed! Generated {} components in {:.2}s", 
                        component_count, duration.as_secs_f64()));
                
                if ui.button("🔄 Generate Again").clicked() {
                    *self.generation_status.lock().unwrap() = GenerationStatus::Idle;
                }
            }
            GenerationStatus::Error(error) => {
                ui.colored_label(egui::Color32::from_rgb(255, 69, 58), format!("❌ Error: {}", error));
                
                if ui.button("🔄 Try Again").clicked() {
                    *self.generation_status.lock().unwrap() = GenerationStatus::Idle;
                }
            }
        }
    }
    
    fn show_preview_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("👁️ Component Preview");
        ui.add_space(10.0);
        
        ui.label("Preview of generated components will appear here:");
        ui.add_space(10.0);
        
        // Example preview content
        ui.group(|ui| {
            ui.label("📋 Sample Components:");
            ui.separator();
            ui.label("• R0603_1.00 - 1.00Ω, 0603, 1%, 1/10W");
            ui.label("• R0603_1.05K - 1.05KΩ, 0603, 1%, 1/10W"); 
            ui.label("• R0805_10.0K - 10.0KΩ, 0805, 1%, 1/8W");
            ui.label("• R1206_100K - 100KΩ, 1206, 1%, 1/4W");
        });
        
        ui.add_space(10.0);
        
        ui.group(|ui| {
            ui.label("🏭 Manufacturer Info:");
            ui.separator();
            ui.label("• Manufacturer: Vishay");
            ui.label("• MPN: CRCW06031K05FKEA");
            ui.label("• Supplier: Digikey");
            ui.label("• Supplier PN: 541-1.05KHCT-ND");
            ui.label("• Supplier URL: https://www.digikey.com/products/en?keywords=541-1.05KHCT-ND");
        });
    }
    
    fn show_logs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Generation Logs");
        ui.add_space(5.0);
        
        // Show the egui_logger output (temporarily disabled)
        ui.label("📋 Logs will be displayed here when egui_logger version is updated");
    }
}

impl eframe::App for AtlantixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for progress updates
        ctx.request_repaint();
        
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("💾 Save Configuration").clicked() {
                        info!("Save configuration requested");
                        ui.close_menu();
                    }
                    if ui.button("📂 Load Configuration").clicked() {
                        info!("Load configuration requested");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🚪 Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.button("🔄 Reset Layout").clicked() {
                        // Reset dock layout
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("ℹ️ About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Atlantix EDA v0.2.0 | {} components", self.preview_component_count));
                });
            });
        });
        
        // Main dock area with manual borrowing split
        let AtlantixApp { dock_state, config, generation_status, log_messages, file_dialog, preview_component_count, show_about, .. } = self;
        
        DockArea::new(dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut AtlantixTabViewer { 
                config, 
                generation_status: generation_status.clone(), 
                log_messages: log_messages.clone(), 
                file_dialog, 
                preview_component_count: *preview_component_count,
            });
        
        // About dialog
        if self.show_about {
            egui::Window::new("About Atlantix EDA")
                .open(&mut self.show_about)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("🏭 Atlantix EDA Component Library Generator");
                    ui.label("Version 0.2.0");
                    ui.add_space(10.0);
                    ui.label("Professional PCB component library generation tool");
                    ui.label("Supports KiCad and Altium Designer formats");
                    ui.add_space(10.0);
                    ui.label("© 2019-2025 Atlantix Engineering");
                    ui.hyperlink_to("🌐 Visit Website", "https://github.com/saturn77/atlantix-eda");
                });
        }
    }
}

// Separate TabViewer to handle borrowing
struct AtlantixTabViewer<'a> {
    app: &'a mut AtlantixApp,
}

impl<'a> TabViewer for AtlantixTabViewer<'a> {
    type Tab = AtlantixTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title.clone().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.tab_type {
            TabType::Configuration => self.app.show_configuration_tab(ui),
            TabType::Generation => self.app.show_generation_tab(ui),
            TabType::Preview => self.app.show_preview_tab(ui),
            TabType::Logs => self.app.show_logs_tab(ui),
        }
    }
}

fn get_metric_name(package: &str) -> String {
    match package {
        "0402" => "1005Metric",
        "0603" => "1608Metric",
        "0805" => "2012Metric",
        "1206" => "3216Metric",
        "1210" => "3225Metric",
        "2512" => "6332Metric",
        _ => "UnknownMetric",
    }.to_string()
}