use crate::apu::{Apu, SequenceMode};
use crate::gui::DebugInfo;
use egui::Ui;

impl DebugInfo for Apu {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Registers");
            ui.separator();
            self.pulse1.print(ui);
            ui.separator();
            self.pulse2.print(ui);
            ui.separator();
            self.triangle.print(ui);
            ui.separator();
            self.noise.print(ui);
            ui.separator();
            self.dmc.print(ui);
            ui.separator();
            ui.vertical(|ui| {
                ui.label("Frame counter");
                let _ = ui.radio(self.interrupt.get(), "IRQ");
                ui.label(format!(
                    "Mode: {}",
                    if matches!(self.mode, SequenceMode::FourStep) {
                        "4-step"
                    } else {
                        "5-step"
                    }
                ));
                let _ = ui.radio(self.irq_inhibit, "IRQ inhibit");
            })
        });
    }
}
