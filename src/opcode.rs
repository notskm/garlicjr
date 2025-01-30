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

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum Opcode {
    NOP,
    LDRI8(Register8Bit),
    LDR8R8 {
        source: Register8Bit,
        destination: Register8Bit,
    },
    LDR8HLAddr(Register8Bit),
    LDAR16Addr(Register16Bit),
    LDAHLIAddr,
    LDAHLDAddr,
    LDHLAddrI8,
    LDR16I16(Register16Bit),
    LDHLAddrR8(Register8Bit),
    HALT,
    Unimplemented(u8),
}

impl Opcode {
    #[allow(dead_code)]
    pub fn decode(data: u8) -> Opcode {
        if data == 0b00000000 {
            return Opcode::NOP;
        }

        if data == 0b01110110 {
            return Opcode::HALT;
        }

        let opcode = Self::decode_top_2(data);
        if let Some(opcode) = opcode {
            return opcode;
        }

        let opcode = Self::decode_top_2_bottom_3(data);
        if let Some(opcode) = opcode {
            return opcode;
        }

        let opcode = Self::decode_top_2_bottom_4(data);
        opcode.unwrap_or_else(|| Opcode::Unimplemented(data))
    }

    fn decode_top_2(data: u8) -> Option<Opcode> {
        let top_2 = data & 0b11000000;

        match top_2 {
            0b01000000 => {
                let source = data & 0b00000111;
                let destination = (data >> 3) & 0b00000111;

                let source = Register8Bit::from_u8(source);
                let destination = Register8Bit::from_u8(destination);

                // This is a special case because LD [HL], [HL] results in 01110110, which is the
                // HALT opcode.
                if source == Register8Bit::HLAddr && destination == Register8Bit::HLAddr {
                    return Some(Opcode::HALT);
                }

                if source == Register8Bit::HLAddr {
                    return Some(Opcode::LDR8HLAddr(destination));
                }

                if destination == Register8Bit::HLAddr {
                    return Some(Opcode::LDHLAddrR8(source));
                }

                Some(Opcode::LDR8R8 {
                    source,
                    destination,
                })
            }
            _ => None,
        }
    }

    fn decode_top_2_bottom_3(data: u8) -> Option<Opcode> {
        let top_2 = data & 0b11000000;
        let bot_3 = data & 0b00000111;

        match (top_2, bot_3) {
            (0b00000000, 0b00000110) => {
                let reg_num = (data & 0b00111000) >> 3;
                let register = Register8Bit::from_u8(reg_num);
                if register == Register8Bit::HLAddr {
                    Some(Opcode::LDHLAddrI8)
                } else {
                    Some(Opcode::LDRI8(register))
                }
            }
            _ => None,
        }
    }

