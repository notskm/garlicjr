#[derive(Default)]
pub struct Timer {
    pub registers: Registers,
    tima_counter: u16,
}

impl Timer {
    pub fn tick(&mut self) {
        if self.should_increment_tima() {
            self.registers.tima = self.registers.tima.wrapping_add(1);
        }

        self.tima_counter += 1;
        if self.tima_counter >= self.increment_frequency() {
            self.tima_counter = 0;
        }
    }

    fn should_increment_tima(&self) -> bool {
        self.is_tima_enabled() && self.is_time_to_increment_tima()
    }

    fn is_tima_enabled(&self) -> bool {
        self.registers.tac & 0b00000100 > 0
    }

    fn is_time_to_increment_tima(&self) -> bool {
        self.tima_counter == self.increment_frequency() - 1
    }

    fn increment_frequency(&self) -> u16 {
        const M_CYCLE_LENGTH: u16 = 4;

        M_CYCLE_LENGTH
            * match self.registers.tac & 0b00000011 {
                0b00000000 => 256,
                0b00000001 => 4,
                0b00000010 => 16,
                0b00000011 => 64,
                _ => u16::MAX,
            }
    }
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

    #[rstest]
    #[case(0b00000100, 256)]
    #[case(0b00000101, 4)]
    #[case(0b00000110, 16)]
    #[case(0b00000111, 64)]
    fn should_increment_tima_when_tac_bit_2_is_on(
        #[case] tac: u8,
        #[case] increment_frequency_in_m_cycles: u16,
    ) {
        let mut timer = Timer::default();
        timer.registers.set_tac(tac);
        timer.registers.tma = 0;

        let t_cycles = increment_frequency_in_m_cycles * 4;

        for expected in 1..=10 {
            // Skip a cycle to detect the exact moment tima increments
            for _ in 0..t_cycles - 1 {
                timer.tick();
                assert_eq!(timer.registers.tima, expected - 1);
            }

            // Detect the increment
            timer.tick();
            assert_eq!(timer.registers.tima, expected);
        }
    }

    #[rstest]
    fn should_not_panic_when_running_for_a_long_time(#[values(0b00000000, 0b00000101)] tac: u8) {
        let mut timer = Timer::default();
        timer.registers.tac = tac;

        for _ in 0..2 {
            for _ in 0..u16::MAX {
                timer.tick();
            }
        }
    }
}
