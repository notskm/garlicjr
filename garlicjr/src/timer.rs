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

/// A timer that produces interrupts on overflow.
#[derive(Default)]
pub struct Timer {
    /// The timer's registers
    ///
    /// The registers should be memory mapped according to the Pan Docs:
    /// <https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#timer-and-divider-registers>
    pub registers: TimerRegisters,
    tima_counter: u16,
    request_interrupt: bool,
}

impl Timer {
    /// Runs the timer for 1 T-Cycle (1/4 M-cycle) and requests interrupts
    ///
    /// After a call to this function, the timer may request an interrupt.
    /// The subsequent call may disable the interrupt, request.
    /// See [Timer::interrupt_requested] for details.
    ///
    /// # Panics
    /// This function does not panic.
    ///
    /// # Examples
    /// ```
    /// use garlicjr::Timer;
    /// let mut timer = Timer::default();
    /// timer.tick();
    /// ```
    pub fn tick(&mut self) {
        self.request_interrupt = false;

        if self.should_increment_tima() {
            let (new_tima, overflow) = self.registers.tima.overflowing_add(1);

            self.registers.tima = if overflow {
                self.request_interrupt = true;
                self.registers.tma
            } else {
                new_tima
            }
        }

        self.tima_counter += 1;
        if self.tima_counter >= self.increment_frequency() {
            self.tima_counter = 0;
        }
    }

    /// Returns whether a timer interrupt should be triggered
    ///
    /// In a full system, the CPU may reject interrupt requests.
    ///
    /// Calls to [Timer::tick] will enable and disable this intterupt request out of
    /// sync with the CPU. To avoid missing interrupt requests, the return
    /// value of this function should be bitwise ORed onto the CPU's intterupt
    /// request line.
    //
    /// # Panics
    /// This function does not panic.
    ///
    /// # Examples
    /// ```
    /// use garlicjr::Timer;
    /// let mut timer = Timer::default();
    /// timer.registers.tima = 0xFF; // Prepare to overflow
    /// timer.registers.tma = 0xFF; // Overflow on every increment
    /// timer.registers.set_tac(0b00000101); // increment every 4 M-cycles
    /// for _ in 0..16 {
    ///     timer.tick();
    /// }
    /// assert!(timer.interrupt_requested());
    /// ```
    pub fn interrupt_requested(&self) -> bool {
        self.request_interrupt
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

/// [Timer]'s register file, determines the behavior of [Timer]
///
/// See the Pan Docs for more details:
/// <https://gbdev.io/pandocs/Timer_and_Divider_Registers.html#timer-and-divider-registers>
pub struct TimerRegisters {
    /// Increments at a frequency determined by [TimerRegisters::set_tac]
    pub tima: u8,

    /// Determines when timer interrupts are requested
    pub tma: u8,

    tac: u8,
}

impl Default for TimerRegisters {
    /// Returns a set of [TimerRegisters] with each register
    /// set to 0. Note that [TimerRegisters::get_tac] returns `0b11111000`,
    /// since only the bottom 3 bits of the TAC register are relevant.
    fn default() -> Self {
        Self {
            tima: 0,
            tma: 0,
            tac: 0b11111000,
        }
    }
}

impl TimerRegisters {
    /// Sets the TAC register, ignoring the top 5 bits
    ///
    /// Determines how often [TimerRegisters::tima] increments
    pub fn set_tac(&mut self, value: u8) {
        self.tac = 0b11111000 | (value & 0b00000111)
    }

    /// Returns the value of the TAC register
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

    #[rstest]
    fn should_reset_tima_to_tma_on_overflow(#[values(0x00, 0xFF, 0xFE, 0x42)] tma: u8) {
        let mut timer = Timer::default();
        timer.registers.tma = tma;
        timer.registers.tac = 0b00000101;

        while timer.registers.tima < 0xFF {
            timer.tick();
        }

        for _ in 0..16 {
            timer.tick();
        }

        assert_eq!(timer.registers.tima, timer.registers.tma);
    }

    #[rstest]
    fn should_not_request_interrupt_only_when_tima_does_not_overflow(
        #[values(0x00, 0xFF, 0xFE, 0x42)] tma: u8,
    ) {
        let mut timer = Timer::default();
        timer.registers.tma = tma;
        timer.registers.tac = 0b00000101;
        assert!(!timer.interrupt_requested());

        for _ in 0..5 {
            while timer.registers.tima < 0xFF {
                timer.tick();
                assert!(!timer.interrupt_requested());
            }

            for _ in 0..15 {
                timer.tick();
                assert!(!timer.interrupt_requested());
            }

            timer.tick();
            assert!(timer.interrupt_requested());
        }
    }
}
