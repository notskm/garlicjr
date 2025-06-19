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

use crate::number::OverflowHalfCarry;
use crate::opcode::{Cond, Opcode, Register8Bit, Register16Bit, Register16BitStack};
use crate::{Bus, ReadWriteMode};

pub struct SharpSM83 {
    pub registers: Registers,
    #[allow(dead_code)]
    interrupt_master_enable: bool,
    current_tick: u8,
    opcode: Opcode,
    phase: Phase,
    decode_as_prefix_opcode: bool,
    temp_16_bit: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
    pub interrupt_enable: u8,
}

enum Phase {
    Execute,
    Decode,
    Fetch,
}

enum Flags {
    Z = 0b10000000,
    N = 0b01000000,
    H = 0b00100000,
    C = 0b00010000,
}

#[derive(PartialEq)]
enum IncrementMode {
    Increment,
    Decrement,
    None,
}

impl SharpSM83 {
    pub fn new() -> SharpSM83 {
        SharpSM83 {
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                f: 0,
                h: 0,
                l: 0,
                stack_pointer: 0,
                program_counter: 0,
                interrupt_enable: 0,
            },
            interrupt_master_enable: false,
            current_tick: 0,
            opcode: Opcode::Nop,
            phase: Phase::Decode,
            decode_as_prefix_opcode: false,
            temp_16_bit: 0,
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        match self.phase {
            Phase::Decode => {
                match self.current_tick {
                    0 => {
                        self.read_opcode(bus);
                    }
                    1 => {
                        self.phase = Phase::Execute;
                    }
                    _ => (),
                }
                self.current_tick += 1;
            }
            Phase::Execute => {
                self.execute_opcode(bus);
                self.current_tick = self.current_tick.saturating_add(1);
            }
            Phase::Fetch => {
                self.write_program_counter(bus);
                self.phase = Phase::Decode;
                self.current_tick = 0;
                self.increment_program_counter();
            }
        }
    }

    fn write_program_counter(&mut self, bus: &mut Bus) {
        bus.address = self.registers.program_counter;
        bus.mode = ReadWriteMode::Read;
    }

    fn read_opcode(&mut self, bus: &mut Bus) {
        self.opcode = if self.decode_as_prefix_opcode {
            self.decode_as_prefix_opcode = false;
            Opcode::decode_as_prefix(bus.data)
        } else {
            Opcode::decode(bus.data)
        }
    }

    fn increment_program_counter(&mut self) {
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
    }

    fn execute_opcode(&mut self, bus: &mut Bus) {
        match self.opcode {
            Opcode::Nop => self.no_op(),
            Opcode::Prefix => self.prefix(),

            Opcode::LdReg8Imm8(dest) => self.ld_r_n8(dest, bus),
            Opcode::LdReg8Reg8 {
                source,
                destination,
            } => self.ld_r8_r8(source, destination),
            Opcode::LdReg16Imm16(register) => self.ld_r16_imm16(register, bus),
            Opcode::LdReg16AddrA(register) => {
                self.ld_r16_r8(register, Register8Bit::A, IncrementMode::None, bus)
            }
            Opcode::LdHliAddrA => self.ld_r16_r8(
                Register16Bit::HL,
                Register8Bit::A,
                IncrementMode::Increment,
                bus,
            ),
            Opcode::LdHldAddrA => self.ld_r16_r8(
                Register16Bit::HL,
                Register8Bit::A,
                IncrementMode::Decrement,
                bus,
            ),
            Opcode::LdHlAddrReg8(register) => {
                self.ld_r16_r8(Register16Bit::HL, register, IncrementMode::None, bus)
            }
            Opcode::LdAReg16Addr(register) => {
                self.ld_r8_r16addr(Register8Bit::A, register, bus, IncrementMode::None)
            }
            Opcode::LdAHliAddr => self.ld_r8_r16addr(
                Register8Bit::A,
                Register16Bit::HL,
                bus,
                IncrementMode::Increment,
            ),
            Opcode::LdAHldAddr => self.ld_r8_r16addr(
                Register8Bit::A,
                Register16Bit::HL,
                bus,
                IncrementMode::Decrement,
            ),
            Opcode::LdReg8HlAddr(register) => {
                self.ld_r8_r16addr(register, Register16Bit::HL, bus, IncrementMode::None);
            }
            Opcode::LdCAddrA => self.ld_caddr_a(bus),
            Opcode::LdACAddr => self.ld_a_caddr(bus),
            Opcode::LdImm16AddrA => self.ld_imm16addr_a(bus),
            Opcode::LdAImm16Addr => self.ld_a_imm16addr(bus),
            Opcode::LdhImm8AddrA => self.ldh_imm8addr_a(bus),
            Opcode::LdhAImm8Addr => self.ldh_a_imm8addr(bus),

            Opcode::AddAReg8(register) => self.add_a_r8(register),
            Opcode::SubAReg8(register) => self.sub_a_r8(register),
            Opcode::AddAHlAddr => self.add_a_hladdr(bus),
            Opcode::SubAHlAddr => self.sub_a_hladdr(bus),
            Opcode::AdcAReg8(register) => self.adc_a_r8(register),
            Opcode::SbcAReg8(register) => self.sbc_a_r8(register),
            Opcode::AddAImm8 => self.add_a_imm8(bus),
            Opcode::SubImm8 => self.sub_a_imm8(bus),

            Opcode::IncReg8(register) => self.inc_r8(register),
            Opcode::DecReg8(register) => self.dec_r8(register),
            Opcode::IncReg16(register) => self.inc_r16(register),
            Opcode::DecReg16(register) => self.dec_r16(register),

            Opcode::AndAReg8(register) => self.and_a_r8(register),
            Opcode::OrAReg8(register) => self.or_a_r8(register),
            Opcode::XorAReg8(register) => self.xor_a_r8(register),
            Opcode::Rla => self.rla(),
            Opcode::AndImm8 => self.and_a_imm8(bus),
            Opcode::XorAHlAddr => self.xor_a_hl_addr(bus),

            Opcode::CpReg8(register) => self.cp_a_r8(register),
            Opcode::CpImm8 => self.cp_a_imm8(bus),
            Opcode::CpHlAddr => self.cp_a_hladdr(bus),

            Opcode::JpImm16 => self.jp_imm16(bus),
            Opcode::JrCondImm8(condition) => self.jr_cond_imm8(condition, bus),
            Opcode::JrImm8 => self.jr_imm8(bus),

            Opcode::CallImm16 => self.call_imm16(bus),
            Opcode::CallCondImm16(condition) => self.call_cond_imm16(condition, bus),
            Opcode::Ret => self.ret(bus),
            Opcode::PushReg16Stack(register) => self.push_r16_stack(register, bus),
            Opcode::PopReg16Stack(register) => self.pop_r16_stack(register, bus),

            Opcode::RlcReg8(register) => self.rlc_r8(register),
            Opcode::RlcHlAddr => self.rlc_hladdr(bus),
            Opcode::Rl(register) => self.rl_r8(register),
            Opcode::Bit(mask, register) => self.bit(mask, register),
            Opcode::Srl(register) => self.srl(register),

            Opcode::Unimplemented(_) => {}
            _ => {}
        }
    }

