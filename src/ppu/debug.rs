use crate::gui::DebugInfo;
use crate::ppu::oam::Attributes;
use crate::ppu::Ppu;
use crate::util::bit::Bit;
use byte_slice_cast::AsByteSlice;
use egui::{ColorImage, TextureId, TextureOptions, Ui, Vec2};
use serde::{Deserialize, Serialize};

impl Ppu {
    fn dump_nt(&self) -> [[u32; 256 * 240]; 4] {
        let mut nametables = [[0; 256 * 240]; 4];

        for nt in 0..4 {
            for cy in 0..30 {
                for fy in 0..8 {
                    for cx in 0..32 {
                        for fx in 0..8 {
                            let pt = self.ctrl.bg_pt_addr;

                            let v = fy << 12 | nt << 10 | cy << 5 | cx;
                            let nt_byte = self.read(0x2000 | (v & 0xFFF)) as u16;
                            let pt_low = self.read(pt + nt_byte * 16 + v.bits(12, 14));
                            let pt_high = self.read(pt + nt_byte * 16 + v.bits(12, 14) + 8);
                            let at_byte = self
                                .read(0x23C0 | (v & 0xC00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x7));

                            let palette_low = pt_low.bit(7 - fx) as u8;
                            let palette_high = pt_high.bit(7 - fx) as u8;
                            let palette_idx = palette_high << 1 | palette_low;

                            let at_bits = (at_byte
                                >> ((v.bit(6) as u8) * 4 + (v.bit(1) as u8) * 2))
                                .bits_abs(0, 1);

                            let pixel = match palette_idx {
                                0 => 0,
                                idx @ 1..=3 => (at_bits << 2) | idx,
                                _ => unreachable!(),
                            };

                            let color = {
                                let palette_idx = self.pram[pixel as usize].bits_abs(0, 5);

                                self.palette[palette_idx]
                            };

                            nametables[nt as usize]
                                [((cy * 8 + fy) * 256 + cx * 8 + fx as u16) as usize] =
                                <u32>::from(color);
                        }
                    }
                }
            }
        }

        nametables
    }

    fn dump_pt(&self) -> [[u32; 128 * 128]; 2] {
        let mut pattern_tables = [[0; 128 * 128]; 2];

        for pt in 0..2 {
            for cy in 0..16 {
                for fy in 0..8 {
                    for cx in 0..16 {
                        for fx in 0..8 {
                            let addr = pt << 12 | cy << 8 | cx << 4 | fy;
                            let pt_low = self.read(addr);
                            let pt_high = self.read(addr + 8);

                            let palette_low = pt_low.bit(7 - fx) as u8;
                            let palette_high = pt_high.bit(7 - fx) as u8;
                            let palette_idx = palette_high << 1 | palette_low;

                            let color = match palette_idx {
                                0 => 0xFF000000,
                                1 => 0xFF555555,
                                2 => 0xFFAAAAAA,
                                3 => 0xFFFFFFFF,
                                _ => unreachable!(),
                            };

                            pattern_tables[pt as usize]
                                [((cy * 8 + fy) * 128 + cx * 8 + fx as u16) as usize] = color
                        }
                    }
                }
            }
        }

        pattern_tables
    }

    fn dump_spr(&self) -> [u32; 256 * 240] {
        let bg = self.palette[self.pram[0].bits_abs(0, 5)];
        let mut sprites = [<u32>::from(bg); 256 * 240];

        for sprite in self.oam.sprites().chunks_exact(4).rev() {
            let height = self.ctrl.size / 8;
            let spr_y = sprite[0] as u16;
            let idx = sprite[1];
            let at = Attributes::from(sprite[2]);
            let spr_x = sprite[3] as u16;

            for y in 0..height {
                let y = if at.flip_vert { height - 1 - y } else { y } as u16;

                if spr_y + y >= 0xEF {
                    continue;
                }

                let pt_low = self.spr_fetch_pt_low(idx, y);
                let pt_high = self.spr_fetch_pt_high(idx, y);

                let attrib = at.palette;

                for x in 0..8 {
                    let xoff = if at.flip_horiz { 7 - x } else { x } as usize;

                    let palette_low = pt_low.bit(7 - xoff) as u8;
                    let palette_high = pt_high.bit(7 - xoff) as u8;
                    let palette_idx = palette_high << 1 | palette_low;

                    let pixel = match palette_idx {
                        0 => 0,
                        idx @ 1..=3 => (attrib << 2) | idx,
                        _ => unreachable!(),
                    };

                    let color = {
                        let palette_idx = self.pram[pixel as usize].bits_abs(0, 5);

                        self.palette[palette_idx]
                    };

                    sprites[((spr_y + y) * 256 + spr_x + x as u16) as usize] = <u32>::from(color);
                }
            }
        }

        sprites
    }
}

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub(super) enum Menu {
    #[default]
    Registers,
    Nametable,
    PatternTable,
    Sprites,
}

impl DebugInfo for Ppu {
    fn print(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Registers").clicked() {
                self.selected_menu.replace(Menu::Registers);
            };

            if ui.button("Nametable viewer").clicked() {
                self.selected_menu.replace(Menu::Nametable);
            };

            if ui.button("Pattern table viewer").clicked() {
                self.selected_menu.replace(Menu::PatternTable);
            };

