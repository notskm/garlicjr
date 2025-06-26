/*
    Copyright 2024-2025 notskm

    This file is part of garlicjr.

    garlicjr is free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by the Free
    Software Foundation, either version 3 of the License, or (at your option)
    any later version.

    garlicjr is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
    more details.

    You should have received a copy of the GNU General Public License along
    with garlicjr. If not, see <https: //www.gnu.org/licenses/>.
*/

use egui::Grid;
use garlicjr::System;

pub fn cpu_gui(ui: &mut egui::Ui, dmg: &mut System, running: &mut bool) {
    Grid::new("CPU Register Grid")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("AF");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.a).range(0..=255));
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.f).range(0..=255));
            });
            ui.end_row();

            ui.label("BC");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.b).range(0..=255));
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.c).range(0..=255));
            });
            ui.end_row();

            ui.label("DE");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.d).range(0..=255));
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.e).range(0..=255));
            });
            ui.end_row();

            ui.label("HL");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.h).range(0..=255));
                ui.add(egui::DragValue::new(&mut dmg.cpu.registers.l).range(0..=255));
            });
            ui.end_row();

            ui.label("PC");
            ui.add(
                egui::DragValue::new(&mut dmg.cpu.registers.program_counter)
                    .range(0..=u16::MAX)
                    .hexadecimal(4, true, true),
            );
            ui.end_row();

            ui.label("SP");
            ui.add(
                egui::DragValue::new(&mut dmg.cpu.registers.stack_pointer)
                    .range(0..=u16::MAX)
                    .hexadecimal(4, true, true),
            );
            ui.end_row();

            if ui.button("Step").clicked() {
                dmg.run_cycle();
            }

            ui.checkbox(running, "Run");
        });
}
