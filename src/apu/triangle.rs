use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

use super::LENGTHS;

const SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Triangle {
    pub(super) length: u8,
    clockrate: u32,
    enabled: bool,
    halt: bool,
    linear_counter: u8,
    linear_counter_reload: u8,
    pos: usize,
    reload_flag: bool,
    timer: u16,
    timer_reload: u16,
}

impl Triangle {
    pub fn new(clockrate: u32) -> Self {
        Self {
            length: 0,
            clockrate,
            enabled: false,
            halt: false,
            linear_counter: 0,
            linear_counter_reload: 0,
            pos: 0,
            reload_flag: false,
            timer: 0,
            timer_reload: 0,
        }
    }

    pub fn dec_len(&mut self) {
        if self.halt {
            return;
        }

        self.length = self.length.saturating_sub(1);
    }

    pub fn dec_lin(&mut self) {
        if self.reload_flag {
            self.linear_counter = self.linear_counter_reload;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.halt {
            self.reload_flag = false;
        }
    }

    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.length = 0;
        }
    }

    pub fn output(&self) -> f32 {
        if self.timer_reload < 2 {
            0.0
        } else {
            SEQUENCE[self.pos] as f32
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled || self.linear_counter == 0 || self.length == 0 {
            return;
        }

        if self.timer == 0 {
            self.timer = self.timer_reload;
            self.pos = (self.pos + 1) % 32;
        } else {
            self.timer -= 1;
        }
    }

    pub fn write(&mut self, reg: usize, data: u8) {
        match reg {
            0 => {
                let halt = data.bit(7);
                let linear_counter = data.bits_abs(0, 6);

                self.halt = halt;
                self.linear_counter = linear_counter;
                self.linear_counter_reload = linear_counter;
            }
            2 => {
                let data = data as u16;

                self.timer_reload = self.timer_reload.bits_abs(8, 10) | data;
            }
            3 => {
                let data = data as u16;

                self.timer_reload = self.timer_reload.bits_abs(0, 7) | data.bits_abs(0, 2) << 8;
                let len_idx = data.bits(3, 7);

                if self.enabled {
                    self.length = LENGTHS[len_idx as usize];
                }

                self.reload_flag = true;
            }
            _ => unreachable!(),
        }
    }
}

impl DebugInfo for Triangle {
    fn print(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Triangle");
            ui.vertical(|ui| {
                let _ = ui.radio(self.enabled, "Enabled");
                ui.label(format!("Sequencer position: {}", self.pos));
                ui.label(format!("Linear counter: {} ", self.linear_counter));
                let _ = ui.radio(self.halt, "Control flag");
                let _ = ui.radio(self.reload_flag, "Linear counter reload flag");
                let reload_freq = self.clockrate / (16 * (self.timer_reload as u32 + 1));
                ui.label(format!(
                    "Timer reload: {} ({} Hz)",
                    self.timer_reload, reload_freq
                ));
                ui.label(format!("Timer : {} ", self.timer));
                ui.label(format!("Length counter: {} ", self.length));
                let _ = ui.radio(self.halt, "Length counter halt");
            });
        });
    }
}
