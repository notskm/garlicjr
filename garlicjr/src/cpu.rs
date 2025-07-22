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

/// An emulator of the SharpSM83 CPU
///
/// For detailed information about this CPU, see the Pan Docs:
/// <https://gbdev.io/pandocs/>
pub struct SharpSM83 {
    /// The register file as described in the Pan Docs:
    /// <https://gbdev.io/pandocs/CPU_Registers_and_Flags.html>
    pub registers: CpuRegisters,

    interrupt_master_enable: InterruptEnableFlag,
    current_tick: u8,
    opcode: Opcode,
    phase: Phase,
    decode_as_prefix_opcode: bool,
    temp_16_bit: u16,
    mode: CpuMode,
}

/// The SharpSM83's registers
///
/// For detailed information about the SharpSM83's registers and how they work,
/// see the Pan Docs: <https://gbdev.io/pandocs/CPU_Registers_and_Flags.html>
///
/// Note that the register file contains an interrupt enable, and interrupt
/// flags, which together control which interrupts the CPU handles. While not
/// explicitly stated to be part of the CPU's register file, this seemed like
/// the most natural place for them.
#[derive(Clone, Debug, PartialEq)]
pub struct CpuRegisters {
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
    pub interrupt_flags: u8,
}

#[derive(PartialEq, Eq, Debug)]
enum InterruptEnableFlag {
    Enabled,
    Disabled,
    ShouldEnable,
}

enum Phase {
    Execute,
    HandleInterrupt,
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

#[derive(PartialEq, Eq)]
enum CpuMode {
    Running,
    Halted,
}

impl SharpSM83 {
    /// Creates a new SharpSM83 with all registers set to 0.
    ///
    /// # Examples
    /// ```
    /// use garlicjr::SharpSM83;
    ///
    /// let cpu = SharpSM83::new();
    /// ```
    pub fn new() -> SharpSM83 {
        SharpSM83 {
            registers: CpuRegisters {
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
                interrupt_flags: 0,
            },
            mode: CpuMode::Running,
            interrupt_master_enable: InterruptEnableFlag::Disabled,
            current_tick: 0,
            opcode: Opcode::Nop,
            phase: Phase::Decode,
            decode_as_prefix_opcode: false,
            temp_16_bit: 0,
        }
    }

    /// Runs the CPU for one T-cycle, or 1/4 of an M-cycle.
    ///
    /// Running this function 4 times constitutes 1 M-cycle.
    ///
    /// The SharpSM83 runs at a rate of 4 mebihertz. To run the CPU in realtime,
    /// call this function 4194304 times per second.
    ///
    /// After every 4 calls to this function, the CPU may request a read or a
    /// write via the `bus`. The read or write should be handled before the next
    /// call to this function. See the examples for details.
    ///
    /// # Panics
    /// This function will panic when trying to execute an instruction that has
    /// not yet been implemented.
    ///
    /// In some future version, this function should not panic.
    ///
    /// # Examples
    /// ```
    /// use garlicjr::{SharpSM83, Bus, ReadWriteMode};
    ///
    /// let mut cpu = SharpSM83::new();
    /// let mut bus = Bus::new();
    /// let mut memory = vec![0u8; u16::MAX as usize];
    ///
    /// // Run 1 M-cycle
    /// for _ in 0..4 {
    ///     cpu.tick(&mut bus);
    /// }
    ///
    /// // After 1 M-cycle, handle read/write requests.
    /// match bus.mode {
    ///     ReadWriteMode::Read => bus.data = memory[bus.address as usize],
    ///     ReadWriteMode::Write => memory[bus.address as usize] = bus.data,
    /// }
    /// ```
    pub fn tick(&mut self, bus: &mut Bus) {
        if self.should_wake_from_halt() {
            self.mode = CpuMode::Running;
        } else if self.mode == CpuMode::Halted {
            self.current_tick += 1;
            if self.current_tick >= 4 {
                self.current_tick = 0;
            }
            return;
        }

        match self.phase {
            Phase::Decode => {
                if self.check_interrupts() {
                    self.phase = Phase::HandleInterrupt;
                } else {
                    self.decode(bus);
                }
                self.current_tick += 1;
            }
            Phase::HandleInterrupt => {
                self.handle_interrupt(bus);
                self.current_tick = self.current_tick.saturating_add(1);
            }
            Phase::Execute => {
                self.execute_opcode(bus);
                self.current_tick = self.current_tick.saturating_add(1);
            }
            Phase::Fetch => {
                self.execute_opcode(bus);

                self.write_program_counter(bus);
                self.phase = Phase::Decode;
                self.current_tick = 0;
                self.increment_program_counter();

                if self.interrupt_master_enable == InterruptEnableFlag::ShouldEnable
                    && self.opcode != Opcode::Ei
                {
                    self.interrupt_master_enable = InterruptEnableFlag::Enabled;
                }
            }
        }
    }

