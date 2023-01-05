use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct Lfsr {
    data: u16,
}

impl Lfsr {
    pub fn new() -> Self {
        Self { data: 1 }
    }

    pub fn tick(&mut self, mode: bool) {
        let data = self.data;
        let bit0 = data.bit(0) as u16;
        let other_bit = if mode {
            data.bit(6) as u16
        } else {
            data.bit(1) as u16
        };

        let feedback = bit0 ^ other_bit;

        self.data = (self.data >> 1) | (feedback << 14);
    }

    pub fn output(&self) -> bool {
        self.data.bit(0)
    }
}

impl DebugInfo for Lfsr {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Linear Feedback Shift Register");
            ui.label(format!("{:015b}", self.data));
        });
    }
}