    fn no_op(&mut self) {
        if self.current_tick == 2 {
            self.phase = Phase::Fetch;
        }
    }

    fn prefix(&mut self) {
        if self.current_tick == 2 {
            self.decode_as_prefix_opcode = true;
            self.phase = Phase::Fetch;
        }
    }

    fn ld_r_n8(&mut self, destination: Register8Bit, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.mode = ReadWriteMode::Read;
                bus.address = self.registers.program_counter;
                self.increment_program_counter();
            }
            4 => {
                self.write_to_register(destination, bus.data);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_r8_r8(&mut self, source: Register8Bit, destination: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(source);
            self.write_to_register(destination, data);
            self.phase = Phase::Fetch;
        }
    }

    fn ld_r16_imm16(&mut self, register: Register16Bit, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                self.increment_program_counter();
            }
            4 => {
                let low = bus.data;
                self.write_to_16_bit_register_low(register, low);

                bus.address = self.registers.program_counter;
                self.increment_program_counter();
            }
            8 => {
                let high = bus.data;
                self.write_to_16_bit_register_high(register, high);
            }
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_r16_r8(
        &mut self,
        destination: Register16Bit,
        source: Register8Bit,
        mode: IncrementMode,
        bus: &mut Bus,
    ) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(destination);
                bus.data = self.read_from_register(source);
                bus.mode = ReadWriteMode::Write;

