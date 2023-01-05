use egui::Ui;
use serde_big_array::BigArray;

use crate::bus::Bus;
use crate::cartridge::{MapperType, Mirroring};
use crate::controller::Controller;
use crate::gui::DebugInfo;
use crate::util::bit::Bit;
use crate::util::shift_reg::ShiftRegister;
use crate::util::{load_ram, save_ram, Config};

use super::*;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
enum PrgRomMode {
    Switch32kB,
    FixFirstBank,
    FixLastBank,
}

#[derive(Serialize, Deserialize, Clone)]
enum ChrRomMode {
    Switch8kB,
    SwitchTwo4kB,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapper1 {
    bank_settings: BankSettings,
    bus: Bus,
    chr_rom_mode: ChrRomMode,
    cur_bank: usize,
    filename: String,
    num_banks: usize,
    prg_rom: Vec<u8>,
    prg_rom_mode: PrgRomMode,
    ram_enabled: bool,
    shift_reg: ShiftRegister<u8, 5>,
    #[serde(with = "BigArray")]
    sram: [u8; 0x2000],
    writes: usize,
}

impl Mapper1 {
    pub fn new(mut cartridge: Cartridge, config: &Config, clockrate: u32) -> Self {
        let prg_rom = cartridge.prg_rom.take().unwrap();

        let num_banks = prg_rom.len() / 0x4000;

        Self {
            bank_settings: BankSettings::new(vec![
                (0, (0x8000..0xC000)),
                (num_banks - 1, (0xC000..0x10000)),
            ]),
            bus: Bus::new(cartridge, MapperType::MMC1, clockrate),
            chr_rom_mode: ChrRomMode::Switch8kB,
            cur_bank: 0,
            sram: load_ram(&config.filename),
            filename: config.filename.clone(),
            num_banks,
            prg_rom,
            prg_rom_mode: PrgRomMode::FixLastBank,
            ram_enabled: true,
            shift_reg: ShiftRegister::new(),
            writes: 0,
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

        let offset = if self.prg_rom_mode == PrgRomMode::Switch32kB {
            addr % 0x8000
        } else {
            addr % 0x4000
        } as usize;

        self.prg_rom.chunks_exact(bank_size).nth(*bank).unwrap()[offset]
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => {
                let mirroring = data.bits_abs(0, 1);

                let prg_rom_mode = data.bits(2, 3);

                let chr_rom_mode = data.bit(4);

                let mirroring = match mirroring {
                    0 => Mirroring::OneScreenLowerBank,
                    1 => Mirroring::OneScreenUpperBank,
                    2 => Mirroring::Vertical,
                    3 => Mirroring::Horizontal,
                    _ => unreachable!(),
                };

                self.bus.ppu.set_mirroring_mode(mirroring);

                self.prg_rom_mode = match prg_rom_mode {
                    0 | 1 => {
                        self.bank_settings
                            .replace(BankSettings::new(vec![(self.cur_bank, (0x8000..0x10000))]));
                        PrgRomMode::Switch32kB
                    }
                    2 => {
                        self.bank_settings.replace(BankSettings::new(vec![
                            (0, (0x8000..0xC000)),
                            (self.cur_bank, (0xC000..0x10000)),
                        ]));
                        PrgRomMode::FixFirstBank
                    }
                    3 => {
                        self.bank_settings.replace(BankSettings::new(vec![
                            (self.cur_bank, (0x8000..0xC000)),
                            (self.num_banks - 1, (0xC000..0x10000)),
                        ]));
                        PrgRomMode::FixLastBank
                    }
                    _ => unreachable!(),
                };

                self.chr_rom_mode = if chr_rom_mode {
                    self.ppu().bank_settings.replace(BankSettings::new(vec![
                        (0, (0..0x1000)),
                        (1, (0x1000..0x2000)),
                    ]));
                    ChrRomMode::SwitchTwo4kB
                } else {
                    self.ppu()
                        .bank_settings
                        .replace(BankSettings::new(vec![(0, 0x0000..0x2000)]));
                    ChrRomMode::Switch8kB
                };
            }
            0xA000..=0xBFFF => {
                let prg_ram_disable = data.bit(4);

                if !prg_ram_disable {
                    self.ram_enabled = true;
                };

                if matches!(self.chr_rom_mode, ChrRomMode::SwitchTwo4kB) {
                    // TODO non-SNROM
                    let data = data.bits_abs(0, 4) as usize;

                    self.ppu().bank_settings.set_bank(data, 0x0000..0x1000);
                } else {
                    let data = data.bits_abs(1, 4) as usize;

                    self.ppu().bank_settings.set_bank(data, 0x0000..0x2000);
                }
            }
            0xC000..=0xDFFF => {
                if matches!(self.chr_rom_mode, ChrRomMode::SwitchTwo4kB) {
                    // TODO non-SNROM
                    let data = data.bits_abs(0, 4) as usize;

                    self.ppu().bank_settings.set_bank(data, 0x1000..0x2000);
                }
            }
            0xE000..=0xFFFF => {
                // TODO MMC1A bypass
                let prg_ram_disable = data.bit(4);

                self.ram_enabled = !prg_ram_disable;

                match self.prg_rom_mode {
                    PrgRomMode::Switch32kB => {
                        let data = data.bits(1, 3) as usize;

                        self.cur_bank = data;
                        self.bank_settings.set_bank(data, 0x8000..0x10000);
                    }
                    PrgRomMode::FixFirstBank => {
                        let data = data.bits(0, 3) as usize;

                        self.cur_bank = data;
                        self.bank_settings.set_bank(data, 0xC000..0x10000);
                    }
                    PrgRomMode::FixLastBank => {
                        let data = data.bits(0, 3) as usize;

                        self.cur_bank = data;
                        self.bank_settings.set_bank(data, 0x8000..0xC000);
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl MapperTrait for Mapper1 {
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
        match addr {
            0x6000..=0x7FFF => {
                if self.ram_enabled {
                    let offset = addr as usize % 0x6000;

                    self.sram[offset] = data;
                }
            }
            // TODO consecutive-cycle writes
            0x8000..=0xFFFF => {
                let bit7 = data.bit(7);

                if bit7 {
                    self.prg_rom_mode = PrgRomMode::FixLastBank;
                    self.bank_settings.replace(BankSettings::new(vec![
                        (self.cur_bank, (0x8000..0xC000)),
                        (self.num_banks - 1, (0xC000..0x10000)),
                    ]));

                    self.shift_reg.clear();
                    self.writes = 0;
                } else if self.writes < 4 {
                    let data_bit = data.bit(0) as u8;

                    self.shift_reg.push(data_bit);
                    self.writes += 1;
                } else if self.writes == 4 {
                    let data_bit = data.bit(0) as u8;

                    self.shift_reg.push(data_bit);

                    let data = self.shift_reg.take();
                    self.writes = 0;

                    self.write(addr, data);
                }
            }
            _ => self.bus.write_u8(addr, data),
        }
    }
}

impl DebugInfo for Mapper1 {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("PRG ROM size: {}K", self.prg_rom.len() / 1024));
                ui.vertical(|ui| {
                    for (bank, addresses) in self.bank_settings.iter() {
                        ui.label(format!(
                            "${:04X}-${:04X}: Bank {bank}",
                            addresses.start, addresses.end
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
                            "${:04X}-${:04X}: Bank {bank}",
                            addresses.start, addresses.end
                        ));
                    }
                })
            });
            ui.label(format!("Mirroring: {:?}", self.bus.ppu.mirroring));
        });
    }
}
