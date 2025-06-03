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
                self.current_tick += 1;
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
            Opcode::IncReg8(register) => self.inc_r8(register),
            Opcode::DecReg8(register) => self.dec_r8(register),
            Opcode::XorAReg8(register) => self.xor_a_r8(register),
            Opcode::Rla => self.rla(),

            Opcode::JrCondImm8(condition) => self.jr_cond_imm8(condition, bus),

            Opcode::CallImm16 => self.call_imm16(bus),
            Opcode::Ret => self.ret(bus),
            Opcode::PushReg16Stack(register) => self.push_r16_stack(register, bus),
            Opcode::PopReg16Stack(register) => self.pop_r16_stack(register, bus),

            Opcode::RlcReg8(register) => self.rlc_r8(register),
            Opcode::RlcHlAddr => self.rlc_hladdr(bus),
            Opcode::Rl(register) => self.rl_r8(register),
            Opcode::Bit(mask, register) => self.bit(mask, register),

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

            let (new_value, overflow_from_7) = self.registers.a.overflowing_add(data);
            let overflow_from_3 = new_value & 0b00001111 < self.registers.a & 0b00001111;

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, overflow_from_3);
            self.set_flag(Flags::C, overflow_from_7);
            self.registers.a = new_value;

            self.phase = Phase::Fetch;
        }
    }

    fn sub_a_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let (new_value, borrow) = self.registers.a.overflowing_sub(data);
        let borrow_from_4 = new_value & 0b00001111 > self.registers.a & 0b00001111;

        self.set_flag(Flags::Z, new_value == 0);
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::H, borrow_from_4);
        self.set_flag(Flags::C, borrow);
        self.registers.a = new_value;

        self.phase = Phase::Fetch;
    }

    fn inc_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let new_value = data.wrapping_add(1);
        let overflow_from_3 = new_value & 0b00001111 < data & 0b00001111;

        self.write_to_register(register, new_value);

        self.set_flag(Flags::Z, new_value == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, overflow_from_3);

        self.phase = Phase::Fetch;
    }

    fn dec_r8(&mut self, register: Register8Bit) {
        let data = self.read_from_register(register);

        let new_value = data.wrapping_sub(1);
        let borrow_from_4 = new_value & 0b00001111 > data & 0b00001111;

        self.write_to_register(register, new_value);

        self.set_flag(Flags::Z, new_value == 0);
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::H, borrow_from_4);

        self.phase = Phase::Fetch;
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

    fn rla(&mut self) {
        self.rl_r8(Register8Bit::A);
        self.set_flag(Flags::Z, false);
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
            12 => {
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
    #[case("00.json", "")]
    #[case("01.json", "")]
    #[case("02.json", "")]
    #[case("04.json", "")]
    #[case("05.json", "")]
    #[case("06.json", "")]
    #[case("0e.json", "")]
    #[case("0a.json", "")]
    #[case("0c.json", "")]
    #[case("0d.json", "")]
    #[case("11.json", "")]
    #[case("12.json", "")]
    #[case("14.json", "")]
    #[case("15.json", "")]
    #[case("16.json", "")]
    #[case("17.json", "")]
    #[case("1a.json", "")]
    #[case("1c.json", "")]
    #[case("1d.json", "")]
    #[case("1e.json", "")]
    #[case("20.json", "")]
    #[case("21.json", "")]
    #[case("22.json", "")]
    #[case("24.json", "")]
    #[case("25.json", "")]
    #[case("26.json", "")]
    #[case("28.json", "")]
    #[case("2a.json", "")]
    #[case("2c.json", "")]
    #[case("2d.json", "")]
    #[case("2e.json", "")]
    #[case("30.json", "")]
    #[case("31.json", "")]
    #[case("32.json", "")]
    #[case("3a.json", "")]
    #[case("3c.json", "")]
    #[case("3d.json", "")]
    #[case("3e.json", "")]
    #[case("38.json", "")]
    #[case("40.json", "")]
    #[case("41.json", "")]
    #[case("42.json", "")]
    #[case("43.json", "")]
    #[case("44.json", "")]
    #[case("45.json", "")]
    #[case("46.json", "")]
    #[case("47.json", "")]
    #[case("48.json", "")]
    #[case("49.json", "")]
    #[case("4a.json", "")]
    #[case("4b.json", "")]
    #[case("4c.json", "")]
    #[case("4d.json", "")]
    #[case("4e.json", "")]
    #[case("4f.json", "")]
    #[case("50.json", "")]
    #[case("51.json", "")]
    #[case("52.json", "")]
    #[case("53.json", "")]
    #[case("54.json", "")]
    #[case("55.json", "")]
    #[case("56.json", "")]
    #[case("57.json", "")]
    #[case("58.json", "")]
    #[case("59.json", "")]
    #[case("5a.json", "")]
    #[case("5b.json", "")]
    #[case("5c.json", "")]
    #[case("5d.json", "")]
    #[case("5e.json", "")]
    #[case("5f.json", "")]
    #[case("60.json", "")]
    #[case("61.json", "")]
    #[case("62.json", "")]
    #[case("63.json", "")]
    #[case("64.json", "")]
    #[case("65.json", "")]
    #[case("66.json", "")]
    #[case("67.json", "")]
    #[case("68.json", "")]
    #[case("69.json", "")]
    #[case("6a.json", "")]
    #[case("6b.json", "")]
    #[case("6c.json", "")]
    #[case("6d.json", "")]
    #[case("6e.json", "")]
    #[case("6f.json", "")]
    #[case("70.json", "")]
    #[case("71.json", "")]
    #[case("72.json", "")]
    #[case("73.json", "")]
    #[case("74.json", "")]
    #[case("75.json", "")]
    #[case("77.json", "")]
    #[case("78.json", "")]
    #[case("79.json", "")]
    #[case("7a.json", "")]
    #[case("7b.json", "")]
    #[case("7c.json", "")]
    #[case("7d.json", "")]
    #[case("7e.json", "")]
    #[case("7f.json", "")]
    #[case("80.json", "")]
    #[case("81.json", "")]
    #[case("82.json", "")]
    #[case("83.json", "")]
    #[case("84.json", "")]
    #[case("85.json", "")]
    #[case("87.json", "")]
    #[case("90.json", "")]
    #[case("91.json", "")]
    #[case("92.json", "")]
    #[case("93.json", "")]
    #[case("94.json", "")]
    #[case("95.json", "")]
    #[case("97.json", "")]
    #[case("a8.json", "")]
    #[case("a9.json", "")]
    #[case("aa.json", "")]
    #[case("ab.json", "")]
    #[case("ac.json", "")]
    #[case("ad.json", "")]
    #[case("af.json", "")]
    #[case("c1.json", "")]
    #[case("c5.json", "")]
    #[case("c9.json", "")]
    #[case("cd.json", "")]
    #[case("d1.json", "")]
    #[case("d5.json", "")]
    #[case("e0.json", "")]
    #[case("e1.json", "")]
    #[case("e2.json", "")]
    #[case("e5.json", "")]
    #[case("ea.json", "")]
    #[case("f0.json", "")]
    #[case("f1.json", "")]
    #[case("f2.json", "")]
    #[case("f5.json", "")]
    #[case("fa.json", "")]
    #[case("cb.json", "cb 00")]
    #[case("cb.json", "cb 01")]
    #[case("cb.json", "cb 02")]
    #[case("cb.json", "cb 03")]
    #[case("cb.json", "cb 04")]
    #[case("cb.json", "cb 05")]
    #[case("cb.json", "cb 06")]
    #[case("cb.json", "cb 07")]
    #[case("cb.json", "cb 10")]
    #[case("cb.json", "cb 11")]
    #[case("cb.json", "cb 12")]
    #[case("cb.json", "cb 13")]
    #[case("cb.json", "cb 14")]
    #[case("cb.json", "cb 15")]
    #[case("cb.json", "cb 17")]
    #[case("cb.json", "cb 40")]
    #[case("cb.json", "cb 41")]
    #[case("cb.json", "cb 42")]
    #[case("cb.json", "cb 43")]
    #[case("cb.json", "cb 44")]
    #[case("cb.json", "cb 45")]
    #[case("cb.json", "cb 47")]
    #[case("cb.json", "cb 48")]
    #[case("cb.json", "cb 49")]
    #[case("cb.json", "cb 4a")]
    #[case("cb.json", "cb 4b")]
    #[case("cb.json", "cb 4c")]
    #[case("cb.json", "cb 4d")]
    #[case("cb.json", "cb 4f")]
    #[case("cb.json", "cb 50")]
    #[case("cb.json", "cb 51")]
    #[case("cb.json", "cb 52")]
    #[case("cb.json", "cb 53")]
    #[case("cb.json", "cb 54")]
    #[case("cb.json", "cb 55")]
    #[case("cb.json", "cb 57")]
    #[case("cb.json", "cb 58")]
    #[case("cb.json", "cb 59")]
    #[case("cb.json", "cb 5a")]
    #[case("cb.json", "cb 5b")]
    #[case("cb.json", "cb 5c")]
    #[case("cb.json", "cb 5d")]
    #[case("cb.json", "cb 5f")]
    #[case("cb.json", "cb 60")]
    #[case("cb.json", "cb 61")]
    #[case("cb.json", "cb 62")]
    #[case("cb.json", "cb 63")]
    #[case("cb.json", "cb 64")]
    #[case("cb.json", "cb 65")]
    #[case("cb.json", "cb 67")]
    #[case("cb.json", "cb 68")]
    #[case("cb.json", "cb 69")]
    #[case("cb.json", "cb 6a")]
    #[case("cb.json", "cb 6b")]
    #[case("cb.json", "cb 6c")]
    #[case("cb.json", "cb 6d")]
    #[case("cb.json", "cb 6f")]
    #[case("cb.json", "cb 70")]
    #[case("cb.json", "cb 71")]
    #[case("cb.json", "cb 72")]
    #[case("cb.json", "cb 73")]
    #[case("cb.json", "cb 74")]
    #[case("cb.json", "cb 75")]
    #[case("cb.json", "cb 77")]
    #[case("cb.json", "cb 78")]
    #[case("cb.json", "cb 79")]
    #[case("cb.json", "cb 7a")]
    #[case("cb.json", "cb 7b")]
    #[case("cb.json", "cb 7c")]
    #[case("cb.json", "cb 7d")]
    #[case("cb.json", "cb 7f")]
    fn should_pass_gameboycputtests_json_tests(#[case] test_file: &str, #[case] filter: &str) {
        let test_data = {
            let test_filepath = Path::new("test-data")
                .join("json-tests")
                .join("GameBoyCPUTests")
                .join("v2")
                .join(test_file);

            let buf = fs::read_to_string(test_filepath).unwrap();
            let test_data: Vec<JsonTest> = serde_json::from_str(&buf).unwrap();
            test_data
        };

        let test_data = test_data
            .iter()
            .filter(|test| test.name.starts_with(filter));

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