                match mode {
                    IncrementMode::Increment => self.add_to_16_bit_register(destination, 1),
                    IncrementMode::Decrement => self.sub_from_16_bit_register(destination, 1),
                    IncrementMode::None => (),
                };
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_r8_r16addr(
        &mut self,
        destination: Register8Bit,
        source: Register16Bit,
        bus: &mut Bus,
        mode: IncrementMode,
    ) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(source);
            }
            4 => {
                self.write_to_register(destination, bus.data);

                match mode {
                    IncrementMode::Increment => {
                        self.add_to_16_bit_register(source, 1);
                    }
                    IncrementMode::Decrement => {
                        self.sub_from_16_bit_register(source, 1);
                    }
                    _ => (),
                }
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_caddr_a(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = 0xFF00 + self.registers.c as u16;
                bus.data = self.registers.a;
                bus.mode = ReadWriteMode::Write;
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_a_caddr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = 0xFF00 + self.registers.c as u16;
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                self.registers.a = bus.data;
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_imm16addr_a(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();

                self.temp_16_bit = bus.data as u16;
            }
            8 => {
                self.temp_16_bit |= (bus.data as u16) << 8;

                bus.address = self.temp_16_bit;
                bus.data = self.registers.a;
                bus.mode = ReadWriteMode::Write;
            }
            14 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_a_imm16addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                self.temp_16_bit = bus.data as u16;

                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            8 => {
                self.temp_16_bit |= (bus.data as u16) << 8;

                bus.address = self.temp_16_bit;
                bus.mode = ReadWriteMode::Read;
            }
            12 => {
                self.registers.a = bus.data;
            }
            14 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ldh_imm8addr_a(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                bus.address = 0xFF00 + bus.data as u16;
                bus.data = self.registers.a;
                bus.mode = ReadWriteMode::Write;
            }
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ldh_a_imm8addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                bus.address = 0xFF00 + bus.data as u16;
                bus.mode = ReadWriteMode::Read;
            }
            8 => {
                self.registers.a = bus.data;
            }
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn read_from_register(&mut self, register: Register8Bit) -> u8 {
        match register {
            Register8Bit::A => self.registers.a,
            Register8Bit::B => self.registers.b,
            Register8Bit::C => self.registers.c,
            Register8Bit::D => self.registers.d,
            Register8Bit::E => self.registers.e,
            Register8Bit::H => self.registers.h,
            Register8Bit::L => self.registers.l,
        }
    }

    fn write_to_register(&mut self, dest: Register8Bit, data: u8) {
        match dest {
            Register8Bit::A => self.registers.a = data,
            Register8Bit::B => self.registers.b = data,
            Register8Bit::C => self.registers.c = data,
            Register8Bit::D => self.registers.d = data,
            Register8Bit::E => self.registers.e = data,
            Register8Bit::H => self.registers.h = data,
            Register8Bit::L => self.registers.l = data,
        };
    }

    fn read_from_16_bit_register(&mut self, register: Register16Bit) -> u16 {
        match register {
            Register16Bit::BC => u16::from_be_bytes([self.registers.b, self.registers.c]),
            Register16Bit::DE => u16::from_be_bytes([self.registers.d, self.registers.e]),
            Register16Bit::HL => u16::from_be_bytes([self.registers.h, self.registers.l]),
            Register16Bit::SP => self.registers.stack_pointer,
        }
    }

    fn read_from_16_bit_stack_register(&mut self, register: Register16BitStack) -> u16 {
        match register {
            Register16BitStack::BC => u16::from_be_bytes([self.registers.b, self.registers.c]),
            Register16BitStack::DE => u16::from_be_bytes([self.registers.d, self.registers.e]),
            Register16BitStack::HL => u16::from_be_bytes([self.registers.h, self.registers.l]),
            Register16BitStack::AF => u16::from_be_bytes([self.registers.a, self.registers.f]),
        }
    }

    fn write_to_16_bit_register(&mut self, register: Register16Bit, value: u16) {
        let [high, low] = value.to_be_bytes();
        self.write_to_16_bit_register_high(register, high);
        self.write_to_16_bit_register_low(register, low);
    }

    fn write_to_16_bit_register_low(&mut self, dest: Register16Bit, data: u8) {
        match dest {
            Register16Bit::BC => self.write_to_register(Register8Bit::C, data),
            Register16Bit::DE => self.write_to_register(Register8Bit::E, data),
            Register16Bit::HL => self.write_to_register(Register8Bit::L, data),
            Register16Bit::SP => Self::write_to_16_bit_low(&mut self.registers.stack_pointer, data),
        }
    }

    fn write_to_16_bit_register_high(&mut self, dest: Register16Bit, data: u8) {
        match dest {
            Register16Bit::BC => self.write_to_register(Register8Bit::B, data),
            Register16Bit::DE => self.write_to_register(Register8Bit::D, data),
            Register16Bit::HL => self.write_to_register(Register8Bit::H, data),
            Register16Bit::SP => {
                Self::write_to_16_bit_high(&mut self.registers.stack_pointer, data)
            }
        }
    }

    fn write_to_16_bit_stack_register_low(&mut self, dest: Register16BitStack, data: u8) {
        match dest {
            Register16BitStack::BC => self.write_to_register(Register8Bit::C, data),
            Register16BitStack::DE => self.write_to_register(Register8Bit::E, data),
            Register16BitStack::HL => self.write_to_register(Register8Bit::L, data),
            Register16BitStack::AF => self.registers.f = data,
        }
    }

    fn write_to_16_bit_stack_register_high(&mut self, dest: Register16BitStack, data: u8) {
        match dest {
            Register16BitStack::BC => self.write_to_register(Register8Bit::B, data),
            Register16BitStack::DE => self.write_to_register(Register8Bit::D, data),
            Register16BitStack::HL => self.write_to_register(Register8Bit::H, data),
            Register16BitStack::AF => self.write_to_register(Register8Bit::A, data),
        }
    }

    fn add_to_16_bit_register(&mut self, register: Register16Bit, value: u16) {
        let data = self.read_from_16_bit_register(register);
        let result = data.wrapping_add(value);
        self.write_to_16_bit_register(register, result);
    }

    fn sub_from_16_bit_register(&mut self, register: Register16Bit, value: u16) {
        let data = self.read_from_16_bit_register(register);
        let result = data.wrapping_sub(value);
        self.write_to_16_bit_register(register, result);
    }

    fn write_to_16_bit_low(destination: &mut u16, data: u8) {
        *destination &= 0b1111111100000000;
        *destination |= data as u16;
    }

    fn write_to_16_bit_high(destination: &mut u16, data: u8) {
        let data_16_bit = (data as u16) << 8;

        *destination &= 0b0000000011111111;
        *destination |= data_16_bit;
    }

    fn add_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);

            let (new_value, carry, half_carry) =
                self.registers.a.overflowing_add_with_half_carry(data);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, half_carry);
            self.set_flag(Flags::C, carry);
            self.registers.a = new_value;

            self.phase = Phase::Fetch;
        }
    }

    fn adc_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);
            let carry_flag = self.get_flag(Flags::C);

            let (new_value, carry, half_carry) =
                self.registers.a.full_overflowing_add(data, carry_flag);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, half_carry);
            self.set_flag(Flags::C, carry);
            self.registers.a = new_value;

            self.phase = Phase::Fetch;
        }
    }

    fn sbc_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);
            let carry_flag = self.get_flag(Flags::C);

            let (new_value, carry, half_carry) =
                self.registers.a.full_overflowing_sub(data, carry_flag);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, true);
            self.set_flag(Flags::H, half_carry);
            self.set_flag(Flags::C, carry);
            self.registers.a = new_value;

            self.phase = Phase::Fetch;
        }
    }

    fn add_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let (new_value, carry, half_carry) =
                    self.registers.a.overflowing_add_with_half_carry(bus.data);

                self.set_flag(Flags::Z, new_value == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, half_carry);
                self.set_flag(Flags::C, carry);
                self.registers.a = new_value;
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn sub_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let (new_value, carry, half_carry) =
                    self.registers.a.overflowing_sub_with_half_carry(bus.data);

                self.set_flag(Flags::Z, new_value == 0);
                self.set_flag(Flags::N, true);
                self.set_flag(Flags::H, half_carry);
                self.set_flag(Flags::C, carry);
                self.registers.a = new_value;
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn sub_a_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let (new_value, borrow, half_borrow) =
            self.registers.a.overflowing_sub_with_half_carry(data);

        self.set_flag(Flags::Z, new_value == 0);
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::H, half_borrow);
        self.set_flag(Flags::C, borrow);
        self.registers.a = new_value;

        self.phase = Phase::Fetch;
    }

    fn add_a_hladdr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let (new_value, carry, half_carry) =
                    self.registers.a.overflowing_add_with_half_carry(bus.data);

                self.set_flag(Flags::Z, new_value == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, half_carry);
                self.set_flag(Flags::C, carry);
                self.registers.a = new_value;
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn sub_a_hladdr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let (new_value, borrow, half_borrow) =
                    self.registers.a.overflowing_sub_with_half_carry(bus.data);

                self.set_flag(Flags::Z, new_value == 0);
                self.set_flag(Flags::N, true);
                self.set_flag(Flags::H, half_borrow);
                self.set_flag(Flags::C, borrow);
                self.registers.a = new_value;
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn inc_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let (new_value, _, half_carry) = data.overflowing_add_with_half_carry(1);

        self.write_to_register(register, new_value);

        self.set_flag(Flags::Z, new_value == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, half_carry);

        self.phase = Phase::Fetch;
    }

    fn dec_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let (new_value, _, half_borrow) = data.overflowing_sub_with_half_carry(1);

        self.write_to_register(register, new_value);

        self.set_flag(Flags::Z, new_value == 0);
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::H, half_borrow);

        self.phase = Phase::Fetch;
    }

    fn inc_r16(&mut self, register: Register16Bit) {
        match self.current_tick {
            2 => {
                self.add_to_16_bit_register(register, 1);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn dec_r16(&mut self, register: Register16Bit) {
        match self.current_tick {
            2 => {
                self.sub_from_16_bit_register(register, 1);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn xor_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            self.registers.a ^= self.read_from_register(register);
            self.set_flag(Flags::Z, self.registers.a == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, false);
            self.phase = Phase::Fetch;
        }
    }

    fn and_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            self.registers.a &= self.read_from_register(register);
            self.set_flag(Flags::Z, self.registers.a == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, true);
            self.set_flag(Flags::C, false);
            self.phase = Phase::Fetch;
        }
    }

    fn or_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            self.registers.a |= self.read_from_register(register);
            self.set_flag(Flags::Z, self.registers.a == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, false);
            self.phase = Phase::Fetch;
        }
    }

    fn rla(&mut self) {
        self.rl_r8(Register8Bit::A);
        self.set_flag(Flags::Z, false);
    }

    fn and_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                self.registers.a &= bus.data;
                self.set_flag(Flags::Z, self.registers.a == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, true);
                self.set_flag(Flags::C, false);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn xor_a_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                let address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
                bus.address = address;
            }
            4 => {
                self.registers.a ^= bus.data;
                self.set_flag(Flags::Z, self.registers.a == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, false);
                self.set_flag(Flags::C, false);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn cp_a_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let (result, borrow, half_borrow) = self.registers.a.overflowing_sub_with_half_carry(data);

        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::H, half_borrow);
        self.set_flag(Flags::C, borrow);
        self.phase = Phase::Fetch;
    }

    fn cp_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let (result, borrow, half_borrow) =
                    self.registers.a.overflowing_sub_with_half_carry(bus.data);
                self.set_flag(Flags::Z, result == 0);
                self.set_flag(Flags::N, true);
                self.set_flag(Flags::H, half_borrow);
                self.set_flag(Flags::C, borrow);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn cp_a_hladdr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let (result, borrow, half_borrow) =
                    self.registers.a.overflowing_sub_with_half_carry(bus.data);

                self.set_flag(Flags::Z, result == 0);
                self.set_flag(Flags::N, true);
                self.set_flag(Flags::H, half_borrow);
                self.set_flag(Flags::C, borrow);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn jp_imm16(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                self.increment_program_counter();
            }
            4 => {
                bus.address = self.registers.program_counter;
                self.registers.program_counter &= 0b1111111100000000;
                self.registers.program_counter |= bus.data as u16;
            }
            8 => {
                self.registers.program_counter &= 0b0000000011111111;
                self.registers.program_counter |= (bus.data as u16) << 8;
            }
            14 => self.phase = Phase::Fetch,
            _ => (),
        }
    }

    fn jr_cond_imm8(&mut self, condition: crate::opcode::Cond, bus: &mut Bus) {
        let should_jump = match condition {
            Cond::Z => self.registers.f & Flags::Z as u8 > 0,
            Cond::Nz => self.registers.f & Flags::Z as u8 == 0,
            Cond::C => self.registers.f & Flags::C as u8 > 0,
            Cond::Nc => self.registers.f & Flags::C as u8 == 0,
        };

        match (self.current_tick, should_jump) {
            (2, _) => {
                bus.address = self.registers.program_counter;
                self.increment_program_counter();
            }
            (4, true) => {
                let new_pc = self
                    .registers
                    .program_counter
                    .wrapping_add_signed(bus.data as i8 as i16);
                self.registers.program_counter = new_pc;
            }
            (6, false) => self.phase = Phase::Fetch,
            (10, true) => self.phase = Phase::Fetch,
            (_, _) => (),
        }
    }

    fn jr_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                self.increment_program_counter();
            }
            4 => {
                let new_pc = self
                    .registers
                    .program_counter
                    .wrapping_add_signed(bus.data as i8 as i16);
                self.registers.program_counter = new_pc;
            }
            10 => self.phase = Phase::Fetch,
            _ => (),
        }
    }

    fn call_imm16(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                self.temp_16_bit &= 0b1111111100000000;
                self.temp_16_bit |= bus.data as u16;

                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            8 => {
                self.temp_16_bit &= 0b0000000011111111;
                self.temp_16_bit |= (bus.data as u16) << 8u16;
            }
            12 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                bus.address = self.registers.stack_pointer;
                bus.data = ((self.registers.program_counter & 0xFF00u16) >> 8u16) as u8;
                bus.mode = ReadWriteMode::Write;
            }
            16 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                bus.address = self.registers.stack_pointer;
                bus.data = (self.registers.program_counter & 0x00FFu16) as u8;
                bus.mode = ReadWriteMode::Write;

                self.registers.program_counter = self.temp_16_bit;
            }
            22 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn call_cond_imm16(&mut self, condition: Cond, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                self.temp_16_bit &= 0b1111111100000000;
                self.temp_16_bit |= bus.data as u16;

                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            8 => {
                self.temp_16_bit &= 0b0000000011111111;
                self.temp_16_bit |= (bus.data as u16) << 8u16;
            }
            10 => {
                let should_jump = match condition {
                    Cond::Nz => !self.get_flag(Flags::Z),
                    Cond::Z => self.get_flag(Flags::Z),
                    Cond::Nc => !self.get_flag(Flags::C),
                    Cond::C => self.get_flag(Flags::C),
                };
                if !should_jump {
                    self.phase = Phase::Fetch;
                }
            }
            12 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                bus.address = self.registers.stack_pointer;
                bus.data = ((self.registers.program_counter & 0xFF00u16) >> 8u16) as u8;
                bus.mode = ReadWriteMode::Write;
            }
            16 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                bus.address = self.registers.stack_pointer;
                bus.data = (self.registers.program_counter & 0x00FFu16) as u8;
                bus.mode = ReadWriteMode::Write;

                self.registers.program_counter = self.temp_16_bit;
            }
            22 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ret(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.stack_pointer;
                bus.mode = ReadWriteMode::Read;
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
            }
            4 => {
                self.registers.program_counter &= 0xFF00;
                self.registers.program_counter |= bus.data as u16;

                bus.address = self.registers.stack_pointer;
                bus.mode = ReadWriteMode::Read;
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
            }
            8 => {
                self.registers.program_counter &= 0x00FF;
                self.registers.program_counter |= (bus.data as u16) << 8;
            }
            14 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn push_r16_stack(&mut self, register: Register16BitStack, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_stack_register(register);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let address = self.read_from_16_bit_stack_register(register);
                let address_high = ((address & 0xFF00u16) >> 8) as u8;

                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                bus.address = self.registers.stack_pointer;
                bus.data = address_high;
                bus.mode = ReadWriteMode::Write;
            }
            8 => {
                let address = self.read_from_16_bit_stack_register(register);
                let address_low = (address & 0x00FFu16) as u8;

                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                bus.address = self.registers.stack_pointer;
                bus.data = address_low;
                bus.mode = ReadWriteMode::Write;
            }
            14 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn pop_r16_stack(&mut self, register: Register16BitStack, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.stack_pointer;
                bus.mode = ReadWriteMode::Read;
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
            }
            4 => {
                let data = if register == Register16BitStack::AF {
                    bus.data & 0b11110000
                } else {
                    bus.data
                };

                self.write_to_16_bit_stack_register_low(register, data);

                bus.address = self.registers.stack_pointer;
                bus.mode = ReadWriteMode::Read;
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
            }
            8 => {
                self.write_to_16_bit_stack_register_high(register, bus.data);
            }
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn rlc_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);
            let overflow = data & 0b10000000 > 0;
            let data = data.rotate_left(1);
            self.write_to_register(register, data);

            self.set_flag(Flags::Z, data == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, overflow);

            self.phase = Phase::Fetch;
        }
    }

    fn rl_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);
            let overflow = data & 0b10000000 > 0;
            let data = data << 1;
            let data = data | self.get_flag(Flags::C) as u8;

            self.write_to_register(register, data);

            self.set_flag(Flags::Z, data == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, overflow);

            self.phase = Phase::Fetch;
        }
    }

    fn bit(&mut self, bit: u8, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);

            let mask = 0b00000001 << bit;
            let bit_is_0 = data & mask == 0;

            self.set_flag(Flags::Z, bit_is_0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, true);

            self.phase = Phase::Fetch;
        }
    }

    fn srl(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let value = self.read_from_register(register);
            let carry = value & 1 > 0;
            let new_value = value >> 1;
            self.write_to_register(register, new_value);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, carry);

            self.phase = Phase::Fetch;
        }
    }

    fn rlc_hladdr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let overflow = bus.data & 0b10000000 > 0;

                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.data = bus.data.rotate_left(1);
                bus.mode = ReadWriteMode::Write;

                self.set_flag(Flags::Z, bus.data == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, false);
                self.set_flag(Flags::C, overflow);
            }
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn get_flag(&mut self, flag: Flags) -> bool {
        let mask = flag as u8;
        self.registers.f & mask > 0
    }

    fn set_flag(&mut self, flag: Flags, value: bool) {
        let mask = flag as u8;
        if value {
            self.registers.f |= mask;
        } else {
            self.registers.f &= !mask;
        }
    }
}

