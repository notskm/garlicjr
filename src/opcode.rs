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

#[derive(Debug, PartialEq, Clone, Copy)]
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
    Cpl,
    Scf,
    Ccf,
    JrImm8,
    JrCondImm8(Cond),
    AddAReg8(Register8Bit),
    AddAHlAddr,
    AdcAReg8(Register8Bit),
    AdcAHlAddr,
    SubAReg8(Register8Bit),
    SubAHlAddr,
    SbcAReg8(Register8Bit),
    SbcAHlAddr,
    AndAReg8(Register8Bit),
    AndAHlAddr,
    XorAReg8(Register8Bit),
    XorAHlAddr,
    OrReg8(Register8Bit),
    OrHLAddr,
    CpReg8(Register8Bit),
    CpHlAddr,
    RetCond(Cond),
    Ret,
    Reti,
    PopReg16Stack(Register16BitStack),
    PushReg16Stack(Register16BitStack),
    JpCondImm16(Cond),
    JpImm16,
    JpHlAddr,
    CallCondImm16(Cond),
    CallImm16,
    Rst(RstTarget),
    Prefix,
    AddAImm8,
    SubImm8,
    AndImm8,
    OrImm8,
    AdcAImm8,
    SbcAImm8,
    XorImm8,
    CpImm8,
    LdhImm8AddrA,
    LdhAImm8Addr,
    LdCAddrA,
    LdACAddr,
    LdImm16AddrA,
    LdAImm16Addr,
    AddSpImm8,
    Unimplemented(u8),
}

