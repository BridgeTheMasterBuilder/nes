use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Irq {
    pub counter: u8,
    pub enabled: bool,
    pub occurred: bool,
    pub reload: u8,
    old_counter: u8,
    filter: usize,
}

impl Irq {
    pub fn new() -> Self {
        Self {
            counter: 0,
            enabled: false,
            occurred: false,
            reload: 0,
            old_counter: 0,
            filter: 0,
        }
    }

    pub fn clock(&mut self) {
        // println!("counter before: {}", self.counter);

        self.old_counter = self.counter;

        if self.counter == 0 {
            self.counter = self.reload;
        } else {
            // println!("decrementing counter");
            self.counter -= 1;
        }
        //
        // println!("counter after: {}", self.counter);
        //
        // println!("IRQ enabled = {}", self.enabled);

        if self.old_counter == 1 && self.enabled {
            // if self.enabled {
            // if self.enabled {
            // println!("Triggering IRQ");
            self.occurred = true;
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mmc3 {
    pub banks: [usize; 6],
    pub irq: Irq,
    pub selected_bank_register: usize,
}

impl Mmc3 {
    pub fn new() -> Self {
        Self {
            banks: [0; 6],
            irq: Irq::new(),
            selected_bank_register: 0,
        }
    }
}

impl DebugInfo for Mmc3 {
    fn print(&self, ui: &mut Ui) {
        ui.label("MMC3");
        ui.label(format!("IRQ counter reload: {}", self.irq.reload));
        ui.label(format!("IRQ counter: {}", self.irq.counter));
        let _ = ui.radio(self.irq.enabled, "IRQ enabled");
    }
}