impl Default for SharpSM83 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;
    use serde::Deserialize;
    use std::fs;
    use std::path::Path;

    #[derive(Deserialize)]
    struct JsonTest {
        pub name: String,

        #[serde(rename = "initial")]
        pub initial_state: JsonTestState,

        #[serde(rename = "final")]
        pub final_state: JsonTestState,
        pub cycles: Vec<Option<JsonTestCycleEntry>>,
    }

    #[derive(Deserialize)]
    struct JsonTestState {
        pub pc: u16,
        pub sp: u16,
        pub a: u8,
        pub b: u8,
        pub c: u8,
        pub d: u8,
        pub e: u8,
        pub f: u8,
        pub h: u8,
        pub l: u8,

        #[serde(deserialize_with = "deserialize_optional_int_as_bool")]
        #[serde(default)]
        pub ime: Option<bool>,

        #[serde(deserialize_with = "deserialize_optional_int_as_bool")]
        #[serde(default)]
        pub ie: Option<bool>,

        pub ram: Vec<JsonTestRamEntry>,
    }

    #[derive(Deserialize)]
    struct JsonTestRamEntry {
        pub address: u16,
        pub value: u8,
    }

    #[derive(Deserialize, Clone)]
    struct JsonTestCycleEntry {
        pub address: u16,
        pub data: Option<u8>,
        pub flags: String,
    }

    use serde::de;
    fn deserialize_optional_int_as_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s: Option<u64> = de::Deserialize::deserialize(deserializer)?;

        match s {
            Some(1) => Ok(Some(true)),
            Some(0) => Ok(Some(false)),
            None => Ok(None),
            Some(value) => Err(de::Error::invalid_value(
                de::Unexpected::Unsigned(value),
                &"0 or 1",
            )),
        }
    }

    #[rstest]
    #[case("00.json")]
    #[case("01.json")]
    #[case("02.json")]
    #[case("03.json")]
    #[case("04.json")]
    #[case("05.json")]
    #[case("06.json")]
    #[case("0e.json")]
    #[case("0a.json")]
    #[case("0b.json")]
    #[case("0c.json")]
    #[case("0d.json")]
    #[case("11.json")]
    #[case("12.json")]
    #[case("13.json")]
    #[case("14.json")]
    #[case("15.json")]
    #[case("16.json")]
    #[case("17.json")]
    #[case("18.json")]
    #[case("1a.json")]
    #[case("1b.json")]
    #[case("1c.json")]
    #[case("1d.json")]
    #[case("1e.json")]
    #[case("20.json")]
    #[case("21.json")]
    #[case("22.json")]
    #[case("23.json")]
    #[case("24.json")]
    #[case("25.json")]
    #[case("26.json")]
    #[case("28.json")]
    #[case("2a.json")]
    #[case("2b.json")]
    #[case("2c.json")]
    #[case("2d.json")]
    #[case("2e.json")]
    #[case("30.json")]
    #[case("31.json")]
    #[case("32.json")]
    #[case("33.json")]
    #[case("3a.json")]
    #[case("3b.json")]
    #[case("3c.json")]
    #[case("3d.json")]
    #[case("3e.json")]
    #[case("38.json")]
    #[case("40.json")]
    #[case("41.json")]
    #[case("42.json")]
    #[case("43.json")]
    #[case("44.json")]
    #[case("45.json")]
    #[case("46.json")]
    #[case("47.json")]
    #[case("48.json")]
    #[case("49.json")]
    #[case("4a.json")]
    #[case("4b.json")]
    #[case("4c.json")]
    #[case("4d.json")]
    #[case("4e.json")]
    #[case("4f.json")]
    #[case("50.json")]
    #[case("51.json")]
    #[case("52.json")]
    #[case("53.json")]
    #[case("54.json")]
    #[case("55.json")]
    #[case("56.json")]
    #[case("57.json")]
    #[case("58.json")]
    #[case("59.json")]
    #[case("5a.json")]
    #[case("5b.json")]
    #[case("5c.json")]
    #[case("5d.json")]
    #[case("5e.json")]
    #[case("5f.json")]
    #[case("60.json")]
    #[case("61.json")]
    #[case("62.json")]
    #[case("63.json")]
    #[case("64.json")]
    #[case("65.json")]
    #[case("66.json")]
    #[case("67.json")]
    #[case("68.json")]
    #[case("69.json")]
    #[case("6a.json")]
    #[case("6b.json")]
    #[case("6c.json")]
    #[case("6d.json")]
    #[case("6e.json")]
    #[case("6f.json")]
    #[case("70.json")]
    #[case("71.json")]
    #[case("72.json")]
    #[case("73.json")]
    #[case("74.json")]
    #[case("75.json")]
    #[case("77.json")]
    #[case("78.json")]
    #[case("79.json")]
    #[case("7a.json")]
    #[case("7b.json")]
    #[case("7c.json")]
    #[case("7d.json")]
    #[case("7e.json")]
    #[case("7f.json")]
    #[case("80.json")]
    #[case("81.json")]
    #[case("82.json")]
    #[case("83.json")]
    #[case("84.json")]
    #[case("85.json")]
    #[case("86.json")]
    #[case("87.json")]
    #[case("88.json")]
    #[case("89.json")]
    #[case("8a.json")]
    #[case("8b.json")]
    #[case("8c.json")]
    #[case("8d.json")]
    #[case("8f.json")]
    #[case("90.json")]
    #[case("91.json")]
    #[case("92.json")]
    #[case("93.json")]
    #[case("94.json")]
    #[case("95.json")]
    #[case("96.json")]
    #[case("97.json")]
    #[case("98.json")]
    #[case("99.json")]
    #[case("9a.json")]
    #[case("9b.json")]
    #[case("9c.json")]
    #[case("9d.json")]
    #[case("9f.json")]
    #[case("a0.json")]
    #[case("a1.json")]
    #[case("a2.json")]
    #[case("a3.json")]
    #[case("a4.json")]
    #[case("a5.json")]
    #[case("a7.json")]
    #[case("a8.json")]
    #[case("a9.json")]
    #[case("aa.json")]
    #[case("ab.json")]
    #[case("ac.json")]
    #[case("ad.json")]
    #[case("ae.json")]
    #[case("af.json")]
    #[case("b0.json")]
    #[case("b1.json")]
    #[case("b2.json")]
    #[case("b3.json")]
    #[case("b4.json")]
    #[case("b5.json")]
    #[case("b7.json")]
    #[case("b8.json")]
    #[case("b9.json")]
    #[case("ba.json")]
    #[case("bb.json")]
    #[case("bc.json")]
    #[case("bd.json")]
    #[case("be.json")]
    #[case("bf.json")]
    #[case("c1.json")]
    #[case("c3.json")]
    #[case("c4.json")]
    #[case("c5.json")]
    #[case("c6.json")]
    #[case("c9.json")]
    #[case("cc.json")]
    #[case("cd.json")]
    #[case("d1.json")]
    #[case("d4.json")]
    #[case("d5.json")]
    #[case("d6.json")]
    #[case("dc.json")]
    #[case("e0.json")]
    #[case("e1.json")]
    #[case("e2.json")]
    #[case("e5.json")]
    #[case("e6.json")]
    #[case("ea.json")]
    #[case("f0.json")]
    #[case("f1.json")]
    #[case("f2.json")]
    #[case("f5.json")]
    #[case("fa.json")]
    #[case("fe.json")]
    #[case("cb_00.json")]
    #[case("cb_01.json")]
    #[case("cb_02.json")]
    #[case("cb_03.json")]
    #[case("cb_04.json")]
    #[case("cb_05.json")]
    #[case("cb_06.json")]
    #[case("cb_07.json")]
    #[case("cb_10.json")]
    #[case("cb_11.json")]
    #[case("cb_12.json")]
    #[case("cb_13.json")]
    #[case("cb_14.json")]
    #[case("cb_15.json")]
    #[case("cb_17.json")]
    #[case("cb_38.json")]
    #[case("cb_38.json")]
    #[case("cb_39.json")]
    #[case("cb_3a.json")]
    #[case("cb_3b.json")]
    #[case("cb_3c.json")]
    #[case("cb_3d.json")]
    #[case("cb_3f.json")]
    #[case("cb_40.json")]
    #[case("cb_41.json")]
    #[case("cb_42.json")]
    #[case("cb_43.json")]
    #[case("cb_44.json")]
    #[case("cb_45.json")]
    #[case("cb_47.json")]
    #[case("cb_48.json")]
    #[case("cb_49.json")]
    #[case("cb_4a.json")]
    #[case("cb_4b.json")]
    #[case("cb_4c.json")]
    #[case("cb_4d.json")]
    #[case("cb_4f.json")]
    #[case("cb_50.json")]
    #[case("cb_51.json")]
    #[case("cb_52.json")]
    #[case("cb_53.json")]
    #[case("cb_54.json")]
    #[case("cb_55.json")]
    #[case("cb_57.json")]
    #[case("cb_58.json")]
    #[case("cb_59.json")]
    #[case("cb_5a.json")]
    #[case("cb_5b.json")]
    #[case("cb_5c.json")]
    #[case("cb_5d.json")]
    #[case("cb_5f.json")]
    #[case("cb_60.json")]
    #[case("cb_61.json")]
    #[case("cb_62.json")]
    #[case("cb_63.json")]
    #[case("cb_64.json")]
    #[case("cb_65.json")]
    #[case("cb_67.json")]
    #[case("cb_68.json")]
    #[case("cb_69.json")]
    #[case("cb_6a.json")]
    #[case("cb_6b.json")]
    #[case("cb_6c.json")]
    #[case("cb_6d.json")]
    #[case("cb_6f.json")]
    #[case("cb_70.json")]
    #[case("cb_71.json")]
    #[case("cb_72.json")]
    #[case("cb_73.json")]
    #[case("cb_74.json")]
    #[case("cb_75.json")]
    #[case("cb_77.json")]
    #[case("cb_78.json")]
    #[case("cb_79.json")]
    #[case("cb_7a.json")]
    #[case("cb_7b.json")]
    #[case("cb_7c.json")]
    #[case("cb_7d.json")]
    #[case("cb_7f.json")]
    fn should_pass_gameboycputtests_json_tests(#[case] test_file: &str) {
        let test_data: Vec<JsonTest> = {
            let test_filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("test-data")
                .join("json-tests")
                .join("GameBoyCPUTests")
                .join("v2")
                .join(test_file);

            let buf = fs::read_to_string(test_filepath).unwrap();
            serde_json::from_str(&buf).unwrap()
        };

        for test in test_data {
            println!("Test name: {}", test.name);

            // These tests don't use the IE and IME fields.
            // This is a sanity check to ensure we're not ignoring test data.
            assert_eq!(test.initial_state.ie, None);
            assert_eq!(test.initial_state.ime, None);
            assert_eq!(test.final_state.ie, None);
            assert_eq!(test.final_state.ime, None);

            let mut cpu = SharpSM83::new();
            let mut ram = [0u8; 64 * 1024];
            let mut bus = Bus::new();

            let initial_state = Registers {
                a: test.initial_state.a,
                b: test.initial_state.b,
                c: test.initial_state.c,
                d: test.initial_state.d,
                e: test.initial_state.e,
                f: test.initial_state.f,
                h: test.initial_state.h,
                l: test.initial_state.l,
                interrupt_enable: 0u8,
                program_counter: test.initial_state.pc,
                stack_pointer: test.initial_state.sp,
            };

            cpu.registers = initial_state.clone();

            for ram_data in &test.initial_state.ram {
                let ram_index = ram_data.address as usize;
                ram[ram_index] = ram_data.value;
            }

            bus.address = test.initial_state.pc - 1;
            bus.data = ram[bus.address as usize];

            assert_eq!(
                cpu.registers, initial_state,
                "CPU is not in the correct initial state"
            );

            for i in 0..test.cycles.len() {
                for _ in 0..4 {
                    cpu.tick(&mut bus);
                }

                if bus.mode == ReadWriteMode::Read {
                    bus.data = ram[bus.address as usize];
                } else if bus.mode == ReadWriteMode::Write {
                    ram[bus.address as usize] = bus.data;
                }

                if let Some(cycle) = &test.cycles[i] {
                    let read = cycle.flags.contains("read");
                    let write = cycle.flags.contains("write");

                    if read {
                        assert_eq!(
                            bus.mode,
                            ReadWriteMode::Read,
                            "Expected bus in read mode on cycle {}",
                            i
                        );
                    }

                    if write {
                        assert_eq!(
                            bus.mode,
                            ReadWriteMode::Write,
                            "Expected bus in write mode on cycle {}",
                            i
                        );
                    }

                    assert_eq!(
                        bus.address, cycle.address,
                        "Expected address {} on bus after M-cycle {}, got {}",
                        cycle.address, i, bus.address
                    );

                    assert_eq!(
                        bus.data,
                        cycle.data.unwrap(),
                        "Expected data {} on bus after M-cycle {}, got {}",
                        cycle.data.unwrap(),
                        i,
                        bus.data
                    );
                }
            }

            let final_state = Registers {
                a: test.final_state.a,
                b: test.final_state.b,
                c: test.final_state.c,
                d: test.final_state.d,
                e: test.final_state.e,
                f: test.final_state.f,
                h: test.final_state.h,
                l: test.final_state.l,
                interrupt_enable: 0u8,
                program_counter: test.final_state.pc,
                stack_pointer: test.final_state.sp,
            };

            assert_eq!(cpu.registers, final_state);

            for ram_data in &test.final_state.ram {
                let ram_index = ram_data.address as usize;
                let data = ram[ram_index];
                assert_eq!(data, ram_data.value);
            }
        }
    }
}
