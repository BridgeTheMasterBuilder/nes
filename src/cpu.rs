use std::error::Error;

use serde::{Deserialize, Serialize};

use status::*;
use Register::*;

use crate::bus::mapper::{
    Mapper, Mapper0, Mapper1, Mapper2, Mapper3, Mapper4, Mapper7, MapperTrait, MockBus,
};
use crate::cartridge::{Cartridge, MapperType};
use crate::util::crosses_page;
use crate::Config;

use crate::util::bit::Bit;
use status::Flag::*;

mod arithmetic;
mod bitwise;
mod branch;
mod compare;
mod debug;
mod disasm;
mod flags;
mod interrupts;
mod load;
mod stack;
pub mod status;
mod store;
mod transfer;

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Register {
    A,
    X,
    Y,
}

pub enum AddressingMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Implied,
    IndexedIndirect,
    Indirect,
    IndirectIndexed,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

#[derive(Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Cpu {
    pub bus: Mapper,
    pub cyc: usize,
    pub disasm: bool,
    pub nmi_acknowledged: bool,
    pub pc: u16,
    pub p: Status,
    config: Config,
    real_cyc: usize,
    regs: [u8; 3],
    sp: u8,
}

impl Cpu {
    pub fn new(config: &Config, cartridge: Cartridge, clockrate: u32, mock: bool) -> Cpu {
        Cpu {
            config: config.clone(),
            bus: {
                let mapper_type = cartridge.mapper_type;

                if mock {
                    Mapper::MockBus(MockBus::new(cartridge, clockrate))
                } else {
                    match mapper_type {
                        MapperType::Nrom => Mapper::Mapper0(Mapper0::new(cartridge, clockrate)),
                        MapperType::MMC1 => {
                            Mapper::Mapper1(Mapper1::new(cartridge, config, clockrate))
                        }
                        MapperType::Uxrom => Mapper::Mapper2(Mapper2::new(cartridge, clockrate)),
                        MapperType::Cnrom => Mapper::Mapper3(Mapper3::new(cartridge, clockrate)),
                        MapperType::MMC3 => {
                            Mapper::Mapper4(Mapper4::new(cartridge, config, clockrate))
                        }
                        MapperType::Axrom => Mapper::Mapper7(Mapper7::new(cartridge, clockrate)),
                    }
                }
            },
            cyc: 0,
            disasm: false,
            nmi_acknowledged: false,
            pc: 0,
            p: Status::new(),
            real_cyc: 0,
            regs: [0, 0, 0],
            sp: 0,
        }
    }

    pub fn dma(&mut self, page: u8) {
        if self.cyc % 2 == 1 {
            self.tick(1);
        }

        let addr = ((page as u16) << 8) as usize;

        for addr in addr..addr + 0x100 {
            let data = self.bus.read_u8(addr as u16);
            self.tick(1);

            self.bus.ppu().oam.write(data);
            self.tick(1);
        }
    }

    fn ea_zp(&self, addr: u8) -> u16 {
        addr as u16
    }

    fn ea_zp_idx(&self, addr: u8, idx: u8) -> u16 {
        addr.wrapping_add(idx) as u16
    }

    fn ea_abs(&self, addr: u16) -> u16 {
        addr
    }

    fn ea_abs_idx(&self, addr: u16, idx: u8) -> u16 {
        addr.wrapping_add(idx as u16)
    }

    fn ea_ind(&mut self, addr: u16) -> (u16, u16) {
        let l = (addr & 0xFF) as u8;
        let h = ((addr & 0xFF00) >> 8) as u8;
        let adl = ((h as u16) << 8) | (l as u16);
        let adh = ((h as u16) << 8) | (l.wrapping_add(1) as u16);

        (adl, adh)
    }

    fn ea_idx_ind(&mut self, addr: u8, idx: u8) -> u16 {
        let l = self.bus.read_u8(addr.wrapping_add(idx) as u16);
        let h = self
            .bus
            .read_u8(addr.wrapping_add(1).wrapping_add(idx) as u16);
        ((h as u16) << 8) | (l as u16)
    }

    fn ea_ind_idx(&mut self, addr: u8, idx: u8) -> (u16, bool) {
        let l = self.bus.read_u8(addr as u16);
        let h = self.bus.read_u8(addr.wrapping_add(1) as u16);
        let target = ((h as u16) << 8) | (l as u16);
        let (_, cycle_penalty) = l.overflowing_add(idx);

        (target.wrapping_add(idx as u16), cycle_penalty)
    }

