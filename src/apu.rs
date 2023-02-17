use std::cell::Cell;

use serde::{Deserialize, Serialize};

use dmc::Dmc;
use noise::Noise;
use pulse::sweep::Mode;
use pulse::Pulse;
use triangle::Triangle;

use crate::util::bit::Bit;

mod debug;
mod dmc;
mod envelope;
mod noise;
mod pulse;
mod triangle;

const LENGTHS: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const CYCLES: [[usize; 6]; 2] = [
    [7457, 14913, 22371, 29828, 29829, 29830],
    [7457, 14913, 22371, 29829, 37281, 37282],
];

#[derive(Serialize, Deserialize, Clone)]
enum SequenceMode {
    FourStep,
    FiveStep,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Apu {
    pub dmc: Dmc,
    pub interrupt: Cell<bool>,
    clockrate: u32,
    frame_counter: usize,
    irq_inhibit: bool,
    mode: SequenceMode,
    cycles: usize,
    noise: Noise,
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    request_cycles: usize,
    reset_requested: bool,
}

impl Apu {
    pub fn new(clockrate: u32) -> Self {
        Self {
            dmc: Dmc::new(),
            interrupt: Cell::new(false),
            clockrate,
            cycles: 0,
            frame_counter: 0,
            irq_inhibit: false,
            mode: SequenceMode::FourStep,
            noise: Noise::new(),
            pulse1: Pulse::new(Mode::OnesComplement, clockrate),
            pulse2: Pulse::new(Mode::TwosComplement, clockrate),
            request_cycles: 0,
            reset_requested: false,
            triangle: Triangle::new(clockrate),
        }
    }

    pub fn output(&self) -> [f32; 6] {
        let pulse1 = self.pulse1.output();
        let pulse2 = self.pulse2.output();
        let triangle = self.triangle.output();
        let noise = self.noise.output();
        let dmc = self.dmc.output();
        let mixed = Self::mix([pulse1, pulse2, triangle, noise, dmc]);

        [pulse1, pulse2, triangle, noise, dmc, mixed]
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x4000..=0x4014 => 0,
            0x4015 => {
                let dmc_interrupt = self.dmc.irq as u8;
                let frame_interrupt = self.interrupt.get() as u8;
                let dmc_active = (self.dmc.length > 0) as u8;
                let noise_len = (self.noise.length > 0) as u8;
                let triangle_len = (self.triangle.length > 0) as u8;
                let pulse1_len = (self.pulse1.length > 0) as u8;
                let pulse2_len = (self.pulse2.length > 0) as u8;

                self.interrupt.replace(false);

                dmc_interrupt << 7
                    | frame_interrupt << 6
                    | dmc_active << 4
                    | noise_len << 3
                    | triangle_len << 2
                    | pulse2_len << 1
                    | pulse1_len
            }
            _ => unreachable!(),
        }
    }

    pub fn _set_clockrate(&mut self, clockrate: u32) {
        self.clockrate = clockrate;
    }

    pub fn tick(&mut self) {
        if self.reset_requested {
            if self.request_cycles == 0 {
                self.update_envs();
                self.triangle.dec_lin();

                self.reset_requested = false;
                self.frame_counter = 0;
                self.cycles = 0;
            } else {
                self.request_cycles -= 1;
            }
        }

        self.triangle.tick();

        if self.cycles % 2 == 0 {
            self.pulse1.tick();
            self.pulse2.tick();
            self.noise.tick();
            self.dmc.tick();
        }

        let mode_idx = match self.mode {
            SequenceMode::FourStep => 0,
            SequenceMode::FiveStep => 1,
        };

        if self.cycles == CYCLES[mode_idx][self.frame_counter] {
            self.tock();
        }

        self.cycles += 1;
    }

