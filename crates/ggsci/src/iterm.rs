use std::{fmt, str::FromStr};

use crate::{continuous, generated, normalize::key_matches, Error, PaletteKind, Rgb, Rgba};

/// A normal or bright iTerm theme variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItermVariant {
    /// The theme's normal terminal colors.
    Normal,
    /// The theme's bright terminal colors.
    Bright,
}

impl ItermVariant {
    /// Returns the canonical lowercase variant name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Bright => "bright",
        }
    }

    /// Parses a normal or bright iTerm variant case-insensitively.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UnknownItermVariant`] when `input` is not `normal` or
    /// `bright`.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("normal") {
            Ok(Self::Normal)
        } else if trimmed.eq_ignore_ascii_case("bright") {
            Ok(Self::Bright)
        } else {
            Err(Error::UnknownItermVariant {
                variant: input.to_owned(),
            })
        }
    }
}

impl fmt::Display for ItermVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ItermVariant {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

/// A terminal color channel in canonical ggsci iTerm order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItermChannel {
    /// Blue terminal channel.
    Blue,
    /// Yellow terminal channel.
    Yellow,
    /// Red terminal channel.
    Red,
    /// Cyan terminal channel.
    Cyan,
    /// Green terminal channel.
    Green,
    /// Magenta terminal channel.
    Magenta,
}

impl ItermChannel {
    /// Returns the canonical channel name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blue => "Blue",
            Self::Yellow => "Yellow",
            Self::Red => "Red",
            Self::Cyan => "Cyan",
            Self::Green => "Green",
            Self::Magenta => "Magenta",
        }
    }

    /// Returns the channel's index in [`ITERM_CHANNELS`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Blue => 0,
            Self::Yellow => 1,
            Self::Red => 2,
            Self::Cyan => 3,
            Self::Green => 4,
            Self::Magenta => 5,
        }
    }
}

/// Canonical iTerm channel order from ggsci for R.
pub const ITERM_CHANNELS: [ItermChannel; 6] = [
    ItermChannel::Blue,
    ItermChannel::Yellow,
    ItermChannel::Red,
    ItermChannel::Cyan,
    ItermChannel::Green,
    ItermChannel::Magenta,
];

/// A fixed discrete iTerm theme with normal and bright variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItermPalette {
    name: &'static str,
    normal: &'static [Rgb; 6],
    bright: &'static [Rgb; 6],
}

impl ItermPalette {
    /// Creates an iTerm palette from canonical metadata and terminal colors.
    #[must_use]
    pub const fn new(
        name: &'static str,
        normal: &'static [Rgb; 6],
        bright: &'static [Rgb; 6],
    ) -> Self {
        Self {
            name,
            normal,
            bright,
        }
    }

    /// Returns the canonical theme name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the scale semantics of this iTerm palette.
    #[must_use]
    pub const fn kind(&self) -> PaletteKind {
        PaletteKind::Discrete
    }

