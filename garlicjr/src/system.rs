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

use crate::{Bus, Cartridge, DmgBootrom, PPU, RandomAccessMemory, ReadWriteMode, SharpSM83, Timer};

pub struct System {
    pub cpu: SharpSM83,
    pub ppu: PPU,
    pub timer: Timer,
    pub bus: Bus,
    pub bootrom: Option<DmgBootrom>,
    pub cartridge: Option<Cartridge>,
    pub work_ram_1: RandomAccessMemory,
    pub work_ram_2: RandomAccessMemory,
    pub high_ram: RandomAccessMemory,
    pub bootrom_enable_register: u8,
}

impl System {
    pub fn new() -> Self {
        Self {
            cpu: SharpSM83::new(),
            ppu: PPU::new(),
            timer: Timer::default(),
            bus: Bus::new(),
            bootrom: None,
            cartridge: None,
            work_ram_1: RandomAccessMemory::new(4096),
            work_ram_2: RandomAccessMemory::new(4096),
            high_ram: RandomAccessMemory::new(126),
            bootrom_enable_register: 0,
        }
    }

    pub fn run_cycle(&mut self) {
        for _ in 0..4 {
            self.cpu.tick(&mut self.bus);
            self.ppu.tick();
            self.timer.tick();
            if self.timer.interrupt_requested() {
                self.write(0xFF0F, 0b00000100);
            }
        }

        match self.bus.mode {
            ReadWriteMode::Read => self.bus.data = self.read(self.bus.address),
            ReadWriteMode::Write => self.write(self.bus.address, self.bus.data),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..0x0100 if self.bootrom_enabled() => self
                .bootrom
                .as_ref()
                .map(|rom| rom.data().get(address as usize).cloned().unwrap_or(0xFF))
                .unwrap_or(0xFF),
            0x0000..0x0100 if !self.bootrom_enabled() => self
                .cartridge
                .as_ref()
                .map(|cart| cart.read(address).unwrap_or(0xFF))
                .unwrap_or(0xFF),
            0x0100..=0x7FFF => self
                .cartridge
                .as_ref()
                .map(|cart| cart.read(address).unwrap_or(0xFF))
                .unwrap_or(0xFF),
            0x8000..=0x9FFF => self.ppu.read_vram(address - 0x8000),
            0xC000..=0xCFFF => self.work_ram_1.read(address - 0xC000).unwrap_or(0xFF),
            0xD000..=0xDFFF => self.work_ram_2.read(address - 0xD000).unwrap_or(0xFF),
            0xFF05 => self.timer.registers.tima,
            0xFF06 => self.timer.registers.tma,
            0xFF07 => self.timer.registers.get_tac(),
            0xFF0F => self.cpu.registers.interrupt_flags,
            0xFF40 => self.ppu.registers.lcdc,
            0xFF41 => self.ppu.registers.get_stat(),
            0xFF42 => self.ppu.registers.scy,
            0xFF43 => self.ppu.registers.scx,
            0xFF44 => self.ppu.registers.ly,
            0xFF45 => self.ppu.registers.lyc,
            0xFF4A => self.ppu.registers.wy,
            0xFF4B => self.ppu.registers.wx,
            0xFF50 => self.bootrom_enable_register,
            0xFF80..=0xFFFE => self.high_ram.read(address - 0xFF80).unwrap_or(0xFF),
            0xFFFF => self.cpu.registers.interrupt_enable,
            _ => 0xFFu8,
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x8000..=0x9FFF => self.ppu.write_vram(address - 0x8000, data),
            0xC000..=0xCFFF => self.work_ram_1.write(address - 0xC000, data),
            0xD000..=0xDFFF => self.work_ram_2.write(address - 0xD000, data),
            0xFF05 => self.timer.registers.tima = data,
            0xFF06 => self.timer.registers.tma = data,
            0xFF07 => self.timer.registers.set_tac(data),
            0xFF0F => self.cpu.registers.interrupt_flags = data & 0b00011111,
            0xFF40 => self.ppu.registers.lcdc = data,
            0xFF41 => self.ppu.registers.set_stat(data),
            0xFF42 => self.ppu.registers.scy = data,
            0xFF43 => self.ppu.registers.scx = data,
            0xFF45 => self.ppu.registers.lyc = data,
            0xFF4A => self.ppu.registers.wy = data,
            0xFF4B => self.ppu.registers.wx = data,
            0xFF50 => self.bootrom_enable_register = data,
            0xFF80..=0xFFFE => self.high_ram.write(address - 0xFF80, data),
            0xFFFF => self.cpu.registers.interrupt_enable = data & 0b00011111,
            _ => (),
        }
    }

    pub fn bootrom_enabled(&self) -> bool {
        self.bootrom_enable_register == 0
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn should_return_a_default_system() {
        let default_system = System::default();
        let new_system = System::new();

        assert_eq!(default_system.cpu.registers.a, new_system.cpu.registers.a);
        assert_eq!(default_system.cpu.registers.b, new_system.cpu.registers.b);
        assert_eq!(default_system.cpu.registers.c, new_system.cpu.registers.c);
        assert_eq!(default_system.cpu.registers.d, new_system.cpu.registers.d);
        assert_eq!(default_system.cpu.registers.e, new_system.cpu.registers.e);
        assert_eq!(default_system.cpu.registers.f, new_system.cpu.registers.f);
        assert_eq!(default_system.cpu.registers.h, new_system.cpu.registers.h);
        assert_eq!(default_system.cpu.registers.l, new_system.cpu.registers.l);
        assert_eq!(
            default_system.cpu.registers.program_counter,
            new_system.cpu.registers.program_counter
        );
        assert_eq!(
            default_system.cpu.registers.stack_pointer,
            new_system.cpu.registers.stack_pointer
        );
        assert_eq!(
            default_system.bootrom.is_none(),
            new_system.bootrom.is_none()
        );
    }

    #[test]
    fn should_return_a_new_system() {
        let system = System::new();

        assert_eq!(system.cpu.registers.a, 0);
        assert_eq!(system.cpu.registers.b, 0);
        assert_eq!(system.cpu.registers.c, 0);
        assert_eq!(system.cpu.registers.d, 0);
        assert_eq!(system.cpu.registers.e, 0);
        assert_eq!(system.cpu.registers.f, 0);
        assert_eq!(system.cpu.registers.h, 0);
        assert_eq!(system.cpu.registers.l, 0);
        assert_eq!(system.cpu.registers.program_counter, 0);
        assert_eq!(system.cpu.registers.stack_pointer, 0);
        assert!(system.bootrom.is_none());
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    fn should_run_a_cycle(#[case] start_address: u16) {
        let mut system = System::new();
        system.cpu.registers.program_counter = start_address;

        system.run_cycle();

        assert_eq!(system.cpu.registers.program_counter, start_address + 1);
    }
}
