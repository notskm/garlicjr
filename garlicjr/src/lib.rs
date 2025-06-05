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

mod bootrom;
mod bus;
mod cpu;
mod memory;
mod opcode;
mod ppu;
mod system;

pub use bootrom::*;
pub use bus::*;
pub use cpu::*;
pub use memory::*;
pub use ppu::*;
pub use system::*;

pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
