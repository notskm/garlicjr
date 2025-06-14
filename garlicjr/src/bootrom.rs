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

#[derive(Debug)]
pub struct DmgBootrom {
    data: [u8; 256],
}

impl DmgBootrom {
    pub fn from_reader(mut readable: impl Read) -> std::io::Result<Self> {
        let mut data = [0; 256];
        readable.read_exact(&mut data)?;
        Ok(Self { data })
    }

    pub fn data(&self) -> &[u8; 256] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use rstest::rstest;

    use super::*;

    struct BootromFile<'a> {
        data: &'a [u8],
        position: usize,
    }

    impl<'a> BootromFile<'a> {
        fn new(data: &'a [u8]) -> Self {
            Self { data, position: 0 }
        }
    }

    impl<'a> Read for BootromFile<'a> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.position > self.data.len() {
                return Ok(0);
            }

            let remaining = &self.data[self.position..];
            let bytes_to_read = remaining.len().min(buf.len());
            buf[..bytes_to_read].copy_from_slice(&remaining[..bytes_to_read]);
            self.position += bytes_to_read;

            Ok(bytes_to_read)
        }
    }

    #[rstest]
    #[case([0u8;256])]
    #[case([255u8;256])]
    fn should_return_bootrom_if_given_256_bytes(#[case] raw_data: [u8; 256]) {
        let file = BootromFile::new(&raw_data);
        let bootrom = DmgBootrom::from_reader(file);
        assert_eq!(*bootrom.unwrap().data(), raw_data);
    }

    #[rstest]
    #[case(&[0u8;255])]
    #[case(&[0u8;0])]
    fn should_return_error_when_given_less_than_256_bytes(#[case] raw_data: &'static [u8]) {
        let file = BootromFile::new(raw_data);
        let error = DmgBootrom::from_reader(file).unwrap_err();
        let expected_kind = std::io::ErrorKind::UnexpectedEof;
        assert_eq!(error.kind(), expected_kind);
    }
}
