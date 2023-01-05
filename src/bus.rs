use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::apu::Apu;
use crate::cartridge::{Cartridge, MapperType};
use crate::controller::Controller;
use crate::ppu::Ppu;

pub mod mapper;

#[derive(Serialize, Deserialize, Clone)]
pub struct Bus {
    pub dma_interrupt: Option<u8>,
    apu: Apu,
    controller1: Controller,
    ppu: Ppu,
    prg_rom: Vec<u8>,
    #[serde(with = "BigArray")]
    ram: [u8; 0x800],
}

impl Bus {
    pub fn new(mut cartridge: Cartridge, mapper_type: MapperType, clockrate: u32) -> Self {
        let prg_rom = cartridge.prg_rom.take().unwrap_or_default();

        assert!(
            prg_rom.is_empty() || prg_rom.len() == 0x4000 || prg_rom.len() == 0x8000,
            "NROM PRGROM size is not 16K or 32K"
        );

        Self {
            dma_interrupt: None,
            apu: Apu::new(clockrate),
            controller1: Controller::new(),
            ppu: Ppu::new(cartridge, mapper_type),
            prg_rom,
            ram: [0; 0x800],
        }
    }

    fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let addr = addr % 0x800;

                self.ram[addr as usize]
            }
            0x2000..=0x3FFF => {
                let addr = addr % 8;

                self.ppu.read_reg(addr as u8)
            }
            0x4000..=0x4015 => self.apu.read(addr),
            0x4016 => self.controller1.read(),
            // TODO Second controller
            0x4017 => 0,
            0x4018..=0x5FFF => 0,
            0x6000..=0x7FFF => 0,
            0x8000..=0xFFFF => {
                let addr = addr % 0x8000;

                match self.prg_rom.len() {
                    0x4000 => {
                        let offset = addr % 0x4000;

                        self.prg_rom[offset as usize]
                    }
                    0x8000 => self.prg_rom[addr as usize],
                    _ => unreachable!(),
                }
            }
        }
    }

    fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let addr = addr % 0x800;

                self.ram[addr as usize] = data;
            }
            0x2000..=0x3FFF => {
                let addr = addr % 8;

                self.ppu.write_reg(addr as u8, data)
            }
            0x4014 => {
                self.dma_interrupt.replace(data);
            }
            0x4000..=0x4015 => self.apu.write(addr, data),
            0x4016 => self.controller1.write(data),
            0x4017 => self.apu.write(addr, data),
            0x4018..=0x5FFF => {}
            0x6000..=0x7FFF => {}
            0x8000..=0xFFFF => {}
        }
    }
}
