use crate::util::bit::Bit;

use super::*;

impl Cpu {
    pub fn lda(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.load_imm(A, addr as u8),
            AddressingMode::ZeroPage => self.load_zp(A, addr as u8),
            AddressingMode::ZeroPageX => self.load_zp_idx(A, addr as u8, X),
            AddressingMode::Absolute => self.load_abs(A, addr),
            AddressingMode::AbsoluteX => self.load_abs_idx(A, addr, X),
            AddressingMode::AbsoluteY => self.load_abs_idx(A, addr, Y),
            AddressingMode::IndexedIndirect => self.load_idx_ind(A, addr as u8),
            AddressingMode::IndirectIndexed => self.load_ind_idx(A, addr as u8),
            _ => unreachable!(),
        }
    }

    pub fn ldx(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.load_imm(X, addr as u8),
            AddressingMode::ZeroPage => self.load_zp(X, addr as u8),
            AddressingMode::ZeroPageY => self.load_zp_idx(X, addr as u8, Y),
            AddressingMode::Absolute => self.load_abs(X, addr),
            AddressingMode::AbsoluteY => self.load_abs_idx(X, addr, Y),
            _ => unreachable!(),
        }
    }

    pub fn ldy(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.load_imm(Y, addr as u8),
            AddressingMode::ZeroPage => self.load_zp(Y, addr as u8),
            AddressingMode::ZeroPageX => self.load_zp_idx(Y, addr as u8, X),
            AddressingMode::Absolute => self.load_abs(Y, addr),
            AddressingMode::AbsoluteX => self.load_abs_idx(Y, addr, X),
            _ => unreachable!(),
        }
    }

    #[allow(unused_parens)]
    fn load(&mut self, reg: Register, val: u8) {
        self.regs[reg as usize] = val;

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7)
    }

    fn load_imm(&mut self, reg: Register, imm: u8) {
        self.tick(1);
        self.load(reg, imm);
        self.tick(1);
    }

    fn load_zp(&mut self, reg: Register, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.load(reg, mem);
    }

    fn load_zp_idx(&mut self, reg: Register, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.load(reg, mem);
    }

    fn load_abs(&mut self, reg: Register, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.load(reg, mem);
    }
    fn load_abs_idx(&mut self, reg1: Register, addr: u16, reg2: Register) {
        let offset = self.regs[reg2 as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.load(reg1, mem);
    }

    fn load_idx_ind(&mut self, reg: Register, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.load(reg, mem);
    }

    fn load_ind_idx(&mut self, reg: Register, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.load(reg, mem);
    }
}
