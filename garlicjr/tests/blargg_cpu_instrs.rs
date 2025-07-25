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

use std::{fs::File, path::Path};

use garlicjr::{Cartridge, ReadWriteMode, System};
use rstest::rstest;

#[rstest]
// #[case::cpu_instrs_01_special("01-special", 0)]
#[case::cpu_instrs_02_interrupts("02-interrupts", 1)]
#[case::cpu_instrs_03_op_sp_hl("03-op sp,hl", 3)]
#[case::cpu_instrs_04_op_r_imm("04-op r,imm", 3)]
#[case::cpu_instrs_05_op_rp("05-op rp", 4)]
#[case::cpu_instrs_06_ld_r_r("06-ld r,r", 2)]
// #[case::cpu_instrs_07_jr_jp_call_ret_rst("07-jr,jp,call,ret,rst", 0)]
#[case::cpu_instrs_08_misc_instrs("08-misc instrs", 1)]
// #[case::cpu_instrs_09_op_r_r("09-op r,r", 0)]
// #[case::cpu_instrs_10_bit_ops("10-bit ops", 0)]
// #[case::cpu_instrs_11_op_a_hl("11-op a,(hl)", 0)]
fn should_pass_blargg_cpu_instrs_tests(#[case] test_file: &str, #[case] seconds: i32) {
    let test_filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test-data")
        .join("blargg-gb-tests")
        .join("cpu_instrs")
        .join("individual")
        .join(format!("{test_file}.gb"));

    let mut dmg = initialize_dmg(test_filepath.as_path());

    let mut result = String::new();
    let mut last_char = '\0';
    const ONE_MEBIHERTZ: i32 = 1048576;
    for _ in 0..ONE_MEBIHERTZ * seconds {
        dmg.run_cycle();

        // These tests write ASCII data to the link port at 0xFF01. They
        // write 0x81 to 0xFF02 immediately afterward. It's important to
        // check writes to 0xFF02 to ensure we're reading the test results.
        if dmg.bus.address == 0xFF01 && dmg.bus.mode == ReadWriteMode::Write {
            last_char = dmg.bus.data as char;
        } else if dmg.bus.address == 0xFF02
            && dmg.bus.data == 0x81
            && dmg.bus.mode == ReadWriteMode::Write
        {
            result.push(last_char);
        }
    }

    let expected = format!("{test_file}\n\n\nPassed\n");
    assert_eq!(result, expected);
}

fn initialize_dmg(rom_filepath: &Path) -> System {
    let mut dmg = System::new();
    dmg.cartridge = Some(load_cartridge(rom_filepath));
    dmg.cpu.registers.program_counter = 0x0100;
    dmg.cpu.registers.a = 0x01;
    dmg.cpu.registers.f = 0xB0;
    dmg.cpu.registers.b = 0x00;
    dmg.cpu.registers.c = 0x13;
    dmg.cpu.registers.d = 0x00;
    dmg.cpu.registers.e = 0xD8;
    dmg.cpu.registers.h = 0x01;
    dmg.cpu.registers.l = 0x4D;
    dmg.cpu.registers.stack_pointer = 0xFFFE;
    dmg.cpu.registers.program_counter = 0x0100;
    dmg.bootrom_enable_register = 0x01;
    dmg
}

fn load_cartridge(file_path: &Path) -> Cartridge {
    let file = File::open(file_path).unwrap();
    Cartridge::from_reader(file).unwrap()
}
