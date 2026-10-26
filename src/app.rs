//! GUI Application built using eframe / egui.

use crate::data::{default_achievements_dir, load_from_dir, AchievementsData, GameItem};
use crate::generator::{
    generate_passport_svg, render_svg_to_png, PassportOptions, PassportTheme,
};
use eframe::egui;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone)]
enum PassportMode {
    Overall,
    SingleGame,
}

pub struct PassportApp {
    achievements_dir: PathBuf,
    data: Option<AchievementsData>,
    error_message: Option<String>,

    // Options
    mode: PassportMode,
    selected_game_name: String,
    game_search_query: String,
    username: String,
    title_line: String,
    theme: PassportTheme,
    output_path: PathBuf,

    // Feedback
    status_notification: Option<(String, bool)>, // (message, is_error)

    // Preview
    preview_texture: Option<egui::TextureHandle>,
    last_rendered_svg: Option<String>,
}

impl PassportApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up dark cyberpunk styling
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(240, 246, 252));
        visuals.panel_fill = egui::Color32::from_rgb(5, 15, 26);
        visuals.window_fill = egui::Color32::from_rgb(7, 24, 41);
        cc.egui_ctx.set_visuals(visuals);

        let default_dir = default_achievements_dir();
        let default_output = dirs::picture_dir()
            .or_else(dirs::desktop_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("achievements_passport.png");

        let mut app = Self {
            achievements_dir: default_dir.clone(),
            data: None,
            error_message: None,

            mode: PassportMode::Overall,
            selected_game_name: String::new(),
            game_search_query: String::new(),
            username: "MOHANDL3G".to_string(),
            title_line: "Lv. 80   |   Day 665".to_string(),
            theme: PassportTheme::CyberpunkCyan,
            output_path: default_output,

            status_notification: None,
            preview_texture: None,
            last_rendered_svg: None,
        };

        app.reload_data(&cc.egui_ctx);
        app
    }

    fn reload_data(&mut self, ctx: &egui::Context) {
        match load_from_dir(&self.achievements_dir) {
            Ok(data) => {
                if !data.username.is_empty() {
                    self.username = data.username.clone();
                }
                if let Some(first_game) = data.games.first() {
                    if self.selected_game_name.is_empty() {
                        self.selected_game_name = first_game.name.clone();
                    }
                }
                self.data = Some(data);
                self.error_message = None;

                // Auto-generate initial preview
                self.generate_and_render_preview(ctx);
            }
            Err(e) => {
                self.error_message = Some(e);
                self.data = None;
            }
        }
    }

    fn selected_game(&self) -> Option<&GameItem> {
        let data = self.data.as_ref()?;
        data.games.iter().find(|g| g.name == self.selected_game_name)
    }

    fn generate_svg(&self) -> Option<String> {
        let data = self.data.as_ref()?;
        let selected_game = if self.mode == PassportMode::SingleGame {
            self.selected_game()
        } else {
            None
        };

        let options = PassportOptions {
            username: &self.username,
            title: &self.title_line,
            theme: self.theme,
            selected_game,
        };

        Some(generate_passport_svg(data, &options))
    }

    fn generate_and_render_preview(&mut self, ctx: &egui::Context) {
        if let Some(svg) = self.generate_svg() {
            self.last_rendered_svg = Some(svg.clone());
            // Render 1200x850 PNG for preview
            match render_svg_to_png(&svg, 1200, 850) {
                Ok(png_bytes) => {
                    if let Ok(image) = image::load_from_memory(&png_bytes) {
                        let size = [image.width() as _, image.height() as _];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_slice(),
                        );
                        self.preview_texture = Some(ctx.load_texture(
                            "passport_preview",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        ));
                    }
                }
                Err(err) => {
                    self.status_notification = Some((format!("Render error: {}", err), true));
                }
            }
        }
    }

    fn save_png(&mut self, ctx: &egui::Context) {
        self.generate_and_render_preview(ctx);
        if let Some(svg) = &self.last_rendered_svg {
            match render_svg_to_png(svg, 1200, 850) {
                Ok(bytes) => {
                    if let Some(parent) = self.output_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Err(e) = fs::write(&self.output_path, bytes) {
                        self.status_notification =
                            Some((format!("Failed to save image: {}", e), true));
                    } else {
                        self.status_notification = Some((
                            format!("✓ Passport saved to: {}", self.output_path.display()),
                            false,
                        ));
                    }
                }
                Err(e) => {
                    self.status_notification = Some((format!("PNG Render Error: {}", e), true));
                }
            }
        }
    }

    fn save_svg(&mut self) {
        if let Some(svg) = self.generate_svg() {
            let svg_path = self.output_path.with_extension("svg");
            if let Some(parent) = svg_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(&svg_path, svg) {
                self.status_notification = Some((format!("Failed to save SVG: {}", e), true));
            } else {
                self.status_notification = Some((
                    format!("✓ Vector SVG saved to: {}", svg_path.display()),
                    false,
                ));
            }
        }
    }
}

