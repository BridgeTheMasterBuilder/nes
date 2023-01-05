use crate::apu::{Apu, SequenceMode};
use crate::gui::DebugInfo;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub(super) enum Menu {
    #[default]
    Registers,
    Visualizer,
}

impl DebugInfo for Apu {
    fn print(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Registers").clicked() {
                self.selected_menu.replace(Menu::Registers);
            };

            if ui.button("Visualizer").clicked() {
                self.selected_menu.replace(Menu::Visualizer);
            };
        });

        match self.selected_menu.get() {
            Menu::Registers => {
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
            Menu::Visualizer => {}
        }
    }
}
