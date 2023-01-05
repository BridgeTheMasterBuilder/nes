use egui::Ui;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::bus::Bus;
use crate::cartridge::MapperType;
use crate::controller::Controller;
use crate::gui::DebugInfo;

use super::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct MockBus {
    bus: Bus,
    #[serde(with = "BigArray")]
    ram: [u8; 0x10000],
}

impl MockBus {
    pub fn new(cartridge: Cartridge, clockrate: u32) -> Self {
        Self {
            bus: Bus::new(cartridge, MapperType::Nrom, clockrate),
            ram: [0; 0x10000],
        }
    }
}

impl MapperTrait for MockBus {
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
        &self.ram
    }

    fn ppu(&mut self) -> &mut Ppu {
        &mut self.bus.ppu
    }

    fn read_u8(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn save_data(&mut self) {}

    fn write_u8(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}

impl DebugInfo for MockBus {
    fn print(&self, _ui: &mut Ui) {}
}