    /// Returns all six colors for the requested variant in canonical channel
    /// order.
    #[must_use]
    pub const fn colors(&self, variant: ItermVariant) -> &'static [Rgb; 6] {
        match variant {
            ItermVariant::Normal => self.normal,
            ItermVariant::Bright => self.bright,
        }
    }

    /// Returns one terminal channel color from the requested variant.
    #[must_use]
    pub const fn color(&self, variant: ItermVariant, channel: ItermChannel) -> Rgb {
        self.colors(variant)[channel.index()]
    }

    /// Returns the first `n` colors from a variant in canonical channel order.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyItermColorsRequested`] if `n` exceeds six.
    /// This method never cycles colors.
    pub fn take(&self, variant: ItermVariant, n: usize) -> Result<Vec<Rgb>, Error> {
        let colors = self.colors(variant);
        if n > colors.len() {
            return Err(Error::TooManyItermColorsRequested {
                palette: self.name,
                variant: variant.as_str(),
                requested: n,
                available: colors.len(),
            });
        }

        Ok(colors[..n].to_vec())
    }

    /// Returns the first `n` colors as `#RRGGBB` strings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyItermColorsRequested`] if `n` exceeds six.
    pub fn take_hex(&self, variant: ItermVariant, n: usize) -> Result<Vec<String>, Error> {
        self.take(variant, n)
            .map(|colors| colors.into_iter().map(Rgb::to_hex_string).collect())
    }

    /// Returns the first `n` colors with an alpha channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAlpha`] unless `alpha` is finite and in
    /// `(0.0, 1.0]`, or [`Error::TooManyItermColorsRequested`] if `n` exceeds
    /// six.
    pub fn take_rgba(
        &self,
        variant: ItermVariant,
        n: usize,
        alpha: f32,
    ) -> Result<Vec<Rgba>, Error> {
        let alpha = continuous::continuous_alpha(alpha)?;
        self.take(variant, n).map(|colors| {
            colors
                .into_iter()
                .map(|color| color.with_alpha_u8(alpha))
                .collect()
        })
    }

    /// Returns an explicitly infinite iterator over a variant's six colors.
    pub fn cycle(&self, variant: ItermVariant) -> impl Iterator<Item = Rgb> + '_ {
        self.colors(variant).iter().copied().cycle()
    }
}

/// Returns the dedicated iTerm palette registry.
///
/// Every item has [`PaletteKind::Discrete`] scale semantics. These records are
/// not included in [`crate::palettes`] or [`crate::palettes_by_kind`] because
/// each theme carries normal and bright variants plus a fixed terminal-channel
/// ordering.
#[must_use]
pub fn iterm_palettes() -> &'static [ItermPalette] {
    let palettes = generated::iterm::ITERM_PALETTES;
    debug_assert_eq!(palettes.len(), generated::iterm::ITERM_PALETTE_COUNT);
    debug_assert_eq!(generated::iterm::ITERM_VARIANT_COUNT, 2);
    debug_assert_eq!(generated::iterm::ITERM_COLORS_PER_VARIANT, 6);
    debug_assert_eq!(
        palettes.len()
            * generated::iterm::ITERM_VARIANT_COUNT
            * generated::iterm::ITERM_COLORS_PER_VARIANT,
        generated::iterm::ITERM_TOTAL_COLOR_COUNT
    );
    debug_assert_eq!(
        generated::iterm::ITERM_DATA_SOURCE,
        "ggsci/R/palettes-iterm.R"
    );
    palettes
}

/// Looks up an iTerm palette by theme name.
///
/// Lookup is case-insensitive. Underscores, hyphens, and whitespace are
/// interchangeable separators; other punctuation remains significant.
///
/// # Errors
///
/// Returns [`Error::UnknownItermPalette`] when the theme is not known.
pub fn iterm_palette(name: &str) -> Result<&'static ItermPalette, Error> {
    iterm_palettes()
        .iter()
        .find(|palette| key_matches(palette.name(), name))
        .ok_or_else(|| Error::UnknownItermPalette {
            palette: name.to_owned(),
        })
}

/// Returns all canonical iTerm theme names in upstream order.
pub fn iterm_palette_names() -> impl Iterator<Item = &'static str> {
    iterm_palettes().iter().map(ItermPalette::name)
}

/// Returns the number of iTerm themes in the dedicated registry.
#[must_use]
pub const fn iterm_palette_count() -> usize {
    generated::iterm::ITERM_PALETTE_COUNT
}

/// Returns the number of stored colors across both variants of all themes.
#[must_use]
pub const fn iterm_total_color_count() -> usize {
    generated::iterm::ITERM_TOTAL_COLOR_COUNT
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::iterm_palette_names;
    use crate::normalize::normalize_key;

    #[test]
    fn normalized_theme_names_are_unique() {
        let names = iterm_palette_names().collect::<Vec<_>>();
        let normalized = names
            .iter()
            .map(|name| normalize_key(name))
            .collect::<HashSet<_>>();
        assert_eq!(normalized.len(), names.len());
    }
}
