/*
    Copyright 2024 notskm

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
    LdR8HLAddr,
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
                    Some(Opcode::LdR8HLAddr)
                } else {
                    Some(Opcode::LDRI8(register))
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

    #[test]
    fn should_return_ld_r8_addr_hl_given_00110110() {
        let opcode = Opcode::decode(0b00110110);
        assert_eq!(opcode, Opcode::LdR8HLAddr);
    }

    #[test]
    fn should_return_halt_given_01110110() {
        let opcode = Opcode::decode(0b01110110);
        assert_eq!(opcode, Opcode::HALT);
    }

    #[test]
    fn should_return_b_when_given_0() {
        let register = Register8Bit::from_u8(0);
        assert_eq!(register, Register8Bit::B);
    }

    #[test]
    fn should_return_c_when_given_1() {
        let register = Register8Bit::from_u8(1);
        assert_eq!(register, Register8Bit::C);
    }

    #[test]
    fn should_return_d_when_given_2() {
        let register = Register8Bit::from_u8(2);
        assert_eq!(register, Register8Bit::D);
    }

    #[test]
    fn should_return_e_when_given_3() {
        let register = Register8Bit::from_u8(3);
        assert_eq!(register, Register8Bit::E);
    }

    #[test]
    fn should_return_h_when_given_4() {
        let register = Register8Bit::from_u8(4);
        assert_eq!(register, Register8Bit::H);
    }

    #[test]
    fn should_return_l_when_given_5() {
        let register = Register8Bit::from_u8(5);
        assert_eq!(register, Register8Bit::L);
    }

    #[test]
    fn should_return_hladdr_when_given_6() {
        let register = Register8Bit::from_u8(6);
        assert_eq!(register, Register8Bit::HLAddr);
    }

    #[test]
    fn should_return_a_when_given_7() {
        let register = Register8Bit::from_u8(7);
        assert_eq!(register, Register8Bit::A);
    }
}
