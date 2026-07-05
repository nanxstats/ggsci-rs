use crate::Error;

/// A packed RGB color stored as `0xRRGGBB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb(u32);

impl Rgb {
    /// Creates an RGB color from a packed `0xRRGGBB` value.
    #[must_use]
    pub const fn from_hex(value: u32) -> Self {
        Self(value & 0x00FF_FFFF)
    }

    /// Parses a `#RRGGBB` hex color.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidHexColor`] if `input` is not exactly a
    /// six-digit RGB hex color with a leading `#`.
    pub fn parse_hex(input: &str) -> Result<Self, Error> {
        let Some(hex) = input.strip_prefix('#') else {
            return Err(Error::InvalidHexColor {
                input: input.to_owned(),
            });
        };

        if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::InvalidHexColor {
                input: input.to_owned(),
            });
        }

        u32::from_str_radix(hex, 16)
            .map(Self::from_hex)
            .map_err(|_| Error::InvalidHexColor {
                input: input.to_owned(),
            })
    }

    /// Returns the red channel.
    #[must_use]
    pub const fn r(self) -> u8 {
        self.0.to_be_bytes()[1]
    }

    /// Returns the green channel.
    #[must_use]
    pub const fn g(self) -> u8 {
        self.0.to_be_bytes()[2]
    }

    /// Returns the blue channel.
    #[must_use]
    pub const fn b(self) -> u8 {
        self.0.to_be_bytes()[3]
    }

    /// Returns the packed `0xRRGGBB` value.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Formats the color as `#RRGGBB`.
    #[must_use]
    pub fn to_hex_string(self) -> String {
        format!("#{:06X}", self.0)
    }

    /// Returns this RGB color with an 8-bit alpha channel.
    #[must_use]
    pub const fn with_alpha_u8(self, alpha: u8) -> Rgba {
        Rgba::from_rgb_alpha(self, alpha)
    }

    /// Returns this RGB color with a floating-point alpha channel.
    ///
    /// `alpha` must be finite and in the inclusive range `0.0..=1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAlpha`] when `alpha` is outside the valid range
    /// or is not finite.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn with_alpha(self, alpha: f32) -> Result<Rgba, Error> {
        if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
            return Err(Error::InvalidAlpha { alpha });
        }

        Ok(self.with_alpha_u8((alpha * 255.0).round() as u8))
    }
}

/// A packed RGBA color stored as `0xRRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba(u32);

impl Rgba {
    /// Creates an RGBA color from a packed `0xRRGGBBAA` value.
    #[must_use]
    pub const fn from_hex(value: u32) -> Self {
        Self(value)
    }

    /// Creates an RGBA color from RGB and 8-bit alpha components.
    #[must_use]
    pub const fn from_rgb_alpha(rgb: Rgb, alpha: u8) -> Self {
        Self((rgb.to_u32() << 8) | alpha as u32)
    }

    /// Parses a `#RRGGBBAA` hex color.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidHexColor`] if `input` is not exactly an
    /// eight-digit RGBA hex color with a leading `#`.
    pub fn parse_hex(input: &str) -> Result<Self, Error> {
        let Some(hex) = input.strip_prefix('#') else {
            return Err(Error::InvalidHexColor {
                input: input.to_owned(),
            });
        };

        if hex.len() != 8 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::InvalidHexColor {
                input: input.to_owned(),
            });
        }

        u32::from_str_radix(hex, 16)
            .map(Self::from_hex)
            .map_err(|_| Error::InvalidHexColor {
                input: input.to_owned(),
            })
    }

    /// Returns the red channel.
    #[must_use]
    pub const fn r(self) -> u8 {
        self.0.to_be_bytes()[0]
    }

    /// Returns the green channel.
    #[must_use]
    pub const fn g(self) -> u8 {
        self.0.to_be_bytes()[1]
    }

    /// Returns the blue channel.
    #[must_use]
    pub const fn b(self) -> u8 {
        self.0.to_be_bytes()[2]
    }

    /// Returns the alpha channel.
    #[must_use]
    pub const fn a(self) -> u8 {
        self.0.to_be_bytes()[3]
    }

    /// Returns the packed `0xRRGGBBAA` value.
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Formats the color as `#RRGGBBAA`.
    #[must_use]
    pub fn to_hex_string(self) -> String {
        format!("#{:08X}", self.0)
    }
}