impl eframe::App for PassportApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TOP PANEL: Branding & Global Info
        egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("🎮 Achievements Passport Generator")
                        .color(egui::Color32::from_rgb(0, 240, 255))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(data) = &self.data {
                        ui.label(
                            egui::RichText::new(format!(
                                "✓ {} games loaded ({} perfect, {} unlocked)",
                                data.stats.total_configs,
                                data.stats.perfect_games,
                                data.stats.unlocked_achievements
                            ))
                            .color(egui::Color32::from_rgb(0, 255, 136)),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("⚠ No Achievements data loaded")
                                .color(egui::Color32::from_rgb(255, 85, 85)),
                        );
                    }
                });
            });
            ui.add_space(6.0);
        });

        // LEFT SIDEBAR: Controls & Customization
        egui::SidePanel::left("control_panel")
            .resizable(true)
            .default_width(380.0)
            .min_width(320.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);

                    // --- SOURCE DIRECTORY ---
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new("📁 Achievements AppData Source")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let mut dir_str = self.achievements_dir.to_string_lossy().to_string();
                            if ui.text_edit_singleline(&mut dir_str).changed() {
                                self.achievements_dir = PathBuf::from(dir_str);
                            }
                            if ui.button("Browse…").clicked() {
                                if let Some(folder) = rfd::FileDialog::new()
                                    .set_directory(&self.achievements_dir)
                                    .pick_folder()
                                {
                                    self.achievements_dir = folder;
                                    self.reload_data(ctx);
                                }
                            }
                        });
                        if ui.button("🔄 Reload Data").clicked() {
                            self.reload_data(ctx);
                        }

                        if let Some(err) = &self.error_message {
                            ui.colored_label(egui::Color32::from_rgb(255, 85, 85), err);
                        }
                    });

                    ui.add_space(10.0);

                    // --- PASSPORT CONFIGURATION ---
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new("⚙ Passport Options")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                        ui.add_space(6.0);

                        // Mode Selector
                        ui.label("Passport Type:");
                        ui.horizontal(|ui| {
                            let was_overall = self.mode == PassportMode::Overall;
                            if ui
                                .selectable_label(was_overall, "🏆 Overall Library")
                                .clicked()
                                && !was_overall
                            {
                                self.mode = PassportMode::Overall;
                                self.generate_and_render_preview(ctx);
                            }
                            let was_single = self.mode == PassportMode::SingleGame;
                            if ui
                                .selectable_label(was_single, "🎮 Single Game")
                                .clicked()
                                && !was_single
                            {
                                self.mode = PassportMode::SingleGame;
                                self.generate_and_render_preview(ctx);
                            }
                        });

                        ui.add_space(8.0);

                        // If single game mode: game selector
                        if self.mode == PassportMode::SingleGame {
                            let mut selection_changed = false;
                            if let Some(data) = &self.data {
                                ui.label("Search & Select Game:");
                                ui.text_edit_singleline(&mut self.game_search_query);

                                let query = self.game_search_query.to_lowercase();
                                let filtered_games: Vec<(String, String)> = data
                                    .games
                                    .iter()
                                    .filter(|g| {
                                        query.is_empty()
                                            || g.name.to_lowercase().contains(&query)
                                    })
                                    .take(50)
                                    .map(|g| {
                                        (
                                            g.name.clone(),
                                            format!(
                                                "{} ({}/{} - {}%)",
                                                g.name, g.unlocked, g.total, g.pct
                                            ),
                                        )
                                    })
                                    .collect();

                                egui::ComboBox::from_id_salt("game_select_box")
                                    .selected_text(&self.selected_game_name)
                                    .width(ui.available_width() - 10.0)
                                    .show_ui(ui, |ui| {
                                        for (name, label) in filtered_games {
                                            if ui
                                                .selectable_value(
                                                    &mut self.selected_game_name,
                                                    name,
                                                    label,
                                                )
                                                .clicked()
                                            {
                                                selection_changed = true;
                                            }
                                        }
                                    });
                                ui.add_space(8.0);
                            }

                            if selection_changed {
                                self.generate_and_render_preview(ctx);
                            }
                        }

                        // Player Identity
                        ui.label("Player / Gamer Tag:");
                        if ui.text_edit_singleline(&mut self.username).changed() {
                            self.generate_and_render_preview(ctx);
                        }

                        ui.add_space(6.0);
                        ui.label("Subtitle / Level & Day:");
                        if ui.text_edit_singleline(&mut self.title_line).changed() {
                            self.generate_and_render_preview(ctx);
                        }

                        ui.add_space(6.0);
                        ui.label("HUD Color Theme:");
                        let prev_theme = self.theme;
                        egui::ComboBox::from_id_salt("theme_select_box")
                            .selected_text(self.theme.name())
                            .show_ui(ui, |ui| {
                                for t in PassportTheme::all() {
                                    ui.selectable_value(&mut self.theme, t, t.name());
                                }
                            });
                        if self.theme != prev_theme {
                            self.generate_and_render_preview(ctx);
                        }
                    });

                    ui.add_space(10.0);

                    // --- OUTPUT & EXPORT ---
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new("💾 Export Destination")
                                .strong()
                                .color(egui::Color32::from_rgb(0, 240, 255)),
                        );
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            let mut path_str = self.output_path.to_string_lossy().to_string();
                            if ui.text_edit_singleline(&mut path_str).changed() {
                                self.output_path = PathBuf::from(path_str);
                            }
                            if ui.button("Browse…").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_file_name("achievements_passport.png")
                                    .add_filter("PNG Image", &["png"])
                                    .save_file()
                                {
                                    self.output_path = path;
                                }
                            }
                        });

                        ui.add_space(12.0);

                        // ACTION BUTTONS
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("✨ Generate & Save Passport (PNG)")
                                        .size(16.0)
                                        .strong()
                                        .color(egui::Color32::BLACK),
                                )
                                .fill(egui::Color32::from_rgb(0, 240, 255))
                                .min_size(egui::vec2(ui.available_width(), 42.0)),
                            )
                            .clicked()
                        {
                            self.save_png(ctx);
                        }

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            if ui.button("💾 Save SVG Vector").clicked() {
                                self.save_svg();
                            }
                            if ui.button("📁 Open Folder").clicked() {
                                if let Some(parent) = self.output_path.parent() {
                                    let _ = open::that(parent);
                                }
                            }
                        });

                        // Notification feedback
                        if let Some((msg, is_err)) = &self.status_notification {
                            ui.add_space(8.0);
                            let color = if *is_err {
                                egui::Color32::from_rgb(255, 85, 85)
                            } else {
                                egui::Color32::from_rgb(0, 255, 136)
                            };
                            ui.label(egui::RichText::new(msg).color(color).strong());
                        }
                    });

                    ui.add_space(15.0);
                });
            });

        // CENTRAL PANEL: Live Passport Preview
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(
                egui::RichText::new("Live Passport Preview")
                    .color(egui::Color32::from_rgb(141, 164, 190))
                    .size(16.0),
            );
            ui.separator();

            if let Some(texture) = &self.preview_texture {
                let avail_size = ui.available_size();
                // Passport aspect ratio: 1200 / 850 = ~1.4117
                let aspect_ratio = 1200.0 / 850.0;

                let mut w = avail_size.x;
                let mut h = w / aspect_ratio;
                if h > avail_size.y {
                    h = avail_size.y;
                    w = h * aspect_ratio;
                }

                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.image(egui::load::SizedTexture::new(texture.id(), egui::vec2(w, h)));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("Select a valid Achievements folder to preview passport")
                            .color(egui::Color32::GRAY),
                    );
                });
            }
        });
    }
}
