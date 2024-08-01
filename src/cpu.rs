use crate::opcode::{Dest8Bit, Opcode};
use crate::{Bus, ReadWriteMode};

pub struct SharpSM83 {
    pub registers: Registers,
    current_tick: u8,
    opcode: Opcode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
}

impl SharpSM83 {
    pub fn new() -> SharpSM83 {
        SharpSM83 {
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                f: 0,
                h: 0,
                l: 0,
                stack_pointer: 0,
                program_counter: 0,
            },
            current_tick: 1,
            opcode: Opcode::NOP,
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        match self.current_tick {
            1 => self.write_program_counter(bus),
            2 => self.read_opcode(bus),
            3 => self.increment_program_counter(),
            _ => self.execute_opcode(bus),
        }

        self.current_tick += 1;
    }

    fn write_program_counter(&mut self, bus: &mut Bus) {
        bus.address = self.registers.program_counter;
    }

    fn read_opcode(&mut self, bus: &mut Bus) {
        self.opcode = Opcode::decode(bus.data);
    }

    fn increment_program_counter(&mut self) {
        self.registers.program_counter += 1;
    }

    fn execute_opcode(&mut self, bus: &mut Bus) {
        match self.opcode {
            Opcode::NOP => self.no_op(),
            Opcode::LDRI8(dest) => self.ld_r_n8(dest, bus),
            Opcode::Unimplemented(_) => {}
        }
    }

    fn no_op(&mut self) {
        self.current_tick = 0;
    }

    fn ld_r_n8(&mut self, destination: Dest8Bit, bus: &mut Bus) {
        match self.current_tick {
            5 => {
                bus.mode = ReadWriteMode::Read;
                bus.address = self.registers.program_counter;
            }
            8 => {
                self.write_to_register(destination, bus.data);
                self.increment_program_counter();
                self.current_tick = 0;
            }
            _ => (),
        }
    }

    fn write_to_register(&mut self, dest: Dest8Bit, data: u8) {
        match dest {
            Dest8Bit::A => self.registers.a = data,
            Dest8Bit::B => self.registers.b = data,
            Dest8Bit::C => self.registers.c = data,
            Dest8Bit::D => self.registers.d = data,
            Dest8Bit::E => self.registers.e = data,
            Dest8Bit::H => self.registers.h = data,
            Dest8Bit::L => self.registers.l = data,
            _ => panic!("Tried to write to invalid destination"),
        };
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    use crate::{opcode::Dest8Bit, ReadWriteMode};

    #[test]
    fn should_return_true_when_register_contents_are_the_same() {
        let lhs = Registers {
            a: 2,
            b: 8,
            c: 2,
            d: 0,
            e: 4,
            f: 1,
            h: 7,
            l: 6,
            stack_pointer: 100,
            program_counter: 300,
        };

        let rhs = Registers {
            a: 2,
            b: 8,
            c: 2,
            d: 0,
            e: 4,
            f: 1,
            h: 7,
            l: 6,
            stack_pointer: 100,
            program_counter: 300,
        };

        assert_eq!(lhs == rhs, true);
    }

    #[test]
    fn should_return_false_when_register_contents_differ() {
        let lhs = Registers {
            a: 2,
            b: 8,
            c: 2,
            d: 0,
            e: 4,
            f: 1,
            h: 7,
            l: 6,
            stack_pointer: 100,
            program_counter: 300,
        };

        let rhs = Registers {
            a: 3,
            b: 9,
            c: 3,
            d: 1,
            e: 5,
            f: 2,
            h: 8,
            l: 7,
            stack_pointer: 101,
            program_counter: 301,
        };

        assert_eq!(lhs == rhs, false);
    }

    #[test]
    fn should_clone_registers() {
        let registers = Registers {
            a: 2,
            b: 8,
            c: 2,
            d: 0,
            e: 4,
            f: 1,
            h: 7,
            l: 6,
            stack_pointer: 100,
            program_counter: 300,
        };

        let clone = registers.clone();
        assert_eq!(clone, registers);
    }

    #[test]
    fn initializes_registers_to_0() {
        let cpu = SharpSM83::new();
        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.b, 0);
        assert_eq!(cpu.registers.c, 0);
        assert_eq!(cpu.registers.d, 0);
        assert_eq!(cpu.registers.e, 0);
        assert_eq!(cpu.registers.f, 0);
        assert_eq!(cpu.registers.h, 0);
        assert_eq!(cpu.registers.l, 0);
        assert_eq!(cpu.registers.stack_pointer, 0);
        assert_eq!(cpu.registers.program_counter, 0);
    }

