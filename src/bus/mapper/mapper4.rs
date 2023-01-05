use egui::Ui;
use serde_big_array::BigArray;

use crate::bus::Bus;
use crate::cartridge::{MapperType, Mirroring};
use crate::controller::Controller;
use crate::gui::DebugInfo;
use crate::util::bit::Bit;
use crate::util::{load_ram, save_ram, Config};

use super::*;

#[derive(Serialize, Deserialize, Clone)]
enum PrgRomMode {
    SwappableFixed,
    FixedSwappable,
}

#[derive(Serialize, Deserialize, Clone)]
enum ChrRomMode {
    TwoFour,
    FourTwo,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapper4 {
    bank_settings: BankSettings,
    banks: [usize; 10],
    bus: Bus,
    chr_rom_mode: ChrRomMode,
    filename: String,
    num_banks: usize,
    prg_rom: Vec<u8>,
    prg_rom_mode: PrgRomMode,
    ram_enabled: bool,
    ram_protected: bool,
    selected_bank_register: usize,
    #[serde(with = "BigArray")]
    sram: [u8; 0x2000],
}

impl Mapper4 {
    pub fn new(mut cartridge: Cartridge, config: &Config, clockrate: u32) -> Self {
        let prg_rom = cartridge.prg_rom.take().unwrap();

        let num_banks = prg_rom.len() / 0x2000;

        Self {
            prg_rom,
            bus: Bus::new(cartridge, MapperType::MMC3, clockrate),
            prg_rom_mode: PrgRomMode::SwappableFixed,
            banks: [0, 0, 0, 0, 0, 0, 0, 0, num_banks - 2, num_banks - 1],
            bank_settings: BankSettings::new(vec![
                (0, (0x8000..0xA000)),
                (0, (0xA000..0xC000)),
                (0, (0xC000..0xE000)),
                (9, (0xE000..0x10000)),
            ]),
            chr_rom_mode: ChrRomMode::TwoFour,
            sram: load_ram(&config.filename),
            selected_bank_register: 0,
            ram_enabled: true,
            ram_protected: false,
            filename: config.filename.clone(),
            num_banks,
        }
    }

    fn read_prg(&self, addr: u16) -> u8 {
        let addr = addr as i32;

        let (bank, addresses) = self
            .bank_settings
            .iter()
            .find(|(_, addresses)| addresses.contains(&addr))
            .unwrap();

        let start = addresses.start as usize;
        let end = addresses.end as usize;
        let bank_size = end - start;

        let offset = addr as usize % 0x2000;

        let bank = self.banks[*bank];

        self.prg_rom.chunks_exact(bank_size).nth(bank).unwrap()[offset]
    }
}

impl MapperTrait for Mapper4 {
    fn apu(&mut self) -> &mut Apu {
        &mut self.bus.apu
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn bus(&mut self) -> &mut Bus {
        &mut self.bus
    }

    fn controller(&mut self) -> &mut Controller {
        &mut self.bus.controller1
    }

    fn memory(&self) -> &[u8] {
        &self.bus.ram
    }

    fn ppu(&mut self) -> &mut Ppu {
        &mut self.bus.ppu
    }

    fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => {
                if self.ram_enabled {
                    let offset = addr as usize % 0x6000;

                    self.sram[offset]
                } else {
                    0
                }
            }
            0x8000..=0xFFFF => self.read_prg(addr),
            _ => self.bus.read_u8(addr),
        }
    }

    fn save_data(&mut self) {
        save_ram(&self.filename, self.sram.as_slice());
    }