            if ui.button("Sprite viewer").clicked() {
                self.selected_menu.replace(Menu::Sprites);
            };
        });

        match self.selected_menu.get() {
            Menu::Registers => {
                ui.vertical(|ui| {
                    ui.label("Registers");

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("PPUCTRL");

                        ui.vertical(|ui| {
                            let _ = ui.radio(self.ctrl.nmi, "NMI");

                            let _ = ui.radio(self.ctrl.master_slave, "Master/slave mode");

                            let _ = ui.radio(self.ctrl.size == 8 * 16, "8x16 Sprite mode");

                            ui.label(format!(
                                "Background pattern table address: ${:04X}",
                                self.ctrl.bg_pt_addr
                            ));

                            ui.label(format!(
                                "Sprite pattern table address: ${:04X}",
                                self.ctrl.spr_pt_addr
                            ));

                            let _ = ui.radio(self.ctrl.inc_vert, "Vertical increment");

                            ui.label(format!(
                                "Base nametable address: ${:04X}",
                                self.ctrl.base_nt_addr
                            ));
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("PPUMASK");

                        ui.vertical(|ui| {
                            let _ = ui.radio(self.mask.emphasize_blue, "Emphasize blue");

                            let _ = ui.radio(self.mask.emphasize_green, "Emphasize green");

                            let _ = ui.radio(self.mask.emphasize_red, "Emphasize red");

                            let _ = ui.radio(self.mask.show_sprites, "Show sprites");

                            let _ = ui.radio(self.mask.show_background, "Show background");

                            let _ = ui.radio(self.mask.show_sprites_leftmost_8, "Show sprites leftmost 8");

                            let _ = ui.radio(
                                self.mask.show_background_leftmost_8,
                                "Show background leftmost 8",
                            );

                            let _ = ui.radio(self.mask.greyscale, "Greyscale");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("PPUSTATUS");

                        let status = self.status.get();

                        ui.vertical(|ui| {
                            let _ = ui.radio(status.vblank, "Vblank");

                            let _ = ui.radio(status.spr_0_hit, "Sprite zero hit");

                            let _ = ui.radio(status.spr_overflow, "Sprite overflow");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("OAMADDR");

                        ui.label(format!("${:04X}", self.oam.addr()));
                    });

                    ui.horizontal(|ui| {
                        ui.label("PPUADDR");

                        let v = self.v.address();
                        let t = self.t.address();

                        ui.label(format!("v: ${:04X}\nCoarse x: {}\nCoarse y: {}\nFine x: {}\nFine y: {}\nNametable: {}\nw: {}", v, v.bits_abs(0, 4), v.bits(5, 9), self.x, v.bits(12,14), v.bits(10,11), self.w.get()));

                        ui.label(format!("t: ${:04X}\nCoarse x: {}\nCoarse y: {}\nFine x: {}\nFine y: {}\nNametable: {}\nw: {}", t, t.bits_abs(0, 4), t.bits(5, 9), self.x, t.bits(12,14), t.bits(10,11), self.w.get()));
                    });

                    ui.label(format!(
                        "Frame: {} Scanline: {} dot: {}",
                        self.frame, self.scanline, self.dot
                    ));
                    if let Some(mmc3) = self.mmc3.as_ref() {
                        ui.separator();
                        mmc3.print(ui);
                        ui.label(format!("A12: {}", self.a12.get() as u8));
                    }
                });
            }
            Menu::Nametable => {
                let nts = self.dump_nt();
                let mut nt_ids = [TextureId::default(); 4];

                for nt in 0..4 {
                    let texture = ui
                        .ctx()
                        .load_texture(
                            "Nametables",
                            ColorImage::from_rgba_unmultiplied([256, 240], nts[nt].as_byte_slice()),
                            TextureOptions::NEAREST,
                        )
                        .id();

                    nt_ids[nt] = texture;
                }

                ui.vertical(|ui| {
                    let Vec2 { x: w, y: h } = ui.available_size();

                    ui.horizontal(|ui| {
                        ui.image(nt_ids[0], Vec2::new(w / 2.0, h / 2.0));
                        ui.image(nt_ids[1], Vec2::new(w / 2.0, h / 2.0));
                    });
                    ui.horizontal(|ui| {
                        ui.image(nt_ids[2], Vec2::new(w / 2.0, h / 2.0));
                        ui.image(nt_ids[3], Vec2::new(w / 2.0, h / 2.0));
                    });
                });
            }
            Menu::PatternTable => {
                let pts = self.dump_pt();
                let mut pt_ids = [TextureId::default(); 2];

                for pt in 0..2 {
                    let texture = ui
                        .ctx()
                        .load_texture(
                            "Pattern tables",
                            ColorImage::from_rgba_unmultiplied([128, 128], pts[pt].as_byte_slice()),
                            TextureOptions::NEAREST,
                        )
                        .id();

                    pt_ids[pt] = texture;
                }

                let Vec2 { x: w, y: h } = ui.available_size();

                ui.horizontal(|ui| {
                    ui.image(pt_ids[0], Vec2::new(w / 2.0, h));
                    ui.image(pt_ids[1], Vec2::new(w / 2.0, h));
                });
            }
            Menu::Sprites => {
                let sprites = self.dump_spr();

                let texture = ui
                    .ctx()
                    .load_texture(
                        "Sprites",
                        ColorImage::from_rgba_unmultiplied([256, 240], sprites.as_byte_slice()),
                        TextureOptions::NEAREST,
                    )
                    .id();

                ui.image(texture, ui.available_size());
            }
        }
    }
}
