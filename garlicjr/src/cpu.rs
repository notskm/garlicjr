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

use crate::opcode::{Cond, Opcode, Register8Bit, Register16Bit};
use crate::{Bus, ReadWriteMode};

pub struct SharpSM83 {
    pub registers: Registers,
    #[allow(dead_code)]
    interrupt_master_enable: bool,
    current_tick: u8,
    opcode: Opcode,
    phase: Phase,
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
        self.opcode = Opcode::decode(bus.data);
    }

    fn increment_program_counter(&mut self) {
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
    }

    fn execute_opcode(&mut self, bus: &mut Bus) {
        match self.opcode {
            Opcode::Nop => self.no_op(),
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
            Opcode::AddAReg8(register) => self.add_a_r8(register),
            Opcode::SubAReg8(register) => self.sub_a_r8(register),
            Opcode::XorAReg8(register) => self.xor_a_r8(register),
            Opcode::JrCondImm8(condition) => self.jr_cond_imm8(condition, bus),
            Opcode::Unimplemented(_) => {}
            _ => {}
        }
    }

    fn no_op(&mut self) {
        if self.current_tick == 2 {
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
    use std::path::Path;
    use std::{fs::File, io::BufReader};

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
    #[case("06.json")]
    #[case("0e.json")]
    #[case("0a.json")]
    #[case("11.json")]
    #[case("12.json")]
    #[case("16.json")]
    #[case("1a.json")]
    #[case("1e.json")]
    #[case("20.json")]
    #[case("21.json")]
    #[case("22.json")]
    #[case("26.json")]
    #[case("28.json")]
    #[case("2a.json")]
    #[case("2e.json")]
    #[case("30.json")]
    #[case("31.json")]
    #[case("32.json")]
    #[case("3a.json")]
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
    #[case("87.json")]
    #[case("90.json")]
    #[case("91.json")]
    #[case("92.json")]
    #[case("93.json")]
    #[case("94.json")]
    #[case("95.json")]
    #[case("97.json")]
    #[case("a8.json")]
    #[case("a9.json")]
    #[case("aa.json")]
    #[case("ab.json")]
    #[case("ac.json")]
    #[case("ad.json")]
    #[case("af.json")]
    fn should_pass_gameboycputtests_json_tests(#[case] test_file: &str) {
        let test_filepath = Path::new("test-data/json-tests/GameBoyCPUTests/v2/").join(test_file);

        let file = File::open(test_filepath).unwrap();
        let reader = BufReader::new(file);
        let test_data: Vec<JsonTest> = serde_json::from_reader(reader).unwrap();

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

            for ram_data in test.initial_state.ram {
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

                    assert_eq!(
                        bus.mode == ReadWriteMode::Read,
                        read,
                        "Expected bus in read mode on cycle {}",
                        i
                    );
                    assert_eq!(
                        bus.mode == ReadWriteMode::Write,
                        write,
                        "Expected bus in write mode on cycle {}",
                        i
                    );

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

            for ram_data in test.final_state.ram {
                let ram_index = ram_data.address as usize;
                let data = ram[ram_index];
                assert_eq!(data, ram_data.value);
            }
        }
    }
}
