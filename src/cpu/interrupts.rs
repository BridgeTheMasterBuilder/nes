use super::*;

impl Cpu {
    pub fn brk(&mut self) {
        self.tick(7);

        self.push_u16(self.pc + 1);

        let flags = self.flags();
        let flags = flags | 0b00110000;

        self.push(flags);
        self.p[InterruptDisable as usize] = true;

        self.pc = self.bus.read_u16(0xFFFE);
    }

    pub fn rti(&mut self) {
        self.tick(6);

        let flags = self.pull_flags();

        self.p = flags;
        self.pc = self.pull_u16();
    }
}