    fn decode_top_2_bottom_4(data: u8) -> Option<Opcode> {
        let top_2 = data & 0b11000000;
        let bot_4 = data & 0b00001111;

        match (top_2, bot_4) {
            (0b00000000, 0b00000001) => {
                let reg_num = (data & 0b00110000) >> 4;
                let register = Register16Bit::from_u8(reg_num);
                Some(Opcode::LDR16I16(register))
            }
            (0b00000000, 0b00001010) => {
                let source = (data & 0b00110000) >> 4;
                let source = Register16BitMemory::from_u8(source);

                match source {
                    Register16BitMemory::HLI => Some(Opcode::LDAHLIAddr),
                    Register16BitMemory::HLD => Some(Opcode::LDAHLDAddr),
                    Register16BitMemory::BC => Some(Opcode::LDAR16Addr(Register16Bit::BC)),
                    Register16BitMemory::DE => Some(Opcode::LDAR16Addr(Register16Bit::DE)),
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Register8Bit {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

impl Register8Bit {
    pub fn from_u8(data: u8) -> Register8Bit {
        match data {
            0 => Register8Bit::B,
            1 => Register8Bit::C,
            2 => Register8Bit::D,
            3 => Register8Bit::E,
            4 => Register8Bit::H,
            5 => Register8Bit::L,
            6 => Register8Bit::HLAddr,
            7 => Register8Bit::A,
            _ => panic!("Invalid register"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Register16Bit {
    BC,
    DE,
    HL,
    SP,
}

impl Register16Bit {
    pub fn from_u8(data: u8) -> Register16Bit {
        match data {
            0 => Register16Bit::BC,
            1 => Register16Bit::DE,
            2 => Register16Bit::HL,
            3 => Register16Bit::SP,
            _ => panic!("Invalid register"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Register16BitMemory {
    BC,
    DE,
    HLI,
    HLD,
}

impl Register16BitMemory {
    pub fn from_u8(data: u8) -> Register16BitMemory {
        match data {
            0 => Register16BitMemory::BC,
            1 => Register16BitMemory::DE,
            2 => Register16BitMemory::HLI,
            3 => Register16BitMemory::HLD,
            _ => panic!("Invalid register"),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[test]
    fn should_return_unimplemented_opcode_when_data_is_0xd3() {
        let opcode = Opcode::decode(0xd3);
        assert_eq!(opcode, Opcode::Unimplemented(0xd3));
    }

    #[rstest]
    #[case(0xD3)]
    #[case(0xE3)]
    #[case(0xE4)]
    #[case(0xF4)]
    #[case(0xEB)]
    #[case(0xEC)]
    #[case(0xFC)]
    #[case(0xDB)]
    #[case(0xDD)]
    #[case(0xDE)]
    #[case(0xDF)]
    fn should_return_unimplemented_opcode(#[case] code: u8) {
        let opcode = Opcode::decode(code);
        assert_eq!(opcode, Opcode::Unimplemented(code));
    }

    #[test]
    fn should_return_nop_when_data_is_0b00000000() {
        let opcode = Opcode::decode(0b00000000);
        assert_eq!(opcode, Opcode::NOP);
    }

    #[rstest]
    #[case(Register8Bit::A, 0b00111110)]
    #[case(Register8Bit::B, 0b00000110)]
    #[case(Register8Bit::C, 0b00001110)]
    #[case(Register8Bit::D, 0b00010110)]
    #[case(Register8Bit::E, 0b00011110)]
    #[case(Register8Bit::H, 0b00100110)]
    #[case(Register8Bit::L, 0b00101110)]
    fn should_return_ldri8_containing_destination_given_00xxx110(
        #[case] destination: Register8Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, Opcode::LDRI8(destination));
    }

    #[rstest]
    #[case(Register8Bit::A, Register8Bit::A, 0b01111111)]
    #[case(Register8Bit::B, Register8Bit::A, 0b01111000)]
    #[case(Register8Bit::C, Register8Bit::A, 0b01111001)]
    #[case(Register8Bit::D, Register8Bit::A, 0b01111010)]
    #[case(Register8Bit::E, Register8Bit::A, 0b01111011)]
    #[case(Register8Bit::H, Register8Bit::A, 0b01111100)]
    #[case(Register8Bit::L, Register8Bit::A, 0b01111101)]
    #[case(Register8Bit::A, Register8Bit::B, 0b01000111)]
    #[case(Register8Bit::B, Register8Bit::B, 0b01000000)]
    #[case(Register8Bit::C, Register8Bit::B, 0b01000001)]
    #[case(Register8Bit::D, Register8Bit::B, 0b01000010)]
    #[case(Register8Bit::E, Register8Bit::B, 0b01000011)]
    #[case(Register8Bit::H, Register8Bit::B, 0b01000100)]
    #[case(Register8Bit::L, Register8Bit::B, 0b01000101)]
    #[case(Register8Bit::A, Register8Bit::C, 0b01001111)]
    #[case(Register8Bit::B, Register8Bit::C, 0b01001000)]
    #[case(Register8Bit::C, Register8Bit::C, 0b01001001)]
    #[case(Register8Bit::D, Register8Bit::C, 0b01001010)]
    #[case(Register8Bit::E, Register8Bit::C, 0b01001011)]
    #[case(Register8Bit::H, Register8Bit::C, 0b01001100)]
    #[case(Register8Bit::L, Register8Bit::C, 0b01001101)]
    #[case(Register8Bit::A, Register8Bit::D, 0b01010111)]
    #[case(Register8Bit::B, Register8Bit::D, 0b01010000)]
    #[case(Register8Bit::C, Register8Bit::D, 0b01010001)]
    #[case(Register8Bit::D, Register8Bit::D, 0b01010010)]
    #[case(Register8Bit::E, Register8Bit::D, 0b01010011)]
    #[case(Register8Bit::H, Register8Bit::D, 0b01010100)]
    #[case(Register8Bit::L, Register8Bit::D, 0b01010101)]
    #[case(Register8Bit::A, Register8Bit::E, 0b01011111)]
    #[case(Register8Bit::B, Register8Bit::E, 0b01011000)]
    #[case(Register8Bit::C, Register8Bit::E, 0b01011001)]
    #[case(Register8Bit::D, Register8Bit::E, 0b01011010)]
    #[case(Register8Bit::E, Register8Bit::E, 0b01011011)]
    #[case(Register8Bit::H, Register8Bit::E, 0b01011100)]
    #[case(Register8Bit::L, Register8Bit::E, 0b01011101)]
    #[case(Register8Bit::A, Register8Bit::H, 0b01100111)]
    #[case(Register8Bit::B, Register8Bit::H, 0b01100000)]
    #[case(Register8Bit::C, Register8Bit::H, 0b01100001)]
    #[case(Register8Bit::D, Register8Bit::H, 0b01100010)]
    #[case(Register8Bit::E, Register8Bit::H, 0b01100011)]
    #[case(Register8Bit::H, Register8Bit::H, 0b01100100)]
    #[case(Register8Bit::L, Register8Bit::H, 0b01100101)]
    #[case(Register8Bit::A, Register8Bit::L, 0b01101111)]
    #[case(Register8Bit::B, Register8Bit::L, 0b01101000)]
    #[case(Register8Bit::C, Register8Bit::L, 0b01101001)]
    #[case(Register8Bit::D, Register8Bit::L, 0b01101010)]
    #[case(Register8Bit::E, Register8Bit::L, 0b01101011)]
    #[case(Register8Bit::H, Register8Bit::L, 0b01101100)]
    #[case(Register8Bit::L, Register8Bit::L, 0b01101101)]
    fn should_return_ld_r8_r8_given_01xxxxxx(
        #[case] source: Register8Bit,
        #[case] destination: Register8Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(
            opcode,
            Opcode::LDR8R8 {
                source,
                destination
            }
        );
    }

    #[rstest]
    #[case(Register16Bit::BC, 0b00001010)]
    #[case(Register16Bit::DE, 0b00011010)]
    fn should_return_ld_a_r16_addr_with_source(
        #[case] source: Register16Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, Opcode::LDAR16Addr(source));
    }

    #[rstest]
    #[case(Register8Bit::A, 0b01111110)]
    #[case(Register8Bit::B, 0b01000110)]
    #[case(Register8Bit::C, 0b01001110)]
    #[case(Register8Bit::D, 0b01010110)]
    #[case(Register8Bit::E, 0b01011110)]
    #[case(Register8Bit::H, 0b01100110)]
    #[case(Register8Bit::L, 0b01101110)]
    fn should_return_ld_r8_hl_addr_with_destination(
        #[case] destination: Register8Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, Opcode::LDR8HLAddr(destination));
    }

    #[test]
    fn should_return_ld_a_hli_addr_given_00101010() {
        let opcode = Opcode::decode(0b00101010);
        assert_eq!(opcode, Opcode::LDAHLIAddr);
    }

    #[test]
    fn should_return_ld_a_hld_addr_given_00111010() {
        let opcode = Opcode::decode(0b00111010);
        assert_eq!(opcode, Opcode::LDAHLDAddr);
    }

    #[test]
    fn should_return_ld_hl_addr_i8_given_00110110() {
        let opcode = Opcode::decode(0b00110110);
        assert_eq!(opcode, Opcode::LDHLAddrI8);
    }

    #[test]
    fn should_return_halt_given_01110110() {
        let opcode = Opcode::decode(0b01110110);
        assert_eq!(opcode, Opcode::HALT);
    }

    #[rstest]
    #[case(Register8Bit::A, 0b01110111)]
    #[case(Register8Bit::B, 0b01110000)]
    #[case(Register8Bit::C, 0b01110001)]
    #[case(Register8Bit::D, 0b01110010)]
    #[case(Register8Bit::E, 0b01110011)]
    #[case(Register8Bit::H, 0b01110100)]
    #[case(Register8Bit::L, 0b01110101)]
    fn should_return_ld_hl_addr_r8_with_correct_source(
        #[case] source: Register8Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, Opcode::LDHLAddrR8(source));
    }

    #[rstest]
    #[case(Register16Bit::BC, 0b00000001)]
    #[case(Register16Bit::DE, 0b00010001)]
    #[case(Register16Bit::HL, 0b00100001)]
    #[case(Register16Bit::SP, 0b00110001)]
    fn should_return_ld_r16_i16_with_destination_given_00xx0001(
        #[case] destination: Register16Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, Opcode::LDR16I16(destination));
    }

    #[rstest]
    #[case(0, Register8Bit::B)]
    #[case(1, Register8Bit::C)]
    #[case(2, Register8Bit::D)]
    #[case(3, Register8Bit::E)]
    #[case(4, Register8Bit::H)]
    #[case(5, Register8Bit::L)]
    #[case(6, Register8Bit::HLAddr)]
    #[case(7, Register8Bit::A)]
    fn should_return_correct_8_bit_register(#[case] data: u8, #[case] expected: Register8Bit) {
        let register = Register8Bit::from_u8(data);
        assert_eq!(register, expected);
    }

    #[rstest]
    #[case(0, Register16Bit::BC)]
    #[case(1, Register16Bit::DE)]
    #[case(2, Register16Bit::HL)]
    #[case(3, Register16Bit::SP)]
    fn should_return_correct_16_bit_register(#[case] data: u8, #[case] expected: Register16Bit) {
        let register = Register16Bit::from_u8(data);
        assert_eq!(register, expected);
    }

    #[rstest]
    #[case(0, Register16BitMemory::BC)]
    #[case(1, Register16BitMemory::DE)]
    #[case(2, Register16BitMemory::HLI)]
    #[case(3, Register16BitMemory::HLD)]
    fn should_return_correct_16_bit_address_register(
        #[case] data: u8,
        #[case] expected: Register16BitMemory,
    ) {
        let register = Register16BitMemory::from_u8(data);
        assert_eq!(register, expected);
    }
}
