use std::cell::Cell;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use address::Address;
use ctrl::Control;
use mask::Mask;
use oam::{Oam, Sprite};
pub use palette::{PaletteTable, Rgba};
use status::Status;

use crate::bus::mapper::BankSettings;
use crate::cartridge::{Cartridge, MapperType, Mirroring};
use crate::ppu::mmc3::Mmc3;
use crate::ppu::oam::Attributes;
use crate::util::bit::Bit;
use crate::util::shift_reg::ShiftRegister;
use debug::Menu;

mod address;
mod ctrl;
mod debug;
mod mask;
mod mmc3;
mod oam;
mod palette;
mod status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Nmi;

#[derive(Serialize, Deserialize, Clone)]
pub struct Ppu {
    pub bank_settings: BankSettings,
    pub chr: Vec<u8>,
    pub dot: u16,
    #[serde(with = "BigArray")]
    pub fb: [u32; 256 * 240],
    pub frame: usize,
    pub interrupts: Vec<Nmi>,
    pub mirroring: Mirroring,
    pub mmc3: Option<Mmc3>,
    pub mmc7_vram_page: u8,
    pub nmi_occurred: Cell<bool>,
    pub oam: Oam,
    pub ram: bool,
    pub scanline: u16,
    a12: Cell<bool>,
    a12_filter: Cell<u8>,
    a12_rising: Cell<bool>,
    at_hi_latch: u8,
    at_latch: u8,
    at_lo_latch: u8,
    at_shifters: [ShiftRegister<u16, 16>; 2],
    buf: Cell<u8>,
    bus: Cell<u8>,
    ctrl: Control,
    cur_spr: Option<Sprite>,
    last_nmi_pair: bool,
    mapper_type: MapperType,
    mask: Mask,
    nt_latch: u8,
    odd: bool,
    old_a12: Cell<bool>,
    palette: PaletteTable,
    ppuscroll_delay: (u8, bool),
    pram: [u8; 0x20],
    pt_hi_latch: u8,
    pt_lo_latch: u8,
    pt_shifters: [ShiftRegister<u16, 16>; 2],
    #[serde(skip)]
    selected_menu: Cell<Menu>,
    spr0_present: bool,
    spr_active: [bool; 8],
    spr_counters: [u16; 8],
    spr_idx: usize,
    spr_latches: [Attributes; 8],
    spr_pt_shifters: [[ShiftRegister<u8, 8>; 2]; 8],
    spr_shift_count: [u8; 8],
    status: Cell<Status>,
    sx: u16,
    t: Address,
    v: Address,
    #[serde(with = "BigArray")]
    vram: [u8; 0x800],
    w: Cell<bool>,
    x: u8,
}