    fn decode(&mut self, bus: &mut Bus) {
        match self.current_tick {
            0 => {
                self.read_opcode(bus);
            }
            1 => {
                self.phase = Phase::Execute;
            }
            _ => (),
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

    fn should_wake_from_halt(&self) -> bool {
        self.current_tick == 0 && self.mode == CpuMode::Halted && self.are_interrupts_pending()
    }

    fn are_interrupts_pending(&self) -> bool {
        let i_enable = self.registers.interrupt_enable & 0b00011111;
        let i_flag = self.registers.interrupt_flags & 0b00011111;
        i_enable & i_flag != 0
    }

    fn check_interrupts(&mut self) -> bool {
        self.interrupt_master_enable == InterruptEnableFlag::Enabled
            && self.are_interrupts_pending()
    }

    fn handle_interrupt(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.registers.program_counter = self.registers.program_counter.wrapping_sub(1);
            }
            4 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
            }
            8 => {
                bus.address = self.registers.stack_pointer;
                bus.data = self.registers.program_counter.to_be_bytes()[0];
                bus.mode = ReadWriteMode::Write;

                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
            }
            12 => {
                bus.address = self.registers.stack_pointer;
                bus.data = self.registers.program_counter.to_be_bytes()[1];
                bus.mode = ReadWriteMode::Write;

                let mut mask = 1;
                let mut shift = 0;
                while self.registers.interrupt_enable & mask == 0 {
                    mask <<= 1;
                    shift += 1;
                }

                self.registers.interrupt_flags &= !mask;
                self.interrupt_master_enable = InterruptEnableFlag::Disabled;

                self.registers.program_counter = 0x0040 + shift * 8;
            }
            18 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn increment_program_counter(&mut self) {
        self.registers.program_counter = self.registers.program_counter.wrapping_add(1);
    }

    fn execute_opcode(&mut self, bus: &mut Bus) {
        match self.opcode {
            Opcode::Nop => self.no_op(),
            Opcode::Prefix => self.prefix(),

            Opcode::Halt => self.halt(),

            Opcode::Ei => self.ei(),
            Opcode::Di => self.di(),

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
            Opcode::LdHlAddrImm8 => self.ld_hl_addr_imm8(bus),
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
            Opcode::LdImm16AddrSp => self.ld_imm16addr_sp(bus),
            Opcode::LdImm16AddrA => self.ld_imm16addr_a(bus),
            Opcode::LdAImm16Addr => self.ld_a_imm16addr(bus),
            Opcode::LdhImm8AddrA => self.ldh_imm8addr_a(bus),
            Opcode::LdhAImm8Addr => self.ldh_a_imm8addr(bus),
            Opcode::LdSpHl => self.ld_sp_hl(),
            Opcode::LdHlSpPlusImm8 => self.ld_hl_sp_plus_imm8(bus),

            Opcode::AddAReg8(register) => self.add_a_r8(register),
            Opcode::SubAReg8(register) => self.sub_a_r8(register),
            Opcode::AddAHlAddr => self.add_a_hladdr(bus),
            Opcode::SubAHlAddr => self.sub_a_hladdr(bus),
            Opcode::AdcAReg8(register) => self.adc_a_r8(register),
            Opcode::AdcAImm8 => self.adc_a_imm8(bus),
            Opcode::AdcAHlAddr => self.adc_a_hl_addr(bus),
            Opcode::SbcAReg8(register) => self.sbc_a_r8(register),
            Opcode::SbcAImm8 => self.sbc_a_imm8(bus),
            Opcode::SbcAHlAddr => self.sbc_a_hl_addr(bus),
            Opcode::AddAImm8 => self.add_a_imm8(bus),
            Opcode::SubImm8 => self.sub_a_imm8(bus),
            Opcode::AddHlR16(register) => self.add_hl_r16(register),
            Opcode::AddSpImm8 => self.add_sp_imm8(bus),

            Opcode::IncReg8(register) => self.inc_r8(register),
            Opcode::DecReg8(register) => self.dec_r8(register),
            Opcode::IncReg16(register) => self.inc_r16(register),
            Opcode::DecReg16(register) => self.dec_r16(register),
            Opcode::IncHlAddr => self.inc_hl_addr(bus),
            Opcode::DecHlAddr => self.dec_hl_addr(bus),

            Opcode::AndAReg8(register) => self.and_a_r8(register),
            Opcode::OrAReg8(register) => self.or_a_r8(register),
            Opcode::OrHLAddr => self.or_a_hl_addr(bus),
            Opcode::OrImm8 => self.or_a_imm8(bus),
            Opcode::XorAReg8(register) => self.xor_a_r8(register),
            Opcode::XorImm8 => self.xor_a_imm8(bus),
            Opcode::Rrca => self.rrca(),
            Opcode::Rlca => self.rlca(),
            Opcode::Rla => self.rla(),
            Opcode::Rra => self.rra(),
            Opcode::AndImm8 => self.and_a_imm8(bus),
            Opcode::XorAHlAddr => self.xor_a_hl_addr(bus),
            Opcode::Cpl => self.cpl(),
            Opcode::Ccf => self.ccf(),

            Opcode::CpReg8(register) => self.cp_a_r8(register),
            Opcode::CpImm8 => self.cp_a_imm8(bus),
            Opcode::CpHlAddr => self.cp_a_hladdr(bus),

            Opcode::JpImm16 => self.jp_imm16(bus),
            Opcode::JpHl => self.jp_hl(),
            Opcode::JpCondImm16(condition) => self.jp_cond_imm16(condition, bus),
            Opcode::JrCondImm8(condition) => self.jr_cond_imm8(condition, bus),
            Opcode::JrImm8 => self.jr_imm8(bus),

            Opcode::CallImm16 => self.call_imm16(bus),
            Opcode::CallCondImm16(condition) => self.call_cond_imm16(condition, bus),
            Opcode::Ret => self.ret(bus),
            Opcode::RetCond(condition) => self.ret_cond(condition, bus),
            Opcode::PushReg16Stack(register) => self.push_r16_stack(register, bus),
            Opcode::PopReg16Stack(register) => self.pop_r16_stack(register, bus),

            Opcode::Rst(address) => self.rst(address, bus),

            Opcode::Scf => self.scf(),

            Opcode::RlcReg8(register) => self.rlc_r8(register),
            Opcode::RlcHlAddr => self.rlc_hladdr(bus),
            Opcode::RrcReg8(register) => self.rrc_r8(register),
            Opcode::RrcHlAddr => self.rrc_hl_addr(bus),
            Opcode::Rl(register) => self.rl_r8(register),
            Opcode::Rr(register) => self.rr_r8(register),
            Opcode::RrHlAddr => self.rr_hl_addr(bus),
            Opcode::Bit(mask, register) => self.bit(mask, register),
            Opcode::Srl(register) => self.srl(register),
            Opcode::Swap(register) => self.swap(register),

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

    fn halt(&mut self) {
        self.no_op();
        if self.current_tick == 3 {
            self.mode = CpuMode::Halted;
        }
    }

    fn ei(&mut self) {
        self.interrupt_master_enable = InterruptEnableFlag::ShouldEnable;
        self.no_op();
    }

    fn di(&mut self) {
        self.interrupt_master_enable = InterruptEnableFlag::Disabled;
        self.no_op();
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
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                let low = bus.data;
                self.write_to_16_bit_register_low(register, low);

                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
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

    fn ld_hl_addr_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                // Whatever data we read in the last step needs to be written
                // now. Since it's already on the bus, we leave bus.data alone.
                bus.mode = ReadWriteMode::Write;
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
            }
            10 => {
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
                bus.mode = ReadWriteMode::Read;
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

    fn ld_imm16addr_sp(&mut self, bus: &mut Bus) {
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

                let [_, low] = self
                    .read_from_16_bit_register(Register16Bit::SP)
                    .to_be_bytes();

                bus.address = self.temp_16_bit;
                bus.data = low;
                bus.mode = ReadWriteMode::Write;
            }
            12 => {
                let [high, _] = self
                    .read_from_16_bit_register(Register16Bit::SP)
                    .to_be_bytes();

                bus.address = self.temp_16_bit.wrapping_add(1);
                bus.data = high;
                bus.mode = ReadWriteMode::Write;
            }
            18 => {
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

    fn ld_sp_hl(&mut self) {
        match self.current_tick {
            2 => {
                let value = self.read_from_register(Register8Bit::L);
                self.write_to_16_bit_register_low(Register16Bit::SP, value);
            }
            4 => {
                let value = self.read_from_register(Register8Bit::H);
                self.write_to_16_bit_register_high(Register16Bit::SP, value);
            }
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn ld_hl_sp_plus_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let [_, sp_low] = self.registers.stack_pointer.to_be_bytes();
                let (_, carry, half_carry) = sp_low.overflowing_add_with_half_carry(bus.data);
                self.set_flag(Flags::Z, false);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, half_carry);
                self.set_flag(Flags::C, carry);
            }
            8 => {
                let new_value = self
                    .registers
                    .stack_pointer
                    .wrapping_add_signed(bus.data as i8 as i16);
                self.write_to_16_bit_register(Register16Bit::HL, new_value);
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

    fn adc_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let carry_flag = self.get_flag(Flags::C);

                let (new_value, carry, half_carry) =
                    self.registers.a.full_overflowing_add(bus.data, carry_flag);

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

    fn adc_a_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let carry_flag = self.get_flag(Flags::C);

                let (new_value, carry, half_carry) =
                    self.registers.a.full_overflowing_add(bus.data, carry_flag);

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

    fn sbc_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let carry_flag = self.get_flag(Flags::C);

                let (new_value, carry, half_carry) =
                    self.registers.a.full_overflowing_sub(bus.data, carry_flag);

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

    fn sbc_a_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let carry_flag = self.get_flag(Flags::C);

                let (new_value, carry, half_carry) =
                    self.registers.a.full_overflowing_sub(bus.data, carry_flag);

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
        if self.current_tick == 2 {
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

    fn add_hl_r16(&mut self, register: Register16Bit) {
        match self.current_tick {
            2 => {
                let value = self.read_from_16_bit_register(register);
                let hl = self.read_from_16_bit_register(Register16Bit::HL);

                let [hl_high, hl_low] = hl.to_be_bytes();
                let [val_high, val_low] = value.to_be_bytes();

                let (new_low, low_carry) = hl_low.overflowing_add(val_low);
                let (new_high, carry, half_carry) =
                    hl_high.full_overflowing_add(val_high, low_carry);

                self.write_to_16_bit_register_low(Register16Bit::HL, new_low);
                self.write_to_16_bit_register_high(Register16Bit::HL, new_high);

                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, half_carry);
                self.set_flag(Flags::C, carry);
            }
            4 => {}
            6 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn add_sp_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                let data = bus.data;

                let [_, sp_low] = self.registers.stack_pointer.to_be_bytes();

                let (_, carry, half_carry) = sp_low.overflowing_add_with_half_carry(data);

                self.registers.stack_pointer = self
                    .registers
                    .stack_pointer
                    .wrapping_add_signed(data as i8 as i16);

                self.set_flag(Flags::Z, false);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, half_carry);
                self.set_flag(Flags::C, carry);
            }
            8 => {}
            12 => {}
            14 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn inc_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);

            let (new_value, _, half_carry) = data.overflowing_add_with_half_carry(1);

            self.write_to_register(register, new_value);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, half_carry);

            self.phase = Phase::Fetch;
        }
    }

    fn dec_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);

            let (new_value, _, half_borrow) = data.overflowing_sub_with_half_carry(1);

            self.write_to_register(register, new_value);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, true);
            self.set_flag(Flags::H, half_borrow);

