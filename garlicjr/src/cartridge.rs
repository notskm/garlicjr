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
        let mut buf: Vec<u8> = vec![];

        let result = reader.read_to_end(&mut buf);

        match result {
            Ok(size) if size % 16384 != 0 => {
                return Err(ReadError::BadSize { size });
            }
            Ok(_) => (),
            Err(err) => return Err(ReadError::IoError(err)),
        }

        let title_data = buf[TITLE_RANGE].to_vec();

        if !title_data.is_ascii() {
            return Err(ReadError::NonAsciiTitle { bytes: title_data });
        }

        let title = String::from_utf8(title_data)
            .unwrap()
            .trim_matches('\0')
            .to_string();

        Ok(Cartridge { title })
    }

    pub fn title(&self) -> &str {
        &self.title
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
}