    pub fn fetch_decode_and_execute(&mut self) -> Result<(), Box<dyn Error>> {
        if self.disasm {
            self.disasm();
        }

        let opcode = self.read_opcode();

        match opcode {
            // ADC # Oper
            0x69 => {
                let imm = self.read_byte_operand() as u16;

                self.adc(imm, AddressingMode::Immediate)
            }
            // ADC Oper
            0x65 => {
                let addr = self.read_byte_operand() as u16;

                self.adc(addr, AddressingMode::ZeroPage)
            }
            // ADC Oper, X
            0x75 => {
                let addr = self.read_byte_operand() as u16;

                self.adc(addr, AddressingMode::ZeroPageX)
            }
            // ADC Oper
            0x6D => {
                let addr = self.read_word_operand();

                self.adc(addr, AddressingMode::Absolute)
            }
            // ADC Oper, X
            0x7D => {
                let addr = self.read_word_operand();

                self.adc(addr, AddressingMode::AbsoluteX)
            }
            // ADC Oper, Y
            0x79 => {
                let addr = self.read_word_operand();

                self.adc(addr, AddressingMode::AbsoluteY)
            }
            // ADC (Oper, X)
            0x61 => {
                let addr = self.read_byte_operand() as u16;

                self.adc(addr, AddressingMode::IndexedIndirect)
            }
            // ADC (Oper), Y
            0x71 => {
                let addr = self.read_byte_operand() as u16;

                self.adc(addr, AddressingMode::IndirectIndexed)
            }
            // AND # Oper
            0x29 => {
                let imm = self.read_byte_operand() as u16;

                self.and(imm, AddressingMode::Immediate)
            }
            // AND Oper
            0x25 => {
                let addr = self.read_byte_operand() as u16;

                self.and(addr, AddressingMode::ZeroPage)
            }
            // AND Oper, X
            0x35 => {
                let addr = self.read_byte_operand() as u16;

                self.and(addr, AddressingMode::ZeroPageX)
            }
            // AND Oper
            0x2D => {
                let addr = self.read_word_operand();

                self.and(addr, AddressingMode::Absolute)
            }
            // AND Oper, X
            0x3D => {
                let addr = self.read_word_operand();

                self.and(addr, AddressingMode::AbsoluteX)
            }
            // AND Oper, Y
            0x39 => {
                let addr = self.read_word_operand();

                self.and(addr, AddressingMode::AbsoluteY)
            }
            // AND (Oper, X)
            0x21 => {
                let addr = self.read_byte_operand() as u16;

                self.and(addr, AddressingMode::IndexedIndirect)
            }
            // AND (Oper), Y
            0x31 => {
                let addr = self.read_byte_operand() as u16;

                self.and(addr, AddressingMode::IndirectIndexed)
            }
            // ASL A
            0xA => {
                const NO_ARG: u16 = 0;

                self.asl(NO_ARG, AddressingMode::Accumulator)
            }
            // ASL Oper
            0x6 => {
                let addr = self.read_byte_operand() as u16;

                self.asl(addr, AddressingMode::ZeroPage)
            }
            // ASL Oper, X
            0x16 => {
                let addr = self.read_byte_operand() as u16;

                self.asl(addr, AddressingMode::ZeroPageX)
            }
            // ASL Oper
            0xE => {
                let addr = self.read_word_operand();

                self.asl(addr, AddressingMode::Absolute)
            }
            // ASL Oper, X
            0x1E => {
                let addr = self.read_word_operand();

                self.asl(addr, AddressingMode::AbsoluteX)
            }
            // BPL Oper
            0x10 => {
                // let offset = self.read_i8();
                let offset = self.read_byte_operand() as i8;

                self.bpl(offset)
            }
            // BCC Oper
            0x90 => {
                // let offset = self.read_i8();
                let offset = self.read_byte_operand() as i8;

                self.bcc(offset)
            }
            // BCS Oper
            0xB0 => {
                // let offset = self.read_i8();
                let offset = self.read_byte_operand() as i8;

                self.bcs(offset)
            }
            // BEQ Oper
            0xF0 => {
                // let offset = self.read_i8();
                let offset = self.read_byte_operand() as i8;

                self.beq(offset)
            }
            // BIT Oper
            0x24 => {
                let addr = self.read_byte_operand() as u16;

                self.bit(addr, AddressingMode::ZeroPage)
            }
            // BIT Oper
            0x2C => {
                let addr = self.read_word_operand();

                self.bit(addr, AddressingMode::Absolute)
            }
            // BMI Oper
            0x30 => {
                let offset = self.read_byte_operand() as i8;

                self.bmi(offset)
            }
            // BNE Oper
            0xD0 => {
                let offset = self.read_byte_operand() as i8;

                self.bne(offset)
            }
            // BRK
            0x0 => self.brk(),
            // BVC Oper
            0x50 => {
                let offset = self.read_byte_operand() as i8;

                self.bvc(offset)
            }
            // BVS Oper
            0x70 => {
                let offset = self.read_byte_operand() as i8;

                self.bvs(offset)
            }
            // CLC
            0x18 => self.clc(),
            // CLD
            0xD8 => self.cld(),
            // CLI
            0x58 => self.cli(),
            // CLV
            0xB8 => self.clv(),
            // CMP # Oper
            0xC9 => {
                let imm = self.read_byte_operand() as u16;

                self.cmp(imm, AddressingMode::Immediate)
            }
            // CMP Oper
            0xC5 => {
                let addr = self.read_byte_operand() as u16;

                self.cmp(addr, AddressingMode::ZeroPage)
            }
            // CMP Oper, X
            0xD5 => {
                let addr = self.read_byte_operand() as u16;

                self.cmp(addr, AddressingMode::ZeroPageX)
            }
            // CMP Oper
            0xCD => {
                let addr = self.read_word_operand();

                self.cmp(addr, AddressingMode::Absolute)
            }
            // CMP Oper, X
            0xDD => {
                let addr = self.read_word_operand();

                self.cmp(addr, AddressingMode::AbsoluteX)
            }
            // CMP Oper, Y
            0xD9 => {
                let addr = self.read_word_operand();

                self.cmp(addr, AddressingMode::AbsoluteY)
            }
            // CMP (Oper, X)
            0xC1 => {
                let addr = self.read_byte_operand() as u16;

                self.cmp(addr, AddressingMode::IndexedIndirect)
            }
            // CMP (Oper), Y
            0xD1 => {
                let addr = self.read_byte_operand() as u16;

                self.cmp(addr, AddressingMode::IndirectIndexed)
            }
            // CPX # Oper
            0xE0 => {
                let imm = self.read_byte_operand() as u16;

                self.cpx(imm, AddressingMode::Immediate)
            }
            // CPX Oper
            0xE4 => {
                let addr = self.read_byte_operand() as u16;

                self.cpx(addr, AddressingMode::ZeroPage)
            }
            // CPX Oper
            0xEC => {
                let addr = self.read_word_operand();

                self.cpx(addr, AddressingMode::Absolute)
            }
            // CPY # Oper
            0xC0 => {
                let imm = self.read_byte_operand() as u16;

                self.cpy(imm, AddressingMode::Immediate)
            }
            // CPY Oper
            0xC4 => {
                let addr = self.read_byte_operand() as u16;

                self.cpy(addr, AddressingMode::ZeroPage)
            }
            // CPY Oper
            0xCC => {
                let addr = self.read_word_operand();

                self.cpy(addr, AddressingMode::Absolute)
            }
            // DEC Oper
            0xC6 => {
                let addr = self.read_byte_operand() as u16;

                self.dec(addr, AddressingMode::ZeroPage)
            }
            // DEC Oper, X
            0xD6 => {
                let addr = self.read_byte_operand() as u16;

                self.dec(addr, AddressingMode::ZeroPageX)
            }
            // DEC Oper
            0xCE => {
                let addr = self.read_word_operand();

                self.dec(addr, AddressingMode::Absolute)
            }
            // DEC Oper, X
            0xDE => {
                let addr = self.read_word_operand();

                self.dec(addr, AddressingMode::AbsoluteX)
            }
            // DEX
            0xCA => self.dex(),
            // DEY
            0x88 => self.dey(),
            // EOR # Oper
            0x49 => {
                let imm = self.read_byte_operand() as u16;

                self.eor(imm, AddressingMode::Immediate)
            }
            // EOR Oper
            0x45 => {
                let addr = self.read_byte_operand() as u16;

                self.eor(addr, AddressingMode::ZeroPage)
            }
            // EOR Oper, X
            0x55 => {
                let addr = self.read_byte_operand() as u16;

                self.eor(addr, AddressingMode::ZeroPageX)
            }
            // EOR Oper
            0x4D => {
                let addr = self.read_word_operand();

                self.eor(addr, AddressingMode::Absolute)
            }
            // EOR Oper, X
            0x5D => {
                let addr = self.read_word_operand();

                self.eor(addr, AddressingMode::AbsoluteX)
            }
            // EOR Oper, Y
            0x59 => {
                let addr = self.read_word_operand();

                self.eor(addr, AddressingMode::AbsoluteY)
            }
            // EOR (Oper, X)
            0x41 => {
                let addr = self.read_byte_operand() as u16;

                self.eor(addr, AddressingMode::IndexedIndirect)
            }
            // EOR (Oper), Y
            0x51 => {
                let addr = self.read_byte_operand() as u16;

                self.eor(addr, AddressingMode::IndirectIndexed)
            }
            // INC Oper
            0xE6 => {
                let addr = self.read_byte_operand() as u16;

                self.inc(addr, AddressingMode::ZeroPage)
            }
            // INC Oper, X
            0xF6 => {
                let addr = self.read_byte_operand() as u16;

                self.inc(addr, AddressingMode::ZeroPageX)
            }
            // INC Oper
            0xEE => {
                let addr = self.read_word_operand();

                self.inc(addr, AddressingMode::Absolute)
            }
            // INC Oper, X
            0xFE => {
                let addr = self.read_word_operand();

                self.inc(addr, AddressingMode::AbsoluteX)
            }
            // INX
            0xE8 => self.inx(),
            // INY
            0xC8 => self.iny(),
            // JMP Oper
            0x4C => {
                let addr = self.read_word_operand();

                self.jmp(addr, AddressingMode::Absolute)
            }
            // JMP (Oper)
            0x6C => {
                let addr = self.read_word_operand();

                self.jmp(addr, AddressingMode::Indirect)
            }
            // JSR Oper
            0x20 => {
                let addr = self.read_word_operand();

                self.jsr(addr)
            }
            // LDA # Oper
            0xA9 => {
                let imm = self.read_byte_operand() as u16;

                self.lda(imm, AddressingMode::Immediate)
            }
            // LDA Oper
            0xA5 => {
                let imm = self.read_byte_operand() as u16;

                self.lda(imm, AddressingMode::ZeroPage)
            }
            // LDA Oper, X
            0xB5 => {
                let imm = self.read_byte_operand() as u16;

                self.lda(imm, AddressingMode::ZeroPageX)
            }
            // LDA Oper
            0xAD => {
                let addr = self.read_word_operand();

                self.lda(addr, AddressingMode::Absolute)
            }
            // LDA Oper, X
            0xBD => {
                let addr = self.read_word_operand();

                self.lda(addr, AddressingMode::AbsoluteX)
            }
            // LDA Oper, Y
            0xB9 => {
                let addr = self.read_word_operand();

                self.lda(addr, AddressingMode::AbsoluteY)
            }
            // LDA (Oper, X)
            0xA1 => {
                let addr = self.read_byte_operand() as u16;

                self.lda(addr, AddressingMode::IndexedIndirect)
            }
            // LDA (Oper), Y
            0xB1 => {
                let addr = self.read_byte_operand() as u16;

                self.lda(addr, AddressingMode::IndirectIndexed)
            }
            // LDX # Oper
            0xA2 => {
                let imm = self.read_byte_operand() as u16;

                self.ldx(imm, AddressingMode::Immediate)
            }
            // LDX Oper
            0xA6 => {
                let addr = self.read_byte_operand() as u16;

                self.ldx(addr, AddressingMode::ZeroPage)
            }
            // LDX Oper, Y
            0xB6 => {
                let addr = self.read_byte_operand() as u16;

                self.ldx(addr, AddressingMode::ZeroPageY)
            }
            // LDX Oper
            0xAE => {
                let addr = self.read_word_operand();

                self.ldx(addr, AddressingMode::Absolute)
            }
            // LDX Oper, Y
            0xBE => {
                let addr = self.read_word_operand();

                self.ldx(addr, AddressingMode::AbsoluteY)
            }
            // LDY # Oper
            0xA0 => {
                let imm = self.read_byte_operand() as u16;

                self.ldy(imm, AddressingMode::Immediate)
            }
            // LDY Oper
            0xA4 => {
                let addr = self.read_byte_operand() as u16;

                self.ldy(addr, AddressingMode::ZeroPage)
            }
            // LDY Oper, X
            0xB4 => {
                let addr = self.read_byte_operand() as u16;

                self.ldy(addr, AddressingMode::ZeroPageX)
            }
            // LDY Oper
            0xAC => {
                let addr = self.read_word_operand();

                self.ldy(addr, AddressingMode::Absolute)
            }
            // LDY Oper, X
            0xBC => {
                let addr = self.read_word_operand();

                self.ldy(addr, AddressingMode::AbsoluteX)
            }
            // LSR A
            0x4A => {
                const NO_ARG: u16 = 0;

                self.lsr(NO_ARG, AddressingMode::Accumulator)
            }
            // LSR Oper
            0x46 => {
                let addr = self.read_byte_operand() as u16;

                self.lsr(addr, AddressingMode::ZeroPage)
            }
            // LSR Oper, X
            0x56 => {
                let addr = self.read_byte_operand() as u16;

                self.lsr(addr, AddressingMode::ZeroPageX)
            }
            // LSR Oper
            0x4E => {
                let addr = self.read_word_operand();

                self.lsr(addr, AddressingMode::Absolute)
            }
            // LSR Oper, X
            0x5E => {
                let addr = self.read_word_operand();

                self.lsr(addr, AddressingMode::AbsoluteX)
            }
            // NOP
            0xEA => self.nop(),
            // ORA # Oper
            0x9 => {
                let imm = self.read_byte_operand() as u16;

                self.ora(imm, AddressingMode::Immediate)
            }
            // ORA Oper
            0x5 => {
                let addr = self.read_byte_operand() as u16;

                self.ora(addr, AddressingMode::ZeroPage)
            }
            // ORA Oper, X
            0x15 => {
                let addr = self.read_byte_operand() as u16;

                self.ora(addr, AddressingMode::ZeroPageX)
            }
            // ORA Oper
            0xD => {
                let addr = self.read_word_operand();

                self.ora(addr, AddressingMode::Absolute)
            }
            // ORA Oper, X
            0x1D => {
                let addr = self.read_word_operand();

                self.ora(addr, AddressingMode::AbsoluteX)
            }
            // ORA Oper, Y
            0x19 => {
                let addr = self.read_word_operand();

                self.ora(addr, AddressingMode::AbsoluteY)
            }
            // ORA (Oper, X)
            0x1 => {
                let addr = self.read_byte_operand() as u16;

                self.ora(addr, AddressingMode::IndexedIndirect)
            }
            // ORA (Oper), Y
            0x11 => {
                let addr = self.read_byte_operand() as u16;

                self.ora(addr, AddressingMode::IndirectIndexed)
            }
            // PHA
            0x48 => self.pha(),
            // PHP
            0x08 => self.php(),
            // PLA
            0x68 => self.pla(),
            // PLP
            0x28 => self.plp(),
            // ROL A
            0x2A => {
                const NO_ARG: u16 = 0;

                self.rol(NO_ARG, AddressingMode::Accumulator)
            }
            // ROL Oper
            0x26 => {
                let addr = self.read_byte_operand() as u16;

                self.rol(addr, AddressingMode::ZeroPage)
            }
            // ROL Oper, X
            0x36 => {
                let addr = self.read_byte_operand() as u16;

                self.rol(addr, AddressingMode::ZeroPageX)
            }
            // ROL Oper
            0x2E => {
                let addr = self.read_word_operand();

                self.rol(addr, AddressingMode::Absolute)
            }
            // ROL Oper, X
            0x3E => {
                let addr = self.read_word_operand();

                self.rol(addr, AddressingMode::AbsoluteX)
            }
            // ROR A
            0x6A => {
                const NO_ARG: u16 = 0;

                self.ror(NO_ARG, AddressingMode::Accumulator)
            }
            // ROR Oper
            0x66 => {
                let addr = self.read_byte_operand() as u16;

                self.ror(addr, AddressingMode::ZeroPage)
            }
            // ROR Oper, X
            0x76 => {
                let addr = self.read_byte_operand() as u16;

                self.ror(addr, AddressingMode::ZeroPageX)
            }
            // ROR Oper
            0x6E => {
                let addr = self.read_word_operand();

                self.ror(addr, AddressingMode::Absolute)
            }
            // ROR Oper, X
            0x7E => {
                let addr = self.read_word_operand();

                self.ror(addr, AddressingMode::AbsoluteX)
            }
            // RTI
            0x40 => self.rti(),
            // RTS
            0x60 => self.rts(),
            // SBC # Oper
            0xE9 => {
                let imm = self.read_byte_operand() as u16;

                self.sbc(imm, AddressingMode::Immediate)
            }
            // SBC Oper
            0xE5 => {
                let addr = self.read_byte_operand() as u16;

                self.sbc(addr, AddressingMode::ZeroPage)
            }
            // SBC Oper, X
            0xF5 => {
                let addr = self.read_byte_operand() as u16;

                self.sbc(addr, AddressingMode::ZeroPageX)
            }
            // SBC Oper
            0xED => {
                let addr = self.read_word_operand();

                self.sbc(addr, AddressingMode::Absolute)
            }
            // SBC Oper, X
            0xFD => {
                let addr = self.read_word_operand();

                self.sbc(addr, AddressingMode::AbsoluteX)
            }
            // SBC Oper, Y
            0xF9 => {
                let addr = self.read_word_operand();

                self.sbc(addr, AddressingMode::AbsoluteY)
            }
            // SBC (Oper, X)
            0xE1 => {
                let addr = self.read_byte_operand() as u16;

                self.sbc(addr, AddressingMode::IndexedIndirect)
            }
            // SBC (Oper), Y
            0xF1 => {
                let addr = self.read_byte_operand() as u16;

                self.sbc(addr, AddressingMode::IndirectIndexed)
            }
            // SED
            0xF8 => self.sed(),
            // SEC
            0x38 => self.sec(),
            // SEI
            0x78 => self.sei(),
            // STA Oper
            0x85 => {
                let addr = self.read_byte_operand() as u16;

                self.sta(addr, AddressingMode::ZeroPage)
            }
            // STA Oper, X
            0x95 => {
                let addr = self.read_byte_operand() as u16;

                self.sta(addr, AddressingMode::ZeroPageX)
            }
            // STA Oper
            0x8D => {
                let addr = self.read_word_operand();

                self.sta(addr, AddressingMode::Absolute)
            }
            // STA Oper, X
            0x9D => {
                let addr = self.read_word_operand();

                self.sta(addr, AddressingMode::AbsoluteX)
            }
            // STA Oper, Y
            0x99 => {
                let addr = self.read_word_operand();

                self.sta(addr, AddressingMode::AbsoluteY)
            }
            // STA (Oper, X)
            0x81 => {
                let addr = self.read_byte_operand() as u16;

                self.sta(addr, AddressingMode::IndexedIndirect)
            }
            // STA (Oper), Y
            0x91 => {
                let addr = self.read_byte_operand() as u16;

                self.sta(addr, AddressingMode::IndirectIndexed)
            }
            // STX Oper
            0x86 => {
                let addr = self.read_byte_operand() as u16;

                self.stx(addr, AddressingMode::ZeroPage)
            }
            // STX Oper, Y
            0x96 => {
                let addr = self.read_byte_operand() as u16;

                self.stx(addr, AddressingMode::ZeroPageY)
            }
            // STX Oper
            0x8E => {
                let addr = self.read_word_operand();

                self.stx(addr, AddressingMode::Absolute)
            }
            // STY Oper
            0x84 => {
                let offset = self.read_byte_operand() as u16;

                self.sty(offset, AddressingMode::ZeroPage)
            }
            // STY Oper, X
            0x94 => {
                let offset = self.read_byte_operand() as u16;

                self.sty(offset, AddressingMode::ZeroPageX)
            }
            // STY Oper
            0x8C => {
                let addr = self.read_word_operand();

                self.sty(addr, AddressingMode::Absolute)
            }
            // TAX
            0xAA => self.tax(),
            // TAY
            0xA8 => self.tay(),
            // TYA
            0x98 => self.tya(),
            // TSX
            0xBA => self.tsx(),
            // TXA
            0x8A => self.txa(),
            // TXS
            0x9A => self.txs(),
            _ => return Err(format!("Unimplemented instruction 0x{opcode:x?}").into()),
        };

        Ok(())
    }

