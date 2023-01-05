use crate::util::bit::Bit;

use super::*;

impl Cpu {
    pub fn bit(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::ZeroPage => self.bit_zp(addr as u8),
            AddressingMode::Absolute => self.bit_abs(addr),
            _ => unreachable!(),
        }
    }

    #[allow(unused_parens)]
    fn bit_zp(&mut self, addr: u8) {
        self.tick(2);
        let mem = self.bus.read_u8(addr as u16);
        self.tick(1);

        self.p[Zero as usize] = ((self.regs[A as usize] & mem) == 0);
        self.p[Overflow as usize] = mem.bit(6);
        self.p[Negative as usize] = mem.bit(7);
    }

    #[allow(unused_parens)]
    fn bit_abs(&mut self, addr: u16) {
        self.tick(3);
        let mem = self.bus.read_u8(addr);
        self.tick(1);

        self.p[Zero as usize] = ((self.regs[A as usize] & mem) == 0);
        self.p[Overflow as usize] = mem.bit(6);
        self.p[Negative as usize] = mem.bit(7);
    }

    pub fn cmp(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.cmp_imm(A, addr as u8),
            AddressingMode::ZeroPage => self.cmp_zp(A, addr as u8),
            AddressingMode::ZeroPageX => self.cmp_zp_idx(A, addr as u8, X),
            AddressingMode::Absolute => self.cmp_abs(A, addr),
            AddressingMode::AbsoluteX => self.cmp_abs_idx(A, addr, X),
            AddressingMode::AbsoluteY => self.cmp_abs_idx(A, addr, Y),
            AddressingMode::IndexedIndirect => self.cmp_idx_ind(A, addr as u8),
            AddressingMode::IndirectIndexed => self.cmp_ind_idx(A, addr as u8),
            _ => unreachable!(),
        }
    }

    pub fn cpx(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.cmp_imm(X, addr as u8),
            AddressingMode::ZeroPage => self.cmp_zp(X, addr as u8),
            AddressingMode::Absolute => self.cmp_abs(X, addr),
            _ => unreachable!(),
        }
    }

    pub fn cpy(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.cmp_imm(Y, addr as u8),
            AddressingMode::ZeroPage => self.cmp_zp(Y, addr as u8),
            AddressingMode::Absolute => self.cmp_abs(Y, addr),
            _ => unreachable!(),
        }
    }

    fn cmp_imm(&mut self, reg: Register, imm: u8) {
        self.tick(1);
        self.compare(reg, imm);
        self.tick(1);
    }

    fn cmp_zp(&mut self, reg: Register, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.compare(reg, mem);
    }

    fn cmp_zp_idx(&mut self, reg: Register, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.compare(reg, mem);
    }

    fn cmp_abs(&mut self, reg: Register, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.compare(reg, mem);
    }

    fn cmp_abs_idx(&mut self, reg1: Register, addr: u16, reg2: Register) {
        let offset = self.regs[reg2 as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.compare(reg1, mem);
    }

    fn cmp_idx_ind(&mut self, reg: Register, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.compare(reg, mem);
    }

    fn cmp_ind_idx(&mut self, reg: Register, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.compare(reg, mem);
    }

    #[allow(unused_parens)]
    fn compare(&mut self, reg: Register, mem: u8) {
        let val = self.regs[reg as usize];

        let result = val.wrapping_sub(mem);

        self.p[Zero as usize] = (result == 0);
        self.p[Negative as usize] = result.bit(7);
        self.p[Carry as usize] = (mem <= val);
    }
}
