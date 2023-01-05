use crate::util::bit::Bit;

use super::*;

impl Cpu {
    pub fn tax(&mut self) {
        self.transfer(A, X)
    }

    pub fn tay(&mut self) {
        self.transfer(A, Y)
    }

    pub fn tya(&mut self) {
        self.transfer(Y, A)
    }

    #[allow(unused_parens)]
    pub fn tsx(&mut self) {
        self.tick(2);

        let val = self.sp;

        self.regs[X as usize] = val;

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7);
    }

    pub fn txa(&mut self) {
        self.transfer(X, A)
    }

    pub fn txs(&mut self) {
        self.tick(2);

        self.sp = self.regs[X as usize];
    }

    #[allow(unused_parens)]
    fn transfer(&mut self, reg1: Register, reg2: Register) {
        self.tick(2);

        let val = self.regs[reg1 as usize];

        self.regs[reg2 as usize] = val;

        self.p[Zero as usize] = (val == 0);
        self.p[Negative as usize] = val.bit(7);
    }
}
