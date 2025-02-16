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
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Unimplemented(u8),
}

impl Opcode {
    #[allow(dead_code)]
    pub fn decode(data: u8) -> Opcode {
        let opcode = Self::decode_whole(data);
        if let Some(opcode) = opcode {
            return opcode;
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

    fn decode_whole(data: u8) -> Option<Opcode> {
        match data {
            0b00000000 => Some(Opcode::Nop),
            0b01110110 => Some(Opcode::Halt),
            0b00010000 => Some(Opcode::Stop),
            0b00001000 => Some(Opcode::LdImm16AddrSp),
            0b00000111 => Some(Opcode::Rlca),
            0b00001111 => Some(Opcode::Rrca),
            0b00010111 => Some(Opcode::Rla),
            0b00011111 => Some(Opcode::Rra),
            0b00100111 => Some(Opcode::Daa),
            _ => None,
        }
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

    #[rstest]
    #[case(0b00000000, Opcode::Nop)]
    #[case(0b00010000, Opcode::Stop)]
    #[case(0b01110110, Opcode::Halt)]
    #[case(0xD3, Opcode::Unimplemented(0xD3))]
    #[case(0xE3, Opcode::Unimplemented(0xE3))]
    #[case(0xE4, Opcode::Unimplemented(0xE4))]
    #[case(0xF4, Opcode::Unimplemented(0xF4))]
    #[case(0xEB, Opcode::Unimplemented(0xEB))]
    #[case(0xEC, Opcode::Unimplemented(0xEC))]
    #[case(0xFC, Opcode::Unimplemented(0xFC))]
    #[case(0xDB, Opcode::Unimplemented(0xDB))]
    #[case(0xDD, Opcode::Unimplemented(0xDD))]
    #[case(0xDE, Opcode::Unimplemented(0xDE))]
    #[case(0xDF, Opcode::Unimplemented(0xDF))]
    #[case(0b00111110, Opcode::LdReg8Imm8(Register8Bit::A))]
    #[case(0b00000110, Opcode::LdReg8Imm8(Register8Bit::B))]
    #[case(0b00001110, Opcode::LdReg8Imm8(Register8Bit::C))]
    #[case(0b00010110, Opcode::LdReg8Imm8(Register8Bit::D))]
    #[case(0b00011110, Opcode::LdReg8Imm8(Register8Bit::E))]
    #[case(0b00100110, Opcode::LdReg8Imm8(Register8Bit::H))]
    #[case(0b00101110, Opcode::LdReg8Imm8(Register8Bit::L))]
    #[case(0b01111111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::A})]
    #[case(0b01111000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::A})]
    #[case(0b01111001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::A})]
    #[case(0b01111010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::A})]
    #[case(0b01111011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::A})]
    #[case(0b01111100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::A})]
    #[case(0b01111101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::A})]
    #[case(0b01000111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::B})]
    #[case(0b01000000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::B})]
    #[case(0b01000001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::B})]
    #[case(0b01000010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::B})]
    #[case(0b01000011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::B})]
    #[case(0b01000100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::B})]
    #[case(0b01000101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::B})]
    #[case(0b01001111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::C})]
    #[case(0b01001000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::C})]
    #[case(0b01001001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::C})]
    #[case(0b01001010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::C})]
    #[case(0b01001011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::C})]
    #[case(0b01001100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::C})]
    #[case(0b01001101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::C})]
    #[case(0b01010111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::D})]
    #[case(0b01010000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::D})]
    #[case(0b01010001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::D})]
    #[case(0b01010010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::D})]
    #[case(0b01010011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::D})]
    #[case(0b01010100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::D})]
    #[case(0b01010101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::D})]
    #[case(0b01011111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::E})]
    #[case(0b01011000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::E})]
    #[case(0b01011001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::E})]
    #[case(0b01011010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::E})]
    #[case(0b01011011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::E})]
    #[case(0b01011100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::E})]
    #[case(0b01011101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::E})]
    #[case(0b01100111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::H})]
    #[case(0b01100000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::H})]
    #[case(0b01100001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::H})]
    #[case(0b01100010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::H})]
    #[case(0b01100011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::H})]
    #[case(0b01100100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::H})]
    #[case(0b01100101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::H})]
    #[case(0b01101111, Opcode::LdReg8Reg8{source: Register8Bit::A, destination: Register8Bit::L})]
    #[case(0b01101000, Opcode::LdReg8Reg8{source: Register8Bit::B, destination: Register8Bit::L})]
    #[case(0b01101001, Opcode::LdReg8Reg8{source: Register8Bit::C, destination: Register8Bit::L})]
    #[case(0b01101010, Opcode::LdReg8Reg8{source: Register8Bit::D, destination: Register8Bit::L})]
    #[case(0b01101011, Opcode::LdReg8Reg8{source: Register8Bit::E, destination: Register8Bit::L})]
    #[case(0b01101100, Opcode::LdReg8Reg8{source: Register8Bit::H, destination: Register8Bit::L})]
    #[case(0b01101101, Opcode::LdReg8Reg8{source: Register8Bit::L, destination: Register8Bit::L})]
    #[case(0b00001010, Opcode::LdAReg16Addr(Register16Bit::BC))]
    #[case(0b00011010, Opcode::LdAReg16Addr(Register16Bit::DE))]
    #[case(0b01111110, Opcode::LdReg8HlAddr(Register8Bit::A))]
    #[case(0b01000110, Opcode::LdReg8HlAddr(Register8Bit::B))]
    #[case(0b01001110, Opcode::LdReg8HlAddr(Register8Bit::C))]
    #[case(0b01010110, Opcode::LdReg8HlAddr(Register8Bit::D))]
    #[case(0b01011110, Opcode::LdReg8HlAddr(Register8Bit::E))]
    #[case(0b01100110, Opcode::LdReg8HlAddr(Register8Bit::H))]
    #[case(0b01101110, Opcode::LdReg8HlAddr(Register8Bit::L))]
    #[case(0b00101010, Opcode::LdAHliAddr)]
    #[case(0b00111010, Opcode::LdAHldAddr)]
    #[case(0b00100010, Opcode::LdHliAddrA)]
    #[case(0b00110010, Opcode::LdHldAddrA)]
    #[case(0b00110110, Opcode::LdHlAddrImm8)]
    #[case(0b01110111, Opcode::LdHlAddrReg8(Register8Bit::A))]
    #[case(0b01110000, Opcode::LdHlAddrReg8(Register8Bit::B))]
    #[case(0b01110001, Opcode::LdHlAddrReg8(Register8Bit::C))]
    #[case(0b01110010, Opcode::LdHlAddrReg8(Register8Bit::D))]
    #[case(0b01110011, Opcode::LdHlAddrReg8(Register8Bit::E))]
    #[case(0b01110100, Opcode::LdHlAddrReg8(Register8Bit::H))]
    #[case(0b01110101, Opcode::LdHlAddrReg8(Register8Bit::L))]
    #[case(0b00000010, Opcode::LdReg16AddrA(Register16Bit::BC))]
    #[case(0b00010010, Opcode::LdReg16AddrA(Register16Bit::DE))]
    #[case(0b00000001, Opcode::LdReg16Imm16(Register16Bit::BC))]
    #[case(0b00010001, Opcode::LdReg16Imm16(Register16Bit::DE))]
    #[case(0b00100001, Opcode::LdReg16Imm16(Register16Bit::HL))]
    #[case(0b00110001, Opcode::LdReg16Imm16(Register16Bit::SP))]
    #[case(0b00001000, Opcode::LdImm16AddrSp)]
    #[case(0b00000011, Opcode::IncReg16(Register16Bit::BC))]
    #[case(0b00010011, Opcode::IncReg16(Register16Bit::DE))]
    #[case(0b00100011, Opcode::IncReg16(Register16Bit::HL))]
    #[case(0b00110011, Opcode::IncReg16(Register16Bit::SP))]
    #[case(0b00001011, Opcode::DecReg16(Register16Bit::BC))]
    #[case(0b00011011, Opcode::DecReg16(Register16Bit::DE))]
    #[case(0b00101011, Opcode::DecReg16(Register16Bit::HL))]
    #[case(0b00111011, Opcode::DecReg16(Register16Bit::SP))]
    #[case(0b00001001, Opcode::AddHlR16(Register16Bit::BC))]
    #[case(0b00011001, Opcode::AddHlR16(Register16Bit::DE))]
    #[case(0b00101001, Opcode::AddHlR16(Register16Bit::HL))]
    #[case(0b00111001, Opcode::AddHlR16(Register16Bit::SP))]
    #[case(0b00111100, Opcode::IncReg8(Register8Bit::A))]
    #[case(0b00000100, Opcode::IncReg8(Register8Bit::B))]
    #[case(0b00001100, Opcode::IncReg8(Register8Bit::C))]
    #[case(0b00010100, Opcode::IncReg8(Register8Bit::D))]
    #[case(0b00011100, Opcode::IncReg8(Register8Bit::E))]
    #[case(0b00100100, Opcode::IncReg8(Register8Bit::H))]
    #[case(0b00101100, Opcode::IncReg8(Register8Bit::L))]
    #[case(0b00110100, Opcode::IncHlAddr)]
    #[case(0b00111101, Opcode::DecReg8(Register8Bit::A))]
    #[case(0b00000101, Opcode::DecReg8(Register8Bit::B))]
    #[case(0b00001101, Opcode::DecReg8(Register8Bit::C))]
    #[case(0b00010101, Opcode::DecReg8(Register8Bit::D))]
    #[case(0b00011101, Opcode::DecReg8(Register8Bit::E))]
    #[case(0b00100101, Opcode::DecReg8(Register8Bit::H))]
    #[case(0b00101101, Opcode::DecReg8(Register8Bit::L))]
    #[case(0b00110101, Opcode::DecHlAddr)]
    #[case(0b00000111, Opcode::Rlca)]
    #[case(0b00001111, Opcode::Rrca)]
    #[case(0b00010111, Opcode::Rla)]
    #[case(0b00011111, Opcode::Rra)]
    #[case(0b00100111, Opcode::Daa)]
    fn should_return_expected_opcode_given_an_opcode_byte(
        #[case] raw_opcode: u8,
        #[case] result: Opcode,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, result);
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
