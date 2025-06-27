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

use crate::RandomAccessMemory;

pub struct PPU {
    pub registers: PpuRegisters,
    current_dot: u16,
    vram_enabled: bool,
    vram: RandomAccessMemory,
}

impl Default for PPU {
    fn default() -> Self {
        Self::new()
    }
}

impl PPU {
    pub fn new() -> Self {
        Self {
            registers: PpuRegisters {
                ly: 0,
                lyc: 0,
                scx: 0,
                scy: 0,
                wx: 0,
                wy: 0,
                lcdc: 0,
                stat: 0,
            },
            current_dot: 0,
            vram_enabled: true,
            vram: RandomAccessMemory::new(0x2000),
        }
    }

    pub fn tick(&mut self) {
        if !self.is_ppu_on() {
            return;
        }

        self.vram_enabled =
            self.current_dot < 80 || self.current_dot > 368 || self.registers.ly >= 144;

        self.set_stat_register();

        self.current_dot += 1;
        self.current_dot %= 456;

        if self.current_dot == 0 {
            self.registers.ly += 1;
            self.registers.ly %= 154;
        }
    }

    fn is_ppu_on(&self) -> bool {
        self.registers.lcdc & 0b10000000 > 0
    }

    fn set_stat_register(&mut self) {
        self.registers.stat &= 0b11111000;

        if self.registers.ly == self.registers.lyc {
            self.registers.stat |= 0b00000100;
        }

        let state = match (self.registers.ly, self.current_dot) {
            (144..=153, _) => 0b00000001,
            (_, 0..80) => 0b00000010,
            (_, 80..369) => 0b00000011,
            (_, 369..) => 0b00000000,
        };

        self.registers.stat |= state;
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.vram_enabled {
            self.vram.read(address).unwrap()
        } else {
            0xFF
        }
    }

    pub fn write_vram(&mut self, address: u16, data: u8) {
        self.vram.write(address, data);
    }

    pub fn dump_tile_data(&self) -> ([usize; 2], Vec<u8>) {
        let mut buffer = vec![0u8; 16 * 8 * 24 * 8 * 4];

        let mut last_x = 0;
        let mut last_y = 0;
        for tile_index in 0..16 * 24 {
            let tile = self.dump_tile(tile_index);

            for (y, row) in tile.iter().enumerate() {
                for (x, component) in row.iter().enumerate() {
                    let x = last_x + x;
                    let y = last_y + y;
                    buffer[y * 16 * 8 * 4 + x] = *component;
                }
            }

            last_x += 8 * 4;
            last_x %= 16 * 8 * 4;
            if last_x == 0 {
                last_y += 8;
            }
        }

        ([16 * 8, 24 * 8], buffer)
    }

    fn dump_tile(&self, index: u16) -> Vec<[u8; 32]> {
        let mut tile = vec![];
        let start = index * 8 * 2;
        let end = start + 8 * 2;
        for idx in (start..end).step_by(2) {
            let lsb = self.vram.read(idx).unwrap_or(0);
            let msb = self.vram.read(idx + 1).unwrap_or(0);
            let pixels = to_pixels(lsb, msb);
            tile.push(pixels);
        }

        tile
    }
}