impl Ppu {
    pub fn new(mut cartridge: Cartridge, mapper_type: MapperType) -> Self {
        let mirroring = cartridge.mirroring;
        let chr_rom = cartridge.chr_rom.take().unwrap();
        let bank_settings = BankSettings::new(match mapper_type {
            MapperType::Nrom => {
                vec![(0, (0..0x2000))]
            }
            MapperType::MMC1 => {
                vec![(0, (0..0x1000)), (1, (0x1000..0x2000))]
            }
            MapperType::Uxrom => {
                vec![(0, (0..0x2000))]
            }
            MapperType::Cnrom => {
                vec![(0, (0..0x2000))]
            }
            MapperType::MMC3 => {
                vec![
                    (0, 0x0000..0x0800),
                    (1, 0x0800..0x1000),
                    (2, 0x1000..0x1400),
                    (3, 0x1400..0x1800),
                    (4, 0x1800..0x1C00),
                    (5, 0x1C00..0x2000),
                ]
            }
            MapperType::Axrom => {
                vec![(0, (0..0x2000))]
            }
        });
        let ram = chr_rom.is_empty();

        Self {
            bank_settings,
            chr: if chr_rom.is_empty() {
                vec![0; 0x2000]
            } else {
                chr_rom
            },
            dot: 0,
            fb: [0; 256 * 240],
            frame: 1,
            interrupts: Vec::new(),
            mirroring,
            mmc3: if mapper_type == MapperType::MMC3 {
                Some(Mmc3::new())
            } else {
                None
            },
            mmc7_vram_page: 0,
            nmi_occurred: Cell::new(false),
            oam: Oam::new(),
            ram,
            scanline: 261,
            a12: Cell::new(false),
            a12_filter: Cell::new(9),
            a12_rising: Cell::new(false),
            at_hi_latch: 0,
            at_latch: 0,
            at_lo_latch: 0,
            at_shifters: [ShiftRegister::new(); 2],
            buf: Cell::new(0),
            bus: Cell::new(0),
            ctrl: Control::new(),
            cur_spr: None,
            last_nmi_pair: false,
            mapper_type,
            mask: Mask::new(),
            nt_latch: 0,
            odd: false,
            old_a12: Cell::new(false),
            palette: PaletteTable::new(),
            ppuscroll_delay: (0, false),
            pram: [0; 0x20],
            pt_hi_latch: 0,
            pt_lo_latch: 0,
            pt_shifters: [ShiftRegister::new(); 2],
            selected_menu: Cell::new(Menu::Registers),
            spr0_present: true,
            spr_active: [false; 8],
            spr_counters: [0; 8],
            spr_idx: 0,
            spr_latches: [Attributes::new(); 8],
            spr_pt_shifters: [[ShiftRegister::new(); 2]; 8],
            spr_shift_count: [0; 8],
            status: Cell::new(Status::new()),
            sx: 0,
            t: Address::new(),
            v: Address::new(),
            vram: [0; 0x800],
            w: Cell::new(false),
            x: 0,
        }
    }

    pub fn read_reg(&self, reg: u8) -> u8 {
        match reg {
            0 => self.bus.get(),
            1 => self.bus.get(),
            2 => {
                let status = u8::from(self.status.get());
                let mut new_status = Status::from(status);

                new_status.vblank = self.nmi_occurred.get();
                self.nmi_occurred.replace(false);
                self.status.replace(new_status);

                self.w.replace(false);

                let bus = self.bus.get();

                let data = status.bits_abs(5, 7) | bus.bits_abs(0, 4);

                self.bus.replace(data);

                data
            }
            3 => self.bus.get(),
            4 => {
                let blank = self.status.get().vblank
                    || (!self.mask.show_background && !self.mask.show_sprites);
                let data = self.oam.read(!blank);

                self.bus.replace(data);

                data
            }
            5 => self.bus.get(),
            6 => self.bus.get(),
            7 => {
                let addr = self.v.address().bits_abs(0, 14);

                let data;

                if addr < 0x3F00 {
                    data = self.buf.get();
                    self.buf.replace(self.read(addr));
                } else {
                    data = self.read(addr);
                }

                self.bus.replace(data);

                self.v.increment(self.ctrl.inc_vert);

                self.watch_a12(self.v.address().bit(12));

                data
            }
            _ => unreachable!(),
        }
    }

    pub fn set_mirroring_mode(&mut self, mode: Mirroring) {
        self.mirroring = mode;
    }

    pub fn tick(&mut self) {
        let show_bg = self.mask.show_background;
        let show_spr = self.mask.show_sprites;
        let rendering = show_bg || show_spr;

        // Idle on first dot
        if self.dot != 0 {
            match self.scanline {
                0..=239 => self.tick_visible_scanline(rendering),
                241 if self.dot == 1 => {
                    self.status.get_mut().vblank = true;

                    if self.ctrl.nmi {
                        self.nmi_occurred.replace(true);
                        self.interrupts.push(Nmi {});
                    }
                }
                261 => self.tick_prerender_scanline(rendering),
                _ => {}
            }
        }

        self.dot = (self.dot + 1) % 341;

        if self.dot == 0 {
            self.scanline = (self.scanline + 1) % 262;
            self.sx = 0;
        }

        if let (cycles, true) = &mut self.ppuscroll_delay {
            if *cycles == 0 {
                self.v = self.t.clone();

                self.watch_a12(self.v.address().bit(12));

                self.ppuscroll_delay = (0, false);
            } else {
                *cycles -= 1;
            }
        }

        if let Some(mmc3) = self.mmc3.as_mut() {
            if !self.a12.get() {
                self.a12_filter
                    .replace(self.a12_filter.get().saturating_sub(1));
            } else {
                self.a12_filter.replace(9);
            }

            let a12_rising = self.a12_rising.get();

            if a12_rising {
                self.a12_rising.replace(false);

                mmc3.irq.clock();
            }
        }
    }

