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

use egui::{Color32, ColorImage};

const TILE_DATA_BEGIN: u16 = 0x8000;

pub fn tile_data(ram: &[u8], buffer: &mut ColorImage) {
    let mut last_x = 0;
    let mut last_y = 0;
    for tile_index in 0..16 * 24 {
        let tile = get_tile(ram, tile_index);

        for (y, row) in tile.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let x = last_x + x;
                let y = last_y + y;
                buffer.pixels[y * buffer.size[0] + x] = *pixel;
            }
        }

        last_x += 8;
        last_x %= buffer.size[0];
        if last_x == 0 {
            last_y += 8;
        }
    }
}

fn get_tile(ram: &[u8], index: u16) -> Vec<[Color32; 8]> {
    let mut tile = vec![];
    let start = (index * 8 * 2 + TILE_DATA_BEGIN) as usize;
    let end = start + 8 * 2;
    for idx in (start..end).step_by(2) {
        let lsb = ram[idx];
        let msb = ram[idx + 1];
        let pixels = to_pixels(lsb, msb);
        tile.push(pixels);
    }

    tile
}

fn to_pixels(lsb: u8, msb: u8) -> [Color32; 8] {
    let pixel_values = raw_pixel_values(lsb, msb);
    [
        map_to_color(pixel_values[0]),
        map_to_color(pixel_values[1]),
        map_to_color(pixel_values[2]),
        map_to_color(pixel_values[3]),
        map_to_color(pixel_values[4]),
        map_to_color(pixel_values[5]),
        map_to_color(pixel_values[6]),
        map_to_color(pixel_values[7]),
    ]
}

fn raw_pixel_values(lsb: u8, msb: u8) -> [u8; 8] {
    let p0 = ((msb & 0b10000000) >> 6) | ((lsb & 0b10000000) >> 7);
    let p1 = ((msb & 0b01000000) >> 5) | ((lsb & 0b01000000) >> 6);
    let p2 = ((msb & 0b00100000) >> 4) | ((lsb & 0b00100000) >> 5);
    let p3 = ((msb & 0b00010000) >> 3) | ((lsb & 0b00010000) >> 4);
    let p4 = ((msb & 0b00001000) >> 2) | ((lsb & 0b00001000) >> 3);
    let p5 = ((msb & 0b00000100) >> 1) | ((lsb & 0b00000100) >> 2);
    let p6 = (msb & 0b00000010) | ((lsb & 0b00000010) >> 1);
    let p7 = ((msb & 0b00000001) << 1) | (lsb & 0b00000001);
    [p0, p1, p2, p3, p4, p5, p6, p7]
}

fn map_to_color(pixel_value: u8) -> Color32 {
    match pixel_value {
        0b00000000 => Color32::GRAY,
        0b00000001 => Color32::LIGHT_GRAY,
        0b00000010 => Color32::DARK_GRAY,
        0b00000011 => Color32::BLACK,
        _ => {
            println!("{pixel_value}");
            Color32::CYAN
        }
    }
}
