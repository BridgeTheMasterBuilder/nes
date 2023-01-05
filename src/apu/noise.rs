use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::gui::DebugInfo;
use lfsr::Lfsr;

use crate::util::bit::Bit;

use super::envelope::Envelope;
use super::LENGTHS;

mod lfsr;

const PERIODS: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Noise {
    pub(super) length: u8,
    enabled: bool,
    envelope: Envelope,
    halt: bool,
    lfsr: Lfsr,
    mode: bool,
    reload: u16,
    timer: u16,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            length: 0,
            enabled: false,
            envelope: Envelope::new(),
            halt: false,
            lfsr: Lfsr::new(),
            mode: false,
            reload: 0,
            timer: 0,
        }
    }

    pub fn dec_len(&mut self) {
        if self.halt {
            return;
        }

        self.length = self.length.saturating_sub(1);
    }

    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.length = 0;
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || self.lfsr.output() || self.length == 0 {
            0.0
        } else {
            self.envelope.volume() as f32
        }
    }

    pub fn tick(&mut self) {
        if self.timer == 0 {
            self.timer = self.reload;
            self.lfsr.tick(self.mode);
        } else {
            self.timer -= 1;
        }
    }

    pub fn update_env(&mut self) {
        self.envelope.tick();
    }

    pub fn write(&mut self, reg: usize, data: u8) {
        match reg {
            0 => {
                let halt = data.bit(5);
                let constant = data.bit(4);
                let period = data.bits_abs(0, 3);

                self.halt = halt;
                self.envelope.loop_flag = halt;
                self.envelope.constant = constant;
                self.envelope.set_period(period);
            }
            2 => {
                let mode = data.bit(7);
                let reload_idx = data.bits_abs(0, 3);

                self.mode = mode;
                self.reload = PERIODS[reload_idx as usize];
            }
            3 => {
                let len_idx = data.bits(3, 7);

                if self.enabled {
                    self.length = LENGTHS[len_idx as usize];
                }

                self.envelope.start = true;
            }
            _ => unreachable!(),
        }
    }
}

impl DebugInfo for Noise {
    fn print(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Noise");
            ui.vertical(|ui| {
                let _ = ui.radio(self.enabled, "Enabled");
                let _ = ui.radio(self.halt, "Length counter halt");
                let _ = ui.radio(self.mode, "Mode");
                ui.label(format!("Timer : {} ", self.timer));
                ui.label(format!("Length counter: {} ", self.length));
            });
            ui.separator();
            self.envelope.print(ui);
            ui.separator();
            self.lfsr.print(ui);
        });
    }
}
