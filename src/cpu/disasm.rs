use crate::bus::mapper::MapperTrait;
use crate::cpu::{AddressingMode, Cpu};
use crate::util::bit::Bit;

use super::Register::*;

const OPCODE_MAP: [&str; 256] = [
    "BRK", "ORA", "", "", "", "ORA", "ASL", "", "PHP", "ORA", "ASL", "", "", "ORA", "ASL", "",
    "BPL", "ORA", "", "", "", "ORA", "ASL", "", "CLC", "ORA", "", "", "", "ORA", "ASL", "", "JSR",
    "AND", "", "", "BIT", "AND", "ROL", "", "PLP", "AND", "ROL", "", "BIT", "AND", "ROL", "",
    "BMI", "AND", "", "", "", "AND", "ROL", "", "SEC", "AND", "", "", "", "AND", "ROL", "", "RTI",
    "EOR", "", "", "", "EOR", "LSR", "", "PHA", "EOR", "LSR", "", "JMP", "EOR", "LSR", "", "BVC",
    "EOR", "", "", "", "EOR", "LSR", "", "CLI", "EOR", "", "", "", "EOR", "LSR", "", "RTS", "ADC",
    "", "", "", "ADC", "ROR", "", "PLA", "ADC", "ROR", "", "JMP", "ADC", "ROR", "", "BVS", "ADC",
    "", "", "", "ADC", "ROR", "", "SEI", "ADC", "", "", "", "ADC", "ROR", "", "", "STA", "", "",
    "STY", "STA", "STX", "", "DEY", "", "TXA", "", "STY", "STA", "STX", "", "BCC", "STA", "", "",
    "STY", "STA", "STX", "", "TYA", "STA", "TXS", "", "", "STA", "", "", "LDY", "LDA", "LDX", "",
    "LDY", "LDA", "LDX", "", "TAY", "LDA", "TAX", "", "LDY", "LDA", "LDX", "", "BCS", "LDA", "",
    "", "LDY", "LDA", "LDX", "", "CLV", "LDA", "TSX", "", "LDY", "LDA", "LDX", "", "CPY", "CMP",
    "", "", "CPY", "CMP", "DEC", "", "INY", "CMP", "DEX", "", "CPY", "CMP", "DEC", "", "BNE",
    "CMP", "", "", "", "CMP", "DEC", "", "CLD", "CMP", "", "", "", "CMP", "DEC", "", "CPX", "SBC",
    "", "", "CPX", "SBC", "INC", "", "INX", "SBC", "NOP", "", "CPX", "SBC", "INC", "", "BEQ",
    "SBC", "", "", "", "SBC", "INC", "", "SED", "SBC", "", "", "", "SBC", "INC", "",
];

