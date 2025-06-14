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

use egui::RichText;

pub fn ram_table(
    id_salt: impl std::hash::Hash,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    ram: &mut [u8],
    program_counter: u16,
) {
    let font_size = ctx
        .style()
        .text_styles
        .get(&egui::TextStyle::Monospace)
        .unwrap()
        .size;
    let font_width = font_size / 2f32 + 1f32;

    const ROW_LENGTH: usize = 16;

    egui_extras::TableBuilder::new(ui)
        .id_salt(id_salt)
        .striped(true)
        .column(
            egui_extras::Column::auto()
                .resizable(false)
                .at_least(font_width * 4f32),
        )
        .columns(egui_extras::Column::auto().resizable(false), ROW_LENGTH)
        .header(30.0, |mut header| {
            // Address column
            header.col(|_| {});

            // Offset columns
            for i in 0..16 {
                header.col(|ui| {
                    let text = format!("{:02X}", i);
                    let rich_text = RichText::new(text).monospace().strong();
                    ui.label(rich_text);
                });
            }
        })
        .body(|body| {
            body.rows(font_size, ram.len() / ROW_LENGTH, |mut row| {
                let row_index = row.index();

                // Address column
                row.col(|ui| {
                    let address = row_index * ROW_LENGTH;
                    let label_text = format!("{:04X}", address);
                    let label_rich = RichText::new(label_text).monospace().strong();
                    ui.label(label_rich);
                });

                // Data columns
                for col_index in 0..ROW_LENGTH {
                    row.col(|ui| {
                        let ram_offset = row_index * ROW_LENGTH + col_index;
                        let ram_value = ram[ram_offset];

                        let ram_value_text = format!("{:02X}", ram_value);
                        let mut rich_text = egui::RichText::new(ram_value_text).monospace();

                        if ram_offset == program_counter as usize {
                            rich_text = rich_text
                                .color(egui::Color32::BLACK)
                                .background_color(egui::Color32::WHITE);
                        }

                        ui.label(rich_text);
                    });
                }
            });
        });
}
