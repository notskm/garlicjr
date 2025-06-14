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

pub struct RandomAccessMemory(Vec<u8>);

impl RandomAccessMemory {
    pub fn new(size: u16) -> Self {
        Self(vec![0; size as usize])
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        self.0.get(address as usize).copied()
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if let Some(data) = self.0.get_mut(address as usize) {
            *data = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn should_initialize_to_0(#[values(u16::MIN, 0x1234, u16::MAX)] size: u16) {
        let ram = RandomAccessMemory::new(size);
        for i in 0..size {
            assert_eq!(ram.read(i), Some(0));
        }
    }

    #[rstest]
    fn should_read_new_value_after_write(
        #[values(u16::MIN, 0x1234, u16::MAX - 1)] address: u16,
        #[values(u8::MIN, u8::MAX, 123, 92)] data: u8,
    ) {
        let mut ram = RandomAccessMemory::new(u16::MAX);
        ram.write(address, data);
        assert_eq!(ram.read(address), Some(data));
    }

    #[rstest]
    fn should_not_change_memory_locations_that_were_not_written_to(
        #[values(u16::MIN, 0x1234, u16::MAX - 1)] address: u16,
        #[values(u8::MIN, u8::MAX, 123, 92)] data: u8,
    ) {
        let mut ram = RandomAccessMemory::new(u16::MAX);
        ram.write(address, data);
        for i in 0..u16::MAX {
            if i != address {
                assert_eq!(ram.read(i), Some(0));
            }
        }
    }

    #[rstest]
    fn should_return_none_when_reading_outside_range(
        #[values(0x1234, 0x7854, 0xABCD)] size: u16,
        #[values(0, 5, 10, 15)] offset: u16,
    ) {
        let ram = RandomAccessMemory::new(size);
        assert_eq!(ram.read(size + offset), None);
    }

    #[rstest]
    fn should_ignore_writes_above_the_maximum_address(
        #[values(0x1234, 0x7854, 0xABCD)] size: u16,
        #[values(0, 5, 10, 15)] offset: u16,
    ) {
        let mut ram = RandomAccessMemory::new(size);
        ram.write(size + offset, 10);
        for i in 0..size {
            assert_eq!(ram.read(i), Some(0));
        }
    }
}
