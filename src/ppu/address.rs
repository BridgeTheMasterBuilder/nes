use std::cell::Cell;

use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Address {
    addr: Cell<u16>,
}

impl Address {
    pub fn new() -> Self {
        Self { addr: Cell::new(0) }
    }

    pub fn address(&self) -> u16 {
        self.addr.get()
    }

    pub fn inc_horiz(&self) -> u16 {
        let addr = self.addr.get();
        let mut x = addr.bits_abs(0, 4);
        let mut nt_bits = addr.bits(10, 11);

        if x == 31 {
            x = 0;
            nt_bits ^= 1;
        } else {
            x += 1;
        }

        addr.bits_abs(12, 14) | addr.bits_abs(5, 9) | (nt_bits << 10) | x
    }

    pub fn inc_vert(&self) -> u16 {
        let addr = self.addr.get();
        let mut fine_y = addr.bits(12, 14);
        let mut coarse_y = addr.bits(5, 9);
        let mut nt_bits = addr.bits(10, 11);

        if fine_y < 7 {
            fine_y += 1;
        } else {
            fine_y = 0;

            if coarse_y == 29 {
                coarse_y = 0;
                nt_bits ^= 0b10;
            } else if coarse_y == 31 {
                coarse_y = 0
            } else {
                coarse_y += 1;
            }
        }

        addr.bits_abs(0, 4) | (fine_y << 12) | (nt_bits << 10) | (coarse_y << 5)
    }

    pub fn increment(&self, inc_vert: bool) {
        let mut addr = self.addr.get();

        if inc_vert {
            addr += 32;
        } else {
            addr += 1;
        }

        if addr > 0x3FFF {
            addr = (addr % 0x4000) + 0x2000;
        }

        self.addr.replace(addr);
    }

    pub fn update(&self, addr: u16) {
        self.addr.replace(addr);
    }

    pub fn write_addr(&mut self, byte: u8, w: bool) {
        if !w {
            let byte = byte as u16;
            let part = byte.bits_abs(0, 5);
            let addr = self.addr.get().bits_abs(0, 7);
            self.addr.replace(addr | (part << 8));
        } else {
            let byte = byte as u16;
            let addr = self.addr.get().bits_abs(8, 14);
            self.addr.replace(addr | byte);
        }
    }

    pub fn write_scroll(&mut self, byte: u8, w: bool) {
        if !w {
            let byte = byte as u16;
            let x = byte.bits(3, 7);
            let addr = self.addr.get().bits_abs(5, 14);
            self.addr.replace(addr | x);
        } else {
            let byte = byte as u16;
            let course_y = byte.bits(3, 7);
            let addr = self.addr.get();
            let addr = addr.bits_abs(10, 14) | addr.bits_abs(0, 4);
            self.addr.replace(addr | (course_y << 5));

            let fine_y = byte.bits_abs(0, 2);
            let addr = self.addr.get().bits_abs(0, 11);
            self.addr.replace(addr | (fine_y << 12));
        }
    }
}
