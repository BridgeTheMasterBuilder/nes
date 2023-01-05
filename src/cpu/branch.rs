use super::status::Flag::*;
use super::*;

impl Cpu {
    pub fn bpl(&mut self, offset: i8) {
        self.branch_if_not(Negative as u8, offset)
    }

    pub fn bcc(&mut self, offset: i8) {
        self.branch_if_not(Carry as u8, offset)
    }

    pub fn bcs(&mut self, offset: i8) {
        self.branch_if(Carry as u8, offset)
    }

    pub fn beq(&mut self, offset: i8) {
        self.branch_if(Zero as u8, offset)
    }

    pub fn bmi(&mut self, offset: i8) {
        self.branch_if(Negative as u8, offset)
    }

    pub fn bne(&mut self, offset: i8) {
        self.branch_if_not(Zero as u8, offset)
    }

    pub fn bvc(&mut self, offset: i8) {
        self.branch_if_not(Overflow as u8, offset)
    }

    pub fn bvs(&mut self, offset: i8) {
        self.branch_if(Overflow as u8, offset)
    }

    pub fn jmp(&mut self, addr: u16, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => {
                self.tick(3);

                self.pc = addr;
            }
            AddressingMode::Indirect => {
                self.tick(5);

                let target = self.read_ind(addr);

                self.pc = target;
            }
            _ => unreachable!(),
        }
    }

    pub fn jsr(&mut self, addr: u16) {
        self.tick(6);

        self.push_u16(self.pc.wrapping_sub(1));

        self.pc = addr;
    }

    pub fn rts(&mut self) {
        self.tick(6);

        self.pc = self.pull_u16();

        self.pc = self.pc.wrapping_add(1);
    }

    fn branch(&mut self, condition: bool, offset: i8) {
        let pc = self.pc as i32;
        let offset = offset as i32;
        let target = pc.wrapping_add(offset) as u16;

        if condition {
            let cycles = if crosses_page(self.pc, offset) { 4 } else { 3 };

            self.tick(cycles);

            self.pc = target;
        } else {
            self.tick(2);
        }
    }

    fn branch_if(&mut self, flag: u8, offset: i8) {
        self.branch(self.p[flag as usize], offset)
    }

    fn branch_if_not(&mut self, flag: u8, offset: i8) {
        self.branch(!self.p[flag as usize], offset)
    }
}
