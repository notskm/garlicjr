use crate::opcode::Opcode;
use crate::Bus;

pub struct SharpSM83 {
    pub registers: Registers,
    current_tick: u8,
    opcode: Opcode,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
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
            _ => self.execute_opcode(),
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

    fn execute_opcode(&mut self) {
        match self.opcode {
            Opcode::NOP => self.no_op(),
            _ => (),
        }

        self.current_tick = 0;
    }

    fn no_op(&mut self) {
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadWriteMode;

    use super::*;

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
}
