#[derive(Default)]
pub struct Timer {
    pub registers: Registers,
}

pub struct Registers {
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            tima: 0,
            tma: 0,
            tac: 0b11111000,
        }
    }
}

impl Registers {
    pub fn set_tac(&mut self, value: u8) {
        self.tac = 0b11111000 | (value & 0b00000111)
    }

    pub fn get_tac(&self) -> u8 {
        self.tac
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn should_default_tac_to_0b11111000() {
        let timer = Timer::default();
        assert_eq!(timer.registers.get_tac(), 0b11111000);
    }

    #[test]
    fn should_default_tima_to_0() {
        let timer = Timer::default();
        assert_eq!(timer.registers.tima, 0);
    }

    #[test]
    fn should_default_tma_to_0() {
        let timer = Timer::default();
        assert_eq!(timer.registers.tma, 0);
    }

    #[rstest]
    #[case(0b00000000, 0b11111000)]
    #[case(0b00000001, 0b11111001)]
    #[case(0b00000010, 0b11111010)]
    #[case(0b00000100, 0b11111100)]
    #[case(0b00000111, 0b11111111)]
    fn should_only_set_bottom_3_bits_of_tac_register(#[case] tac: u8, #[case] expected: u8) {
        let mut timer = Timer::default();
        timer.registers.set_tac(tac);
        assert_eq!(timer.registers.get_tac(), expected);
    }
}