    pub fn write_reg(&mut self, reg: u8, data: u8) {
        self.bus.replace(data);

        match reg {
            0 => {
                self.ctrl = Control::from(data);

                let nt_bits = data as u16 & 0b11;
                let addr = self.t.address();
                let addr = addr.bits_abs(12, 14) | addr.bits_abs(0, 9) | (nt_bits << 10);

                self.t.update(addr);

                let nmi_pair = self.ctrl.nmi && self.nmi_occurred.get();

                if !self.last_nmi_pair && nmi_pair {
                    self.last_nmi_pair = nmi_pair;
                    self.interrupts.push(Nmi {});
                }
            }
            1 => {
                self.mask = Mask::from(data);
            }
            2 => {}
            3 => self.oam.set_addr(data),
            4 => self.oam.write(data),
            5 => {
                let w = self.w.get();

                if !w {
                    self.x = data & 0b111;
                }

                self.t.write_scroll(data, w);
                self.w.replace(!self.w.get());
            }
            6 => {
                let w = self.w.get();

                self.t.write_addr(data, w);
                self.w.replace(!self.w.get());

                if w {
                    self.ppuscroll_delay = (3, true);
                }
            }
            7 => {
                let addr = self.v.address().bits_abs(0, 14);

                self.write(addr, data);
                self.v.increment(self.ctrl.inc_vert);
                self.watch_a12(self.v.address().bit(12));
            }
            _ => unreachable!(),
        }
    }

    fn addr_to_bank_and_offset(&self, addr: u16) -> (usize, usize, usize) {
        let addr = addr as i32;

        let (bank, addresses) = self
            .bank_settings
            .iter()
            .find(|(_, addresses)| addresses.contains(&addr))
            .unwrap();

        let start = addresses.start as usize;
        let end = addresses.end as usize;
        let bank_size = end - start;

        if let Some(mmc3) = self.mmc3.as_ref() {
            let bank = mmc3.banks[*bank];
            let addr = addr as usize % bank_size;

            let bank = if addr >= 0x400 { bank + 1 } else { bank };

            (bank, 0x400, addr % 0x400)
        } else {
            (*bank, bank_size, addr as usize % bank_size)
        }
    }

    fn calculate_spr_pixel(&mut self) -> Option<(usize, u8, bool)> {
        if self.sx < 8 && !self.mask.show_sprites_leftmost_8 {
            return None;
        }

        let pixel = self
            .spr_active
            .iter_mut()
            .enumerate()
            .filter(|(_, &mut active)| active)
            .find_map(|(idx, _)| {
                let palette_low = self.spr_pt_shifters[idx][0].peek();
                let palette_high = self.spr_pt_shifters[idx][1].peek();
                let palette_idx = palette_high << 1 | palette_low;

                if palette_idx == 0 {
                    return None;
                }

                let attrib = self.spr_latches[idx].palette;
                let priority = self.spr_latches[idx].priority;
                let pixel = 1 << 4 | (attrib << 2) | palette_idx;

                Some((idx, pixel, priority))
            });

        self.spr_active
            .iter_mut()
            .enumerate()
            .for_each(|(idx, active)| {
                if !*active {
                    return;
                }

                self.spr_pt_shifters[idx][0].pop();
                self.spr_pt_shifters[idx][1].pop();
                self.spr_shift_count[idx] += 1;
                if self.spr_shift_count[idx] == 8 {
                    self.spr_shift_count[idx] = 0;
                    *active = false;
                }
            });

        pixel
    }

