use std::sync::Arc;
use std::sync::Mutex;

use eframe::egui;
use egui::ScrollArea;
use egui::{CentralPanel, Ui};
use egui::{ColorImage, FontFamily, TextureOptions, Vec2};
use egui::{FontId, TextureHandle};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::{SurfaceCanvas, Texture};

use crate::bus::mapper::MapperTrait;
use crate::{EmulatorCore, OSCILLOSCOPE_DEPTH, OSCILLOSCOPE_SAMPLES};

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

enum ApuMenu {
    Registers,
    Visualizer,
}

const LABELS: [&str; 6] = [
    "Pulse I ", "Pulse II", "Triangle", "Noise   ", "DMC     ", "Mixed   ",
];

pub struct Gui<'a> {
    core: Arc<Mutex<EmulatorCore>>,
    selected_menu: Menu,
    selected_apu_menu: ApuMenu,
    scratch_surface: SurfaceCanvas<'a>,
    oscilloscopes: [Texture; 6],
    oscilloscope_handles: [Option<TextureHandle>; 6],
}

impl Gui<'_> {
    pub fn new(core: Arc<Mutex<EmulatorCore>>) -> Self {
        let scratch_surface = sdl2::surface::Surface::new(
            OSCILLOSCOPE_SAMPLES as u32,
            OSCILLOSCOPE_DEPTH as u32,
            PixelFormatEnum::ABGR8888,
        )
        .unwrap()
        .into_canvas()
        .unwrap();

        let texture_creator = scratch_surface.texture_creator();

        let oscilloscopes = [
            texture_creator
                .create_texture_target(
                    PixelFormatEnum::ABGR8888,
                    OSCILLOSCOPE_SAMPLES as u32,
                    OSCILLOSCOPE_DEPTH as u32,
                )
                .unwrap(),
            texture_creator
                .create_texture_target(
                    PixelFormatEnum::ABGR8888,
                    OSCILLOSCOPE_SAMPLES as u32,
                    OSCILLOSCOPE_DEPTH as u32,
                )
                .unwrap(),
            texture_creator
                .create_texture_target(
                    PixelFormatEnum::ABGR8888,
                    OSCILLOSCOPE_SAMPLES as u32,
                    OSCILLOSCOPE_DEPTH as u32,
                )
                .unwrap(),
            texture_creator
                .create_texture_target(
                    PixelFormatEnum::ABGR8888,
                    OSCILLOSCOPE_SAMPLES as u32,
                    OSCILLOSCOPE_DEPTH as u32,
                )
                .unwrap(),
            texture_creator
                .create_texture_target(
                    PixelFormatEnum::ABGR8888,
                    OSCILLOSCOPE_SAMPLES as u32,
                    OSCILLOSCOPE_DEPTH as u32,
                )
                .unwrap(),
            texture_creator
                .create_texture_target(
                    PixelFormatEnum::ABGR8888,
                    OSCILLOSCOPE_SAMPLES as u32,
                    OSCILLOSCOPE_DEPTH as u32,
                )
                .unwrap(),
        ];

        Self {
            core,
            // selected_menu: Menu::General,
            selected_menu: Menu::Apu,
            selected_apu_menu: ApuMenu::Visualizer,
            scratch_surface,
            oscilloscopes,
            oscilloscope_handles: [None, None, None, None, None, None],
        }
    }
}

impl eframe::App for Gui<'_> {
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
                    ui.horizontal(|ui| {
                        if ui.button("Registers").clicked() {
                            self.selected_apu_menu = ApuMenu::Registers;
                        };

                        if ui.button("Visualizer").clicked() {
                            self.selected_apu_menu = ApuMenu::Visualizer;
                        };
                    });

                    match self.selected_apu_menu {
                        ApuMenu::Registers => {
                            core.cpu.bus.apu().print(ui);
                        }
                        ApuMenu::Visualizer => {
                            ui.vertical(|ui| {
                                let Vec2 { y: h, .. } = ui.available_size();
                                let ymargin = h / 15.0;
                                let rect = Rect::new(
                                    0,
                                    0,
                                    OSCILLOSCOPE_SAMPLES as u32,
                                    OSCILLOSCOPE_DEPTH as u32,
                                );

                                for fb in 0..self.oscilloscopes.len() {
                                    let mut pixels = Vec::with_capacity(
                                        OSCILLOSCOPE_SAMPLES * OSCILLOSCOPE_DEPTH * 4,
                                    );

                                    let buf = &mut core.sample_buffers[fb];

                                    let len = buf.len();

                                    self.scratch_surface
                                        .with_texture_canvas(
                                            &mut self.oscilloscopes[fb],
                                            |canvas| {
                                                if len >= OSCILLOSCOPE_SAMPLES {
                                                    canvas.set_draw_color(Color::BLUE);
                                                    canvas.clear();

                                                    canvas.set_draw_color(Color::WHITE);

                                                    canvas
                                                        .draw_line(
                                                            Point::new(0, buf[0] as i32),
                                                            Point::new(1, buf[1] as i32),
                                                        )
                                                        .unwrap();

                                                    for x in 1..len - 1 {
                                                        let x = x as i32;

                                                        canvas
                                                            .draw_line(
                                                                Point::new(
                                                                    x,
                                                                    buf[x as usize] as i32,
                                                                ),
                                                                Point::new(
                                                                    x + 1,
                                                                    buf[x as usize + 1] as i32,
                                                                ),
                                                            )
                                                            .unwrap();
                                                    }

                                                    canvas
                                                        .draw_line(
                                                            Point::new(
                                                                (len - 2) as i32,
                                                                buf[len - 2] as i32,
                                                            ),
                                                            Point::new(
                                                                (len - 1) as i32,
                                                                buf[len - 1] as i32,
                                                            ),
                                                        )
                                                        .unwrap();

                                                    buf.clear();
                                                }

                                                pixels = canvas
                                                    .read_pixels(rect, PixelFormatEnum::ABGR8888)
                                                    .unwrap();
                                            },
                                        )
                                        .unwrap();

                                    let handle = self.oscilloscope_handles[fb].get_or_insert(
                                        ui.ctx().load_texture(
                                            "",
                                            ColorImage::from_rgba_unmultiplied(
                                                [OSCILLOSCOPE_SAMPLES, OSCILLOSCOPE_DEPTH],
                                                pixels.as_slice(),
                                            ),
                                            TextureOptions::NEAREST,
                                        ),
                                    );

                                    handle.set(
                                        ColorImage::from_rgba_unmultiplied(
                                            [OSCILLOSCOPE_SAMPLES, OSCILLOSCOPE_DEPTH],
                                            pixels.as_slice(),
                                        ),
                                        TextureOptions::NEAREST,
                                    );

                                    ui.horizontal(|ui| {
                                        ui.label(LABELS[fb]);
                                        let Vec2 { x: w, .. } = ui.available_size();
                                        let xmargin = w / 10.0;

                                        ui.image(
                                            self.oscilloscope_handles[fb].as_ref().unwrap().id(),
                                            Vec2::new(w - xmargin, (h / 6.0) - ymargin),
                                        );
                                    });
                                }
                            });
                        }
                    }
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
