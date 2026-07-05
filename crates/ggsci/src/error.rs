use std::fmt;

/// Error type for palette lookup and color conversion.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// The requested palette family does not exist.
    UnknownFamily {
        /// Requested family name.
        family: String,
    },
    /// The requested variant does not exist within a known family.
    UnknownVariant {
        /// Requested family name.
        family: String,
        /// Requested variant name.
        variant: String,
    },
    /// A palette specification was not written as `family:variant`.
    InvalidPaletteSpec {
        /// Requested palette specification.
        spec: String,
    },
    /// A hex color string was malformed.
    InvalidHexColor {
        /// Requested hex color.
        input: String,
    },
    /// A floating-point alpha value was outside `0.0..=1.0`.
    InvalidAlpha {
        /// Requested alpha value.
        alpha: f32,
    },
    /// More colors were requested than a static palette contains.
    TooManyColorsRequested {
        /// Palette family.
        family: &'static str,
        /// Palette variant.
        variant: &'static str,
        /// Requested number of colors.
        requested: usize,
        /// Available number of colors.
        available: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFamily { family } => {
                write!(f, "unknown ggsci palette family `{family}`")
            }
            Self::UnknownVariant { family, variant } => {
                write!(f, "unknown ggsci palette variant `{variant}` for family `{family}`")
            }
            Self::InvalidPaletteSpec { spec } => {
                write!(f, "invalid palette spec `{spec}`; expected `family:variant`")
            }
            Self::InvalidHexColor { input } => {
                write!(f, "invalid hex color `{input}`; expected `#RRGGBB`")
            }
            Self::InvalidAlpha { alpha } => {
                write!(f, "invalid alpha `{alpha}`; expected a finite value in 0.0..=1.0")
            }
            Self::TooManyColorsRequested {
                family,
                variant,
                requested,
                available,
            } => write!(
                f,
                "requested {requested} colors from `{family}:{variant}`, but only {available} are available"
            ),
        }
    }
}

impl std::error::Error for Error {}