    fn calculate_bg_pixel(&self) -> u8 {
        if self.sx < 8 && !self.mask.show_background_leftmost_8 {
            return 0;
        }

        let palette_low = self.pt_shifters[0].peek_n(8);
        let palette_high = self.pt_shifters[1].peek_n(8);

        let palette_low = palette_low.bit(self.x as usize) as u8;
        let palette_high = palette_high.bit(self.x as usize) as u8;
        let palette_idx = palette_high << 1 | palette_low;

        let attrib_low = self.at_shifters[0].peek_n(8);
        let attrib_high = self.at_shifters[1].peek_n(8);

        let attrib_low = attrib_low.bit(self.x as usize) as u8;
        let attrib_high = attrib_high.bit(self.x as usize) as u8;
        let attrib = attrib_high << 1 | attrib_low;

        match palette_idx {
            0 => 0,
            idx @ 1..=3 => (attrib << 2) | idx,
            _ => unreachable!(),
        }
    }

    fn draw_pixel(&mut self, rgb: Option<Rgba>, x: usize, y: usize) {
        let idx = x + y * 256;

        self.fb[idx] = rgb.map_or(0, <u32>::from);
    }

    fn fetch_at_byte(&self) -> u8 {
        let v = self.v.address();
        let addr = 0x23C0 | (v & 0xC00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x7);

        self.read(addr)
    }

    fn fetch_nt_byte(&self) -> u8 {
        let v = self.v.address();
        let addr = 0x2000 | (v & 0xFFF);

        self.watch_a12(addr.bit(12));

        self.read(addr)
    }

    fn fetch_pt_low(&self, nt_byte: u8) -> u8 {
        let pt = self.ctrl.bg_pt_addr;
        let tile_idx = nt_byte as u16;
        let addr = pt + tile_idx * 16 + self.v.address().bits(12, 14);

        self.watch_a12(addr.bit(12));

        self.read(addr)
    }

    fn fetch_pt_high(&self, nt_byte: u8) -> u8 {
        let pt = self.ctrl.bg_pt_addr;
        let tile_idx = nt_byte as u16;
        let addr = pt + tile_idx * 16 + self.v.address().bits(12, 14) + 8;

        self.watch_a12(addr.bit(12));

        self.read(addr)
    }

    fn mirrored_address(&self, addr: u16) -> u16 {
        let addr = match addr {
            0x3000..=0x3EFF => addr - 0x1000,
            0x0..=0x1FFF => return addr,
            _ => addr,
        };

        match self.mirroring {
            Mirroring::Vertical => {
                let addr = if addr >= 0x2800 { addr - 0x800 } else { addr };

                addr - 0x2000
            }
            Mirroring::Horizontal => {
                let addr = match addr {
                    0x2400..=0x27FF => addr - 0x400,
                    0x2800..=0x2BFF => addr - 0x400,
                    0x2C00..=0x2FFF => addr - 0x800,
                    _ => addr,
                };

                addr - 0x2000
            }
            Mirroring::OneScreenLowerBank => {
                let addr = match addr {
                    0x2400..=0x27FF => addr - 0x400,
                    0x2800..=0x2BFF => addr - 0x800,
                    0x2C00..=0x2FFF => addr - 0xC00,
                    _ => addr,
                };

                addr - 0x2000
            }
            Mirroring::OneScreenUpperBank => {
                let addr = match addr {
                    0x2000..=0x23FF => addr + 0x400,
                    0x2800..=0x2BFF => addr - 0x400,
                    0x2C00..=0x2FFF => addr - 0x800,
                    _ => addr,
                };

                addr - 0x2000
            }
            Mirroring::FourScreen => addr - 0x2000,
            Mirroring::SingleScreen => {
                let addr = match addr {
                    0x2400..=0x27FF => addr - 0x400,
                    0x2800..=0x2BFF => addr - 0x800,
                    0x2C00..=0x2FFF => addr - 0xC00,
                    _ => addr,
                };

                addr - 0x2000 + (self.mmc7_vram_page as u16 * 0x400)
            }
        }
    }