    pub fn tock(&mut self) {
        match self.mode {
            SequenceMode::FourStep => {
                match self.frame_counter {
                    0 => {
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    1 => {
                        self.update_lens();
                        self.update_sweep();
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    2 => {
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    3 => {
                        self.interrupt.replace(!self.irq_inhibit);
                    }
                    4 => {
                        self.update_lens();
                        self.update_sweep();
                        self.update_envs();
                        self.triangle.dec_lin();

                        self.interrupt.replace(!self.irq_inhibit);
                    }
                    5 => {
                        self.interrupt.replace(!self.irq_inhibit);

                        self.cycles = 0;
                    }
                    _ => unreachable!(),
                }

                self.frame_counter = (self.frame_counter + 1) % 6;
            }
            SequenceMode::FiveStep => {
                match self.frame_counter {
                    0 => {
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    1 => {
                        self.update_lens();
                        self.update_sweep();
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    2 => {
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    3 => {}
                    4 => {
                        self.update_lens();
                        self.update_sweep();
                        self.update_envs();
                        self.triangle.dec_lin();
                    }
                    5 => {
                        self.cycles = 0;
                    }

                    _ => unreachable!(),
                }

                self.frame_counter = (self.frame_counter + 1) % 6;
            }
        }
    }

    fn mix(audio_samples: [f32; 5]) -> f32 {
        let [pulse1, pulse2, triangle, noise, dmc] = audio_samples;

        let pulse_out = if pulse1 == 0.0 && pulse2 == 0.0 {
            0.0
        } else {
            95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0)
        };
        let tnd_out = if triangle == 0.0 && noise == 0.0 && dmc == 0.0 {
            0.0
        } else {
            159.79 / ((1.0 / ((triangle / 8227.0) + (noise / 12241.0) + (dmc / 22638.0))) + 100.0)
        };

        let mixed = pulse_out + tnd_out;

        mixed
    }

    fn update_lens(&mut self) {
        self.pulse1.dec_len();
        self.pulse2.dec_len();
        self.triangle.dec_len();
        self.noise.dec_len();
    }

    fn update_sweep(&mut self) {
        self.pulse1.update_sweep();
        self.pulse2.update_sweep();
    }

    fn update_envs(&mut self) {
        self.pulse1.update_env();
        self.pulse2.update_env();
        self.noise.update_env();
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => self.pulse1.write(0, data),
            0x4001 => self.pulse1.write(1, data),
            0x4002 => self.pulse1.write(2, data),
            0x4003 => self.pulse1.write(3, data),
            0x4004 => self.pulse2.write(0, data),
            0x4005 => self.pulse2.write(1, data),
            0x4006 => self.pulse2.write(2, data),
            0x4007 => self.pulse2.write(3, data),
            0x4008 => self.triangle.write(0, data),
            0x4009 => {}
            0x400A => self.triangle.write(2, data),
            0x400B => self.triangle.write(3, data),
            0x400C => self.noise.write(0, data),
            0x400D => {}
            0x400E => self.noise.write(2, data),
            0x400F => self.noise.write(3, data),
            0x4010 => self.dmc.write(0, data),
            0x4011 => self.dmc.write(1, data),
            0x4012 => self.dmc.write(2, data),
            0x4013 => self.dmc.write(3, data),
            0x4015 => {
                let dmc_enabled = data.bit(4);
                let noise_enabled = data.bit(3);
                let triangle_enabled = data.bit(2);
                let pulse2_enabled = data.bit(1);
                let pulse1_enabled = data.bit(0);

                self.dmc.irq = false;

                self.pulse1.enable(pulse1_enabled);
                self.pulse2.enable(pulse2_enabled);
                self.triangle.enable(triangle_enabled);
                self.noise.enable(noise_enabled);
                self.dmc.enable(dmc_enabled);
            }
            0x4017 => {
                let mode = data.bit(7);
                let irq_inhibit = data.bit(6);

                self.mode = if mode {
                    SequenceMode::FiveStep
                } else {
                    SequenceMode::FourStep
                };

                self.irq_inhibit = irq_inhibit;

                if mode {
                    self.update_lens();
                    self.update_sweep();

                    self.reset_requested = true;

                    self.request_cycles = if self.cycles % 2 == 0 { 3 } else { 4 };
                }

                if irq_inhibit {
                    self.interrupt.replace(false);
                }
            }
            _ => {}
        }
    }
}
