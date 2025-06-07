use crate::RandomAccessMemory;

pub struct PPU {
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
            current_dot: 0,
            vram_enabled: true,
            vram: RandomAccessMemory::new(0x1FFF),
        }
    }

    pub fn tick(&mut self) {
        self.vram_enabled = self.current_dot < 80 || self.current_dot > 368;

        self.current_dot += 1;
        self.current_dot %= 456;
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
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const OAM_SCAN_LENGTH: u16 = 80;
    const DRAWING_PIXELS_MAX_LENGTH: u16 = 289;
    const HBLANK_MIN_LENGTH: u16 = 87;

    #[test]
    fn should_read_0xff_from_vram_while_drawing_pixels() {
        let mut ppu = PPU::default();
        for _ in 0..144 {
            for _ in 0..OAM_SCAN_LENGTH {
                ppu.tick();
            }

            for _ in 0..DRAWING_PIXELS_MAX_LENGTH {
                ppu.tick();
                for i in 0..0x1FFF {
                    assert_eq!(ppu.read_vram(i), 0xFF);
                }
            }

            for _ in 0..HBLANK_MIN_LENGTH {
                ppu.tick();
            }
        }
    }

    #[rstest]
    fn should_read_valid_data_from_vram_during_oam_scan(
        #[values(0x1234, 0x0000, 0x1FFF)] address: u16,
        #[values(0x12, 0x42, 0xFE)] data: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.write_vram(address, data);

        for _ in 0..144 {
            for _ in 0..OAM_SCAN_LENGTH {
                ppu.tick();
                for i in 0..0x1FFF {
                    let expected = if i == address { data } else { 0 };
                    assert_eq!(ppu.read_vram(i), expected);
                }
            }

            for _ in 0..DRAWING_PIXELS_MAX_LENGTH + HBLANK_MIN_LENGTH {
                ppu.tick();
            }
        }
    }

    #[rstest]
    fn should_read_valid_data_from_vram_during_hblank(
        #[values(0x1234, 0x0000, 0x1FFF)] address: u16,
        #[values(0x12, 0x42, 0xFE)] data: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.write_vram(address, data);

        for _ in 0..144 {
            for _ in 0..OAM_SCAN_LENGTH + DRAWING_PIXELS_MAX_LENGTH {
                ppu.tick();
            }

            for _ in 0..HBLANK_MIN_LENGTH {
                ppu.tick();
                for i in 0..0x1FFF {
                    let expected = if i == address { data } else { 0 };
                    assert_eq!(ppu.read_vram(i), expected);
                }
            }
        }
    }
}