const OPTABLE: [Opcode; 256] = [
    Opcode::Nop,
    Opcode::LdReg16Imm16(Register16Bit::BC),
    Opcode::LdReg16AddrA(Register16Bit::BC),
    Opcode::IncReg16(Register16Bit::BC),
    Opcode::IncReg8(Register8Bit::B),
    Opcode::DecReg8(Register8Bit::B),
    Opcode::LdReg8Imm8(Register8Bit::B),
    Opcode::Rlca,
    Opcode::LdImm16AddrSp,
    Opcode::AddHlR16(Register16Bit::BC),
    Opcode::LdAReg16Addr(Register16Bit::BC),
    Opcode::DecReg16(Register16Bit::BC),
    Opcode::IncReg8(Register8Bit::C),
    Opcode::DecReg8(Register8Bit::C),
    Opcode::LdReg8Imm8(Register8Bit::C),
    Opcode::Rrca,
    Opcode::Stop,
    Opcode::LdReg16Imm16(Register16Bit::DE),
    Opcode::LdReg16AddrA(Register16Bit::DE),
    Opcode::IncReg16(Register16Bit::DE),
    Opcode::IncReg8(Register8Bit::D),
    Opcode::DecReg8(Register8Bit::D),
    Opcode::LdReg8Imm8(Register8Bit::D),
    Opcode::Rla,
    Opcode::JrImm8,
    Opcode::AddHlR16(Register16Bit::DE),
    Opcode::LdAReg16Addr(Register16Bit::DE),
    Opcode::DecReg16(Register16Bit::DE),
    Opcode::IncReg8(Register8Bit::E),
    Opcode::DecReg8(Register8Bit::E),
    Opcode::LdReg8Imm8(Register8Bit::E),
    Opcode::Rra,
    Opcode::JrCondImm8(Cond::Nz),
    Opcode::LdReg16Imm16(Register16Bit::HL),
    Opcode::LdHliAddrA,
    Opcode::IncReg16(Register16Bit::HL),
    Opcode::IncReg8(Register8Bit::H),
    Opcode::DecReg8(Register8Bit::H),
    Opcode::LdReg8Imm8(Register8Bit::H),
    Opcode::Daa,
    Opcode::JrCondImm8(Cond::Z),
    Opcode::AddHlR16(Register16Bit::HL),
    Opcode::LdAHliAddr,
    Opcode::DecReg16(Register16Bit::HL),
    Opcode::IncReg8(Register8Bit::L),
    Opcode::DecReg8(Register8Bit::L),
    Opcode::LdReg8Imm8(Register8Bit::L),
    Opcode::Cpl,
    Opcode::JrCondImm8(Cond::Nc),
    Opcode::LdReg16Imm16(Register16Bit::SP),
    Opcode::LdHldAddrA,
    Opcode::IncReg16(Register16Bit::SP),
    Opcode::IncHlAddr,
    Opcode::DecHlAddr,
    Opcode::LdHlAddrImm8,
    Opcode::Scf,
    Opcode::JrCondImm8(Cond::C),
    Opcode::AddHlR16(Register16Bit::SP),
    Opcode::LdAHldAddr,
    Opcode::DecReg16(Register16Bit::SP),
    Opcode::IncReg8(Register8Bit::A),
    Opcode::DecReg8(Register8Bit::A),
    Opcode::LdReg8Imm8(Register8Bit::A),
    Opcode::Ccf,
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8HlAddr(Register8Bit::B),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::B,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8HlAddr(Register8Bit::C),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::C,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8HlAddr(Register8Bit::D),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::D,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8HlAddr(Register8Bit::E),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::E,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8HlAddr(Register8Bit::H),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::H,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::L,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::L,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::L,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::L,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::L,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::L,
    },
    Opcode::LdReg8HlAddr(Register8Bit::L),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::L,
    },
    Opcode::LdHlAddrReg8(Register8Bit::B),
    Opcode::LdHlAddrReg8(Register8Bit::C),
    Opcode::LdHlAddrReg8(Register8Bit::D),
    Opcode::LdHlAddrReg8(Register8Bit::E),
    Opcode::LdHlAddrReg8(Register8Bit::H),
    Opcode::LdHlAddrReg8(Register8Bit::L),
    Opcode::Halt,
    Opcode::LdHlAddrReg8(Register8Bit::A),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::B,
        destination: Register8Bit::A,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::C,
        destination: Register8Bit::A,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::D,
        destination: Register8Bit::A,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::E,
        destination: Register8Bit::A,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::H,
        destination: Register8Bit::A,
    },
    Opcode::LdReg8Reg8 {
        source: Register8Bit::L,
        destination: Register8Bit::A,
    },
    Opcode::LdReg8HlAddr(Register8Bit::A),
    Opcode::LdReg8Reg8 {
        source: Register8Bit::A,
        destination: Register8Bit::A,
    },
    Opcode::AddAReg8(Register8Bit::B),
    Opcode::AddAReg8(Register8Bit::C),
    Opcode::AddAReg8(Register8Bit::D),
    Opcode::AddAReg8(Register8Bit::E),
    Opcode::AddAReg8(Register8Bit::H),
    Opcode::AddAReg8(Register8Bit::L),
    Opcode::AddAHlAddr,
    Opcode::AddAReg8(Register8Bit::A),
    Opcode::AdcAReg8(Register8Bit::B),
    Opcode::AdcAReg8(Register8Bit::C),
    Opcode::AdcAReg8(Register8Bit::D),
    Opcode::AdcAReg8(Register8Bit::E),
    Opcode::AdcAReg8(Register8Bit::H),
    Opcode::AdcAReg8(Register8Bit::L),
    Opcode::AdcAHlAddr,
    Opcode::AdcAReg8(Register8Bit::A),
    Opcode::SubAReg8(Register8Bit::B),
    Opcode::SubAReg8(Register8Bit::C),
    Opcode::SubAReg8(Register8Bit::D),
    Opcode::SubAReg8(Register8Bit::E),
    Opcode::SubAReg8(Register8Bit::H),
    Opcode::SubAReg8(Register8Bit::L),
    Opcode::SubAHlAddr,
    Opcode::SubAReg8(Register8Bit::A),
    Opcode::SbcAReg8(Register8Bit::B),
    Opcode::SbcAReg8(Register8Bit::C),
    Opcode::SbcAReg8(Register8Bit::D),
    Opcode::SbcAReg8(Register8Bit::E),
    Opcode::SbcAReg8(Register8Bit::H),
    Opcode::SbcAReg8(Register8Bit::L),
    Opcode::SbcAHlAddr,
    Opcode::SbcAReg8(Register8Bit::A),
    Opcode::AndAReg8(Register8Bit::B),
    Opcode::AndAReg8(Register8Bit::C),
    Opcode::AndAReg8(Register8Bit::D),
    Opcode::AndAReg8(Register8Bit::E),
    Opcode::AndAReg8(Register8Bit::H),
    Opcode::AndAReg8(Register8Bit::L),
    Opcode::AndAHlAddr,
    Opcode::AndAReg8(Register8Bit::A),
    Opcode::XorAReg8(Register8Bit::B),
    Opcode::XorAReg8(Register8Bit::C),
    Opcode::XorAReg8(Register8Bit::D),
    Opcode::XorAReg8(Register8Bit::E),
    Opcode::XorAReg8(Register8Bit::H),
    Opcode::XorAReg8(Register8Bit::L),
    Opcode::XorAHlAddr,
    Opcode::XorAReg8(Register8Bit::A),
    Opcode::OrReg8(Register8Bit::B),
    Opcode::OrReg8(Register8Bit::C),
    Opcode::OrReg8(Register8Bit::D),
    Opcode::OrReg8(Register8Bit::E),
    Opcode::OrReg8(Register8Bit::H),
    Opcode::OrReg8(Register8Bit::L),
    Opcode::OrHLAddr,
    Opcode::OrReg8(Register8Bit::A),
    Opcode::CpReg8(Register8Bit::B),
    Opcode::CpReg8(Register8Bit::C),
    Opcode::CpReg8(Register8Bit::D),
    Opcode::CpReg8(Register8Bit::E),
    Opcode::CpReg8(Register8Bit::H),
    Opcode::CpReg8(Register8Bit::L),
    Opcode::CpHlAddr,
    Opcode::CpReg8(Register8Bit::A),
    Opcode::RetCond(Cond::Nz),
    Opcode::PopReg16Stack(Register16BitStack::BC),
    Opcode::JpCondImm16(Cond::Nz),
    Opcode::JpImm16,
    Opcode::CallCondImm16(Cond::Nz),
    Opcode::PushReg16Stack(Register16BitStack::BC),
    Opcode::AddAImm8,
    Opcode::Rst(RstTarget::Addr00),
    Opcode::RetCond(Cond::Z),
    Opcode::Ret,
    Opcode::JpCondImm16(Cond::Z),
    Opcode::Prefix,
    Opcode::CallCondImm16(Cond::Z),
    Opcode::CallImm16,
    Opcode::AdcAImm8,
    Opcode::Rst(RstTarget::Addr08),
    Opcode::RetCond(Cond::Nc),
    Opcode::PopReg16Stack(Register16BitStack::DE),
    Opcode::JpCondImm16(Cond::Nc),
    Opcode::Unimplemented(0xD3),
    Opcode::CallCondImm16(Cond::Nc),
    Opcode::PushReg16Stack(Register16BitStack::DE),
    Opcode::SubImm8,
    Opcode::Rst(RstTarget::Addr10),
    Opcode::RetCond(Cond::C),
    Opcode::Reti,
    Opcode::JpCondImm16(Cond::C),
    Opcode::Unimplemented(0xDB),
    Opcode::CallCondImm16(Cond::C),
    Opcode::Unimplemented(0xDD),
    Opcode::SbcAImm8,
    Opcode::Rst(RstTarget::Addr18),
    Opcode::LdhImm8AddrA,
    Opcode::PopReg16Stack(Register16BitStack::HL),
    Opcode::LdCAddrA,
    Opcode::Unimplemented(0xE3),
    Opcode::Unimplemented(0xE4),
    Opcode::PushReg16Stack(Register16BitStack::HL),
    Opcode::AndImm8,
    Opcode::Rst(RstTarget::Addr20),
    Opcode::AddSpImm8,
    Opcode::JpHlAddr,
    Opcode::LdImm16AddrA,
    Opcode::Unimplemented(0xEB),
    Opcode::Unimplemented(0xEC),
    Opcode::Unimplemented(0xED),
    Opcode::XorImm8,
    Opcode::Rst(RstTarget::Addr28),
    Opcode::LdhAImm8Addr,
    Opcode::PopReg16Stack(Register16BitStack::AF),
    Opcode::LdACAddr,
    Opcode::Unimplemented(0xF3),
    Opcode::Unimplemented(0xF4),
    Opcode::PushReg16Stack(Register16BitStack::AF),
    Opcode::OrImm8,
    Opcode::Rst(RstTarget::Addr30),
    Opcode::Unimplemented(0xF8),
    Opcode::Unimplemented(0xF9),
    Opcode::LdAImm16Addr,
    Opcode::Unimplemented(0xFB),
    Opcode::Unimplemented(0xFC),
    Opcode::Unimplemented(0xFD),
    Opcode::CpImm8,
    Opcode::Rst(RstTarget::Addr38),
];

