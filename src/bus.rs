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
