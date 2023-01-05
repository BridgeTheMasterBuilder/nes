use crate::bus::Bus;
use crate::cartridge::MapperType;
use crate::controller::Controller;

use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapper2 {
    bank_settings: BankSettings,
    bus: Bus,
    cur_bank: usize,
    num_banks: usize,
    prg_rom: Vec<u8>,
}

impl Mapper2 {
    pub fn new(mut cartridge: Cartridge, clockrate: u32) -> Self {
        let prg_rom = cartridge.prg_rom.take().unwrap();

        let num_banks = prg_rom.len() / 0x4000;

        Self {
            bank_settings: BankSettings::new(vec![
                (0, (0x8000..0xC000)),
                (num_banks - 1, (0xC000..0x10000)),
            ]),
            bus: Bus::new(cartridge, MapperType::Uxrom, clockrate),
            cur_bank: 0,
            num_banks,
            prg_rom,
        }
    }
}

impl MapperTrait for Mapper2 {
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
            0x8000..=0xBFFF => {
                let offset = addr as usize % 0x8000;

                self.prg_rom
                    .chunks_exact(0x4000)
                    .nth(self.cur_bank)
                    .unwrap()[offset]
            }
            0xC000..=0xFFFF => {
                let offset = addr as usize % 0x4000;

                let num_banks = self.prg_rom.len() / 0x4000;

                self.prg_rom
                    .chunks_exact(0x4000)
                    .nth(num_banks - 1)
                    .unwrap()[offset]
            }
            _ => self.bus.read_u8(addr),
        }
    }

    fn save_data(&mut self) {}

    fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {
                // TODO UOROM
                let bank = data as usize & 0b111;

                self.cur_bank = bank;
                self.bank_settings.set_bank(bank, 0x8000..0xC000);
            }
            _ => self.bus.write_u8(addr, data),
        }
    }
}

impl DebugInfo for Mapper2 {
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

            ui.label(format!(
                "CHR size: {}K ({})",
                self.bus.ppu.chr.len() / 1024,
                if self.bus.ppu.ram { "RAM" } else { "ROM" }
            ));
            ui.label(format!("Mirroring: {:?}", self.bus.ppu.mirroring));
        });
    }
}
