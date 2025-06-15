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

#[derive(Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };

    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    pub const GRAY: Self = Self {
        r: 100,
        g: 100,
        b: 100,
    };

    pub const LIGHT_GRAY: Self = Self {
        r: 150,
        g: 150,
        b: 150,
    };

    pub const DARK_GRAY: Self = Self {
        r: 50,
        g: 50,
        b: 50,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    fn should_construct_a_color_with_the_given_components(
        #[values(0, 255, 5)] r: u8,
        #[values(0, 255, 5)] g: u8,
        #[values(0, 255, 5)] b: u8,
    ) {
        let color = Color::from_rgb(r, g, b);
        assert_eq!(color.r, r);
        assert_eq!(color.g, g);
        assert_eq!(color.b, b);
    }
}
