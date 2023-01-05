use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Mode {
    OnesComplement,
    TwosComplement,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct Sweep {
    pub(super) enabled: bool,
    pub(super) negate: bool,
    pub(super) reload_flag: bool,
    pub(super) shift: u8,
    counter: u8,
    mode: Mode,
    reload: u8,
    target_period: u16,
}

impl Sweep {
    pub fn new(mode: Mode) -> Self {
        Self {
            enabled: false,
            negate: false,
            reload_flag: false,
            shift: 0,
            counter: 0,
            mode,
            reload: 0,
            target_period: 0,
        }
    }

    pub fn calculate_target_period(&mut self, cur_period: u16) {
        let change_amount = cur_period >> self.shift;

        self.target_period = if self.negate {
            let change_amount = match self.mode {
                Mode::OnesComplement => change_amount + 1,
                Mode::TwosComplement => change_amount,
            };

            cur_period.wrapping_sub(change_amount)
        } else {
            cur_period.wrapping_add(change_amount)
        }
    }

    pub fn muted(&self, cur_period: u16) -> bool {
        cur_period < 8 || (!self.negate && self.target_period > 0x7FF)
    }

    pub fn set_period(&mut self, period: u8) {
        self.reload = period;
    }

    pub fn tick(&mut self, cur_period: u16) {
        self.calculate_target_period(cur_period);
    }

    pub fn tock(&mut self, cur_period: u16) -> u16 {
        self.counter = self.counter.saturating_sub(1);

        let mut target_period = cur_period;

        if self.counter == 0 {
            if self.shift > 0 && self.enabled && cur_period >= 8 && self.target_period <= 0x7FF {
                target_period = self.target_period;
            }

            self.counter = self.reload;
        }

        if self.reload_flag {
            self.counter = self.reload;
            self.reload_flag = false;
        }

        target_period
    }
}

impl DebugInfo for Sweep {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Sweep");
            let _ = ui.radio(self.enabled, "Enabled");
            ui.label(format!("Divider period: {}", self.reload));
            let _ = ui.radio(self.negate, "Negate");
            ui.label(format!("Shift count: {}", self.shift));
            let _ = ui.radio(self.reload_flag, "Reload");
        });
    }
}
