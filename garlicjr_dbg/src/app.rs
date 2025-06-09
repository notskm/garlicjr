/*
    Copyright 2024-2025 notskm

    This file is part of garlicjr.

    garlicjr is free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by the Free
    Software Foundation, either version 3 of the License, or (at your option)
    any later version.

    garlicjr is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
    more details.

    You should have received a copy of the GNU General Public License along
    with garlicjr. If not, see <https: //www.gnu.org/licenses/>.
*/

use std::sync::mpsc::{Receiver, Sender, channel};

use crate::ui::*;
use egui::TextureHandle;
use garlicjr::*;
use rfd::AsyncFileDialog;

const REPO_URL: Option<&str> = option_env!("GARLICJR_REPO_URL");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GarlicJrApp {
    #[serde(skip)]
    bootrom_channel: (Sender<DmgBootrom>, Receiver<DmgBootrom>),

    license_window_open: bool,
    about_window_open: bool,

    #[serde(skip)]
    disclaimer_window_open: bool,

    #[serde(skip)]
    running: bool,

    #[serde(skip)]
    cpu: SharpSM83,

    #[serde(skip)]
    ram: Vec<u8>,

    #[serde(skip)]
    bus: Bus,

    #[serde(skip)]
    framebuffer: egui::ColorImage,

    #[serde(skip)]
    screen_texture: Option<TextureHandle>,

    #[serde(skip)]
    tile_data_buffer: egui::ColorImage,

    #[serde(skip)]
    tile_data_texture: Option<TextureHandle>,
}

impl Default for GarlicJrApp {
    fn default() -> Self {
        let color = egui::Color32::GRAY;

        // Provide a default program for now
        let mut ram = vec![0; 65536];
        ram[0] = 0x06;
        ram[1] = 0x42;
        ram[2] = 0x16;
        ram[3] = 0x20;
        ram[4] = 0x78;
        ram[5] = 0x42;
        ram[6] = 0x57;
        ram[7] = 0x3E;
        ram[8] = 0x00;

        // Add a tileset to VRAM
        const GARLICJR_TILES: [u8; 128] = [
            0x00, 0x00, 0x1C, 0x1C, 0x22, 0x22, 0x20, 0x20, 0x2E, 0x2E, 0x22, 0x22, 0x1C, 0x1C,
            0x00, 0x00, 0x18, 0x18, 0x24, 0x24, 0x24, 0x24, 0x3C, 0x3C, 0x42, 0x42, 0x42, 0x42,
            0x42, 0x42, 0x00, 0x00, 0x1C, 0x1C, 0x22, 0x22, 0x42, 0x42, 0x44, 0x44, 0x78, 0x78,
            0x50, 0x50, 0x48, 0x48, 0x46, 0x46, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
            0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x3C, 0x3C, 0x6C, 0x6C, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x6C, 0x6C, 0x3C, 0x3C, 0x42, 0x42,
            0x82, 0x82, 0x80, 0x80, 0x80, 0x80, 0x82, 0x82, 0x42, 0x42, 0x3C, 0x3C, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x22, 0x22, 0x22, 0x22, 0x1C, 0x1C,
            0x1C, 0x1C, 0x22, 0x22, 0x42, 0x42, 0x44, 0x44, 0x78, 0x78, 0x50, 0x50, 0x48, 0x48,
            0x46, 0x46,
        ];

        ram.splice(0x8000..0x8000 + GARLICJR_TILES.len(), GARLICJR_TILES);

        Self {
            bootrom_channel: channel(),
            license_window_open: false,
            about_window_open: false,
            disclaimer_window_open: true,
            running: false,
            cpu: SharpSM83::new(),
            ram,
            bus: Bus::new(),
            screen_texture: None,
            framebuffer: egui::ColorImage {
                pixels: [color; 160 * 144].to_vec(),
                size: [160, 144],
            },
            tile_data_buffer: egui::ColorImage {
                pixels: [color; 8 * 16 * 8 * 24].to_vec(),
                size: [8 * 16, 8 * 24],
            },
            tile_data_texture: None,
        }
    }
}

impl GarlicJrApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        match cc.storage {
            Some(storage) => eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
            None => Self::default(),
        }
    }

    fn change_bootrom_and_reset(&mut self, bootrom: DmgBootrom) {
        let data = bootrom.data();
        self.ram.fill(0);
        self.ram.splice(0..bootrom.data().len(), *data);
        self.cpu = SharpSM83::new();
    }
}

