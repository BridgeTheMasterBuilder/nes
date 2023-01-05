use std::ops::Index;
use std::ops::IndexMut;

use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[repr(usize)]
pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    Decimal,
    Break,
    Unused,
    Overflow,
    Negative,
}

#[derive(Default, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Status {
    flags: [bool; 8],
}

impl Status {
    pub fn new() -> Self {
        Self {
            flags: [false, false, false, false, false, true, false, false],
        }
    }
}

impl From<&Status> for u8 {
    fn from(flags: &Status) -> u8 {
        flags.flags.iter().rev().fold(
            0,
            |accum, &elem| if elem { accum * 2 + 1 } else { accum * 2 },
        )
    }
}

impl From<u8> for Status {
    fn from(bits: u8) -> Status {
        let mut flags: [bool; 8] = [false; 8];

        flags
            .iter_mut()
            .enumerate()
            .for_each(|(index, flag)| *flag = bits.bit(index));

        Status { flags }
    }
}

impl Index<usize> for Status {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.flags[index]
    }
}

impl IndexMut<usize> for Status {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.flags[index]
    }
}
