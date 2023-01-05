#![allow(clippy::large_enum_variant)]

use std::any::Any;
use std::ops::Range;

use egui::Ui;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

pub use mapper0::Mapper0;
pub use mapper1::Mapper1;
pub use mapper2::Mapper2;
pub use mapper3::Mapper3;
pub use mapper4::Mapper4;
pub use mapper7::Mapper7;
pub use mockbus::MockBus;

use crate::apu::Apu;
use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::controller::Controller;
use crate::gui::DebugInfo;
use crate::ppu::Ppu;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper3;
mod mapper4;
mod mapper7;
mod mockbus;

pub type Setting = (usize, Range<i32>);

#[enum_dispatch]
pub trait MapperTrait: DebugInfo {
    fn apu(&mut self) -> &mut Apu;

    fn as_any(&self) -> &dyn Any;

    fn bus(&mut self) -> &mut Bus;

    fn controller(&mut self) -> &mut Controller;

    fn memory(&self) -> &[u8];

    fn ppu(&mut self) -> &mut Ppu;

    fn print_debug_info(&self, ui: &mut Ui) {
        self.print(ui);
    }

    fn read_u8(&self, addr: u16) -> u8;

    fn read_u16(&self, addr: u16) -> u16 {
        let l = self.read_u8(addr);

        let h = self.read_u8(addr.wrapping_add(1));

        ((h as u16) << 8) | (l as u16)
    }

    fn save_data(&mut self);

    fn write_u8(&mut self, addr: u16, data: u8);

    fn write_u16(&mut self, addr: u16, data: u16) {
        let h = ((data & 0xFF00) >> 8) as u8;

        let l = (data & 0xFF) as u8;

        self.write_u8(addr, l);

        self.write_u8(addr.wrapping_add(1), h);
    }
}

#[enum_dispatch(MapperTrait)]
#[derive(Serialize, Deserialize, Clone)]
pub enum Mapper {
    Mapper0(Mapper0),
    Mapper1(Mapper1),
    Mapper2(Mapper2),
    Mapper3(Mapper3),
    Mapper4(Mapper4),
    Mapper7(Mapper7),
    MockBus(MockBus),
}

impl DebugInfo for Mapper {
    fn print(&self, _ui: &mut Ui) {}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BankSettings(Vec<Setting>);

impl BankSettings {
    pub fn new(settings: Vec<Setting>) -> Self {
        Self(settings)
    }

    pub fn set_bank(&mut self, bank: usize, addresses: Range<i32>) {
        let (selected, _) = self
            .0
            .iter_mut()
            .find(|(_, addr)| addr.start == addresses.start)
            .unwrap();

        *selected = bank;
    }

    pub fn replace(&mut self, settings: BankSettings) {
        *self = settings;
    }

    pub fn iter(&self) -> impl Iterator<Item = &(usize, Range<i32>)> + '_ {
        self.0.iter()
    }
}
