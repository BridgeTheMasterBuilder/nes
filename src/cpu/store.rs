use super::*;

impl Cpu {
    pub fn sta(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::ZeroPage => self.store_zp(A, addr as u8),
            AddressingMode::ZeroPageX => self.store_zp_idx(A, addr as u8, X),
            AddressingMode::Absolute => self.store_abs(A, addr),
            AddressingMode::AbsoluteX => self.store_abs_idx(A, addr, X),
            AddressingMode::AbsoluteY => self.store_abs_idx(A, addr, Y),
            AddressingMode::IndexedIndirect => self.store_idx_ind(A, addr as u8),
            AddressingMode::IndirectIndexed => self.store_ind_idx(A, addr as u8),
            _ => unreachable!(),
        }
    }

    pub fn stx(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::ZeroPage => self.store_zp(X, addr as u8),
            AddressingMode::ZeroPageY => self.store_zp_idx(X, addr as u8, Y),
            AddressingMode::Absolute => self.store_abs(X, addr),
            _ => unreachable!(),
        }
    }

    pub fn sty(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::ZeroPage => self.store_zp(Y, addr as u8),
            AddressingMode::ZeroPageX => self.store_zp_idx(Y, addr as u8, X),
            AddressingMode::Absolute => self.store_abs(Y, addr),
            _ => unreachable!(),
        }
    }

    fn store_zp(&mut self, reg: Register, addr: u8) {
        self.tick(2);
        self.store(reg, self.ea_zp(addr));
        self.tick(1);
    }

    fn store_zp_idx(&mut self, reg: Register, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        self.tick(1);

        self.store(reg, self.ea_zp_idx(addr, offset));
    }

    fn store_abs(&mut self, reg: Register, addr: u16) {
        self.tick(3);
        self.store(reg, self.ea_abs(addr));
        self.tick(1);
    }

    fn store_abs_idx(&mut self, reg1: Register, addr: u16, reg2: Register) {
        self.tick(4);
        let offset = self.regs[reg2 as usize];

        self.store(reg1, self.ea_abs_idx(addr, offset));
        self.tick(1);
    }

    fn store_idx_ind(&mut self, reg: Register, addr: u8) {
        self.tick(5);

        let offset = self.regs[X as usize];
        let target = self.ea_idx_ind(addr, offset);

        self.store(reg, target);
        self.tick(1);
    }

    fn store_ind_idx(&mut self, reg: Register, addr: u8) {
        self.tick(5);

        let offset = self.regs[Y as usize];
        let (target, _) = self.ea_ind_idx(addr, offset);

        self.store(reg, target);
        self.tick(1);
    }

    fn store(&mut self, reg: Register, addr: u16) {
        let val = self.regs[reg as usize];

        self.bus.write_u8(addr, val);
    }
}
