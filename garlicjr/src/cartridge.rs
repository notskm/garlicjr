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

use std::io::Read;

pub struct Cartridge {
    title: String,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum ReadError {
    BadSize { size: usize },
    NonAsciiTitle { bytes: Vec<u8> },
    IoError(std::io::Error),
}

const TITLE_RANGE: std::ops::RangeInclusive<usize> = 0x0134..=0x143;

impl Cartridge {
    pub fn from_reader(mut reader: impl Read) -> Result<Self, ReadError> {
        let mut data: Vec<u8> = vec![];

        let result = reader.read_to_end(&mut data);

        match result {
            Ok(size) if size % 16384 != 0 => {
                return Err(ReadError::BadSize { size });
            }
            Ok(_) => (),
            Err(err) => return Err(ReadError::IoError(err)),
        }

        let mut title_data = data[TITLE_RANGE].to_vec();

        let cgb_enhanced = title_data[title_data.len() - 1] == 0x80;
        let cgb_only = title_data[title_data.len() - 1] == 0xC0;

        if cgb_enhanced || cgb_only {
            title_data.pop();
        }

        if !title_data.is_ascii() {
            return Err(ReadError::NonAsciiTitle { bytes: title_data });
        }

        let title = String::from_utf8(title_data)
            .unwrap()
            .trim_matches('\0')
            .to_string();

        Ok(Cartridge { title, data })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        self.data.get(address as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn should_return_the_software_title(
        #[values("GAME", "ANOTHER GAME", "A MAX SIZE TITLE")] title: &str,
    ) {
        let title_bytes = title.as_bytes();

        let mut cartridge_data = [0u8; 16384];
        cartridge_data[0x134..0x134 + title_bytes.len()].copy_from_slice(title_bytes);

        let cartridge = Cartridge::from_reader(&cartridge_data[..]).unwrap();
        assert_eq!(cartridge.title(), title);
    }

    #[test]
    fn should_not_interpret_bytes_beyond_0x0143_as_title() {
        let title = "THIS TITLE IS TOO LONG";
        let expected = "THIS TITLE IS TO";

        let title_bytes = title.as_bytes();

        let mut cartridge_data = [0u8; 16384];
        cartridge_data[0x134..0x134 + title_bytes.len()].copy_from_slice(title_bytes);

        let cartridge = Cartridge::from_reader(&cartridge_data[..]).unwrap();
        assert_eq!(cartridge.title(), expected);
    }

    #[rstest]
    fn should_return_error_if_title_cannot_be_interpreted_as_ascii(
        #[values("元abc\0\0\0\0\0\0\0\0\0\0", "元気元気元a")] title: &str,
    ) {
        let title_bytes = title.as_bytes();

        let mut cartridge_data = [0u8; 16384];
        cartridge_data[0x134..0x134 + title_bytes.len()].copy_from_slice(title_bytes);

        let result = Cartridge::from_reader(&cartridge_data[..]);
        assert!(matches!(result, Err(ReadError::NonAsciiTitle { bytes }) if bytes == title_bytes));
    }

    #[rstest]
    fn should_return_title_when_address_0x0143_is_0x80() {
        let title = "THIS IS A TITLE";
        let mut title_bytes = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80];
        title_bytes[0..15].copy_from_slice(title.as_bytes());

        let mut cartridge_data = [0u8; 16384];
        cartridge_data[0x134..=0x143].copy_from_slice(&title_bytes);

        let cartridge = Cartridge::from_reader(&cartridge_data[..]).unwrap();
        assert_eq!(cartridge.title(), title)
    }

    #[rstest]
    fn should_return_cartridge_if_the_rom_is_a_multiple_of_16_kib(
        #[values(16384, 16384*2, 16384*3)] size: usize,
    ) {
        let cartridge_data = vec![0u8; size];
        let result = Cartridge::from_reader(&cartridge_data[..]);
        assert!(result.is_ok());
    }

    #[rstest]
    fn should_return_error_if_the_rom_is_not_a_multiple_of_16_kib(
        #[values(1, 1000, 16)] size: usize,
    ) {
        let cartridge_data = vec![0u8; size];
        let result = Cartridge::from_reader(&cartridge_data[..]);
        assert!(matches!(result, Err(ReadError::BadSize{size: bytes}) if bytes == size));
    }

    #[rstest]
    fn should_read_0000_to_7fff_when_rom_has_no_memory_bank_controller(
        #[values(0x0000, 0x7FFF, 0x1234, 0x4242)] address: u16,
        #[values(0xFF, 0xAB, 0x42)] data: u8,
    ) {
        let mut cartridge_data = [0u8; 16384 * 2];
        cartridge_data[0x0147] = 0x00; // This 'cartridge' is ROM only
        cartridge_data[0x0148] = 0x00; // This 'cartridge' has 32KiB of ROM
        cartridge_data[0x0149] = 0x00; // This 'cartridge' has no RAM
        cartridge_data[address as usize] = data;

        let cartridge = Cartridge::from_reader(&cartridge_data[..]).unwrap();
        assert_eq!(cartridge.read(address).unwrap(), data);
    }
}
