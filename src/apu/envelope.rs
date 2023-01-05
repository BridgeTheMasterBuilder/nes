use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct Envelope {
    pub(super) constant: bool,
    pub(super) loop_flag: bool,
    pub(super) start: bool,
    decay: u8,
    period: u8,
    reload: u8,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            constant: false,
            loop_flag: false,
            start: false,
            decay: 0,
            period: 0,
            reload: 0,
        }
    }

    pub fn set_period(&mut self, period: u8) {
        self.period = period;
        self.reload = period;
    }

    pub fn tick(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.period = self.reload;
        } else {
            self.tock();
        }
    }

    pub fn tock(&mut self) {
        if self.period == 0 {
            self.period = self.reload;

            if self.decay > 0 {
                self.decay -= 1;
            } else if self.loop_flag {
                self.decay = 15;
            }
        } else {
            self.period -= 1;
        }
    }

    pub fn volume(&self) -> u8 {
        if self.constant {
            self.reload
        } else {
            self.decay
        }
    }
}

impl DebugInfo for Envelope {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Envelope");
            let _ = ui.radio(self.start, "Start");
            let _ = ui.radio(self.constant, "Constant volume");
            ui.label(format!("Volume: {}", self.volume()));
            ui.label(format!("Divider period: {}", self.reload));
            ui.label(format!("Divider value: {}", self.period));
            ui.label(format!("Decay level: {}", self.decay));
            let _ = ui.radio(self.loop_flag, "Loop");
        });
    }
}
