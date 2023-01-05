use crate::bus::Bus;
use crate::cartridge::MapperType;
use crate::controller::Controller;
use crate::util::bit::Bit;

use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapper7 {
    cur_bank: usize,
    num_banks: usize,
    prg_rom: Vec<u8>,
    pub bus: Bus,
    vram_page: u8,
}

impl Mapper7 {
    pub fn new(mut cartridge: Cartridge, clockrate: u32) -> Self {
        let prg_rom = cartridge.prg_rom.take().unwrap();

        let num_banks = prg_rom.len() / 0x4000;

        Self {
            bus: Bus::new(cartridge, MapperType::Axrom, clockrate),
            cur_bank: 0,
            num_banks,
            prg_rom,
            vram_page: 0,
        }
    }
}

impl MapperTrait for Mapper7 {
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
            0x8000..=0xFFFF => {
                let addr = addr as usize % 0x8000;

                self.prg_rom
                    .chunks_exact(0x8000)
                    .nth(self.cur_bank)
                    .unwrap()[addr]
            }
            _ => self.bus.read_u8(addr),
        }
    }

    fn save_data(&mut self) {}

    fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {
                let bank = data.bits_abs(0, 2) as usize;
                let vram_page = data.bit(4) as u8;

                self.cur_bank = bank;
                self.bus.ppu.mmc7_vram_page = vram_page;
            }
            _ => self.bus.write_u8(addr, data),
        }
    }
}

impl DebugInfo for Mapper7 {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("PRG ROM size: {}K", self.prg_rom.len() / 1024));
                ui.vertical(|ui| {
                    ui.label(format!("$8000-$FFFF: Bank {}", self.cur_bank));
                })
            });

            ui.label(format!(
                "CHR size: {}K ({})",
                self.bus.ppu.chr.len() / 1024,
                if self.bus.ppu.ram { "RAM" } else { "ROM" }
            ));
            ui.label(format!(
                "Mirroring: Single-screen (VRAM page #{})",
                self.bus.ppu.mmc7_vram_page
            ));
        });
    }
}