    pub fn handle_nmi(&mut self) {
        self.tick(7 + 7);

        self.push_u16(self.pc);

        let flags = self.flags();
        let flags = flags | 0b00100000;
        let flags = flags & !0b00010000;

        self.push(flags);
        self.p[InterruptDisable as usize] = true;

        self.pc = self.bus.read_u16(0xFFFA);
    }

    pub fn handle_irq(&mut self) {
        if self.p[InterruptDisable as usize] {
            return;
        }

        self.tick(7);
        self.push_u16(self.pc);

        let flags = self.flags();
        let flags = flags | 0b00100000;
        let flags = flags & !0b00010000;

        self.push(flags);
        self.p[InterruptDisable as usize] = true;

        self.pc = self.bus.read_u16(0xFFFE);
    }

    fn nop(&mut self) {
        self.tick(2);
    }

    fn pull(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);

        let sp = self.sp as u16;

        self.bus.read_u8(0x100 + sp)
    }

    fn pull_u16(&mut self) -> u16 {
        let low = self.pull() as u16;
        let high = self.pull() as u16;

        (high << 8) | low
    }

    fn push(&mut self, data: u8) {
        let sp = self.sp as u16;

        self.bus.write_u8(0x100 + sp, data);

        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_u16(&mut self, data: u16) {
        let low = data.bits_abs(0, 7) as u8;
        let high = data.bits(8, 15) as u8;

        self.push(high);
        self.push(low);
    }

    fn read_zp(&mut self, addr: u8) -> u8 {
        self.bus.read_u8(self.ea_zp(addr))
    }

    fn read_zp_idx(&mut self, addr: u8, idx: u8) -> u8 {
        self.bus.read_u8(self.ea_zp_idx(addr, idx))
    }

    fn read_abs(&mut self, addr: u16) -> u8 {
        self.bus.read_u8(self.ea_abs(addr))
    }

    fn read_abs_idx(&mut self, addr: u16, idx: u8) -> u8 {
        self.bus.read_u8(self.ea_abs_idx(addr, idx))
    }

    fn read_ind(&mut self, addr: u16) -> u16 {
        let (adl, adh) = self.ea_ind(addr);

        let l = self.bus.read_u8(adl);
        let h = self.bus.read_u8(adh);

        ((h as u16) << 8) | (l as u16)
    }

    fn read_idx_ind(&mut self, addr: u8, idx: u8) -> u8 {
        let target = self.ea_idx_ind(addr, idx);

        self.bus.read_u8(target)
    }

    fn read_ind_idx(&mut self, addr: u8, idx: u8) -> u8 {
        let (target, cycle_penalty) = self.ea_ind_idx(addr, idx);

        if cycle_penalty {
            self.tick(1);
        }

        self.bus.read_u8(target)
    }

    fn read_opcode(&mut self) -> u8 {
        self.read_u8()
    }

    fn read_byte_operand(&mut self) -> u8 {
        self.read_u8()
    }

    fn read_word_operand(&mut self) -> u16 {
        self.read_u16()
    }

    fn read_u8(&mut self) -> u8 {
        let pc = self.pc;

        let byte = self.bus.read_u8(pc);

        self.pc = self.pc.wrapping_add(1);

        byte
    }

    fn read_u16(&mut self) -> u16 {
        let pc = self.pc;

        let word = self.bus.read_u16(pc);

        self.pc = self.pc.wrapping_add(2);

        word
    }

    pub fn reset(&mut self) {
        self.tick(7);

        self.p[InterruptDisable as usize] = true;
        self.sp = self.sp.wrapping_sub(3);
        self.pc = self.bus.read_u16(0xFFFC);
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.bus.ppu().tick();
            self.bus.ppu().tick();
            self.bus.ppu().tick();
            self.bus.apu().tick();

            if self.bus.apu().dmc.buffer_should_be_filled() {
                let sample_addr = self.bus.apu().dmc.addr;
                let data = self.bus.read_u8(sample_addr);

                self.bus.apu().dmc.fill_buffer(data);
            }
        }

        self.cyc += cycles;
        self.real_cyc += cycles;
    }
}