impl Opcode {
    #[allow(dead_code)]
    pub fn decode(data: u8) -> Opcode {
        OPTABLE[data as usize]
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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Register16Bit {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Register16BitStack {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Cond {
    Nz,
    Z,
    Nc,
    C,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RstTarget {
    Addr00,
    Addr10,
    Addr20,
    Addr30,
    Addr08,
    Addr18,
    Addr28,
    Addr38,
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
    #[case(0xDB, Opcode::Unimplemented(0xDB))]
    #[case(0xDD, Opcode::Unimplemented(0xDD))]
    #[case(0xE3, Opcode::Unimplemented(0xE3))]
    #[case(0xE4, Opcode::Unimplemented(0xE4))]
    #[case(0xEB, Opcode::Unimplemented(0xEB))]
    #[case(0xEC, Opcode::Unimplemented(0xEC))]
    #[case(0xED, Opcode::Unimplemented(0xED))]
    #[case(0xF4, Opcode::Unimplemented(0xF4))]
    #[case(0xFC, Opcode::Unimplemented(0xFC))]
    #[case(0xFD, Opcode::Unimplemented(0xFD))]
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
    #[case(0b00101111, Opcode::Cpl)]
    #[case(0b00110111, Opcode::Scf)]
    #[case(0b00111111, Opcode::Ccf)]
    #[case(0b00011000, Opcode::JrImm8)]
    #[case(0b00100000, Opcode::JrCondImm8(Cond::Nz))]
    #[case(0b00101000, Opcode::JrCondImm8(Cond::Z))]
    #[case(0b00110000, Opcode::JrCondImm8(Cond::Nc))]
    #[case(0b00111000, Opcode::JrCondImm8(Cond::C))]
    #[case(0b10000000, Opcode::AddAReg8(Register8Bit::B))]
    #[case(0b10000001, Opcode::AddAReg8(Register8Bit::C))]
    #[case(0b10000010, Opcode::AddAReg8(Register8Bit::D))]
    #[case(0b10000011, Opcode::AddAReg8(Register8Bit::E))]
    #[case(0b10000100, Opcode::AddAReg8(Register8Bit::H))]
    #[case(0b10000101, Opcode::AddAReg8(Register8Bit::L))]
    #[case(0b10000110, Opcode::AddAHlAddr)]
    #[case(0b10000111, Opcode::AddAReg8(Register8Bit::A))]
    #[case(0b10001000, Opcode::AdcAReg8(Register8Bit::B))]
    #[case(0b10001001, Opcode::AdcAReg8(Register8Bit::C))]
    #[case(0b10001010, Opcode::AdcAReg8(Register8Bit::D))]
    #[case(0b10001011, Opcode::AdcAReg8(Register8Bit::E))]
    #[case(0b10001100, Opcode::AdcAReg8(Register8Bit::H))]
    #[case(0b10001101, Opcode::AdcAReg8(Register8Bit::L))]
    #[case(0b10001110, Opcode::AdcAHlAddr)]
    #[case(0b10001111, Opcode::AdcAReg8(Register8Bit::A))]
    #[case(0b10010000, Opcode::SubAReg8(Register8Bit::B))]
    #[case(0b10010001, Opcode::SubAReg8(Register8Bit::C))]
    #[case(0b10010010, Opcode::SubAReg8(Register8Bit::D))]
    #[case(0b10010011, Opcode::SubAReg8(Register8Bit::E))]
    #[case(0b10010100, Opcode::SubAReg8(Register8Bit::H))]
    #[case(0b10010101, Opcode::SubAReg8(Register8Bit::L))]
    #[case(0b10010110, Opcode::SubAHlAddr)]
    #[case(0b10010111, Opcode::SubAReg8(Register8Bit::A))]
    #[case(0b10011000, Opcode::SbcAReg8(Register8Bit::B))]
    #[case(0b10011001, Opcode::SbcAReg8(Register8Bit::C))]
    #[case(0b10011010, Opcode::SbcAReg8(Register8Bit::D))]
    #[case(0b10011011, Opcode::SbcAReg8(Register8Bit::E))]
    #[case(0b10011100, Opcode::SbcAReg8(Register8Bit::H))]
    #[case(0b10011101, Opcode::SbcAReg8(Register8Bit::L))]
    #[case(0b10011110, Opcode::SbcAHlAddr)]
    #[case(0b10011111, Opcode::SbcAReg8(Register8Bit::A))]
    #[case(0b10100000, Opcode::AndAReg8(Register8Bit::B))]
    #[case(0b10100001, Opcode::AndAReg8(Register8Bit::C))]
    #[case(0b10100010, Opcode::AndAReg8(Register8Bit::D))]
    #[case(0b10100011, Opcode::AndAReg8(Register8Bit::E))]
    #[case(0b10100100, Opcode::AndAReg8(Register8Bit::H))]
    #[case(0b10100101, Opcode::AndAReg8(Register8Bit::L))]
    #[case(0b10100110, Opcode::AndAHlAddr)]
    #[case(0b10100111, Opcode::AndAReg8(Register8Bit::A))]
    #[case(0b10101000, Opcode::XorAReg8(Register8Bit::B))]
    #[case(0b10101001, Opcode::XorAReg8(Register8Bit::C))]
    #[case(0b10101010, Opcode::XorAReg8(Register8Bit::D))]
    #[case(0b10101011, Opcode::XorAReg8(Register8Bit::E))]
    #[case(0b10101100, Opcode::XorAReg8(Register8Bit::H))]
    #[case(0b10101101, Opcode::XorAReg8(Register8Bit::L))]
    #[case(0b10101110, Opcode::XorAHlAddr)]
    #[case(0b10101111, Opcode::XorAReg8(Register8Bit::A))]
    #[case(0xB0, Opcode::OrReg8(Register8Bit::B))]
    #[case(0xB1, Opcode::OrReg8(Register8Bit::C))]
    #[case(0xB2, Opcode::OrReg8(Register8Bit::D))]
    #[case(0xB3, Opcode::OrReg8(Register8Bit::E))]
    #[case(0xB4, Opcode::OrReg8(Register8Bit::H))]
    #[case(0xB5, Opcode::OrReg8(Register8Bit::L))]
    #[case(0xB6, Opcode::OrHLAddr)]
    #[case(0xB7, Opcode::OrReg8(Register8Bit::A))]
    #[case(0xB8, Opcode::CpReg8(Register8Bit::B))]
    #[case(0xB9, Opcode::CpReg8(Register8Bit::C))]
    #[case(0xBA, Opcode::CpReg8(Register8Bit::D))]
    #[case(0xBB, Opcode::CpReg8(Register8Bit::E))]
    #[case(0xBC, Opcode::CpReg8(Register8Bit::H))]
    #[case(0xBD, Opcode::CpReg8(Register8Bit::L))]
    #[case(0xBE, Opcode::CpHlAddr)]
    #[case(0xBF, Opcode::CpReg8(Register8Bit::A))]
    #[case(0xC0, Opcode::RetCond(Cond::Nz))]
    #[case(0xC8, Opcode::RetCond(Cond::Z))]
    #[case(0xD0, Opcode::RetCond(Cond::Nc))]
    #[case(0xD8, Opcode::RetCond(Cond::C))]
    #[case(0xC9, Opcode::Ret)]
    #[case(0xD9, Opcode::Reti)]
    #[case(0xC1, Opcode::PopReg16Stack(Register16BitStack::BC))]
    #[case(0xD1, Opcode::PopReg16Stack(Register16BitStack::DE))]
    #[case(0xE1, Opcode::PopReg16Stack(Register16BitStack::HL))]
    #[case(0xF1, Opcode::PopReg16Stack(Register16BitStack::AF))]
    #[case(0xC5, Opcode::PushReg16Stack(Register16BitStack::BC))]
    #[case(0xD5, Opcode::PushReg16Stack(Register16BitStack::DE))]
    #[case(0xE5, Opcode::PushReg16Stack(Register16BitStack::HL))]
    #[case(0xF5, Opcode::PushReg16Stack(Register16BitStack::AF))]
    #[case(0xC2, Opcode::JpCondImm16(Cond::Nz))]
    #[case(0xD2, Opcode::JpCondImm16(Cond::Nc))]
    #[case(0xCA, Opcode::JpCondImm16(Cond::Z))]
    #[case(0xDA, Opcode::JpCondImm16(Cond::C))]
    #[case(0xC3, Opcode::JpImm16)]
    #[case(0xE9, Opcode::JpHlAddr)]
    #[case(0xC4, Opcode::CallCondImm16(Cond::Nz))]
    #[case(0xD4, Opcode::CallCondImm16(Cond::Nc))]
    #[case(0xCC, Opcode::CallCondImm16(Cond::Z))]
    #[case(0xDC, Opcode::CallCondImm16(Cond::C))]
    #[case(0xCD, Opcode::CallImm16)]
    #[case(0xC7, Opcode::Rst(RstTarget::Addr00))]
    #[case(0xD7, Opcode::Rst(RstTarget::Addr10))]
    #[case(0xE7, Opcode::Rst(RstTarget::Addr20))]
    #[case(0xF7, Opcode::Rst(RstTarget::Addr30))]
    #[case(0xCF, Opcode::Rst(RstTarget::Addr08))]
    #[case(0xDF, Opcode::Rst(RstTarget::Addr18))]
    #[case(0xEF, Opcode::Rst(RstTarget::Addr28))]
    #[case(0xFF, Opcode::Rst(RstTarget::Addr38))]
    #[case(0xCB, Opcode::Prefix)]
    #[case(0xC6, Opcode::AddAImm8)]
    #[case(0xD6, Opcode::SubImm8)]
    #[case(0xE6, Opcode::AndImm8)]
    #[case(0xF6, Opcode::OrImm8)]
    #[case(0xCE, Opcode::AdcAImm8)]
    #[case(0xDE, Opcode::SbcAImm8)]
    #[case(0xEE, Opcode::XorImm8)]
    #[case(0xFE, Opcode::CpImm8)]
    #[case(0xE0, Opcode::LdhImm8AddrA)]
    #[case(0xF0, Opcode::LdhAImm8Addr)]
    #[case(0xE2, Opcode::LdCAddrA)]
    #[case(0xF2, Opcode::LdACAddr)]
    #[case(0xEA, Opcode::LdImm16AddrA)]
    #[case(0xFA, Opcode::LdAImm16Addr)]
    #[case(0xE8, Opcode::AddSpImm8)]
    fn should_return_expected_instruction_given_an_opcode_byte(
        #[case] raw_opcode: u8,
        #[case] result: Opcode,
    ) {
        let opcode = Opcode::decode(raw_opcode);
        assert_eq!(opcode, result);
    }
}
