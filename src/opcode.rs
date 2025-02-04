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
    Nop,
    LdReg8Imm8(Register8Bit),
    LdReg8Reg8 {
        source: Register8Bit,
        destination: Register8Bit,
    },
    LdReg8HlAddr(Register8Bit),
    LdAReg16Addr(Register16Bit),
    LdAHliAddr,
    LdAHldAddr,
    LdHlAddrImm8,
    LdReg16Imm16(Register16Bit),
    LdHlAddrReg8(Register8Bit),
    LdReg16AddrA(Register16Bit),
    LdHliAddrA,
    LdHldAddrA,
    LdImm16AddrSp,
    IncReg16(Register16Bit),
    DecReg16(Register16Bit),
    AddHlR16(Register16Bit),
    IncReg8(Register8Bit),
    IncHlAddr,
    DecReg8(Register8Bit),
    DecHlAddr,
    Halt,
    Stop,
    Unimplemented(u8),
}

impl Opcode {
    #[allow(dead_code)]
    pub fn decode(data: u8) -> Opcode {
        if data == 0b00000000 {
            return Opcode::Nop;
        }

        if data == 0b01110110 {
            return Opcode::Halt;
        }

        if data == 0b00010000 {
            return Opcode::Stop;
        }

        if data == 0b00001000 {
            return Opcode::LdImm16AddrSp;
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
        opcode.unwrap_or(Opcode::Unimplemented(data))
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
                    return Some(Opcode::Halt);
                }

                if source == Register8Bit::HLAddr {
                    return Some(Opcode::LdReg8HlAddr(destination));
                }

                if destination == Register8Bit::HLAddr {
                    return Some(Opcode::LdHlAddrReg8(source));
                }

                Some(Opcode::LdReg8Reg8 {
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
                    Some(Opcode::LdHlAddrImm8)
                } else {
                    Some(Opcode::LdReg8Imm8(register))
                }
            }
            (0b00000000, 0b00000100) => {
                let reg_num = (data & 0b00111000) >> 3;

                let register = Register8Bit::from_u8(reg_num);

                if register == Register8Bit::HLAddr {
                    Some(Opcode::IncHlAddr)
                } else {
                    Some(Opcode::IncReg8(register))
                }
            }
            (0b00000000, 0b00000101) => {
                let reg_num = (data & 0b00111000) >> 3;

                let register = Register8Bit::from_u8(reg_num);

                if register == Register8Bit::HLAddr {
                    Some(Opcode::DecHlAddr)
                } else {
                    Some(Opcode::DecReg8(register))
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
                Some(Opcode::LdReg16Imm16(register))
            }
            (0b0000000, 0b00000010) => {
                let reg_num = (data & 0b00110000) >> 4;
                let register = Register16BitMemory::from_u8(reg_num);
                match register {
                    Register16BitMemory::BC => Some(Opcode::LdReg16AddrA(Register16Bit::BC)),
                    Register16BitMemory::DE => Some(Opcode::LdReg16AddrA(Register16Bit::DE)),
                    Register16BitMemory::Hli => Some(Opcode::LdHliAddrA),
                    Register16BitMemory::Hld => Some(Opcode::LdHldAddrA),
                }
            }
            (0b00000000, 0b00000011) => {
                let reg_num = (data & 0b00110000) >> 4;
                let register = Register16Bit::from_u8(reg_num);
                Some(Opcode::IncReg16(register))
            }
            (0b00000000, 0b00001001) => {
                let reg_num = (data & 0b00110000) >> 4;
                let register = Register16Bit::from_u8(reg_num);
                Some(Opcode::AddHlR16(register))
            }
            (0b00000000, 0b00001010) => {
                let source = (data & 0b00110000) >> 4;
                let source = Register16BitMemory::from_u8(source);

                match source {
                    Register16BitMemory::Hli => Some(Opcode::LdAHliAddr),
                    Register16BitMemory::Hld => Some(Opcode::LdAHldAddr),
                    Register16BitMemory::BC => Some(Opcode::LdAReg16Addr(Register16Bit::BC)),
                    Register16BitMemory::DE => Some(Opcode::LdAReg16Addr(Register16Bit::DE)),
                }
            }
            (0b00000000, 0b00001011) => {
                let reg_num = (data & 0b00110000) >> 4;
                let register = Register16Bit::from_u8(reg_num);
                Some(Opcode::DecReg16(register))
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
    Hli,
    Hld,
}

impl Register16BitMemory {
    pub fn from_u8(data: u8) -> Register16BitMemory {
        match data {
            0 => Register16BitMemory::BC,
            1 => Register16BitMemory::DE,
            2 => Register16BitMemory::Hli,
            3 => Register16BitMemory::Hld,
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
        assert_eq!(opcode, Opcode::Nop);
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
        assert_eq!(opcode, Opcode::LdReg8Imm8(destination));
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
            Opcode::LdReg8Reg8 {
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
        assert_eq!(opcode, Opcode::LdAReg16Addr(source));
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
        assert_eq!(opcode, Opcode::LdReg8HlAddr(destination));
    }

    #[test]
    fn should_return_ld_a_hli_addr_given_00101010() {
        let opcode = Opcode::decode(0b00101010);
        assert_eq!(opcode, Opcode::LdAHliAddr);
    }

    #[test]
    fn should_return_ld_a_hld_addr_given_00111010() {
        let opcode = Opcode::decode(0b00111010);
        assert_eq!(opcode, Opcode::LdAHldAddr);
    }

    #[test]
    fn should_return_ld_hli_addr_a_given_00100010() {
        let opcode = Opcode::decode(0b00100010);
        assert_eq!(opcode, Opcode::LdHliAddrA);
    }

    #[test]
    fn should_return_ld_hld_addr_a_given_00110010() {
        let opcode = Opcode::decode(0b00110010);
        assert_eq!(opcode, Opcode::LdHldAddrA);
    }

    #[test]
    fn should_return_ld_hl_addr_i8_given_00110110() {
        let opcode = Opcode::decode(0b00110110);
        assert_eq!(opcode, Opcode::LdHlAddrImm8);
    }

    #[test]
    fn should_return_halt_given_01110110() {
        let opcode = Opcode::decode(0b01110110);
        assert_eq!(opcode, Opcode::Halt);
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
        assert_eq!(opcode, Opcode::LdHlAddrReg8(source));
    }

    #[rstest]
    #[case(Register16Bit::BC, 0b00000010)]
    #[case(Register16Bit::DE, 0b00010010)]
    fn should_return_ld_r16_addr_a_with_correct_destination(
        #[case] destination: Register16Bit,
        #[case] raw_opcode: u8,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, Opcode::LdReg16AddrA(destination));
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
        assert_eq!(opcode, Opcode::LdReg16Imm16(destination));
    }

    #[test]
    fn should_return_stop_when_given_00010000() {
        let opcode = Opcode::decode(0b00010000);
        assert_eq!(opcode, Opcode::Stop);
    }

    #[test]
    fn should_return_ld_imm16_addr_sp_given_00001000() {
        let opcode = Opcode::decode(0b00001000);
        assert_eq!(opcode, Opcode::LdImm16AddrSp);
    }

    #[rstest]
    #[case(Register16Bit::BC, 0b00000011)]
    #[case(Register16Bit::DE, 0b00010011)]
    #[case(Register16Bit::HL, 0b00100011)]
    #[case(Register16Bit::SP, 0b00110011)]
    fn should_return_inc_r16_given_00xx0011(#[case] register: Register16Bit, #[case] data: u8) {
        let opcode = Opcode::decode(data);
        assert_eq!(opcode, Opcode::IncReg16(register));
    }

    #[rstest]
    #[case(Register16Bit::BC, 0b00001011)]
    #[case(Register16Bit::DE, 0b00011011)]
    #[case(Register16Bit::HL, 0b00101011)]
    #[case(Register16Bit::SP, 0b00111011)]
    fn should_return_dec_r16_given_00xx1011(#[case] register: Register16Bit, #[case] data: u8) {
        let opcode = Opcode::decode(data);
        assert_eq!(opcode, Opcode::DecReg16(register));
    }

    #[rstest]
    #[case(Register16Bit::BC, 0b00001001)]
    #[case(Register16Bit::DE, 0b00011001)]
    #[case(Register16Bit::HL, 0b00101001)]
    #[case(Register16Bit::SP, 0b00111001)]
    fn should_return_add_hl_r16_given_00xx1001(#[case] register: Register16Bit, #[case] data: u8) {
        let opcode = Opcode::decode(data);
        assert_eq!(opcode, Opcode::AddHlR16(register));
    }

    #[rstest]
    #[case(Register8Bit::A, 0b00111100)]
    #[case(Register8Bit::B, 0b00000100)]
    #[case(Register8Bit::C, 0b00001100)]
    #[case(Register8Bit::D, 0b00010100)]
    #[case(Register8Bit::E, 0b00011100)]
    #[case(Register8Bit::H, 0b00100100)]
    #[case(Register8Bit::L, 0b00101100)]
    fn should_return_inc_r8_given_00xxx100(#[case] register: Register8Bit, #[case] data: u8) {
        let opcode = Opcode::decode(data);
        assert_eq!(opcode, Opcode::IncReg8(register));
    }

    #[test]
    fn should_return_inc_hl_addr_given_00110100() {
        let opcode = Opcode::decode(0b00110100);
        assert_eq!(opcode, Opcode::IncHlAddr);
    }

    #[rstest]
    #[case(Register8Bit::A, 0b00111101)]
    #[case(Register8Bit::B, 0b00000101)]
    #[case(Register8Bit::C, 0b00001101)]
    #[case(Register8Bit::D, 0b00010101)]
    #[case(Register8Bit::E, 0b00011101)]
    #[case(Register8Bit::H, 0b00100101)]
    #[case(Register8Bit::L, 0b00101101)]
    fn should_return_inc_r8_given_00xxx101(#[case] register: Register8Bit, #[case] data: u8) {
        let opcode = Opcode::decode(data);
        assert_eq!(opcode, Opcode::DecReg8(register));
    }

    #[test]
    fn should_return_dec_hl_addr_given_00110101() {
        let opcode = Opcode::decode(0b00110101);
        assert_eq!(opcode, Opcode::DecHlAddr);
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
    #[case(2, Register16BitMemory::Hli)]
    #[case(3, Register16BitMemory::Hld)]
    fn should_return_correct_16_bit_address_register(
        #[case] data: u8,
        #[case] expected: Register16BitMemory,
    ) {
        let register = Register16BitMemory::from_u8(data);
        assert_eq!(register, expected);
    }
}
