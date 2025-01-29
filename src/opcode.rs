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
    LdR8HLAddr,
    Unimplemented(u8),
}

impl Opcode {
    #[allow(dead_code)]
    pub fn decode(data: u8) -> Opcode {
        if data == 0b00000000 {
            return Opcode::NOP;
        }

        let top_2 = data & 0b11000000;
        let bot_3 = data & 0b00000111;

        match (top_2, bot_3) {
            (0b00000000, 0b00000110) => {
                let reg_num = (data & 0b00111000) >> 3;
                let register = Register8Bit::from_u8(reg_num);
                if register == Register8Bit::HLAddr {
                    Opcode::LdR8HLAddr
                } else {
                    Opcode::LDRI8(register)
                }
            }
            _ => Opcode::Unimplemented(data),
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
    #[case(0xCB)]
    #[case(0xEB)]
    #[case(0xEC)]
    #[case(0xFC)]
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

    #[test]
    fn should_return_ld_r8_addr_hl_given_00110110() {
        let opcode = Opcode::decode(0b00110110);
        assert_eq!(opcode, Opcode::LdR8HLAddr);
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
