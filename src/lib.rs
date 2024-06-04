pub struct SharpSM83 {
    pub registers: Registers,
}

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
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ReadWriteMode {
    Read,
    Write,
}

pub struct Bus {
    pub data: u8,
    pub address: u16,
    pub mode: ReadWriteMode,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            data: 0,
            address: 0,
            mode: ReadWriteMode::Read,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_initialize_data_to_0() {
        let bus = Bus::new();
        assert_eq!(bus.data, 0);
    }

    #[test]
    fn should_initialize_address_to_0() {
        let bus = Bus::new();
        assert_eq!(bus.address, 0);
    }

    #[test]
    fn should_initialize_mode_to_read() {
        let bus = Bus::new();
        assert_eq!(bus.mode, ReadWriteMode::Read);
    }

    #[test]
    fn should_be_5_when_set_to_5() {
        let mut bus = Bus::new();
        bus.data = 5u8;
        assert_eq!(bus.data, 5u8);
    }

    #[test]
    fn should_be_0x1234_when_set_to_0x1234() {
        let mut bus = Bus::new();
        bus.address = 0x1234u16;
        assert_eq!(bus.address, 0x1234u16);
    }

    #[test]
    fn should_be_read_when_set_to_read() {
        let mut bus = Bus::new();
        bus.mode = ReadWriteMode::Read;
        assert_eq!(bus.mode, ReadWriteMode::Read);
    }

    #[test]
    fn should_be_write_when_set_to_write() {
        let mut bus = Bus::new();
        bus.mode = ReadWriteMode::Write;
        assert_eq!(bus.mode, ReadWriteMode::Write);
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
}