impl eframe::App for GarlicJrApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(bootrom) = self.bootrom_channel.1.try_recv() {
            self.change_bootrom_and_reset(bootrom);
        }

        if self.running {
            let cycles = (1_000_000f32 * frame.info().cpu_usage.unwrap_or(0f32)) as u64;
            for _ in 0..cycles {
                run_gameboy_cycle(&mut self.cpu, &mut self.ram, &mut self.bus);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load bootrom...").clicked() {
                        let task = AsyncFileDialog::new().pick_file();

                        let ctx = ui.ctx().clone();
                        let sender = self.bootrom_channel.0.clone();

                        execute(async move {
                            let file = task.await;

                            if let Some(file) = file {
                                let contents = file.read().await;
                                let bootrom = DmgBootrom::from_reader(contents.as_slice());

                                if let Ok(bootrom) = bootrom {
                                    let _ = sender.send(bootrom);
                                }

                                ctx.request_repaint();
                            }
                        });
                    }

                    // NOTE: no File->Quit on web pages!
                    let is_web = cfg!(target_arch = "wasm32");
                    if !is_web && ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("View License").clicked() {
                        self.license_window_open = true;
                        ui.close_menu();
                    }
                    if ui.button("About").clicked() {
                        self.about_window_open = true;
                        ui.close_menu();
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);

                if let Some(url) = REPO_URL {
                    ui.hyperlink_to("Source code.", url);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("GarlicJr");

            ui.label(format!(
                "Update time: {:.3}",
                frame.info().cpu_usage.unwrap_or(0f32)
            ));

            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                ui.label(format!("Core version: {}", garlicjr::version()));
                ui.label(format!("UI version: {}", VERSION));
                egui::warn_if_debug_build(ui);
            });
        });

        egui::Window::new("License")
            .scroll([false, true])
            .open(&mut self.license_window_open)
            .show(ctx, |ui| {
                const LICENSE_INFO: &str = include_str!("../../COPYING");
                ui.label(LICENSE_INFO);
            });

        egui::Window::new("About")
            .open(&mut self.about_window_open)
            .show(ctx, |ui| {
                ui.label(format!("Core version: {}", VERSION));
                ui.label(format!("UI version: {}", VERSION));
            });

        egui::Window::new("Features")
            .open(&mut self.disclaimer_window_open)
            .show(ctx, |ui| {
                let disclaimer = "- Load boot ROM: ✔\n- Load ROMs: ❌\n- Display: ❌\n- Audio: ❌";
                ui.label(disclaimer);
            });

        egui::Window::new("Screen").show(ctx, |ui| {
            let texture: &mut egui::TextureHandle = self.screen_texture.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "Screen",
                    self.framebuffer.clone(),
                    egui::TextureOptions::NEAREST,
                )
            });

            texture.set(self.framebuffer.clone(), egui::TextureOptions::NEAREST);
            ui.image((texture.id(), texture.size_vec2()));
        });

        egui::Window::new("Tile Data").show(ctx, |ui| {
            let texture: &mut egui::TextureHandle =
                self.tile_data_texture.get_or_insert_with(|| {
                    ui.ctx().load_texture(
                        "Tile Data",
                        self.tile_data_buffer.clone(),
                        egui::TextureOptions::NEAREST,
                    )
                });

            tile_data(&self.ram, &mut self.tile_data_buffer);
            texture.set(self.tile_data_buffer.clone(), egui::TextureOptions::NEAREST);
            ui.image((texture.id(), texture.size_vec2()));
        });

        egui::Window::new("CPU")
            .resizable([true, true])
            .show(ctx, |ui| {
                cpu_gui(
                    ui,
                    &mut self.cpu,
                    &mut self.ram,
                    &mut self.bus,
                    &mut self.running,
                    run_gameboy_cycle,
                );
            });

        egui::Window::new("RAM").show(ctx, |ui| {
            ram_table("RAM Table", ctx, ui, &mut self.ram, self.bus.address);
        });

        ctx.request_repaint();
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn run_gameboy_cycle(cpu: &mut SharpSM83, ram: &mut [u8], bus: &mut Bus) {
    for _ in 0..4 {
        cpu.tick(bus);
    }

    match bus.mode {
        ReadWriteMode::Write => ram[bus.address as usize] = bus.data,
        ReadWriteMode::Read => bus.data = ram[bus.address as usize],
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    futures::executor::block_on(f);
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
