use super::status::Flag::*;
use super::*;

impl Cpu {
    pub fn clc(&mut self) {
        self.clear_flag(Carry as u8)
    }

    pub fn cld(&mut self) {
        self.clear_flag(Decimal as u8)
    }

    pub fn cli(&mut self) {
        self.clear_flag(InterruptDisable as u8)
    }

    pub fn clv(&mut self) {
        self.clear_flag(Overflow as u8)
    }

    fn clear_flag(&mut self, flag: u8) {
        self.tick(2);

        self.p[flag as usize] = false;
    }

    pub fn flags(&self) -> u8 {
        u8::from(&self.p)
    }

    pub fn pull_flags(&mut self) -> Status {
        let flags = self.pull();
        let flags = flags | 0b00100000;
        let flags = flags & !0b00010000;

        Status::from(flags)
    }

    pub fn sec(&mut self) {
        self.set_flag(Carry as u8)
    }

    pub fn sed(&mut self) {
        self.set_flag(Decimal as u8)
    }

    pub fn sei(&mut self) {
        self.set_flag(InterruptDisable as u8)
    }

    fn set_flag(&mut self, flag: u8) {
        self.tick(2);

        self.p[flag as usize] = true;
    }
}