    fn nt_byte_to_pt_addr(&self, nt_byte: u8, offset: u16) -> (u16, u8, u16) {
        let spr_height = self.ctrl.size / 8;

        let pt = if spr_height == 16 {
            if nt_byte.bit(0) {
                0x1000
            } else {
                0
            }
        } else {
            self.ctrl.spr_pt_addr
        };

        let tile_idx = if spr_height == 16 {
            let nt_byte = nt_byte.bits_abs(1, 7);

            if offset > 7 {
                nt_byte + 1
            } else {
                nt_byte
            }
        } else {
            nt_byte
        };

        let offset = offset % 8;

        (pt, tile_idx, offset)
    }

    fn watch_a12(&self, a12: bool) {
        if self.mmc3.is_none() {
            return;
        }

        self.old_a12.replace(self.a12.get());
        self.a12.replace(a12);

        if !self.old_a12.get() && self.a12.get() && self.a12_filter.get() == 0 {
            self.a12_rising.replace(true);
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let (bank, size, offset) = self.addr_to_bank_and_offset(addr);

        self.chr.chunks_exact_mut(size).nth(bank).unwrap()[offset] = data;
    }

    fn write(&mut self, addr: u16, data: u8) {
        let addr = addr % 0x4000;

        match addr {
            0x0000..=0x1FFF => {
                if self.ram {
                    self.write_chr(addr, data);
                }
            }
            0x2000..=0x3EFF => {
                let addr = self.mirrored_address(addr);

                self.vram[addr as usize] = data;
            }
            0x3F00..=0x3FFF => {
                let addr = (addr % 0x20) as usize;

                let addr = match addr {
                    0x10 | 0x14 | 0x18 | 0x1C => addr - 0x10,
                    _ => addr,
                };

                self.pram[addr] = data;
            }
            _ => unreachable!(),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        let addr = addr % 0x4000;

        match addr {
            0x0000..=0x1FFF => self.read_chr(addr),
            0x2000..=0x3EFF => {
                let addr = self.mirrored_address(addr);

                self.vram[addr as usize]
            }
            0x3F00..=0x3FFF => {
                let addr = (addr % 0x20) as usize;

                let addr = match addr {
                    0x10 | 0x14 | 0x18 | 0x1C => addr - 0x10,
                    _ => addr,
                };

                let data = self.pram[addr];

                let addr = self.v.address() - 0x1000;
                let vram = self.read(addr);
                self.buf.replace(vram);

                if self.mask.greyscale {
                    data & 0x30
                } else {
                    data
                }
            }
            _ => unreachable!(),
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (bank, size, offset) = self.addr_to_bank_and_offset(addr);

        self.chr.chunks_exact(size).nth(bank).unwrap()[offset]
    }

    fn reload_shifters(&mut self) {
        self.pt_shifters[0].push_n(self.pt_lo_latch, 8);
        self.pt_shifters[1].push_n(self.pt_hi_latch, 8);

        let at_low = self.at_lo_latch;
        let at_high = self.at_hi_latch;

        self.at_shifters[0].push_n(0xFF * at_low, 8);
        self.at_shifters[1].push_n(0xFF * at_high, 8);
    }

    fn render(&mut self) {
        let y = self.scanline as usize;
        let x = self.sx as usize;

        let bg_pixel = self.calculate_bg_pixel();
        let spr_pixel = self.calculate_spr_pixel();

        let show_bg = self.mask.show_background;
        let show_spr = self.mask.show_sprites;

        let pixel = if show_bg && show_spr {
            match (bg_pixel, spr_pixel) {
                (0, Some((_, spr_pixel, _))) => spr_pixel,
                (bg_pixel, None) => bg_pixel,
                (bg_pixel, Some((id, spr_pixel, priority))) => {
                    let spr_0_hit = &mut self.status.get_mut().spr_0_hit;

                    if id == 0 && self.spr0_present && !*spr_0_hit && self.sx != 255 {
                        *spr_0_hit = true;
                    }

                    if priority {
                        bg_pixel
                    } else {
                        spr_pixel
                    }
                }
            }
        } else if show_bg && !show_spr {
            bg_pixel
        } else if !show_bg && show_spr {
            spr_pixel.map_or(0, |(_, spr_pixel, _)| spr_pixel)
        } else {
            0
        };

        // Overscan
        if !(8..=231).contains(&y) {
            return;
        }

        let palette_idx = self.pram[pixel as usize].bits_abs(0, 5);

        let emphasis_bits = u8::from(self.mask).bits(5, 7) as usize;

        self.palette.select_palette(emphasis_bits);

        let palette_idx = if self.mask.greyscale {
            palette_idx & 0x30
        } else {
            palette_idx
        };

        let color = self.palette[palette_idx];

        self.draw_pixel(Some(color), x, y);
    }

    fn spr_fetch_pt(&self, pt: u16, nt_byte: u8, offset: u16) -> u8 {
        let tile_idx = nt_byte as u16;
        let addr = pt + tile_idx * 16 + offset;

        self.watch_a12(addr.bit(12));

        self.read(addr)
    }

    fn spr_fetch_pt_low(&self, nt_byte: u8, offset: u16) -> u8 {
        let (pt, tile_idx, offset) = self.nt_byte_to_pt_addr(nt_byte, offset);

        self.spr_fetch_pt(pt, tile_idx, offset)
    }

    fn spr_fetch_pt_high(&self, nt_byte: u8, offset: u16) -> u8 {
        let (pt, tile_idx, offset) = self.nt_byte_to_pt_addr(nt_byte, offset);

        self.spr_fetch_pt(pt, tile_idx, offset + 8)
    }

    fn tick_visible_scanline(&mut self, rendering: bool) {
        match self.dot {
            // NT
            1 | 321 => self.nt_latch = self.fetch_nt_byte(),
            dot @ (2..=256 | 322..=336) if dot % 8 == 1 => {
                self.reload_shifters();

                self.nt_latch = self.fetch_nt_byte();
            }
            // AT
            dot @ (1..=256 | 321..=336) if dot % 8 == 3 => {
                self.at_latch = self.fetch_at_byte();

                let a6 = self.v.address().bit(6) as u8;
                let a1 = self.v.address().bit(1) as u8;

                let at_bits = self.at_latch >> (a6 * 4 + a1 * 2);

                self.at_lo_latch = at_bits.bit(0) as u8;
                self.at_hi_latch = at_bits.bit(1) as u8;
            }
            // PT low
            dot @ (1..=256 | 321..=336) if dot % 8 == 5 => {
                self.pt_lo_latch = self.fetch_pt_low(self.nt_latch);
            }
            // PT high
            dot @ (1..=256 | 321..=336) if dot % 8 == 7 => {
                self.pt_hi_latch = self.fetch_pt_high(self.nt_latch);
            }
            // Inc hori(v)
            dot @ (1..=255 | 328..=336) if rendering && dot % 8 == 0 => {
                self.v.update(self.v.inc_horiz());
            }
            // Inc vert(v)
            // Inc hori(v)
            256 if rendering => {
                self.v.update(self.v.inc_horiz());
                self.v.update(self.v.inc_vert());
            }

            // Garbage NT
            dot @ (257..=320) if dot % 8 == 1 => {
                if dot == 257 {
                    // Hori(v) = hori(t) + garbage NT
                    if rendering {
                        self.reload_shifters();

                        let addr = self.t.address();

                        let horiz_bits = addr.bits_abs(10, 10) | addr.bits_abs(0, 4);

                        let addr = self.v.address();

                        let addr = addr.bits_abs(11, 14) | addr.bits_abs(5, 9) | horiz_bits;

                        self.v.update(addr);
                    } else {
                        self.reload_shifters();
                    }
                    self.spr_active = [false; 8];
                    self.spr0_present = false;
                }

                self.fetch_nt_byte();

                self.cur_spr = if self.spr_idx < self.oam.sprites.len() {
                    let (id, spr, _) = self.oam.sprites[self.spr_idx];

                    if id == 0 {
                        self.spr0_present = true;
                    }

                    Some(spr)
                } else {
                    None
                };
            }
            // Garbage NT
            dot @ (257..=320) if dot % 8 == 3 => {
                self.fetch_nt_byte();

                if let Some(Sprite { x, attrib, .. }) = self.cur_spr {
                    self.spr_latches[self.spr_idx] = attrib;
                    self.spr_counters[self.spr_idx] = x as u16 + 1;
                } else if self.spr_idx < 8 {
                    self.spr_latches[self.spr_idx] = Attributes::new();
                    self.spr_counters[self.spr_idx] = 0;
                }
            }
            // PT low
            dot @ (257..=320) if dot % 8 == 5 => {
                if let Some(Sprite {
                    tile_idx,
                    y,
                    attrib,
                    ..
                }) = self.cur_spr
                {
                    let spr_off = if attrib.flip_vert {
                        let spr_height = (self.ctrl.size / 8) as u16;

                        (spr_height - 1) - (self.scanline - y as u16)
                    } else {
                        self.scanline - y as u16
                    };

                    let pt_low = self.spr_fetch_pt_low(tile_idx, spr_off);

                    let pt_low = if attrib.flip_horiz {
                        pt_low.reverse_bits()
                    } else {
                        pt_low
                    };

                    self.spr_pt_shifters[self.spr_idx][0].push_n(pt_low, 8);
                } else if self.spr_idx < 8 {
                    self.spr_fetch_pt_low(0xFF, 0);
                    self.spr_pt_shifters[self.spr_idx][0].load(0);
                }
            }
            // PT high
            dot @ (257..=320) if dot % 8 == 7 => {
                if let Some(Sprite {
                    tile_idx,
                    y,
                    attrib,
                    ..
                }) = self.cur_spr
                {
                    let spr_height = (self.ctrl.size / 8) as u16;

                    let spr_off = if attrib.flip_vert {
                        (spr_height - 1) - (self.scanline - y as u16)
                    } else {
                        self.scanline - y as u16
                    };

                    let pt_high = self.spr_fetch_pt_high(tile_idx, spr_off);

                    let pt_high = if attrib.flip_horiz {
                        pt_high.reverse_bits()
                    } else {
                        pt_high
                    };

                    self.spr_pt_shifters[self.spr_idx][1].push_n(pt_high, 8);
                } else if self.spr_idx < 8 {
                    self.spr_fetch_pt_high(0xFF, 0);
                    self.spr_pt_shifters[self.spr_idx][1].load(0);
                }

                self.spr_idx += 1;
            }
            // Unused NT
            337 => {
                self.reload_shifters();
                self.fetch_nt_byte();
            }
            339 => {
                self.fetch_nt_byte();
            }
            _ => {}
        }

        if self.dot <= 256 {
            self.spr_counters
                .iter_mut()
                .enumerate()
                .for_each(|(idx, x)| match *x {
                    0 => {}
                    1 => {
                        self.spr_active[idx] = true;
                        self.spr_shift_count[idx] = 0;
                        *x -= 1;
                    }
                    2.. => {
                        *x -= 1;
                    }
                });

            self.render();
            self.sx += 1;

            if rendering && self.dot == 65 {
                self.oam
                    .calculate_sprites_on_scanline(self.scanline as usize + 1, self.ctrl.size);

                self.status.get_mut().spr_overflow |= self.oam.overflow;

                self.spr_idx = 0;
            }
        }

        if self.dot <= 336 {
            self.pt_shifters[0].pop();
            self.pt_shifters[1].pop();
            self.at_shifters[0].pop();
            self.at_shifters[1].pop();
        }
    }

    fn tick_prerender_scanline(&mut self, rendering: bool) {
        match self.dot {
            1 => {
                self.nmi_occurred.replace(false);

                let status = self.status.get_mut();

                status.vblank = false;
                status.spr_0_hit = false;
                status.spr_overflow = false;
            }
            // Vert(v) = vert(t)
            280..=304 if rendering => {
                let addr = self.t.address();

                let vert_bits = addr.bits_abs(11, 14) | addr.bits_abs(5, 9);

                let addr = self.v.address();

                let addr = addr.bits_abs(10, 10) | addr.bits_abs(0, 4) | vert_bits;

                self.v.update(addr);
            }
            339 => {
                if rendering && self.odd {
                    self.dot = 340;
                }
                self.odd = !self.odd;

                self.frame = self.frame.wrapping_add(1);
            }
            _ => {}
        }

        self.tick_visible_scanline(rendering);
    }
}