fn to_pixels(lsb: u8, msb: u8) -> [u8; 32] {
    let mut pixels = [0u8; 32];

    let pixel_values = raw_pixel_values(lsb, msb);
    pixels[0..4].copy_from_slice(&map_to_color(pixel_values[0]));
    pixels[4..8].copy_from_slice(&map_to_color(pixel_values[1]));
    pixels[8..12].copy_from_slice(&map_to_color(pixel_values[2]));
    pixels[12..16].copy_from_slice(&map_to_color(pixel_values[3]));
    pixels[16..20].copy_from_slice(&map_to_color(pixel_values[4]));
    pixels[20..24].copy_from_slice(&map_to_color(pixel_values[5]));
    pixels[24..28].copy_from_slice(&map_to_color(pixel_values[6]));
    pixels[28..32].copy_from_slice(&map_to_color(pixel_values[7]));

    pixels
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

fn map_to_color(pixel_value: u8) -> [u8; 4] {
    match pixel_value {
        0 => [160, 160, 160, 255],
        1 => [220, 220, 220, 255],
        2 => [96, 96, 96, 255],
        3 => [0, 0, 0, 255],
        _ => [255, 255, 255, 255],
    }
}

pub struct PpuRegisters {
    pub ly: u8,
    pub lyc: u8,
    pub scx: u8,
    pub scy: u8,
    pub wx: u8,
    pub wy: u8,
    pub lcdc: u8,
    stat: u8,
}

impl PpuRegisters {
    pub fn get_stat(&self) -> u8 {
        self.stat
    }

    pub fn set_stat(&mut self, value: u8) {
        self.stat &= 0b00000111;
        self.stat |= value & 0b11111000;
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const OAM_SCAN_LENGTH: u16 = 80;
    const DRAWING_PIXELS_MAX_LENGTH: u16 = 289;
    const HBLANK_MIN_LENGTH: u16 = 87;

    #[test]
    fn should_default_registers_to_0() {
        let ppu = PPU::default();
        assert_eq!(ppu.registers.ly, 0);
        assert_eq!(ppu.registers.scx, 0);
        assert_eq!(ppu.registers.scy, 0);
        assert_eq!(ppu.registers.wx, 0);
        assert_eq!(ppu.registers.wy, 0);
        assert_eq!(ppu.registers.get_stat(), 0);
        assert_eq!(ppu.registers.lcdc, 0);
    }

    #[test]
    fn should_ignore_last_three_bits_when_writing_to_stat() {
        let mut ppu = PPU::default();
        ppu.registers.set_stat(0b11111111);
        assert_eq!(ppu.registers.get_stat(), 0b11111000);
    }

    #[test]
    fn should_do_nothing_when_lcdc_bit_7_is_0() {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b01010101;

        for _ in 0..456 {
            ppu.tick();
            assert_eq!(ppu.registers.ly, 0);
            assert_eq!(ppu.registers.lyc, 0);
            assert_eq!(ppu.registers.scx, 0);
            assert_eq!(ppu.registers.scy, 0);
            assert_eq!(ppu.registers.wx, 0);
            assert_eq!(ppu.registers.wy, 0);
            assert_eq!(ppu.registers.lcdc, 0b01010101);
            assert_eq!(ppu.registers.get_stat(), 0);
        }
    }

    #[rstest]
    fn should_store_2_in_stat_register_bits_0_and_1_during_oam_scan(
        #[values(0, 10, 42, 143)] ly: u8,
        #[values(0b11111111, 0b00000000, 0b10101010)] stat_begin: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b10000000;
        ppu.registers.ly = ly;
        ppu.registers.set_stat(stat_begin);

        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.registers.get_stat() & 0b00000011, 2);
        }
    }

    #[rstest]
    fn should_store_3_in_stat_register_bits_0_and_1_while_drawing_pixels(
        #[values(0, 10, 42, 143)] ly: u8,
        #[values(0b11111111, 0b00000000, 0b10101010)] stat_begin: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b10000000;
        ppu.registers.ly = ly;
        ppu.registers.set_stat(stat_begin);

        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
        }

        for _ in 0..DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
            assert_eq!(ppu.registers.get_stat() & 0b00000011, 3);
        }
    }

    #[rstest]
    fn should_store_0_in_stat_register_bits_0_and_1_during_hblank(
        #[values(0, 10, 42, 143)] ly: u8,
        #[values(0b11111111, 0b00000000, 0b10101010)] stat_begin: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b10000000;
        ppu.registers.ly = ly;
        ppu.registers.set_stat(stat_begin);

        for _ in 0..OAM_SCAN_LENGTH + DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
        }

        for _ in 0..HBLANK_MIN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.registers.get_stat() & 0b00000011, 0);
        }
    }

    #[rstest]
    fn should_store_1_in_stat_register_bits_0_and_1_during_vblank(
        #[values(144, 145, 146, 147, 148, 149, 150, 151, 152, 153)] ly: u8,
        #[values(0b11111111, 0b00000000, 0b10101010)] stat_begin: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b10000000;
        ppu.registers.ly = ly;
        ppu.registers.set_stat(stat_begin);

        for _ in 0..456 {
            ppu.tick();
            assert_eq!(ppu.registers.get_stat() & 0b00000011, 1);
        }
    }

    #[rstest]
    fn should_store_1_in_stat_register_bit_3_when_ly_and_lyc_are_identical(
        #[values(0, 50, 143, 144, 153)] ly: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b10000000;
        ppu.registers.ly = ly;
        ppu.registers.lyc = ly;

        for _ in 0..456 {
            ppu.tick();
            assert!((ppu.registers.get_stat() & 0b00000100) > 0);
        }
    }

    #[rstest]
    fn should_not_change_upper_5_bits_of_stat_register(
        #[values(0b00000000, 0b11111000, 0b10101000, 0b01010000)] stat: u8,
        #[values(0, 50, 143, 144, 153)] ly: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.lcdc = 0b10000000;
        ppu.registers.ly = ly;
        ppu.registers.set_stat(stat);

        for _ in 0..456 {
            ppu.tick();
            assert_eq!(ppu.registers.get_stat() & 0b11111000, stat);
        }
    }

    #[rstest]
    fn should_read_0xff_from_vram_while_drawing_pixels(#[values(0, 10, 42, 143)] ly: u8) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        ppu.registers.lcdc = 0b10000000;
        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
        }

        for _ in 0..DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
            for i in 0..=0x1FFF {
                assert_eq!(ppu.read_vram(i), 0xFF);
            }
        }
    }

    #[rstest]
    fn should_read_valid_data_from_vram_during_oam_scan(
        #[values(0, 10, 42, 143)] ly: u8,
        #[values(0x1234, 0x0000, 0x1FFF)] address: u16,
        #[values(0x12, 0x42, 0xFE)] data: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        ppu.registers.lcdc = 0b10000000;
        ppu.write_vram(address, data);

        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.read_vram(address), data);
        }
    }

    #[rstest]
    fn should_read_valid_data_from_vram_during_hblank(
        #[values(0, 10, 42, 143)] ly: u8,
        #[values(0x1234, 0x0000, 0x1FFF)] address: u16,
        #[values(0x12, 0x42, 0xFE)] data: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        ppu.registers.lcdc = 0b10000000;
        ppu.write_vram(address, data);

        for _ in 0..OAM_SCAN_LENGTH + DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
        }

        for _ in 0..HBLANK_MIN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.read_vram(address), data);
        }
    }

    #[rstest]
    fn should_read_valid_data_from_vram_during_vblank(
        #[values(144, 145, 146, 147, 148, 149, 150, 151, 152, 153)] ly: u8,
        #[values(0x1234, 0x0000, 0x1FFF)] address: u16,
        #[values(0x12, 0x42, 0xFE)] data: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        ppu.registers.lcdc = 0b10000000;
        ppu.write_vram(address, data);

        for _ in 0..OAM_SCAN_LENGTH + DRAWING_PIXELS_MAX_LENGTH + HBLANK_MIN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.read_vram(address), data);
        }
    }

    #[rstest]
    fn should_increment_ly_after_456_dots(#[values(1, 5, 143, 152)] ly: u8) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        ppu.registers.lcdc = 0b10000000;

        for _ in 0..456 {
            ppu.tick();
        }

        let expected = ly + 1;
        assert_eq!(ppu.registers.ly, expected);
    }

    #[test]
    fn should_reset_ly_to_0_after_vblank() {
        let mut ppu = PPU::default();
        ppu.registers.ly = 153;
        ppu.registers.lcdc = 0b10000000;

        for _ in 0..456 {
            ppu.tick();
        }

        assert_eq!(ppu.registers.ly, 0);
    }
}
