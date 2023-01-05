use crate::cartridge::{Cartridge, MapperType};
use crate::cpu::Cpu;
use crate::gui::DebugInfo;
use crate::util::Config;
use crate::State;
use egui::Ui;
use std::error::Error;
use std::time::Duration;

pub struct EmulatorCore {
    pub cpu: Cpu,
    pub cycles_per_frame: usize,
    pub mapper_type: MapperType,
    pub request_termination: bool,
    pub running: bool,
    pub state: State,
    _clockrate: u32,
    adjust: i8,
    avg_fps: f64,
    fps: f64,
    frame: usize,
}

impl EmulatorCore {
    pub fn new(config: &Config, clockrate: u32) -> Result<Self, Box<dyn Error>> {
        let cartridge = Cartridge::new(&config.filename)?;

        let mapper_type = cartridge.mapper_type;

        Ok(Self {
            cpu: Cpu::new(config, cartridge, clockrate, false),
            cycles_per_frame: (341 * 262) / 3,
            mapper_type,
            request_termination: false,
            running: true,
            state: State::Running,
            _clockrate: clockrate,
            avg_fps: 60.0,
            fps: 60.0,
            // For calculating the number of cycles to run this frame
            adjust: 0,
            frame: 0,
        })
    }

    pub fn adjust_cycles_per_frame(&mut self) {
        let cycles_per_frame = self.cycles_per_frame as isize;

        let rem = {
            let cpu_cyc = self.cpu.cyc;
            self.cpu.cyc = 0;

            (cpu_cyc as isize - cycles_per_frame) as i8
        };

        self.frame = (self.frame + 1) % 3;

        if self.frame == 0 {
            self.adjust = 2;
        } else {
            self.adjust = 0;
        }

        self.adjust -= rem;

        // TODO use clockrate to calculate number of cycles to run
        self.cycles_per_frame = ((341 * 262) / 3 + self.adjust as i32) as usize;
    }

    pub fn update_fps(&mut self, dt: Duration) {
        self.fps = Duration::from_secs(1).as_nanos() as f64 / dt.as_nanos() as f64;

        self.avg_fps = (self.avg_fps + self.fps) / 2.0;

        // if frame % (60 * 60) == 0 {
        //     clockrate = ((CLOCKRATE as f64) / (avg_fps / 60.0)) as i32;
        // }
    }
}

impl DebugInfo for EmulatorCore {
    fn print(&self, ui: &mut Ui) {
        ui.label(format!("FPS: {:.3}", self.fps));
    }
}
