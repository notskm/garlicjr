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

use crate::Color;

pub struct Lcd {
    pixel_buffer: [u8; Self::RGB_SIZE],
}

impl Default for Lcd {
    fn default() -> Self {
        Self {
            pixel_buffer: [0; Self::RGB_SIZE],
        }
    }
}

impl Lcd {
    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 144;
    const RGB_SIZE: usize = 3 * Self::WIDTH * Self::HEIGHT;

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), LcdError> {
        if x >= Self::WIDTH || y >= Self::HEIGHT {
            return Err(LcdError::CoordinateOffScreen { x, y });
        }

        let start = y * Self::WIDTH + x;
        self.pixel_buffer[start..start + 3].copy_from_slice(&[color.r, color.g, color.b]);
        Ok(())
    }

    pub fn to_rgb(&self) -> [u8; Self::RGB_SIZE] {
        self.pixel_buffer
    }
}

#[derive(Debug)]
pub enum LcdError {
    CoordinateOffScreen { x: usize, y: usize },
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    use crate::Color;

    #[rstest]
    #[case(10000, 0)]
    #[case(0, 10000)]
    fn should_return_error_when_setting_a_pixel_out_of_bounds(#[case] x: usize, #[case] y: usize) {
        let mut lcd = Lcd::default();
        let error = lcd
            .set_pixel(x, y, Color::from_rgb(0, 0, 0))
            .expect_err("Expected OutOfBounds");

        assert!(
            matches!(error, LcdError::CoordinateOffScreen { x: x1, y: y1 } if x1 == x && y1 == y)
        );
    }

    #[rstest]
    fn should_return_ok_when_setting_a_pixel_within_bounds(
        #[values(0, 1, 2, 3, 159)] x: usize,
        #[values(0, 1, 2, 3, 143)] y: usize,
    ) {
        let mut lcd = Lcd::default();
        let result = lcd.set_pixel(x, y, Color::from_rgb(0, 0, 0));
        assert!(result.is_ok());
    }

    #[rstest]
    fn should_return_pixel_data_as_rgb_array(
        #[values(0, Lcd::WIDTH - 1, 50)] x: usize,
        #[values(0, Lcd::HEIGHT - 1, 50)] y: usize,
        #[values(
            Color::from_rgb(5, 6, 7),
            Color::from_rgb(10, 29, 42),
            Color::from_rgb(0, 0, 0),
            Color::from_rgb(255, 255, 255)
        )]
        color: Color,
    ) {
        let mut lcd = Lcd::default();
        lcd.set_pixel(x, y, color.clone()).expect("");
        let pixels = lcd.to_rgb();

        let expected = {
            let mut arr = [0u8; 3 * Lcd::WIDTH * Lcd::HEIGHT];

            let start = y * Lcd::WIDTH + x;
            arr[start..start + 3].clone_from_slice(&[color.r, color.g, color.b]);
            arr
        };
        assert_eq!(pixels, expected);
    }
}
