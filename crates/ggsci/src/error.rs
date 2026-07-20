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
    /// The requested iTerm palette does not exist.
    UnknownItermPalette {
        /// Requested iTerm theme name.
        palette: String,
    },
    /// The requested iTerm variant does not exist.
    UnknownItermVariant {
        /// Requested iTerm variant name.
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
    /// A floating-point alpha value was outside its operation's valid range.
    InvalidAlpha {
        /// Requested alpha value.
        alpha: f32,
    },
    /// More colors were requested than a discrete palette contains.
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
    /// More colors were requested than an iTerm variant contains.
    TooManyItermColorsRequested {
        /// Canonical iTerm theme name.
        palette: &'static str,
        /// Canonical iTerm variant name.
        variant: &'static str,
        /// Requested number of colors.
        requested: usize,
        /// Available number of colors.
        available: usize,
    },
    /// A discrete-only operation was requested for a continuous palette.
    NotDiscretePalette {
        /// Palette family.
        family: &'static str,
        /// Palette variant.
        variant: &'static str,
    },
    /// A continuous-only operation was requested for a discrete palette.
    NotContinuousPalette {
        /// Palette family.
        family: &'static str,
        /// Palette variant.
        variant: &'static str,
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
            Self::UnknownItermPalette { palette } => {
                write!(f, "unknown ggsci iTerm palette `{palette}`")
            }
            Self::UnknownItermVariant { variant } => {
                write!(f, "unknown ggsci iTerm variant `{variant}`")
            }
            Self::InvalidPaletteSpec { spec } => {
                write!(f, "invalid palette spec `{spec}`; expected `family:variant`")
            }
            Self::InvalidHexColor { input } => {
                write!(f, "invalid hex color `{input}`; expected `#RRGGBB`")
            }
            Self::InvalidAlpha { alpha } => {
                write!(
                    f,
                    "invalid alpha `{alpha}`; expected a finite value in 0.0..=1.0 (continuous and iTerm palette operations require alpha > 0.0)"
                )
            }
            Self::TooManyColorsRequested {
                family,
                variant,
                requested,
                available,
            } => write!(
                f,
                "requested {requested} colors from discrete palette `{family}:{variant}`, but only {available} category colors are available"
            ),
            Self::TooManyItermColorsRequested {
                palette,
                variant,
                requested,
                available,
            } => write!(
                f,
                "requested {requested} colors from iTerm palette `{palette}` ({variant}), but only {available} terminal colors are available"
            ),
            Self::NotDiscretePalette { family, variant } => write!(
                f,
                "`{family}:{variant}` is a continuous palette; this operation requires a discrete palette"
            ),
            Self::NotContinuousPalette { family, variant } => write!(
                f,
                "`{family}:{variant}` is a discrete palette; this operation requires a continuous palette"
            ),
        }
    }
}

impl std::error::Error for Error {}