impl Cpu {
    pub fn disasm(&mut self) {
        let mut pc = self.pc;

        let opcode = self.bus.read_u8(pc) as usize;

        let addr_mode = match opcode {
            0x69 | 0x29 | 0xC9 | 0xE0 | 0xC0 | 0x49 | 0xA9 | 0xA2 | 0xA0 | 0x9 | 0xE9 => {
                AddressingMode::Immediate
            }
            0x65 | 0x25 | 0x6 | 0x24 | 0xC5 | 0xE4 | 0xC4 | 0xC6 | 0x45 | 0xE6 | 0xA5 | 0xA6
            | 0xA4 | 0x46 | 0x5 | 0x26 | 0x66 | 0xE5 | 0x85 | 0x86 | 0x84 => {
                AddressingMode::ZeroPage
            }
            0x75 | 0x35 | 0x16 | 0xD5 | 0xD6 | 0x55 | 0xF6 | 0xB5 | 0xB4 | 0x56 | 0x15 | 0x36
            | 0x76 | 0xF5 | 0x95 | 0x94 => AddressingMode::ZeroPageX,
            0xB6 | 0x96 => AddressingMode::ZeroPageY,
            0x6D | 0x2D | 0xE | 0x2C | 0xCD | 0xEC | 0xCC | 0xCE | 0x4D | 0xEE | 0x4C | 0x20
            | 0xAD | 0xAE | 0xAC | 0x4E | 0xD | 0x2E | 0x6E | 0xED | 0x8D | 0x8E | 0x8C => {
                AddressingMode::Absolute
            }
            0x7D | 0x3D | 0x1E | 0xDD | 0xDE | 0x5D | 0xFE | 0xBD | 0xBC | 0x5E | 0x1D | 0x3E
            | 0x7E | 0xFD | 0x9D => AddressingMode::AbsoluteX,
            0x79 | 0x39 | 0xD9 | 0x59 | 0xB9 | 0xBE | 0x19 | 0xF9 | 0x99 => {
                AddressingMode::AbsoluteY
            }
            0x61 | 0x21 | 0xC1 | 0x41 | 0xA1 | 0x1 | 0xE1 | 0x81 => AddressingMode::IndexedIndirect,
            0x71 | 0x31 | 0xD1 | 0x51 | 0xB1 | 0x11 | 0xF1 | 0x91 => {
                AddressingMode::IndirectIndexed
            }
            0xA | 0x4A | 0x2A | 0x6A => AddressingMode::Accumulator,
            0x10 | 0x90 | 0xB0 | 0xF0 | 0x30 | 0xD0 | 0x50 | 0x70 => AddressingMode::Relative,
            0x0 | 0x18 | 0xD8 | 0x58 | 0xB8 | 0xCA | 0x88 | 0xE8 | 0xC8 | 0xEA | 0x48 | 0x8
            | 0x68 | 0x28 | 0x40 | 0x60 | 0xF8 | 0x38 | 0x78 | 0xAA | 0xA8 | 0x98 | 0xBA | 0x8A
            | 0x9A => AddressingMode::Implied,
            0x6C => AddressingMode::Indirect,
            _ => unimplemented!("Unimplemented instruction 0x{opcode:x?}"),
        };

        let frame = self.bus.ppu().frame;
        let scanline = self.bus.ppu().scanline;
        let dot = self.bus.ppu().dot;

        print!(
            "[{}] ({},{}) {:04X}: {} ",
            frame, scanline, dot, pc, OPCODE_MAP[opcode]
        );

        pc += 1;

        match addr_mode {
            AddressingMode::Absolute => {
                print!("${:04X}", self.bus.read_u16(pc));
            }
            AddressingMode::AbsoluteX => {
                print!("${:04X}, X", self.bus.read_u16(pc));
            }
            AddressingMode::AbsoluteY => {
                print!("${:04X}, Y", self.bus.read_u16(pc));
            }
            AddressingMode::Accumulator => {}
            AddressingMode::Immediate => {
                print!("${:02X}", self.bus.read_u8(pc));
            }
            AddressingMode::Implied => {}
            AddressingMode::IndexedIndirect => {
                print!("(${:02X}, X)", self.bus.read_u8(pc));
            }
            AddressingMode::Indirect => {
                print!("(${:04X})", self.bus.read_u16(pc));
            }
            AddressingMode::IndirectIndexed => {
                print!("(${:02X}), Y", self.bus.read_u8(pc));
            }
            AddressingMode::Relative => {
                let offset = (self.bus.read_u8(pc) as i8) as i32;

                pc += 1;

                let pc = pc as i32;
                let target = pc.wrapping_add(offset) as u16;

                print!("${:04X}", target);
            }
            AddressingMode::ZeroPage => {
                print!("${:02X}", self.bus.read_u8(pc));
            }
            AddressingMode::ZeroPageX => {
                print!("${:02X}, X", self.bus.read_u8(pc));
            }
            AddressingMode::ZeroPageY => {
                print!("${:02X}, Y", self.bus.read_u8(pc));
            }
        };

        let p = u8::from(&self.p);

        println!(
            "\t\tA: {:02X} X: {:02X} Y: {:02X} P: [{}{}{}{}{}{}{}{}] SP: {:02X} Stack: {:X?}",
            self.regs[A as usize],
            self.regs[X as usize],
            self.regs[Y as usize],
            if p.bit(0) { "C" } else { "_" },
            if p.bit(1) { "Z" } else { "_" },
            if p.bit(2) { "I" } else { "_" },
            "_",
            "_",
            "_",
            if p.bit(6) { "V" } else { "_" },
            if p.bit(7) { "N" } else { "_" },
            self.sp,
            &self.bus.memory()[0x100 + 1 + (self.sp as usize)..0x200]
        );
    }
}
