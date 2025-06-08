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
            vram: RandomAccessMemory::new(0x1FFF),
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
        ppu.registers.stat = stat_begin;

        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.registers.stat & 0b00000011, 2);
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
        ppu.registers.stat = stat_begin;

        for _ in 0..OAM_SCAN_LENGTH {
            ppu.tick();
        }

        for _ in 0..DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
            assert_eq!(ppu.registers.stat & 0b00000011, 3);
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
        ppu.registers.stat = stat_begin;

        for _ in 0..OAM_SCAN_LENGTH + DRAWING_PIXELS_MAX_LENGTH {
            ppu.tick();
        }

        for _ in 0..HBLANK_MIN_LENGTH {
            ppu.tick();
            assert_eq!(ppu.registers.stat & 0b00000011, 0);
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
        ppu.registers.stat = stat_begin;

        for _ in 0..456 {
            ppu.tick();
            assert_eq!(ppu.registers.stat & 0b00000011, 1);
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
            assert!((ppu.registers.stat & 0b00000100) > 0);
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
        ppu.registers.stat = stat;

        for _ in 0..456 {
            ppu.tick();
            assert_eq!(ppu.registers.stat & 0b11111000, stat);
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
        ppu.registers.lcdc = 0b10000000;
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
        ppu.registers.lcdc = 0b10000000;
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
        ppu.registers.lcdc = 0b10000000;
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
