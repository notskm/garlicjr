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

use std::{io::Read, string::FromUtf8Error};

pub struct Cartridge {
    title: String,
}

impl Cartridge {
    pub fn from_reader(mut reader: impl Read) -> Result<Self, FromUtf8Error> {
        let mut buf: Vec<u8> = vec![];
        let _bytes_read = reader.read_to_end(&mut buf);

        let title_data = buf[0x134..=0x143].to_ascii_uppercase();
        let title = String::from_utf8(title_data)?
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

        let mut cartridge_data = [0u8; 1000];
        cartridge_data[0x134..0x134 + title_bytes.len()].copy_from_slice(title_bytes);

        let cartridge = Cartridge::from_reader(&cartridge_data[..]).unwrap();
        assert_eq!(cartridge.title(), title);
    }

    #[test]
    fn should_not_interpret_bytes_beyond_0x0143_as_title() {
        let title = "THIS TITLE IS TOO LONG";
        let expected = "THIS TITLE IS TO";

        let title_bytes = title.as_bytes();

        let mut cartridge_data = [0u8; 1024];
        cartridge_data[0x134..0x134 + title_bytes.len()].copy_from_slice(title_bytes);

        let cartridge = Cartridge::from_reader(&cartridge_data[..]).unwrap();
        assert_eq!(cartridge.title(), expected);
    }

    #[test]
    fn should_return_error_if_title_cannot_be_interpreted_as_utf8() {
        let title = "元気元気元気元";
        let title_bytes = title.as_bytes();

        let mut cartridge_data = [0u8; 1000];
        cartridge_data[0x134..0x134 + title_bytes.len()].copy_from_slice(title_bytes);

        let result = Cartridge::from_reader(&cartridge_data[..]);
        assert!(result.is_err());
    }
}