            self.phase = Phase::Fetch;
        }
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

    fn inc_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let (new_value, _, half_carry) = bus.data.overflowing_add_with_half_carry(1);

                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.data = new_value;
                bus.mode = ReadWriteMode::Write;

                self.set_flag(Flags::Z, new_value == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, half_carry);
            }
            8 => {}
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn dec_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let (new_value, _, half_carry) = bus.data.overflowing_sub_with_half_carry(1);

                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.data = new_value;
                bus.mode = ReadWriteMode::Write;

                self.set_flag(Flags::Z, new_value == 0);
                self.set_flag(Flags::N, true);
                self.set_flag(Flags::H, half_carry);
            }
            8 => {}
            10 => {
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

    fn xor_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
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

    fn or_a_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                self.registers.a |= bus.data;
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

    fn or_a_imm8(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.write_program_counter(bus);
                self.increment_program_counter();
            }
            4 => {
                self.registers.a |= bus.data;
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

    fn rrca(&mut self) {
        if self.current_tick == 2 {
            let carry = self.registers.a & 1;
            self.registers.a = self.registers.a.rotate_right(1);
            self.set_flag(Flags::Z, false);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, carry > 0);

            self.phase = Phase::Fetch;
        }
    }

    fn rlca(&mut self) {
        if self.current_tick == 2 {
            let carry = self.registers.a & 0b10000000;
            self.registers.a = self.registers.a.rotate_left(1);
            self.set_flag(Flags::Z, false);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, carry > 0);

            self.phase = Phase::Fetch;
        }
    }

    fn rla(&mut self) {
        self.rl_r8(Register8Bit::A);
        self.set_flag(Flags::Z, false);
    }

    fn rra(&mut self) {
        self.rr_r8(Register8Bit::A);
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

    fn cpl(&mut self) {
        if self.current_tick == 2 {
            self.registers.a = !self.registers.a;

            self.set_flag(Flags::N, true);
            self.set_flag(Flags::H, true);

            self.phase = Phase::Fetch;
        }
    }

    fn ccf(&mut self) {
        if self.current_tick == 2 {
            let carry_complement = !self.get_flag(Flags::C);

            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, carry_complement);

            self.phase = Phase::Fetch;
        }
    }

    fn cp_a_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);

            let (result, borrow, half_borrow) =
                self.registers.a.overflowing_sub_with_half_carry(data);

            self.set_flag(Flags::Z, result == 0);
            self.set_flag(Flags::N, true);
            self.set_flag(Flags::H, half_borrow);
            self.set_flag(Flags::C, borrow);
            self.phase = Phase::Fetch;
        }
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
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
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

    fn jp_hl(&mut self) {
        if self.current_tick == 2 {
            self.registers.program_counter = self.read_from_16_bit_register(Register16Bit::HL);
            self.phase = Phase::Fetch;
        }
    }

    fn jp_cond_imm16(&mut self, condition: Cond, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.increment_program_counter();
            }
            4 => {
                bus.address = self.registers.program_counter;
                bus.mode = ReadWriteMode::Read;
                self.temp_16_bit &= 0b1111111100000000;
                self.temp_16_bit |= bus.data as u16;
                self.increment_program_counter();
            }
            8 => {
                self.temp_16_bit &= 0b0000000011111111;
                self.temp_16_bit |= (bus.data as u16) << 8;
            }
            10 => {
                let should_jump = match condition {
                    Cond::Nz => !self.get_flag(Flags::Z),
                    Cond::Z => self.get_flag(Flags::Z),
                    Cond::Nc => !self.get_flag(Flags::C),
                    Cond::C => self.get_flag(Flags::C),
                };

                if should_jump {
                    self.registers.program_counter = self.temp_16_bit;
                } else {
                    self.phase = Phase::Fetch;
                }
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
                bus.mode = ReadWriteMode::Read;
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
                bus.mode = ReadWriteMode::Read;
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

    fn ret_cond(&mut self, condition: Cond, bus: &mut Bus) {
        match self.current_tick {
            4 => {
                bus.address = self.registers.stack_pointer;
                bus.mode = ReadWriteMode::Read;
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
            }
            6 => {
                let should_return = match condition {
                    Cond::Nz => !self.get_flag(Flags::Z),
                    Cond::Z => self.get_flag(Flags::Z),
                    Cond::Nc => !self.get_flag(Flags::C),
                    Cond::C => self.get_flag(Flags::C),
                };

                if !should_return {
                    self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
                    self.phase = Phase::Fetch;
                }
            }
            8 => {
                self.registers.program_counter &= 0xFF00;
                self.registers.program_counter |= bus.data as u16;

                bus.address = self.registers.stack_pointer;
                bus.mode = ReadWriteMode::Read;
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
            }
            12 => {
                self.registers.program_counter &= 0x00FF;
                self.registers.program_counter |= (bus.data as u16) << 8;
            }
            18 => {
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

    fn rst(&mut self, address: u16, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
            }
            4 => {
                bus.address = self.registers.stack_pointer;
                bus.data = ((self.registers.program_counter & 0xFF00u16) >> 8u16) as u8;
                bus.mode = ReadWriteMode::Write;

                self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
            }
            8 => {
                bus.address = self.registers.stack_pointer;
                bus.data = (self.registers.program_counter & 0x00FFu16) as u8;
                bus.mode = ReadWriteMode::Write;
            }
            12 => {
                self.registers.program_counter = address;
            }
            14 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn scf(&mut self) {
        if self.current_tick == 2 {
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, true);

            self.phase = Phase::Fetch;
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

    fn rrc_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let data = self.read_from_register(register);
            let overflow = data & 0b00000001 > 0;
            let data = data.rotate_right(1);
            self.write_to_register(register, data);

            self.set_flag(Flags::Z, data == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, overflow);

            self.phase = Phase::Fetch;
        }
    }

    fn rrc_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let overflow = bus.data & 0b00000001 > 0;
                bus.data = bus.data.rotate_right(1);

                self.set_flag(Flags::Z, bus.data == 0);
                self.set_flag(Flags::N, false);
                self.set_flag(Flags::H, false);
                self.set_flag(Flags::C, overflow);

                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Write;
            }
            10 => {
                self.phase = Phase::Fetch;
            }
            _ => (),
        }
    }

    fn rr_r8(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let mut data = self.read_from_register(register);
            let overflow = data & 1 > 0;
            let carry_flag = self.get_flag(Flags::C) as u8;

            data >>= 1;
            data |= carry_flag << 7;

            self.write_to_register(register, data);

            self.set_flag(Flags::Z, data == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, overflow);

            self.phase = Phase::Fetch;
        }
    }

    fn rr_hl_addr(&mut self, bus: &mut Bus) {
        match self.current_tick {
            2 => {
                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
                bus.mode = ReadWriteMode::Read;
            }
            4 => {
                let overflow = bus.data & 1 > 0;
                let carry_flag = self.get_flag(Flags::C) as u8;
                bus.data >>= 1;
                bus.data |= carry_flag << 7;

                bus.address = self.read_from_16_bit_register(Register16Bit::HL);
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

    fn swap(&mut self, register: Register8Bit) {
        if self.current_tick == 2 {
            let value = self.read_from_register(register);

            let mut new_value = value << 4;
            new_value |= value >> 4;

            self.write_to_register(register, new_value);

            self.set_flag(Flags::Z, new_value == 0);
            self.set_flag(Flags::N, false);
            self.set_flag(Flags::H, false);
            self.set_flag(Flags::C, false);

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

    const EI: u8 = 0xFB;
    const DI: u8 = 0xF3;
    const NOP: u8 = 0x00;

    #[rstest]
    #[case(0b00000001, 0b00000001, 0x0041)]
    #[case(0b00000010, 0b00000010, 0x0049)]
    #[case(0b00000100, 0b00000100, 0x0051)]
    #[case(0b00001000, 0b00001000, 0x0059)]
    #[case(0b00010000, 0b00010000, 0x0061)]
    fn should_handle_interrupts_one_instruction_after_interrupts_are_enabled(
        #[case] flag: u8,
        #[case] enabled: u8,
        #[case] expected_pc: u16,
    ) {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.interrupt_enable = enabled;
        cpu.registers.interrupt_flags = flag;
        cpu.registers.program_counter = 0x0100;

        // Execute EI, pick up nop
        bus.data = EI;
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP
        }

        assert_eq!(cpu.registers.program_counter, 0x0101);

        // Execute nop
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP
        }

        // Handle interrupt
        for _ in 0..4 * 5 {
            cpu.tick(&mut bus);
        }

        assert_eq!(cpu.registers.program_counter, expected_pc);
        assert_eq!(cpu.registers.interrupt_enable, enabled);
        assert_eq!(cpu.registers.interrupt_flags, 0b00000000);
    }

    #[test]
    fn should_disable_interrupts_after_handling_an_interrupt() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x0101;

        // Execute EI, pick up nop
        bus.data = EI;
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0102);

        cpu.registers.interrupt_enable = 0b00000100;
        cpu.registers.interrupt_flags = 0b00000100;

        // Handle interrupt, execute some nops
        for _ in 0..20 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        cpu.registers.program_counter = 0x101;

        // Execute another nop
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0102);
        assert_eq!(cpu.registers.interrupt_flags, 0b00000000);
        assert_eq!(cpu.registers.interrupt_enable, 0b00000100);
    }

    #[rstest]
    fn should_ignore_top_3_bits_of_interrupt_registers(
        #[values(
            0b00100000, 0b01000000, 0b01100000, 0b10000000, 0b10100000, 0b11000000, 0b11100000
        )]
        requested_interrupt: u8,
    ) {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x0101;

        // Execute EI, pick up NOP
        bus.data = EI;
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0102);

        // Execute NOP, pick up NOP
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = DI;
        }

        assert_eq!(cpu.registers.program_counter, 0x0103);

        cpu.registers.interrupt_enable = requested_interrupt;
        cpu.registers.interrupt_flags = requested_interrupt;

        // Execute NOP, do not handle interrupts
        for _ in 0..4 {
            cpu.tick(&mut bus);
        }

        assert_eq!(cpu.registers.program_counter, 0x0104);
        assert_eq!(cpu.registers.interrupt_flags, requested_interrupt);
        assert_eq!(cpu.registers.interrupt_enable, requested_interrupt);
    }

    #[test]
    fn should_not_handle_interrupts_after_interrupts_are_disabled() {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x0101;

        // Execute EI, pick up NOP
        bus.data = EI;
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0102);

        // Execute NOP, pick up DI
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = DI;
        }

        assert_eq!(cpu.registers.program_counter, 0x0103);

        // Execute DI, pick up NOP
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0104);

        cpu.registers.interrupt_enable = 0b00000100;
        cpu.registers.interrupt_flags = 0b00000100;

        // Execute NOP, do not handle interrupts
        for _ in 0..4 {
            cpu.tick(&mut bus);
        }

        assert_eq!(cpu.registers.program_counter, 0x0105);
        assert_eq!(cpu.registers.interrupt_flags, 0b00000100);
        assert_eq!(cpu.registers.interrupt_enable, 0b00000100);
    }

    #[rstest]
    #[case(0b00000101, 0b00000101, 0b00000100)]
    #[case(0b00010100, 0b00010100, 0b00010000)]
    #[case(0b00011111, 0b00010000, 0b00001111)]
    #[case(0b00010100, 0b00010000, 0b00000100)]
    #[case(0b00000000, 0b00011111, 0b00000000)]
    fn should_prioritize_enabled_low_bit_interrupts(
        #[case] requested: u8,
        #[case] enabled: u8,
        #[case] expected: u8,
    ) {
        let mut cpu = SharpSM83::new();
        let mut bus = Bus::new();

        cpu.registers.program_counter = 0x0101;

        // Execute EI, pick up nop
        bus.data = EI;
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0102);

        cpu.registers.interrupt_enable = enabled;
        cpu.registers.interrupt_flags = requested;

        // Handle interrupt, execute some nops
        for _ in 0..20 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        cpu.registers.program_counter = 0x101;

        // Execute another nop
        for _ in 0..4 {
            cpu.tick(&mut bus);
            bus.data = NOP;
        }

        assert_eq!(cpu.registers.program_counter, 0x0102);
        assert_eq!(cpu.registers.interrupt_flags, expected);
        assert_eq!(cpu.registers.interrupt_enable, enabled);
    }

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
    #[case::opcode_00("00.json")]
    #[case::opcode_01("01.json")]
    #[case::opcode_02("02.json")]
    #[case::opcode_03("03.json")]
    #[case::opcode_04("04.json")]
    #[case::opcode_05("05.json")]
    #[case::opcode_06("06.json")]
    #[case::opcode_07("07.json")]
    #[case::opcode_08("08.json")]
    #[case::opcode_09("09.json")]
    #[case::opcode_0e("0e.json")]
    #[case::opcode_0a("0a.json")]
    #[case::opcode_0b("0b.json")]
    #[case::opcode_0c("0c.json")]
    #[case::opcode_0d("0d.json")]
    #[case::opcode_0f("0f.json")]
    #[case::opcode_11("11.json")]
    #[case::opcode_12("12.json")]
    #[case::opcode_13("13.json")]
    #[case::opcode_14("14.json")]
    #[case::opcode_15("15.json")]
    #[case::opcode_16("16.json")]
    #[case::opcode_17("17.json")]
    #[case::opcode_18("18.json")]
    #[case::opcode_19("19.json")]
    #[case::opcode_1a("1a.json")]
    #[case::opcode_1b("1b.json")]
    #[case::opcode_1c("1c.json")]
    #[case::opcode_1d("1d.json")]
    #[case::opcode_1e("1e.json")]
    #[case::opcode_1f("1f.json")]
    #[case::opcode_20("20.json")]
    #[case::opcode_21("21.json")]
    #[case::opcode_22("22.json")]
    #[case::opcode_23("23.json")]
    #[case::opcode_24("24.json")]
    #[case::opcode_25("25.json")]
    #[case::opcode_26("26.json")]
    #[case::opcode_28("28.json")]
    #[case::opcode_29("29.json")]
    #[case::opcode_2a("2a.json")]
    #[case::opcode_2b("2b.json")]
    #[case::opcode_2c("2c.json")]
    #[case::opcode_2d("2d.json")]
    #[case::opcode_2e("2e.json")]
    #[case::opcode_2f("2f.json")]
    #[case::opcode_30("30.json")]
    #[case::opcode_31("31.json")]
    #[case::opcode_32("32.json")]
    #[case::opcode_33("33.json")]
    #[case::opcode_34("34.json")]
    #[case::opcode_35("35.json")]
    #[case::opcode_36("36.json")]
    #[case::opcode_37("37.json")]
    #[case::opcode_39("39.json")]
    #[case::opcode_3a("3a.json")]
    #[case::opcode_3b("3b.json")]
    #[case::opcode_3c("3c.json")]
    #[case::opcode_3d("3d.json")]
    #[case::opcode_3e("3e.json")]
    #[case::opcode_38("38.json")]
    #[case::opcode_3f("3f.json")]
    #[case::opcode_40("40.json")]
    #[case::opcode_41("41.json")]
    #[case::opcode_42("42.json")]
    #[case::opcode_43("43.json")]
    #[case::opcode_44("44.json")]
    #[case::opcode_45("45.json")]
    #[case::opcode_46("46.json")]
    #[case::opcode_47("47.json")]
    #[case::opcode_48("48.json")]
    #[case::opcode_49("49.json")]
    #[case::opcode_4a("4a.json")]
    #[case::opcode_4b("4b.json")]
    #[case::opcode_4c("4c.json")]
    #[case::opcode_4d("4d.json")]
    #[case::opcode_4e("4e.json")]
    #[case::opcode_4f("4f.json")]
    #[case::opcode_50("50.json")]
    #[case::opcode_51("51.json")]
    #[case::opcode_52("52.json")]
    #[case::opcode_53("53.json")]
    #[case::opcode_54("54.json")]
    #[case::opcode_55("55.json")]
    #[case::opcode_56("56.json")]
    #[case::opcode_57("57.json")]
    #[case::opcode_58("58.json")]
    #[case::opcode_59("59.json")]
    #[case::opcode_5a("5a.json")]
    #[case::opcode_5b("5b.json")]
    #[case::opcode_5c("5c.json")]
    #[case::opcode_5d("5d.json")]
    #[case::opcode_5e("5e.json")]
    #[case::opcode_5f("5f.json")]
    #[case::opcode_60("60.json")]
    #[case::opcode_61("61.json")]
    #[case::opcode_62("62.json")]
    #[case::opcode_63("63.json")]
    #[case::opcode_64("64.json")]
    #[case::opcode_65("65.json")]
    #[case::opcode_66("66.json")]
    #[case::opcode_67("67.json")]
    #[case::opcode_68("68.json")]
    #[case::opcode_69("69.json")]
    #[case::opcode_6a("6a.json")]
    #[case::opcode_6b("6b.json")]
    #[case::opcode_6c("6c.json")]
    #[case::opcode_6d("6d.json")]
    #[case::opcode_6e("6e.json")]
    #[case::opcode_6f("6f.json")]
    #[case::opcode_70("70.json")]
    #[case::opcode_71("71.json")]
    #[case::opcode_72("72.json")]
    #[case::opcode_73("73.json")]
    #[case::opcode_74("74.json")]
    #[case::opcode_75("75.json")]
    #[case::opcode_77("77.json")]
    #[case::opcode_78("78.json")]
    #[case::opcode_79("79.json")]
    #[case::opcode_7a("7a.json")]
    #[case::opcode_7b("7b.json")]
    #[case::opcode_7c("7c.json")]
    #[case::opcode_7d("7d.json")]
    #[case::opcode_7e("7e.json")]
    #[case::opcode_7f("7f.json")]
    #[case::opcode_80("80.json")]
    #[case::opcode_81("81.json")]
    #[case::opcode_82("82.json")]
    #[case::opcode_83("83.json")]
    #[case::opcode_84("84.json")]
    #[case::opcode_85("85.json")]
    #[case::opcode_86("86.json")]
    #[case::opcode_87("87.json")]
    #[case::opcode_88("88.json")]
    #[case::opcode_89("89.json")]
    #[case::opcode_8a("8a.json")]
    #[case::opcode_8b("8b.json")]
    #[case::opcode_8c("8c.json")]
    #[case::opcode_8d("8d.json")]
    #[case::opcode_8e("8e.json")]
    #[case::opcode_8f("8f.json")]
    #[case::opcode_90("90.json")]
    #[case::opcode_91("91.json")]
    #[case::opcode_92("92.json")]
    #[case::opcode_93("93.json")]
    #[case::opcode_94("94.json")]
    #[case::opcode_95("95.json")]
    #[case::opcode_96("96.json")]
    #[case::opcode_97("97.json")]
    #[case::opcode_98("98.json")]
    #[case::opcode_99("99.json")]
    #[case::opcode_9a("9a.json")]
    #[case::opcode_9b("9b.json")]
    #[case::opcode_9c("9c.json")]
    #[case::opcode_9d("9d.json")]
    #[case::opcode_9e("9e.json")]
    #[case::opcode_9f("9f.json")]
    #[case::opcode_a0("a0.json")]
    #[case::opcode_a1("a1.json")]
    #[case::opcode_a2("a2.json")]
    #[case::opcode_a3("a3.json")]
    #[case::opcode_a4("a4.json")]
    #[case::opcode_a5("a5.json")]
    #[case::opcode_a7("a7.json")]
    #[case::opcode_a8("a8.json")]
    #[case::opcode_a9("a9.json")]
    #[case::opcode_aa("aa.json")]
    #[case::opcode_ab("ab.json")]
    #[case::opcode_ac("ac.json")]
    #[case::opcode_ad("ad.json")]
    #[case::opcode_ae("ae.json")]
    #[case::opcode_af("af.json")]
    #[case::opcode_b0("b0.json")]
    #[case::opcode_b1("b1.json")]
    #[case::opcode_b2("b2.json")]
    #[case::opcode_b3("b3.json")]
    #[case::opcode_b4("b4.json")]
    #[case::opcode_b5("b5.json")]
    #[case::opcode_b6("b6.json")]
    #[case::opcode_b7("b7.json")]
    #[case::opcode_b8("b8.json")]
    #[case::opcode_b9("b9.json")]
    #[case::opcode_ba("ba.json")]
    #[case::opcode_bb("bb.json")]
    #[case::opcode_bc("bc.json")]
    #[case::opcode_bd("bd.json")]
    #[case::opcode_be("be.json")]
    #[case::opcode_bf("bf.json")]
    #[case::opcode_c0("c0.json")]
    #[case::opcode_c1("c1.json")]
    #[case::opcode_c2("c2.json")]
    #[case::opcode_c3("c3.json")]
    #[case::opcode_c4("c4.json")]
    #[case::opcode_c5("c5.json")]
    #[case::opcode_c6("c6.json")]
    #[case::opcode_c7("c7.json")]
    #[case::opcode_c8("c8.json")]
    #[case::opcode_c9("c9.json")]
    #[case::opcode_ca("ca.json")]
    #[case::opcode_cc("cc.json")]
    #[case::opcode_cd("cd.json")]
    #[case::opcode_ce("ce.json")]
    #[case::opcode_cf("cf.json")]
    #[case::opcode_d0("d0.json")]
    #[case::opcode_d1("d1.json")]
    #[case::opcode_d2("d2.json")]
    #[case::opcode_d4("d4.json")]
    #[case::opcode_d5("d5.json")]
    #[case::opcode_d6("d6.json")]
    #[case::opcode_d7("d7.json")]
    #[case::opcode_d8("d8.json")]
    #[case::opcode_da("da.json")]
    #[case::opcode_dc("dc.json")]
    #[case::opcode_de("de.json")]
    #[case::opcode_df("df.json")]
    #[case::opcode_e0("e0.json")]
    #[case::opcode_e1("e1.json")]
    #[case::opcode_e2("e2.json")]
    #[case::opcode_e5("e5.json")]
    #[case::opcode_e6("e6.json")]
    #[case::opcode_e7("e7.json")]
    #[case::opcode_e8("e8.json")]
    #[case::opcode_e9("e9.json")]
    #[case::opcode_ea("ea.json")]
    #[case::opcode_ee("ee.json")]
    #[case::opcode_ef("ef.json")]
    #[case::opcode_f0("f0.json")]
    #[case::opcode_f1("f1.json")]
    #[case::opcode_f2("f2.json")]
    #[case::opcode_f5("f5.json")]
    #[case::opcode_f6("f6.json")]
    #[case::opcode_f7("f7.json")]
    #[case::opcode_f8("f8.json")]
    #[case::opcode_f9("f9.json")]
    #[case::opcode_fa("fa.json")]
    #[case::opcode_fe("fe.json")]
    #[case::opcode_ff("ff.json")]
    #[case::opcode_cb_00("cb_00.json")]
    #[case::opcode_cb_01("cb_01.json")]
    #[case::opcode_cb_02("cb_02.json")]
    #[case::opcode_cb_03("cb_03.json")]
    #[case::opcode_cb_04("cb_04.json")]
    #[case::opcode_cb_05("cb_05.json")]
    #[case::opcode_cb_06("cb_06.json")]
    #[case::opcode_cb_07("cb_07.json")]
    #[case::opcode_cb_08("cb_08.json")]
    #[case::opcode_cb_09("cb_09.json")]
    #[case::opcode_cb_0a("cb_0a.json")]
    #[case::opcode_cb_0b("cb_0b.json")]
    #[case::opcode_cb_0c("cb_0c.json")]
    #[case::opcode_cb_0d("cb_0d.json")]
    #[case::opcode_cb_0e("cb_0e.json")]
    #[case::opcode_cb_0f("cb_0f.json")]
    #[case::opcode_cb_10("cb_10.json")]
    #[case::opcode_cb_11("cb_11.json")]
    #[case::opcode_cb_12("cb_12.json")]
    #[case::opcode_cb_13("cb_13.json")]
    #[case::opcode_cb_14("cb_14.json")]
    #[case::opcode_cb_15("cb_15.json")]
    #[case::opcode_cb_17("cb_17.json")]
    #[case::opcode_cb_18("cb_18.json")]
    #[case::opcode_cb_19("cb_19.json")]
    #[case::opcode_cb_1a("cb_1a.json")]
    #[case::opcode_cb_1b("cb_1b.json")]
    #[case::opcode_cb_1c("cb_1c.json")]
    #[case::opcode_cb_1d("cb_1d.json")]
    #[case::opcode_cb_1e("cb_1e.json")]
    #[case::opcode_cb_1f("cb_1f.json")]
    #[case::opcode_cb_30("cb_30.json")]
    #[case::opcode_cb_31("cb_31.json")]
    #[case::opcode_cb_32("cb_32.json")]
    #[case::opcode_cb_33("cb_33.json")]
    #[case::opcode_cb_34("cb_34.json")]
    #[case::opcode_cb_35("cb_35.json")]
    #[case::opcode_cb_37("cb_37.json")]
    #[case::opcode_cb_30("cb_30.json")]
    #[case::opcode_cb_38("cb_38.json")]
    #[case::opcode_cb_39("cb_39.json")]
    #[case::opcode_cb_3a("cb_3a.json")]
    #[case::opcode_cb_3b("cb_3b.json")]
    #[case::opcode_cb_3c("cb_3c.json")]
    #[case::opcode_cb_3d("cb_3d.json")]
    #[case::opcode_cb_3f("cb_3f.json")]
    #[case::opcode_cb_40("cb_40.json")]
    #[case::opcode_cb_41("cb_41.json")]
    #[case::opcode_cb_42("cb_42.json")]
    #[case::opcode_cb_43("cb_43.json")]
    #[case::opcode_cb_44("cb_44.json")]
    #[case::opcode_cb_45("cb_45.json")]
    #[case::opcode_cb_47("cb_47.json")]
    #[case::opcode_cb_48("cb_48.json")]
    #[case::opcode_cb_49("cb_49.json")]
    #[case::opcode_cb_4a("cb_4a.json")]
    #[case::opcode_cb_4b("cb_4b.json")]
    #[case::opcode_cb_4c("cb_4c.json")]
    #[case::opcode_cb_4d("cb_4d.json")]
    #[case::opcode_cb_4f("cb_4f.json")]
    #[case::opcode_cb_50("cb_50.json")]
    #[case::opcode_cb_51("cb_51.json")]
    #[case::opcode_cb_52("cb_52.json")]
    #[case::opcode_cb_53("cb_53.json")]
    #[case::opcode_cb_54("cb_54.json")]
    #[case::opcode_cb_55("cb_55.json")]
    #[case::opcode_cb_57("cb_57.json")]
    #[case::opcode_cb_58("cb_58.json")]
    #[case::opcode_cb_59("cb_59.json")]
    #[case::opcode_cb_5a("cb_5a.json")]
    #[case::opcode_cb_5b("cb_5b.json")]
    #[case::opcode_cb_5c("cb_5c.json")]
    #[case::opcode_cb_5d("cb_5d.json")]
    #[case::opcode_cb_5f("cb_5f.json")]
    #[case::opcode_cb_60("cb_60.json")]
    #[case::opcode_cb_61("cb_61.json")]
    #[case::opcode_cb_62("cb_62.json")]
    #[case::opcode_cb_63("cb_63.json")]
    #[case::opcode_cb_64("cb_64.json")]
    #[case::opcode_cb_65("cb_65.json")]
    #[case::opcode_cb_67("cb_67.json")]
    #[case::opcode_cb_68("cb_68.json")]
    #[case::opcode_cb_69("cb_69.json")]
    #[case::opcode_cb_6a("cb_6a.json")]
    #[case::opcode_cb_6b("cb_6b.json")]
    #[case::opcode_cb_6c("cb_6c.json")]
    #[case::opcode_cb_6d("cb_6d.json")]
    #[case::opcode_cb_6f("cb_6f.json")]
    #[case::opcode_cb_70("cb_70.json")]
    #[case::opcode_cb_71("cb_71.json")]
    #[case::opcode_cb_72("cb_72.json")]
    #[case::opcode_cb_73("cb_73.json")]
    #[case::opcode_cb_74("cb_74.json")]
    #[case::opcode_cb_75("cb_75.json")]
    #[case::opcode_cb_77("cb_77.json")]
    #[case::opcode_cb_78("cb_78.json")]
    #[case::opcode_cb_79("cb_79.json")]
    #[case::opcode_cb_7a("cb_7a.json")]
    #[case::opcode_cb_7b("cb_7b.json")]
    #[case::opcode_cb_7c("cb_7c.json")]
    #[case::opcode_cb_7d("cb_7d.json")]
    #[case::opcode_cb_7f("cb_7f.json")]
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

            let initial_state = CpuRegisters {
                a: test.initial_state.a,
                b: test.initial_state.b,
                c: test.initial_state.c,
                d: test.initial_state.d,
                e: test.initial_state.e,
                f: test.initial_state.f,
                h: test.initial_state.h,
                l: test.initial_state.l,
                interrupt_enable: 0u8,
                interrupt_flags: 0u8,
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

            let final_state = CpuRegisters {
                a: test.final_state.a,
                b: test.final_state.b,
                c: test.final_state.c,
                d: test.final_state.d,
                e: test.final_state.e,
                f: test.final_state.f,
                h: test.final_state.h,
                l: test.final_state.l,
                interrupt_enable: 0u8,
                interrupt_flags: 0u8,
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
