#![allow(clippy::new_without_default)]
#![feature(let_chains)]

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use bus::mapper::MapperTrait;
use controller::Btn::*;
use screen::Screen;
use speaker::Speaker;

use crate::core::EmulatorCore;
use crate::cpu::status::Flag::InterruptDisable;
use crate::cpu::Cpu;
use crate::util::Config;

mod apu;
mod bus;
pub mod cartridge;
mod controller;
pub mod core;
pub mod cpu;
pub mod gui;
mod ppu;
mod screen;
mod speaker;
pub mod util;

const SAMPLERATE: u32 = 44100;
const OSCILLOSCOPE_SAMPLES: usize = (SAMPLERATE as usize / 60) * 2;
const OSCILLOSCOPE_DEPTH: usize = 60;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Halted,
    Running,
    RestartFrame,
    SingleStep,
    StepScanline,
    StepFrame,
}

pub struct Nes {
    actual_cycle: usize,
    dpad_in_use: bool,
    event_queue: EventPump,
    filename: String,
    save_state_slot: usize,
    save_states: Vec<Option<Cpu>>,
    screen: Screen,
    speaker: Speaker,
    _controller: Option<GameController>,
}

impl Nes {
    pub fn new(clockrate: u32, config: &Config) -> Result<Self, Box<dyn Error>> {
        let sdl_context = sdl2::init()?;

        let controller_subsystem = sdl_context.game_controller()?;

        let available = controller_subsystem.num_joysticks()?;

        // TODO multiple controllers
        let _controller = (0..available)
            .find_map(|id| {
                if !controller_subsystem.is_game_controller(id) {
                    return None;
                }

                controller_subsystem.open(id).ok()
            })
            .or_else(|| {
                eprintln!("WARNING: No controller detected. Input not available");
                None
            });

        let filename = config.filename.clone();

        Ok(Self {
            actual_cycle: 0,
            dpad_in_use: false,
            event_queue: sdl_context.event_pump()?,
            save_states: Self::load_save_states(&filename),
            filename,
            save_state_slot: 0,
            screen: Screen::new(&sdl_context)?,
            speaker: Speaker::new(&sdl_context, clockrate)?,
            _controller,
        })
    }

