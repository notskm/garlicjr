#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum Opcode {
    NOP,
    LDRI8(Dest8Bit),
    Unimplemented(u8),
}

impl Opcode {
    #[allow(dead_code)]
    pub fn decode(data: u8) -> Opcode {
        let top_2 = data & 0b11000000;
        let bot_4 = data & 0b00001111;

        match (top_2, bot_4) {
            (0b00000000, 0b00000000) => Opcode::NOP,
            (0b00000000, 0b00000110) => {
                let reg_num = (data & 0b00111000) >> 3;
                let register = Dest8Bit::from_u8(reg_num);
                Opcode::LDRI8(register)
            }
            _ => Opcode::Unimplemented(data),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Dest8Bit {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLAddr,
}

impl Dest8Bit {
    pub fn from_u8(data: u8) -> Dest8Bit {
        match data {
            0 => Dest8Bit::B,
            1 => Dest8Bit::C,
            2 => Dest8Bit::D,
            3 => Dest8Bit::E,
            4 => Dest8Bit::H,
            5 => Dest8Bit::L,
            6 => Dest8Bit::HLAddr,
            7 => Dest8Bit::A,
            _ => panic!("Invalid register"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use rstest::rstest;

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

    #[test]
    fn should_return_ldri8_containing_destination_when_data_is_00xxx110() {
        let opcode = Opcode::decode(0b00100110);
        assert_eq!(opcode, Opcode::LDRI8(Dest8Bit::H));
    }

    #[test]
    fn should_return_b_when_given_0() {
        let register = Dest8Bit::from_u8(0);
        assert_eq!(register, Dest8Bit::B);
    }

    #[test]
    fn should_return_c_when_given_1() {
        let register = Dest8Bit::from_u8(1);
        assert_eq!(register, Dest8Bit::C);
    }

    #[test]
    fn should_return_d_when_given_2() {
        let register = Dest8Bit::from_u8(2);
        assert_eq!(register, Dest8Bit::D);
    }

    #[test]
    fn should_return_e_when_given_3() {
        let register = Dest8Bit::from_u8(3);
        assert_eq!(register, Dest8Bit::E);
    }

    #[test]
    fn should_return_h_when_given_4() {
        let register = Dest8Bit::from_u8(4);
        assert_eq!(register, Dest8Bit::H);
    }

    #[test]
    fn should_return_l_when_given_5() {
        let register = Dest8Bit::from_u8(5);
        assert_eq!(register, Dest8Bit::L);
    }

    #[test]
    fn should_return_hladdr_when_given_6() {
        let register = Dest8Bit::from_u8(6);
        assert_eq!(register, Dest8Bit::HLAddr);
    }

    #[test]
    fn should_return_a_when_given_7() {
        let register = Dest8Bit::from_u8(7);
        assert_eq!(register, Dest8Bit::A);
    }
}
