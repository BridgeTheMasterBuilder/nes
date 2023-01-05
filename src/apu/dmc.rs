use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

const RATES: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dmc {
    pub addr: u16,
    pub irq: bool,
    pub(super) length: u16,
    addr_reload: u16,
    bits_remaining: u8,
    enabled: bool,
    irq_enabled: bool,
    length_reload: u16,
    loop_flag: bool,
    output: u8,
    sample_buf: Option<u8>,
    timer: u16,
    timer_reload: u16,
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            addr: 0,
            irq: false,
            length: 0,
            addr_reload: 0,
            bits_remaining: 0,
            enabled: false,
            irq_enabled: false,
            length_reload: 0,
            loop_flag: false,
            output: 0,
            sample_buf: None,
            timer: 0,
            timer_reload: 0,
        }
    }

    pub fn buffer_should_be_filled(&self) -> bool {
        self.sample_buf.is_none() && self.length > 0
    }

    pub fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;

        if !self.enabled {
            self.length = 0;
        }
    }

    pub fn fill_buffer(&mut self, data: u8) {
        self.sample_buf = Some(data);

        if self.addr == 0xFFFF {
            self.addr = 0x8000;
        } else {
            self.addr += 1;
        }

        self.length -= 1;

        if self.length == 0 {
            if self.loop_flag {
                self.addr = self.addr_reload;
                self.length = self.length_reload;
            } else if self.irq_enabled {
                self.irq = true;
            }
        }
    }

    pub fn output(&self) -> f32 {
        self.output as f32
    }

    pub fn tick(&mut self) {
        if self.enabled {
            if self.timer == 0 {
                self.timer = self.timer_reload;
            } else {
                self.timer -= 1;
            }
        }

        if self.timer == 0 {
            if self.bits_remaining == 0 {
                self.bits_remaining = 8;

                self.enabled = self.sample_buf.is_some();
            }

            if self.enabled && let Some(sample_buf) = self.sample_buf {
                let delta = sample_buf.bit(0);

                if delta {
                    if self.output <= 125 {
                        self.output += 2;
                    }
                } else {
                    self.output = self.output.saturating_sub(2);
                }

                self.sample_buf = self.sample_buf.map(|buf| buf >> 1);
            }

            self.bits_remaining -= 1;

            if self.bits_remaining == 0 {
                self.sample_buf = None;
            }
        }
    }

    pub fn write(&mut self, reg: usize, data: u8) {
        match reg {
            0 => {
                let irq_enabled = data.bit(7);
                let loop_flag = data.bit(6);

                let rate_idx = data.bits_abs(0, 3) as usize;

                self.irq_enabled = irq_enabled;
                self.loop_flag = loop_flag;
                self.timer_reload = RATES[rate_idx] / 2;
            }
            1 => {
                let load = data.bits_abs(0, 6);

                self.output = load;
            }
            2 => {
                let data = u16::from(data);
                let address = 0xC000 + (data * 64);

                self.addr = address;
                self.addr_reload = address;
            }
            3 => {
                let data = u16::from(data);
                let length = data * 16 + 1;

                self.length = length;
                self.length_reload = length;
            }
            _ => unreachable!(),
        }
    }
}

impl DebugInfo for Dmc {
    fn print(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("DMC");
            ui.vertical(|ui| {
                let _ = ui.radio(self.enabled, "Enabled");
                let _ = ui.radio(self.irq_enabled, "IRQ Enabled");
                let _ = ui.radio(self.irq, "IRQ asserted");
                let _ = ui.radio(self.loop_flag, "Loop");
                ui.label(format!("Rate: {} cycles per sample", self.timer_reload));
                ui.label(format!(
                    "Sample buffer: {}",
                    self.sample_buf
                        .map_or_else(|| String::from("Empty"), |sample| sample.to_string())
                ));
                ui.label(format!(
                    "Sample address: {} (sample starts at {})",
                    self.addr, self.addr_reload
                ));
                ui.label(format!(
                    "Sample length: {} ({} left)",
                    self.length_reload, self.length
                ));
            });
        });
    }
}
