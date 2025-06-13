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

pub trait OverflowHalfCarry<T> {
    fn overflowing_add_with_half_carry(self, rhs: T) -> (T, bool, bool);
    fn full_overflowing_add(self, rhs: T, carry: bool) -> (T, bool, bool);
}

impl OverflowHalfCarry<u8> for u8 {
    fn overflowing_add_with_half_carry(self, rhs: u8) -> (u8, bool, bool) {
        let (value, carry) = self.overflowing_add(rhs);
        let half_carry = (self & 0xF) + (rhs & 0xF) > 0xF;
        (value, carry, half_carry)
    }

    fn full_overflowing_add(self, rhs: u8, carry: bool) -> (u8, bool, bool) {
        let carry = carry as u8;

        let (value, c1, h1) = self.overflowing_add_with_half_carry(rhs);
        let (value, c2, h2) = value.overflowing_add_with_half_carry(carry);

        (value, c1 || c2, h1 || h2)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(255, 1)]
    #[case(1, 255)]
    #[case(123, 240)]
    #[case(240, 123)]
    #[case(255, 255)]
    fn should_return_true_full_carry_when_overflowing_add_overflows(
        #[case] lhs: u8,
        #[case] rhs: u8,
    ) {
        let (_, carry, _) = lhs.overflowing_add_with_half_carry(rhs);
        assert!(carry);
    }

    #[rstest]
    #[case(255, 0)]
    #[case(0, 255)]
    #[case(123, 100)]
    #[case(100, 123)]
    #[case(20, 50)]
    fn should_return_false_full_carry_when_overflowing_add_does_not_overflow(
        #[case] lhs: u8,
        #[case] rhs: u8,
    ) {
        let (_, carry, _) = lhs.overflowing_add_with_half_carry(rhs);
        assert!(!carry);
    }

    #[rstest]
    #[case(0b00001111, 1)]
    #[case(1, 0b00001111)]
    #[case(0b11111111, 1)]
    #[case(1, 0b11111111)]
    #[case(0b00001100, 30)]
    #[case(30, 0b00001100)]
    fn should_return_true_half_carry_when_overflowing_add_low_nibble_overflows(
        #[case] lhs: u8,
        #[case] rhs: u8,
    ) {
        let (_, _, half_carry) = lhs.overflowing_add_with_half_carry(rhs);
        assert!(half_carry);
    }

    #[rstest]
    #[case(0b00001111, 0)]
    #[case(0, 0b00001111)]
    #[case(0b11111111, 0)]
    #[case(0, 0b11111111)]
    #[case(0b00001100, 0b00000011)]
    #[case(0b00000011, 0b00001100)]
    fn should_return_false_half_carry_when_overflowing_add_low_nibble_does_not_overflow(
        #[case] lhs: u8,
        #[case] rhs: u8,
    ) {
        let (_, _, half_carry) = lhs.overflowing_add_with_half_carry(rhs);
        assert!(!half_carry);
    }

    #[rstest]
    #[case(0b00001111, 1)]
    #[case(1, 0b00001111)]
    #[case(0b11111111, 1)]
    #[case(1, 0b11111111)]
    #[case(0b00001100, 30)]
    #[case(30, 0b00001100)]
    fn should_return_same_value_as_overflowing_add(#[case] lhs: u8, #[case] rhs: u8) {
        let (value, _, _) = lhs.overflowing_add_with_half_carry(rhs);
        let (expected, _) = lhs.overflowing_add(rhs);
        assert_eq!(value, expected);
    }

    #[rstest]
    #[case(255, 0, true)]
    #[case(0, 255, true)]
    #[case(255, 1, false)]
    #[case(1, 255, false)]
    #[case(123, 240, true)]
    #[case(123, 240, false)]
    fn should_return_true_full_carry_when_full_add_overflows(
        #[case] lhs: u8,
        #[case] rhs: u8,
        #[case] carry: bool,
    ) {
        let (_, new_carry, _) = lhs.full_overflowing_add(rhs, carry);
        assert!(new_carry);
    }

    #[rstest]
    #[case(255, 0, false)]
    #[case(0, 255, false)]
    #[case(254, 1, false)]
    #[case(254, 0, true)]
    #[case(123, 20, true)]
    #[case(123, 20, false)]
    fn should_return_false_full_carry_when_full_add_does_not_overflow(
        #[case] lhs: u8,
        #[case] rhs: u8,
        #[case] carry: bool,
    ) {
        let (_, new_carry, _) = lhs.full_overflowing_add(rhs, carry);
        assert!(!new_carry);
    }

    #[rstest]
    #[case(0b00001111, 0, true)]
    #[case(0, 0b00001111, true)]
    #[case(0b00001111, 1, false)]
    #[case(0b00001111, 1, true)]
    #[case(0b10101111, 0, true)]
    #[case(0b10101111, 1, false)]
    #[case(0b01011110, 1, true)]
    fn should_return_true_half_carry_when_full_add_low_nibble_overflows(
        #[case] lhs: u8,
        #[case] rhs: u8,
        #[case] carry: bool,
    ) {
        let (_, _, half_carry) = lhs.full_overflowing_add(rhs, carry);
        assert!(half_carry);
    }

    #[rstest]
    #[case(0b00001111, 0, false)]
    #[case(0, 0b00001111, false)]
    #[case(0b00001110, 1, false)]
    #[case(0b10101101, 0, true)]
    #[case(0b10101101, 1, true)]
    fn should_return_false_half_carry_when_full_add_low_nibble_does_not_overflow(
        #[case] lhs: u8,
        #[case] rhs: u8,
        #[case] carry: bool,
    ) {
        let (_, _, half_carry) = lhs.full_overflowing_add(rhs, carry);
        assert!(!half_carry);
    }

    #[rstest]
    fn should_return_same_value_as_overflowing_add_plus_carry(
        #[values(0, 5, 255, 123, 32)] lhs: u8,
        #[values(0, 5, 255, 123, 32)] rhs: u8,
        #[values(true, false)] carry: bool,
    ) {
        let (value, _, _) = lhs.full_overflowing_add(rhs, carry);
        let (expected, _) = lhs.wrapping_add(carry as u8).overflowing_add(rhs);
        assert_eq!(value, expected);
    }
}