    pub fn handle_input(&mut self, core: &mut EmulatorCore) {
        for e in self.event_queue.poll_iter() {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    core.cpu.bus.save_data();

                    core.request_termination = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } => {
                    let cpu = core.cpu.clone();

                    self.save_states[self.save_state_slot] = Some(cpu);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F7),
                    ..
                } => {
                    if let Some(cpu) = &self.save_states[self.save_state_slot] {
                        core.cpu = cpu.clone();
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    core.state = if let State::Running = core.state {
                        State::Halted
                    } else {
                        State::Running
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => core.state = State::StepFrame,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => core.state = State::SingleStep,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    core.state = State::StepScanline;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => core.cpu.disasm = !core.cpu.disasm,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => core.cpu.reset(),
                Event::KeyDown {
                    keycode: Some(Keycode::M),
                    ..
                } => self.speaker.muted = !self.speaker.muted,
                Event::ControllerButtonDown { button, .. } => {
                    let controller = core.cpu.bus.controller();

                    match button {
                        Button::A => controller.press(A),
                        Button::B => controller.press(B),
                        Button::Back => controller.press(Select),
                        Button::Start => controller.press(Start),
                        Button::DPadUp => {
                            self.dpad_in_use = true;
                            controller.release(Down);
                            controller.press(Up);
                        }
                        Button::DPadDown => {
                            self.dpad_in_use = true;
                            controller.release(Up);
                            controller.press(Down);
                        }
                        Button::DPadLeft => {
                            self.dpad_in_use = true;
                            controller.release(Left);
                            controller.press(Left);
                        }
                        Button::DPadRight => {
                            self.dpad_in_use = true;
                            controller.release(Right);
                            controller.press(Right);
                        }
                        _ => {}
                    }
                }
                Event::ControllerButtonUp { button, .. } => {
                    let controller = core.cpu.bus.controller();

                    match button {
                        Button::A => controller.release(A),
                        Button::B => controller.release(B),
                        Button::Back => controller.release(Select),
                        Button::Start => controller.release(Start),
                        Button::DPadUp => {
                            self.dpad_in_use = false;
                            controller.release(Up);
                        }
                        Button::DPadDown => {
                            self.dpad_in_use = false;
                            controller.release(Down);
                        }
                        Button::DPadLeft => {
                            self.dpad_in_use = false;
                            controller.release(Left);
                        }
                        Button::DPadRight => {
                            self.dpad_in_use = false;
                            controller.release(Right);
                        }
                        _ => {}
                    }
                }
                Event::ControllerAxisMotion { axis, value, .. } => {
                    let deadzone = 5000;

                    if self.dpad_in_use {
                        continue;
                    }

                    let controller = core.cpu.bus.controller();

                    match axis {
                        sdl2::controller::Axis::LeftX => {
                            controller.release(Left);
                            controller.release(Right);

                            if value < -deadzone {
                                controller.press(Left);
                            } else if value > deadzone {
                                controller.press(Right);
                            }
                        }
                        sdl2::controller::Axis::LeftY => {
                            controller.release(Up);
                            controller.release(Down);

                            if value < -deadzone {
                                controller.press(Up);
                            } else if value > deadzone {
                                controller.press(Down);
                            }
                        }
                        _ => continue,
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => self.save_state_slot = 0,
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => self.save_state_slot = 1,
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => self.save_state_slot = 2,
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => self.save_state_slot = 3,
                Event::KeyDown {
                    keycode: Some(Keycode::Num5),
                    ..
                } => self.save_state_slot = 4,
                Event::KeyDown {
                    keycode: Some(Keycode::Num6),
                    ..
                } => self.save_state_slot = 5,
                Event::KeyDown {
                    keycode: Some(Keycode::Num7),
                    ..
                } => self.save_state_slot = 6,
                Event::KeyDown {
                    keycode: Some(Keycode::Num8),
                    ..
                } => self.save_state_slot = 7,
                Event::KeyDown {
                    keycode: Some(Keycode::Num9),
                    ..
                } => self.save_state_slot = 8,
                _ => {}
            }
        }
    }

    pub fn render(&mut self, core: &mut EmulatorCore) -> Result<(), Box<dyn Error>> {
        let ppu = &core.cpu.bus.ppu();

        self.screen.render(&ppu.fb)
    }

    pub fn run(&mut self, core: &mut EmulatorCore) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();

        self.handle_input(core);
        self.update(core)?;
        self.render(core)?;

        let end = Instant::now();

        core.update_fps(end.duration_since(start));

        Ok(())
    }

    pub fn save_save_states(&self) {
        if self.save_states.iter().all(|state| state.is_none()) {
            return;
        }

        let mut name = Path::new(&self.filename).to_path_buf();

        name.set_extension("stat");

        let mut save_file = File::create(name).expect("Unable to open/create save state file.");

        if let Ok(data) = serde_json::to_vec(&self.save_states) {
            save_file
                .write_all(data.as_slice())
                .unwrap_or_else(|_| eprintln!("Unable to write save states to disk."));
        }
    }

    pub fn update(&mut self, core: &mut EmulatorCore) -> Result<(), Box<dyn Error>> {
        let mut old_state = core.state;
        let mut new_state = old_state;

        loop {
            match new_state {
                State::Running => {
                    while new_state != State::RestartFrame {
                        old_state = new_state;

                        new_state = self.execute(core, new_state)?;
                    }
                }
                State::SingleStep => {
                    old_state = new_state;

                    new_state = self.execute(core, new_state)?;

                    if new_state != State::RestartFrame {
                        new_state = State::Halted
                    }
                }
                State::StepScanline => {
                    loop {
                        old_state = new_state;

                        let ppu_cyc = core.cpu.bus.ppu().dot;

                        new_state = self.execute(core, new_state)?;

                        if core.cpu.bus.ppu().dot < ppu_cyc || new_state == State::RestartFrame {
                            break;
                        }
                    }

                    if new_state != State::RestartFrame {
                        new_state = State::Halted;
                    }
                }
                State::StepFrame => {
                    loop {
                        old_state = new_state;

                        new_state = self.execute(core, new_state)?;

                        if core.cpu.cyc >= core.cycles_per_frame || new_state == State::RestartFrame
                        {
                            break;
                        }
                    }

                    if new_state != State::RestartFrame {
                        new_state = State::Halted;
                    }
                }
                State::RestartFrame => {
                    new_state = if old_state != State::Running {
                        State::Halted
                    } else {
                        State::Running
                    };

                    self.speaker.flush()?;

                    core.adjust_cycles_per_frame();
                    self.actual_cycle = 0;
                    break;
                }
                _ => break,
            }
        }

        core.state = new_state;

        Ok(())
    }

    fn execute(&mut self, core: &mut EmulatorCore, state: State) -> Result<State, Box<dyn Error>> {
        while self.actual_cycle < core.cpu.cyc {
            let output = core.cpu.bus.apu().output();

            self.speaker.push_sample(output)?;

            if let Some(output) = self.speaker.output.take() {
                let diameter = OSCILLOSCOPE_DEPTH / 2;

                for (channel, output) in output.iter().enumerate() {
                    core.sample_buffers[channel]
                        .push(diameter as f32 - output * diameter as f32 * 0.5);
                }
            }

            self.actual_cycle += 1;
        }

        core.cpu.fetch_decode_and_execute()?;

        let nmi_occurred = core.cpu.bus.ppu().nmi_occurred.get();
        let dmc_irq = core.cpu.bus.apu().interrupt.get();
        let mmc3_irq = if let Some(mmc3) = &mut core.cpu.bus.ppu().mmc3.as_ref() {
            mmc3.irq.occurred
        } else {
            false
        };

        let dma = core.cpu.bus.bus().dma_interrupt;

        if let Some(page) = dma {
            core.cpu.dma(page);
            core.cpu.bus.bus().dma_interrupt = None;
        }

        let nmis = &mut core.cpu.bus.ppu().interrupts;
        //
        // if nmi_occurred || dmc_irq || mmc3_irq {
        //     println!(
        //         "Interrupt disable = {}",
        //         core.cpu.p[InterruptDisable as usize]
        //     );
        // }

        if nmi_occurred && !nmis.is_empty() {
            nmis.pop();

            core.cpu.handle_nmi();
        } else if !core.cpu.p[InterruptDisable as usize] && dmc_irq {
            core.cpu.handle_irq();
        } else if !core.cpu.p[InterruptDisable as usize] && mmc3_irq {
            // println!("Handling IRQ");
            core.cpu.bus.ppu().mmc3.as_mut().unwrap().irq.occurred = false;
            core.cpu.handle_irq();
        }

        Ok(if core.cpu.cyc > core.cycles_per_frame {
            State::RestartFrame
        } else {
            state
        })
    }

    fn load_save_states(filename: &str) -> Vec<Option<Cpu>> {
        let mut name = Path::new(filename).to_path_buf();

        name.set_extension("stat");

        let mut states = vec![None; 8];

        if let Ok(mut save_file) = File::open(name) {
            let mut data = Vec::new();

            if save_file.read_to_end(&mut data).is_ok() {
                states = serde_json::from_slice(&data)
                    .map_err(|_| eprintln!("WARNING: Save state data invalid"))
                    .unwrap();
            } else {
                eprintln!("WARNING: Failed to load save states from disk");
            }
        }

        states
    }
}
