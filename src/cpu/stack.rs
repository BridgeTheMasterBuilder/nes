use crate::util::bit::Bit;

use super::*;

impl Cpu {
    pub fn pha(&mut self) {
        self.tick(3);
        self.push(self.regs[A as usize]);
    }

    pub fn php(&mut self) {
        self.tick(3);

        let flags = self.flags();
        let flags = flags | 0b00110000;

        self.push(flags)
    }

    #[allow(unused_parens)]
    pub fn pla(&mut self) {
        self.tick(4);

        let val = self.pull();

        self.regs[A as usize] = val;

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7);
    }

    pub fn plp(&mut self) {
        self.tick(4);

        let flags = self.pull_flags();

        self.p = flags;
    }
}
