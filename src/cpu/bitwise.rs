use crate::util::bit::Bit;

use super::*;

impl Cpu {
    pub fn and(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.and_imm(addr as u8),
            AddressingMode::ZeroPage => self.and_zp(addr as u8),
            AddressingMode::ZeroPageX => self.and_zp_idx(addr as u8, X),
            AddressingMode::Absolute => self.and_abs(addr),
            AddressingMode::AbsoluteX => self.and_abs_idx(addr, X),
            AddressingMode::AbsoluteY => self.and_abs_idx(addr, Y),
            AddressingMode::IndexedIndirect => self.and_idx_ind(addr as u8),
            AddressingMode::IndirectIndexed => self.and_ind_idx(addr as u8),
            _ => unreachable!(),
        }
    }

    fn and_imm(&mut self, imm: u8) {
        self.tick(1);
        self.bitand(imm);
        self.tick(1);
    }

    fn and_zp(&mut self, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.bitand(mem);
    }

    fn and_zp_idx(&mut self, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.bitand(mem);
    }

    fn and_abs(&mut self, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.bitand(mem);
    }

    fn and_abs_idx(&mut self, addr: u16, reg: Register) {
        let offset = self.regs[reg as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.bitand(mem);
    }

    fn and_idx_ind(&mut self, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.bitand(mem);
    }

    fn and_ind_idx(&mut self, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.bitand(mem);
    }

    #[allow(unused_parens)]
    fn bitand(&mut self, mem: u8) {
        let val = self.regs[A as usize];

        let result = val & mem;

        self.p[Zero as usize] = (result == 0);
        self.p[Negative as usize] = result.bit(7);

        self.regs[A as usize] = result;
    }

    pub fn asl(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => self.asl_accum(),
            AddressingMode::ZeroPage => self.asl_zp(addr as u8),
            AddressingMode::ZeroPageX => self.asl_zp_idx(addr as u8),
            AddressingMode::Absolute => self.asl_abs(addr),
            AddressingMode::AbsoluteX => self.asl_abs_idx(addr),
            _ => unreachable!(),
        }
    }

    fn asl_accum(&mut self) {
        self.tick(2);

        let val = self.regs[A as usize];

        let val = self.shl(val);

        self.regs[A as usize] = val;
    }

    fn asl_zp(&mut self, addr: u8) {
        self.tick(5);

        let val = self.read_zp(addr);

        let val = self.shl(val);

        self.bus.write_u8(addr as u16, val);
    }

    fn asl_zp_idx(&mut self, addr: u8) {
        self.tick(6);

        let idx = self.regs[X as usize];
        let target = addr.wrapping_add(idx);
        let val = self.read_zp(target);

        let val = self.shl(val);

        self.bus.write_u8(target as u16, val);
    }

    fn asl_abs(&mut self, addr: u16) {
        self.tick(6);

        let val = self.read_abs(addr);

        let val = self.shl(val);

        self.bus.write_u8(addr, val);
    }

    fn asl_abs_idx(&mut self, addr: u16) {
        self.tick(7);

        let idx = self.regs[X as usize] as u16;
        let target = addr.wrapping_add(idx);
        let val = self.read_abs(target);

        let val = self.shl(val);

        self.bus.write_u8(target, val);
    }

    #[allow(unused_parens)]
    fn shl(&mut self, val: u8) -> u8 {
        self.p[Carry as usize] = val.bit(7);

        let val = val << 1;

        self.p[Negative as usize] = val.bit(7);
        self.p[Zero as usize] = (val == 0);

        val
    }

    pub fn eor(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.eor_imm(addr as u8),
            AddressingMode::ZeroPage => self.eor_zp(addr as u8),
            AddressingMode::ZeroPageX => self.eor_zp_idx(addr as u8, X),
            AddressingMode::Absolute => self.eor_abs(addr),
            AddressingMode::AbsoluteX => self.eor_abs_idx(addr, X),
            AddressingMode::AbsoluteY => self.eor_abs_idx(addr, Y),
            AddressingMode::IndexedIndirect => self.eor_idx_ind(addr as u8),
            AddressingMode::IndirectIndexed => self.eor_ind_idx(addr as u8),
            _ => unreachable!(),
        }
    }

    fn eor_imm(&mut self, imm: u8) {
        self.tick(1);
        self.xor(imm);
        self.tick(1);
    }

    fn eor_zp(&mut self, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.xor(mem);
    }

    fn eor_zp_idx(&mut self, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.xor(mem);
    }

    fn eor_abs(&mut self, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.xor(mem);
    }

    fn eor_abs_idx(&mut self, addr: u16, reg: Register) {
        let offset = self.regs[reg as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.xor(mem);
    }

    fn eor_idx_ind(&mut self, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.xor(mem);
    }

    fn eor_ind_idx(&mut self, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.xor(mem);
    }

    #[allow(unused_parens)]
    fn xor(&mut self, mem: u8) {
        let val = self.regs[A as usize];

        let result = val ^ mem;

        self.p[Zero as usize] = (result == 0);
        self.p[Negative as usize] = result.bit(7);

        self.regs[A as usize] = result;
    }

    pub fn lsr(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => self.lsr_accum(),
            AddressingMode::ZeroPage => self.lsr_zp(addr as u8),
            AddressingMode::ZeroPageX => self.lsr_zp_idx(addr as u8),
            AddressingMode::Absolute => self.lsr_abs(addr),
            AddressingMode::AbsoluteX => self.lsr_abs_idx(addr),
            _ => unreachable!(),
        }
    }

    fn lsr_accum(&mut self) {
        self.tick(2);

        let val = self.regs[A as usize];

        let val = self.shr(val);

        self.regs[A as usize] = val;
    }

    fn lsr_zp(&mut self, addr: u8) {
        self.tick(5);

        let val = self.read_zp(addr);

        let val = self.shr(val);

        self.bus.write_u8(addr as u16, val);
    }

    fn lsr_zp_idx(&mut self, addr: u8) {
        self.tick(6);

        let idx = self.regs[X as usize];
        let target = addr.wrapping_add(idx);
        let val = self.read_zp(target);

        let val = self.shr(val);

        self.bus.write_u8(target as u16, val);
    }

    fn lsr_abs(&mut self, addr: u16) {
        self.tick(6);

        let val = self.read_abs(addr);

        let val = self.shr(val);

        self.bus.write_u8(addr, val);
    }

    fn lsr_abs_idx(&mut self, addr: u16) {
        self.tick(7);

        let idx = self.regs[X as usize] as u16;
        let target = addr.wrapping_add(idx);
        let val = self.read_abs(target);

        let val = self.shr(val);

        self.bus.write_u8(target, val);
    }

    #[allow(unused_parens)]
    fn shr(&mut self, val: u8) -> u8 {
        self.p[Carry as usize] = val.bit(0);

        let val = val >> 1;

        self.p[Negative as usize] = false;
        self.p[Zero as usize] = (val == 0);

        val
    }

    pub fn ora(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => self.ora_imm(addr as u8),
            AddressingMode::ZeroPage => self.ora_zp(addr as u8),
            AddressingMode::ZeroPageX => self.ora_zp_idx(addr as u8, X),
            AddressingMode::Absolute => self.ora_abs(addr),
            AddressingMode::AbsoluteX => self.ora_abs_idx(addr, X),
            AddressingMode::AbsoluteY => self.ora_abs_idx(addr, Y),
            AddressingMode::IndexedIndirect => self.ora_idx_ind(addr as u8),
            AddressingMode::IndirectIndexed => self.ora_ind_idx(addr as u8),
            _ => unreachable!(),
        }
    }

    fn ora_imm(&mut self, imm: u8) {
        self.tick(1);
        self.or(imm);
        self.tick(1);
    }

    fn ora_zp(&mut self, addr: u8) {
        self.tick(2);
        let mem = self.read_zp(addr);
        self.tick(1);

        self.or(mem);
    }

    fn ora_zp_idx(&mut self, addr: u8, idx: Register) {
        self.tick(3);
        let offset = self.regs[idx as usize];
        let mem = self.read_zp_idx(addr, offset);
        self.tick(1);

        self.or(mem);
    }

    fn ora_abs(&mut self, addr: u16) {
        self.tick(3);
        let mem = self.read_abs(addr);
        self.tick(1);

        self.or(mem);
    }

    fn ora_abs_idx(&mut self, addr: u16, reg: Register) {
        let offset = self.regs[reg as usize];

        self.tick(3);
        if crosses_page(addr, offset as i32) {
            self.tick(1);
        };

        let mem = self.read_abs_idx(addr, offset);
        self.tick(1);

        self.or(mem);
    }

    fn ora_idx_ind(&mut self, addr: u8) {
        let offset = self.regs[X as usize];

        self.tick(5);
        let mem = self.read_idx_ind(addr, offset);
        self.tick(1);

        self.or(mem);
    }

    fn ora_ind_idx(&mut self, addr: u8) {
        let offset = self.regs[Y as usize];

        self.tick(4);
        let mem = self.read_ind_idx(addr, offset);
        self.tick(1);

        self.or(mem);
    }

    #[allow(unused_parens)]
    fn or(&mut self, mem: u8) {
        let val = self.regs[A as usize];

        let result = val | mem;

        self.p[Zero as usize] = (result == 0);
        self.p[Negative as usize] = result.bit(7);

        self.regs[A as usize] = result;
    }

    pub fn rol(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => self.rol_accum(),
            AddressingMode::ZeroPage => self.rol_zp(addr as u8),
            AddressingMode::ZeroPageX => self.rol_zp_idx(addr as u8),
            AddressingMode::Absolute => self.rol_abs(addr),
            AddressingMode::AbsoluteX => self.rol_abs_idx(addr),
            _ => unreachable!(),
        }
    }

    fn rol_accum(&mut self) {
        self.tick(2);

        let val = self.regs[A as usize];

        let val = self.lrotate(val);

        self.regs[A as usize] = val;
    }

    fn rol_zp(&mut self, addr: u8) {
        self.tick(5);

        let val = self.read_zp(addr);

        let val = self.lrotate(val);

        self.bus.write_u8(addr as u16, val);
    }

    fn rol_zp_idx(&mut self, addr: u8) {
        self.tick(6);

        let idx = self.regs[X as usize];
        let target = addr.wrapping_add(idx);
        let val = self.read_zp(target);

        let val = self.lrotate(val);

        self.bus.write_u8(target as u16, val);
    }

    fn rol_abs(&mut self, addr: u16) {
        self.tick(6);

        let val = self.read_abs(addr);

        let val = self.lrotate(val);

        self.bus.write_u8(addr, val);
    }

    fn rol_abs_idx(&mut self, addr: u16) {
        self.tick(7);

        let idx = self.regs[X as usize] as u16;
        let target = addr.wrapping_add(idx);
        let val = self.read_abs(target);

        let val = self.lrotate(val);

        self.bus.write_u8(target, val);
    }

    #[allow(unused_parens)]
    fn lrotate(&mut self, val: u8) -> u8 {
        let carry = self.p[Carry as usize] as u8;

        self.p[Carry as usize] = val.bit(7);

        let val = (val << 1) | carry;

        self.p[Negative as usize] = val.bit(7);
        self.p[Zero as usize] = (val == 0);

        val
    }

    pub fn ror(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Accumulator => self.ror_accum(),
            AddressingMode::ZeroPage => self.ror_zp(addr as u8),
            AddressingMode::ZeroPageX => self.ror_zp_idx(addr as u8),
            AddressingMode::Absolute => self.ror_abs(addr),
            AddressingMode::AbsoluteX => self.ror_abs_idx(addr),
            _ => unreachable!(),
        }
    }

    fn ror_accum(&mut self) {
        self.tick(2);

        let val = self.regs[A as usize];

        let val = self.rrotate(val);

        self.regs[A as usize] = val;
    }

    fn ror_zp(&mut self, addr: u8) {
        self.tick(5);

        let val = self.read_zp(addr);

        let val = self.rrotate(val);

        self.bus.write_u8(addr as u16, val);
    }

    fn ror_zp_idx(&mut self, addr: u8) {
        self.tick(6);

        let idx = self.regs[X as usize];
        let target = addr.wrapping_add(idx);
        let val = self.read_zp(target);

        let val = self.rrotate(val);

        self.bus.write_u8(target as u16, val);
    }

    fn ror_abs(&mut self, addr: u16) {
        self.tick(6);

        let val = self.read_abs(addr);

        let val = self.rrotate(val);

        self.bus.write_u8(addr, val);
    }

    fn ror_abs_idx(&mut self, addr: u16) {
        self.tick(7);

        let idx = self.regs[X as usize] as u16;
        let target = addr.wrapping_add(idx);
        let val = self.read_abs(target);

        let val = self.rrotate(val);

        self.bus.write_u8(target, val);
    }

    #[allow(unused_parens)]
    fn rrotate(&mut self, val: u8) -> u8 {
        self.p[Negative as usize] = self.p[Carry as usize];

        let carry = self.p[Carry as usize] as u8;

        self.p[Carry as usize] = val.bit(0);

        let val = (val >> 1) | (carry << 7);

        self.p[Zero as usize] = (val == 0);

        val
    }
}
