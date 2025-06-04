use std::io::Read;

#[derive(Debug)]
pub struct DmgBootrom {
    data: [u8; 256],
}

impl DmgBootrom {
    pub fn from_reader(readable: &mut impl Read) -> std::io::Result<Self> {
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
        let mut file = BootromFile::new(&raw_data);
        let bootrom = DmgBootrom::from_reader(&mut file);
        assert_eq!(*bootrom.unwrap().data(), raw_data);
    }

    #[rstest]
    #[case(&[0u8;255])]
    #[case(&[0u8;0])]
    fn should_return_error_when_given_less_than_256_bytes(#[case] raw_data: &'static [u8]) {
        let mut file = BootromFile::new(raw_data);
        let error = DmgBootrom::from_reader(&mut file).unwrap_err();
        let expected_kind = std::io::ErrorKind::UnexpectedEof;
        assert_eq!(error.kind(), expected_kind);
    }
}