    #[test]
    fn should_write_program_counter_to_bus_on_tick_1() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;
        cpu.tick(&mut bus);

        assert_eq!(bus.address, 0x5555);
        assert_eq!(bus.mode, ReadWriteMode::Read);
    }

    #[test]
    fn should_read_opcode_from_bus_on_tick_2() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.tick(&mut bus);

        bus.data = 0x26;
        cpu.tick(&mut bus);

        assert_eq!(cpu.opcode, Opcode::decode(0x26));
    }

    #[test]
    fn should_not_write_to_bus_on_tick_2() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;
        cpu.tick(&mut bus);

        bus.address = 0x1234;
        bus.data = 0x42;
        cpu.tick(&mut bus);

        assert_eq!(bus.address, 0x1234);
        assert_eq!(bus.data, 0x42);
    }

    #[test]
    fn should_increment_the_program_counter_on_tick_3() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;

        cpu.tick(&mut bus);
        assert_eq!(cpu.registers.program_counter, 0x5555);

        cpu.tick(&mut bus);
        assert_eq!(cpu.registers.program_counter, 0x5555);

        cpu.tick(&mut bus);
        assert_eq!(cpu.registers.program_counter, 0x5556);
    }

    #[test]
    fn should_do_nothing_on_tick_4_when_opcode_is_no_op() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;

        let mut expected_registers = cpu.registers.clone();
        expected_registers.program_counter = 0x5556;

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        assert_eq!(expected_registers, cpu.registers);
    }

    #[test]
    fn should_write_program_counter_after_no_op() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        assert_eq!(bus.address, 0x5555);

        cpu.tick(&mut bus);

        assert_eq!(bus.address, 0x5556);
    }

    #[test]
    fn should_write_program_counter_to_bus_on_tick_5_of_ld_r_n8() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        assert_eq!(bus.address, 0x5556);
        assert_eq!(bus.mode, ReadWriteMode::Read);
    }

    #[test]
    fn should_load_into_register_a_on_tick_8_of_ld_r_n8() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;

        cpu.tick(&mut bus);

        bus.data = 0b00111110;
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        bus.data = 0x42;

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        assert_eq!(cpu.registers.a, 0);
        cpu.tick(&mut bus);

        assert_eq!(cpu.registers.a, 0x42);
    }

    #[rstest]
    #[case(Dest8Bit::A, 0b00111110)]
    #[case(Dest8Bit::B, 0b00000110)]
    #[case(Dest8Bit::C, 0b00001110)]
    #[case(Dest8Bit::D, 0b00010110)]
    #[case(Dest8Bit::E, 0b00011110)]
    #[case(Dest8Bit::H, 0b00100110)]
    #[case(Dest8Bit::L, 0b00101110)]
    fn should_load_into_given_register_on_tick_8_of_ld_r_n8(
        #[case] destination: Dest8Bit,
        #[case] opcode: u8,
    ) {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x5555;

        let registers_before = cpu.registers.clone();

        cpu.tick(&mut bus);

        bus.data = opcode;

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        bus.data = 0x42;

        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);

        let destination_map = vec![
            (Dest8Bit::A, cpu.registers.a, registers_before.a),
            (Dest8Bit::B, cpu.registers.b, registers_before.b),
            (Dest8Bit::C, cpu.registers.c, registers_before.c),
            (Dest8Bit::D, cpu.registers.d, registers_before.d),
            (Dest8Bit::E, cpu.registers.e, registers_before.e),
            (Dest8Bit::H, cpu.registers.h, registers_before.h),
            (Dest8Bit::L, cpu.registers.l, registers_before.l),
        ];

        destination_map
            .iter()
            .for_each(|(dest, register, old_register)| {
                if *dest == destination {
                    assert_eq!(*register, 0x42);
                } else {
                    assert_eq!(*register, *old_register);
                }
            });
    }

    #[test]
    fn should_not_modify_registers_before_tick_8_of_ld_r_n8() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x1234;
        bus.data = 0b00111110;

        for _ in 0..7 {
            cpu.tick(&mut bus);
        }

        let expected = Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            stack_pointer: 0,
            program_counter: 0x1235,
        };

        assert_eq!(cpu.registers, expected);
    }

    #[test]
    fn should_increment_the_program_counter_after_tick_8_of_ld_r_n8() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x1000;
        bus.data = 0b00111110;

        for _ in 0..7 {
            cpu.tick(&mut bus);
        }

        assert_eq!(cpu.registers.program_counter, 0x1001);

        cpu.tick(&mut bus);

        assert_eq!(cpu.registers.program_counter, 0x1002);
    }
}
