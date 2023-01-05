use std::cell::Cell;

use serde::{Deserialize, Serialize};

#[repr(usize)]
pub enum Btn {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Controller {
    buttons: [bool; 8],
    idx: Cell<usize>,
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn press(&mut self, button: Btn) {
        self.buttons[button as usize] = true;
    }

    pub fn read(&self) -> u8 {
        if self.strobe {
            self.buttons[0] as u8
        } else if self.idx.get() == 8 {
            1
        } else {
            let idx = self.idx.get();
            let status = self.buttons[idx] as u8;
            self.idx.replace(idx + 1);

            status
        }
    }

    pub fn release(&mut self, button: Btn) {
        self.buttons[button as usize] = false;
    }

    pub fn write(&mut self, data: u8) {
        let strobe = (data & 1) != 0;

        if !strobe {
            self.idx.replace(0);
        }

        self.strobe = strobe;
    }
}