    fn write_u8(&mut self, addr: u16, data: u8) {
        let mmc3 = self.ppu().mmc3.as_mut().unwrap();

        match addr {
            0x6000..=0x7FFF => {
                // TODO MMC6
                if self.ram_enabled && !self.ram_protected {
                    let offset = addr as usize % 0x6000;

                    self.sram[offset] = data;
                }
            }
            0x8000..=0x9FFE if addr % 2 == 0 => {
                // TODO MMC6
                let bank_register = data.bits_abs(0, 2);
                let prg_rom_mode = data.bit(6);
                let chr_rom_mode = data.bit(7);

                mmc3.selected_bank_register = bank_register as usize;

                self.prg_rom_mode = if prg_rom_mode {
                    self.bank_settings.replace(BankSettings::new(vec![
                        (8, 0x8000..0xA000),
                        (7, 0xA000..0xC000),
                        (6, 0xC000..0xE000),
                        (9, 0xE000..0x10000),
                    ]));
                    PrgRomMode::FixedSwappable
                } else {
                    self.bank_settings.replace(BankSettings::new(vec![
                        (6, 0x8000..0xA000),
                        (7, 0xA000..0xC000),
                        (8, 0xC000..0xE000),
                        (9, 0xE000..0x10000),
                    ]));
                    PrgRomMode::SwappableFixed
                };

                self.chr_rom_mode = if chr_rom_mode {
                    self.ppu().bank_settings.replace(BankSettings::new(vec![
                        (2, 0x0000..0x0400),
                        (3, 0x0400..0x0800),
                        (4, 0x0800..0x0C00),
                        (5, 0x0C00..0x1000),
                        (0, 0x1000..0x1800),
                        (1, 0x1800..0x2000),
                    ]));
                    ChrRomMode::FourTwo
                } else {
                    self.ppu().bank_settings.replace(BankSettings::new(vec![
                        (0, 0x0000..0x0800),
                        (1, 0x0800..0x1000),
                        (2, 0x1000..0x1400),
                        (3, 0x1400..0x1800),
                        (4, 0x1800..0x1C00),
                        (5, 0x1C00..0x2000),
                    ]));
                    ChrRomMode::TwoFour
                };
            }
            0x8001..=0x9FFF => {
                match mmc3.selected_bank_register {
                    0 | 1 => mmc3.banks[mmc3.selected_bank_register] = data.bits_abs(1, 7) as usize,
                    6 | 7 => self.banks[mmc3.selected_bank_register] = data.bits_abs(0, 5) as usize,
                    _ => mmc3.banks[mmc3.selected_bank_register] = data as usize,
                };
            }
            0xA000..=0xBFFE if addr % 2 == 0 => {
                if !matches!(self.ppu().mirroring, Mirroring::FourScreen) {
                    let mirroring = if data.bit(0) {
                        Mirroring::Horizontal
                    } else {
                        Mirroring::Vertical
                    };

                    self.ppu().set_mirroring_mode(mirroring);
                }
            }
            0xA001..=0xBFFF => {
                // TODO MMC6
                let prg_ram_enable = data.bit(7);
                let write_protection = data.bit(6);

                self.ram_enabled = prg_ram_enable;
                self.ram_protected = write_protection;
            }
            0xC000..=0xDFFE if addr % 2 == 0 => {
                // println!("Setting IRQ counter to {data}");
                mmc3.irq.reload = data;
            }
            0xC001..=0xDFFF => {
                // println!("Reloading IRQ counter");
                mmc3.irq.counter = 0;
            }
            0xE000..=0xFFFE if addr % 2 == 0 => {
                mmc3.irq.occurred = false;
                mmc3.irq.enabled = false
            }
            0xE001..=0xFFFF => mmc3.irq.enabled = true,
            _ => self.bus.write_u8(addr, data),
        }
    }
}

impl DebugInfo for Mapper4 {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("PRG ROM size: {}K", self.prg_rom.len() / 1024));
                ui.vertical(|ui| {
                    for (bank, addresses) in self.bank_settings.iter() {
                        ui.label(format!(
                            "${:04X}-${:04X}: Bank {:X}",
                            addresses.start, addresses.end, self.banks[*bank]
                        ));
                    }
                })
            });

            ui.horizontal(|ui| {
                ui.label(format!(
                    "CHR size: {}K ({})",
                    self.bus.ppu.chr.len() / 1024,
                    if self.ram_enabled { "RAM" } else { "ROM" }
                ));
                ui.vertical(|ui| {
                    for (bank, addresses) in self.bus.ppu.bank_settings.iter() {
                        ui.label(format!(
                            "${:04X}-${:04X}: Bank {:X}",
                            addresses.start,
                            addresses.end,
                            self.bus.ppu.mmc3.as_ref().unwrap().banks[*bank]
                        ));
                    }
                })
            });
            ui.label(format!("Mirroring: {:?}", self.bus.ppu.mirroring));
        });
    }
}
