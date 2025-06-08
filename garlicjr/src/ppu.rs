use crate::RandomAccessMemory;

pub struct PPU {
    pub registers: PpuRegisters,
    current_dot: u16,
    vram_enabled: bool,
    vram: RandomAccessMemory,
}

pub struct PpuRegisters {
    pub ly: u8,
    pub scx: u8,
    pub scy: u8,
    pub wx: u8,
    pub wy: u8,
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
                scx: 0,
                scy: 0,
                wx: 0,
                wy: 0,
            },
            current_dot: 0,
            vram_enabled: true,
            vram: RandomAccessMemory::new(0x1FFF),
        }
    }

    pub fn tick(&mut self) {
        self.vram_enabled =
            self.current_dot < 80 || self.current_dot > 368 || self.registers.ly >= 144;

        self.current_dot += 1;
        self.current_dot %= 456;

        if self.current_dot == 0 {
            self.registers.ly += 1;
            self.registers.ly %= 154;
        }
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
    fn should_default_registers_to_0() {
        let ppu = PPU::default();
        assert_eq!(ppu.registers.ly, 0);
        assert_eq!(ppu.registers.scx, 0);
        assert_eq!(ppu.registers.scy, 0);
        assert_eq!(ppu.registers.wx, 0);
        assert_eq!(ppu.registers.wy, 0);
    }

    #[rstest]
    fn should_read_0xff_from_vram_while_drawing_pixels(#[values(0, 10, 42, 143)] ly: u8) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
        }

        for _ in 0..DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
            for i in 0..0x1FFF {
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
        ppu.write_vram(address, data);

        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
            for i in 0..0x1FFF {
                let expected = if i == address { data } else { 0 };
                assert_eq!(ppu.read_vram(i), expected);
            }
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
        ppu.write_vram(address, data);

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

    #[rstest]
    fn should_read_valid_data_from_vram_during_vblank(
        #[values(144, 145, 146, 147, 148, 149, 150, 151, 152, 153)] ly: u8,
        #[values(0x1234, 0x0000, 0x1FFF)] address: u16,
        #[values(0x12, 0x42, 0xFE)] data: u8,
    ) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;
        ppu.write_vram(address, data);

        for _ in 0..OAM_SCAN_LENGTH + DRAWING_PIXELS_MAX_LENGTH + HBLANK_MIN_LENGTH {
            ppu.tick();
            for i in 0..0x1FFF {
                let expected = if i == address { data } else { 0 };
                assert_eq!(ppu.read_vram(i), expected);
            }
        }
    }

    #[rstest]
    fn should_increment_ly_after_456_dots(#[values(1, 5, 143, 152)] ly: u8) {
        let mut ppu = PPU::default();
        ppu.registers.ly = ly;

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

        for _ in 0..456 {
            ppu.tick();
        }

        assert_eq!(ppu.registers.ly, 0);
    }
}
