use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::gui::DebugInfo;
use sweep::{Mode, Sweep};

use crate::util::bit::Bit;

use super::envelope::Envelope;
use super::LENGTHS;

pub mod sweep;

const SEQUENCES: [[bool; 8]; 4] = [
    [false, true, false, false, false, false, false, false],
    [false, true, true, false, false, false, false, false],
    [false, true, true, true, true, false, false, false],
    [true, false, false, true, true, true, true, true],
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pulse {
    pub(super) length: u8,
    clockrate: u32,
    duty: usize,
    enabled: bool,
    envelope: Envelope,
    halt: bool,
    mode: Mode,
    pos: usize,
    reload: u16,
    sweep: Sweep,
    timer: u16,
}

impl Pulse {
    pub fn new(mode: Mode, clockrate: u32) -> Self {
        Self {
            length: 0,
            clockrate,
            duty: 0,
            enabled: false,
            envelope: Envelope::new(),
            halt: false,
            mode,
            pos: 0,
            reload: 0,
            sweep: Sweep::new(mode),
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
        if !self.enabled || self.sweep.muted(self.reload) || self.length == 0 {
            0.0
        } else if SEQUENCES[self.duty][self.pos] {
            self.envelope.volume() as f32
        } else {
            0.0
        }
    }

    pub fn tick(&mut self) {
        self.sweep.tick(self.reload);

        if self.timer == 0 {
            self.timer = self.reload;
            self.pos = (self.pos + 1) % 8;
        } else {
            self.timer -= 1;
        }
    }

    pub fn update_env(&mut self) {
        self.envelope.tick();
    }

    pub fn update_sweep(&mut self) {
        self.reload = self.sweep.tock(self.reload);
    }

    pub fn write(&mut self, reg: usize, data: u8) {
        match reg {
            0 => {
                let duty = data.bits(6, 7) as usize;
                let halt = data.bit(5);
                let constant = data.bit(4);
                let period = data.bits_abs(0, 3);

                self.duty = duty;
                self.halt = halt;
                self.envelope.loop_flag = halt;
                self.envelope.constant = constant;
                self.envelope.set_period(period);
            }
            1 => {
                let enabled = data.bit(7);
                let period = data.bits(4, 6) + 1;
                let negate = data.bit(3);
                let shift = data.bits_abs(0, 2);

                self.sweep.enabled = enabled;
                self.sweep.set_period(period);
                self.sweep.negate = negate;
                self.sweep.shift = shift;

                self.sweep.reload_flag = true;
            }
            2 => {
                let data = data as u16;

                self.reload = self.reload.bits_abs(8, 10) | data;
            }
            3 => {
                let data = data as u16;

                let len_idx = data.bits(3, 7);
                let timer_high = data.bits_abs(0, 2);

                self.reload = self.reload.bits_abs(0, 7) | timer_high << 8;

                if self.enabled {
                    self.length = LENGTHS[len_idx as usize];
                }

                self.pos = 0;
                self.envelope.start = true;
            }
            _ => unreachable!(),
        }
    }
}

impl DebugInfo for Pulse {
    fn print(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Pulse {}",
                if matches!(self.mode, Mode::OnesComplement) {
                    "I"
                } else {
                    "II"
                }
            ));
            ui.vertical(|ui| {
                let _ = ui.radio(self.enabled, "Enabled");
                ui.label(format!(
                    "Duty cycle: {}",
                    match self.duty {
                        0 => "1/8",
                        1 => "1/4",
                        2 => "1/2",
                        3 => "3/4",
                        _ => unreachable!(),
                    }
                ));
                ui.label(format!("Sequencer position: {}", self.pos));
                let _ = ui.radio(self.halt, "Length counter halt");
                let reload_freq = self.clockrate / (16 * (self.reload as u32 + 1));
                ui.label(format!(
                    "Timer reload: {} ({} Hz)",
                    self.reload, reload_freq
                ));
                ui.label(format!("Timer : {} ", self.timer));
                ui.label(format!("Length counter: {} ", self.length));
                let _ = ui.radio(self.sweep.muted(self.reload), "Sweep muting channel");
            });
            ui.separator();
            self.sweep.print(ui);
            ui.separator();
            self.envelope.print(ui);
        });
    }
}
