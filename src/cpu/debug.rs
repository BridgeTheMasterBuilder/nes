use super::status::Flag::*;
use super::Register::*;
use crate::cpu::Cpu;
use crate::gui::DebugInfo;
use egui::Ui;

impl DebugInfo for Cpu {
    fn print(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Registers");

            ui.separator();

            ui.label(format!(
                "A: {:02X} X: {:02X} Y: {:02X} SP: {:02X} \
                             PC: {:04X} cyc: {}",
                self.regs[A as usize],
                self.regs[X as usize],
                self.regs[Y as usize],
                self.sp,
                self.pc,
                self.real_cyc
            ));

            let _ = ui.radio(self.p[Carry as usize], "Carry");

            let _ = ui.radio(self.p[Zero as usize], "Zero");

            let _ = ui.radio(self.p[InterruptDisable as usize], "InterruptDisable");

            let _ = ui.radio(self.p[Overflow as usize], "Overflow");

            let _ = ui.radio(self.p[Negative as usize], "Negative");
        });
    }
}
