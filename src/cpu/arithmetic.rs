use crate::util::bit::Bit;

use super::*;

impl Cpu {
    pub fn adc(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.adc_imm(addr as u8),
            AddressingMode::ZeroPage => self.adc_zp(addr as u8),
            AddressingMode::ZeroPageX => self.adc_zp_idx(addr as u8, X),
            AddressingMode::Absolute => self.adc_abs(addr),
            AddressingMode::AbsoluteX => self.adc_abs_idx(addr, X),
            AddressingMode::AbsoluteY => self.adc_abs_idx(addr, Y),
            AddressingMode::IndexedIndirect => self.adc_idx_ind(addr as u8),
            AddressingMode::IndirectIndexed => self.adc_ind_idx(addr as u8),
            _ => unreachable!(),
        }
    }

    fn adc_imm(&mut self, imm: u8) {
        self.tick(1);
        self.addc(imm);
        self.tick(1);
    }

    fn adc_zp(&mut self, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.addc(mem);
    }

    fn adc_zp_idx(&mut self, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.addc(mem);
    }

    fn adc_abs(&mut self, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.addc(mem);
    }

    fn adc_abs_idx(&mut self, addr: u16, reg: Register) {
        let offset = self.regs[reg as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.addc(mem);
    }

    fn adc_idx_ind(&mut self, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.addc(mem);
    }

    fn adc_ind_idx(&mut self, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.addc(mem);
    }

    #[allow(unused_parens)]
    fn addc(&mut self, mem: u8) {
        let val = self.regs[A as usize];
        let carry = self.p[Carry as usize] as u8;

        let (result, carry_set) = {
            let (temp, temp_v) = val.overflowing_add(mem);
            let (result, result_v) = temp.overflowing_add(carry);

            (result, temp_v || result_v)
        };

        self.p[Carry as usize] = carry_set;
        self.p[Zero as usize] = (result == 0);
        self.p[Negative as usize] = result.bit(7);

        let val = val as i8;
        let mem = mem as i8;
        let carry = carry as i8;

        let overflow_set = {
            let (temp, temp_v) = val.overflowing_add(mem);
            let (_, result_v) = temp.overflowing_add(carry);

            temp_v ^ result_v
        };

        self.p[Overflow as usize] = overflow_set;

        self.regs[A as usize] = result;
    }

    pub fn dec(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::ZeroPage => self.dec_zp(addr as u8),
            AddressingMode::ZeroPageX => self.dec_zp_idx(addr as u8),
            AddressingMode::Absolute => self.dec_abs(addr),
            AddressingMode::AbsoluteX => self.dec_abs_idx(addr),
            _ => unreachable!(),
        }
    }

    pub fn dex(&mut self) {
        self.dec_reg(X)
    }

    pub fn dey(&mut self) {
        self.dec_reg(Y)
    }

    fn dec_zp(&mut self, addr: u8) {
        self.tick(5);
        self.dec_mem(addr as u16);
    }

    fn dec_zp_idx(&mut self, addr: u8) {
        self.tick(6);

        let idx = self.regs[X as usize];
        let target = addr.wrapping_add(idx) as u16;

        self.dec_mem(target)
    }

    fn dec_abs(&mut self, addr: u16) {
        self.tick(6);
        self.dec_mem(addr);
    }

    fn dec_abs_idx(&mut self, addr: u16) {
        self.tick(7);

        let idx = self.regs[X as usize] as u16;
        let target = addr.wrapping_add(idx);

        self.dec_mem(target)
    }

    #[allow(unused_parens)]
    fn decrement(&mut self, val: u8) -> u8 {
        let val = val.wrapping_sub(1);

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7);

        val
    }

    fn dec_reg(&mut self, reg: Register) {
        self.tick(2);

        self.regs[reg as usize] = self.decrement(self.regs[reg as usize]);
    }

    fn dec_mem(&mut self, addr: u16) {
        let mem = self.read_abs(addr);
        let mem = self.decrement(mem);

        self.bus.write_u8(addr, mem);
    }

    pub fn inc(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::ZeroPage => self.inc_zp(addr as u8),
            AddressingMode::ZeroPageX => self.inc_zp_idx(addr as u8),
            AddressingMode::Absolute => self.inc_abs(addr),
            AddressingMode::AbsoluteX => self.inc_abs_idx(addr),
            _ => unreachable!(),
        }
    }

    pub fn inx(&mut self) {
        self.inc_reg(X)
    }

    pub fn iny(&mut self) {
        self.inc_reg(Y)
    }

    fn inc_zp(&mut self, addr: u8) {
        self.tick(5);
        self.inc_mem(addr as u16);
    }

    fn inc_zp_idx(&mut self, addr: u8) {
        self.tick(6);

        let idx = self.regs[X as usize];
        let target = addr.wrapping_add(idx) as u16;

        self.inc_mem(target)
    }

    fn inc_abs(&mut self, addr: u16) {
        self.tick(6);
        self.inc_mem(addr);
    }

    fn inc_abs_idx(&mut self, addr: u16) {
        self.tick(7);

        let idx = self.regs[X as usize] as u16;
        let target = addr.wrapping_add(idx);

        self.inc_mem(target)
    }

    #[allow(unused_parens)]
    fn increment(&mut self, val: u8) -> u8 {
        let val = val.wrapping_add(1);

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7);

        val
    }

    fn inc_mem(&mut self, addr: u16) {
        let mem = self.read_abs(addr);
        let mem = self.increment(mem);

        self.bus.write_u8(addr, mem);
    }

    #[allow(unused_parens)]
    fn inc_reg(&mut self, reg: Register) {
        self.tick(2);

        let val = self.regs[reg as usize].wrapping_add(1);

        self.regs[reg as usize] = val;

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7);
    }

    pub fn sbc(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.sbc_imm(addr as u8),
            AddressingMode::ZeroPage => self.sbc_zp(addr as u8),
            AddressingMode::ZeroPageX => self.sbc_zp_idx(addr as u8, X),
            AddressingMode::Absolute => self.sbc_abs(addr),
            AddressingMode::AbsoluteX => self.sbc_abs_idx(addr, X),
            AddressingMode::AbsoluteY => self.sbc_abs_idx(addr, Y),
            AddressingMode::IndexedIndirect => self.sbc_idx_ind(addr as u8),
            AddressingMode::IndirectIndexed => self.sbc_ind_idx(addr as u8),
            _ => unreachable!(),
        }
    }

    fn sbc_imm(&mut self, imm: u8) {
        self.tick(1);
        self.subb(imm);
        self.tick(1);
    }

    fn sbc_zp(&mut self, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.subb(mem);
    }

    fn sbc_zp_idx(&mut self, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.subb(mem);
    }

    fn sbc_abs(&mut self, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.subb(mem);
    }

    fn sbc_abs_idx(&mut self, addr: u16, reg: Register) {
        let offset = self.regs[reg as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.subb(mem);
    }

    fn sbc_idx_ind(&mut self, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.subb(mem);
    }

    fn sbc_ind_idx(&mut self, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.subb(mem);
    }

    fn subb(&mut self, mem: u8) {
        self.addc(!mem);
    }
}
