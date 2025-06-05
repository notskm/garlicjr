use crate::{Bus, SharpSM83};

pub struct System {
    pub cpu: SharpSM83,
    pub bus: Bus,
}

impl System {
    pub fn new() -> Self {
        Self {
            cpu: SharpSM83::new(),
            bus: Bus::new(),
        }
    }

    pub fn run_cycle(&mut self) {
        for _ in 0..4 {
            self.cpu.tick(&mut self.bus);
        }
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn should_return_a_default_system() {
        let default_system = System::default();
        let new_system = System::new();

        assert_eq!(default_system.cpu.registers.a, new_system.cpu.registers.a);
        assert_eq!(default_system.cpu.registers.b, new_system.cpu.registers.b);
        assert_eq!(default_system.cpu.registers.c, new_system.cpu.registers.c);
        assert_eq!(default_system.cpu.registers.d, new_system.cpu.registers.d);
        assert_eq!(default_system.cpu.registers.e, new_system.cpu.registers.e);
        assert_eq!(default_system.cpu.registers.f, new_system.cpu.registers.f);
        assert_eq!(default_system.cpu.registers.h, new_system.cpu.registers.h);
        assert_eq!(default_system.cpu.registers.l, new_system.cpu.registers.l);
        assert_eq!(
            default_system.cpu.registers.program_counter,
            new_system.cpu.registers.program_counter
        );
        assert_eq!(
            default_system.cpu.registers.stack_pointer,
            new_system.cpu.registers.stack_pointer
        );
    }

    #[test]
    fn should_return_a_new_system() {
        let system = System::new();

        assert_eq!(system.cpu.registers.a, 0);
        assert_eq!(system.cpu.registers.b, 0);
        assert_eq!(system.cpu.registers.c, 0);
        assert_eq!(system.cpu.registers.d, 0);
        assert_eq!(system.cpu.registers.e, 0);
        assert_eq!(system.cpu.registers.f, 0);
        assert_eq!(system.cpu.registers.h, 0);
        assert_eq!(system.cpu.registers.l, 0);
        assert_eq!(system.cpu.registers.program_counter, 0);
        assert_eq!(system.cpu.registers.stack_pointer, 0);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    fn should_run_a_cycle(#[case] start_address: u16) {
        let mut system = System::new();
        system.cpu.registers.program_counter = start_address;

        system.run_cycle();

        assert_eq!(system.cpu.registers.program_counter, start_address + 1);
    }
}
