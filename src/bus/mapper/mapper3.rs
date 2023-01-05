use egui::Ui;

use crate::bus::Bus;
use crate::cartridge::MapperType;
use crate::controller::Controller;
use crate::gui::DebugInfo;
use crate::util::bit::Bit;

use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Mapper3 {
    bus: Bus,
}

impl Mapper3 {
    pub fn new(cartridge: Cartridge, clockrate: u32) -> Self {
        Self {
            bus: Bus::new(cartridge, MapperType::Cnrom, clockrate),
        }
    }
}

impl MapperTrait for Mapper3 {
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
        self.bus.read_u8(addr)
    }

    fn save_data(&mut self) {}

    fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0xFFFF => {
                let bank = data.bits(0, 1) as usize;

                self.bus.ppu.bank_settings.set_bank(bank, 0x0000..0x2000);
            }
            _ => self.bus.write_u8(addr, data),
        }
    }
}

impl DebugInfo for Mapper3 {
    fn print(&self, ui: &mut Ui) {
        let chr_bank_settings = self.bus.ppu.bank_settings.clone();

        ui.vertical(|ui| {
            ui.label(format!("PRG ROM size: {}K", self.bus.prg_rom.len() / 1024));
            ui.horizontal(|ui| {
                ui.label(format!(
                    "CHR size: {}K ({})",
                    self.bus.ppu.chr.len() / 1024,
                    if self.bus.ppu.ram { "RAM" } else { "ROM" }
                ));
                ui.vertical(|ui| {
                    for (bank, addresses) in chr_bank_settings.iter() {
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
