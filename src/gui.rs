use std::sync::Arc;
use std::sync::Mutex;

use eframe::egui;
use egui::FontFamily;
use egui::FontId;
use egui::ScrollArea;
use egui::{CentralPanel, Ui};

use crate::bus::mapper::MapperTrait;
use crate::EmulatorCore;

pub trait DebugInfo {
    fn print(&self, ui: &mut Ui);
}

enum Menu {
    General,
    Cpu,
    Ppu,
    Apu,
    Memory,
    Mapper,
}

pub struct Gui {
    core: Arc<Mutex<EmulatorCore>>,
    selected_menu: Menu,
}

impl Gui {
    pub fn new(core: Arc<Mutex<EmulatorCore>>) -> Self {
        Self {
            core,
            selected_menu: Menu::General,
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let core = &mut *self.core.lock().unwrap();

        if !core.running {
            std::process::exit(0);
        }

        CentralPanel::default().show(ctx, |ui| {
            let mut style = ui.style_mut();

            style.override_font_id = Some(FontId::new(16.0, FontFamily::Monospace));

            ui.horizontal(|ui| {
                if ui.button("Emulator").clicked() {
                    self.selected_menu = Menu::General;
                };

                if ui.button("CPU").clicked() {
                    self.selected_menu = Menu::Cpu;
                };

                if ui.button("PPU").clicked() {
                    self.selected_menu = Menu::Ppu;
                };

                if ui.button("APU").clicked() {
                    self.selected_menu = Menu::Apu;
                };

                if ui.button("Memory").clicked() {
                    self.selected_menu = Menu::Memory;
                };

                if ui.button("Mapper").clicked() {
                    self.selected_menu = Menu::Mapper;
                };
            });

            match self.selected_menu {
                Menu::General => {
                    core.print(ui);
                }
                Menu::Cpu => {
                    core.cpu.print(ui);
                }
                Menu::Ppu => {
                    core.cpu.bus.ppu().print(ui);
                }
                Menu::Apu => {
                    core.cpu.bus.apu().print(ui);
                }
                Menu::Memory => {
                    let text_style = egui::TextStyle::Monospace;

                    let row_height = ui.text_style_height(&text_style);

                    ScrollArea::vertical().show_rows(ui, row_height, 0x10000 / 16, |ui, rows| {
                        ui.horizontal(|ui| {
                            ui.label("    ");

                            (0..=0xF).for_each(|n| {
                                ui.label(format!("{n:02X}"));
                            });
                        });

                        for row in rows {
                            let row_idx = row * 16;

                            ui.horizontal(|ui| {
                                ui.label(format!("{row_idx:04X}"));

                                (0..16).for_each(|offset| {
                                    ui.label(format!(
                                        "{:02X}",
                                        if (0x2000..=0x4017).contains(&(row_idx + offset)) {
                                            0
                                        } else {
                                            core.cpu.bus.read_u8((row_idx + offset) as u16)
                                        }
                                    ));
                                });
                            });
                        }
                    });
                }
                Menu::Mapper => {
                    ui.vertical(|ui| {
                        ui.label(format!("Mapper: {:?}", core.mapper_type));

                        ui.separator();

                        core.cpu.bus.print_debug_info(ui);
                    });
                }
            }

            ctx.request_repaint();
        });
    }
}
