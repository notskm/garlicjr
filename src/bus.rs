#[derive(PartialEq, Debug, Clone, Copy)]
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
}
