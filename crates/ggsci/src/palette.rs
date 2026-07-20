use std::fmt;

use crate::{continuous, Error, Rgb, Rgba};

/// The scale semantics of a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaletteKind {
    /// Maps discrete categories to individual colors.
    Discrete,

    /// Maps a continuous domain through an interpolated color gradient.
    Continuous,
}

impl PaletteKind {
    /// Returns the canonical lowercase palette kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discrete => "discrete",
            Self::Continuous => "continuous",
        }
    }
}

impl fmt::Display for PaletteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Options controlling continuous palette interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContinuousOptions {
    reverse: bool,
}

impl ContinuousOptions {
    /// Creates interpolation options with their defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self { reverse: false }
    }

    /// Sets whether to reverse the colors after interpolation.
    #[must_use]
    pub const fn with_reverse(self, reverse: bool) -> Self {
        Self { reverse }
    }

    /// Returns whether colors are reversed after interpolation.
    #[must_use]
    pub const fn reverse(self) -> bool {
        self.reverse
    }
}

/// A canonical palette specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaletteSpec {
    family: &'static str,
    variant: &'static str,
}

impl PaletteSpec {
    /// Creates a palette specification from canonical family and variant names.
    #[must_use]
    pub const fn new(family: &'static str, variant: &'static str) -> Self {
        Self { family, variant }
    }

    /// Returns the canonical family name.
    #[must_use]
    pub const fn family(self) -> &'static str {
        self.family
    }

    /// Returns the canonical variant name.
    #[must_use]
    pub const fn variant(self) -> &'static str {
        self.variant
    }
}

/// A generated ggsci palette with discrete or continuous scale semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    family: &'static str,
    variant: &'static str,
    kind: PaletteKind,
    colors: &'static [Rgb],
}

impl Palette {
    /// Creates a palette from canonical metadata and stored source colors.
    #[must_use]
    pub const fn new(
        family: &'static str,
        variant: &'static str,
        kind: PaletteKind,
        colors: &'static [Rgb],
    ) -> Self {
        Self {
            family,
            variant,
            kind,
            colors,
        }
    }

    /// Returns the canonical family name.
    #[must_use]
    pub const fn family(&self) -> &'static str {
        self.family
    }

    /// Returns the canonical variant name.
    #[must_use]
    pub const fn variant(&self) -> &'static str {
        self.variant
    }

    /// Returns whether this palette is discrete or continuous.
    #[must_use]
    pub const fn kind(&self) -> PaletteKind {
        self.kind
    }

    /// Returns this palette's canonical specification.
    #[must_use]
    pub const fn spec(&self) -> PaletteSpec {
        PaletteSpec::new(self.family, self.variant)
    }

    /// Returns this palette's canonical source colors.
    ///
    /// These are category colors for a discrete palette and interpolation
    /// anchors for a continuous palette.
    #[must_use]
    pub const fn colors(&self) -> &'static [Rgb] {
        self.colors
    }

    /// Returns the number of stored source colors or interpolation anchors.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns `true` if the palette has no colors.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Returns `true` if this palette maps discrete categories.
    #[must_use]
    pub const fn is_discrete(&self) -> bool {
        matches!(self.kind, PaletteKind::Discrete)
    }

    /// Returns `true` if this palette maps a continuous domain.
    #[must_use]
    pub const fn is_continuous(&self) -> bool {
        matches!(self.kind, PaletteKind::Continuous)
    }

    /// Returns the first `n` category colors from a discrete palette.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotDiscretePalette`] for a continuous palette, or
    /// [`Error::TooManyColorsRequested`] if `n` exceeds the number of stored
    /// category colors. This method does not cycle colors.
    pub fn take(&self, n: usize) -> Result<Vec<Rgb>, Error> {
        self.ensure_discrete()?;
        if n > self.colors.len() {
            return Err(Error::TooManyColorsRequested {
                family: self.family,
                variant: self.variant,
                requested: n,
                available: self.colors.len(),
            });
        }

        Ok(self.colors[..n].to_vec())
    }

    /// Returns the first `n` category colors as `#RRGGBB` strings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotDiscretePalette`] for a continuous palette, or
    /// [`Error::TooManyColorsRequested`] if `n` exceeds the number of stored
    /// category colors. This method does not cycle colors.
    pub fn take_hex(&self, n: usize) -> Result<Vec<String>, Error> {
        self.take(n)
            .map(|colors| colors.into_iter().map(Rgb::to_hex_string).collect())
    }

    /// Returns an explicitly infinite iterator over discrete category colors.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotDiscretePalette`] for a continuous palette.
    pub fn cycle(&self) -> Result<impl Iterator<Item = Rgb> + '_, Error> {
        self.ensure_discrete()?;
        Ok(self.colors.iter().copied().cycle())
    }

    /// Interpolates `n` colors from a continuous palette.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotContinuousPalette`] for a discrete palette.
    pub fn interpolate(&self, n: usize) -> Result<Vec<Rgb>, Error> {
        self.interpolate_with(n, ContinuousOptions::new())
    }

    /// Interpolates `n` colors using the supplied continuous options.
    ///
    /// Reversal is applied after interpolation, matching ggsci for R.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotContinuousPalette`] for a discrete palette.
    pub fn interpolate_with(
        &self,
        n: usize,
        options: ContinuousOptions,
    ) -> Result<Vec<Rgb>, Error> {
        self.ensure_continuous()?;
        let mut colors = continuous::interpolate(self.colors, n);
        if options.reverse() {
            colors.reverse();
        }
        Ok(colors)
    }

    /// Interpolates `n` colors and applies an alpha channel.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotContinuousPalette`] for a discrete palette, or
    /// [`Error::InvalidAlpha`] unless `alpha` is finite and in `(0.0, 1.0]`.
    pub fn interpolate_rgba(&self, n: usize, alpha: f32) -> Result<Vec<Rgba>, Error> {
        self.interpolate_rgba_with(n, alpha, ContinuousOptions::new())
    }

    /// Interpolates `n` colors with an alpha channel and continuous options.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotContinuousPalette`] for a discrete palette, or
    /// [`Error::InvalidAlpha`] unless `alpha` is finite and in `(0.0, 1.0]`.
    pub fn interpolate_rgba_with(
        &self,
        n: usize,
        alpha: f32,
        options: ContinuousOptions,
    ) -> Result<Vec<Rgba>, Error> {
        self.ensure_continuous()?;
        let alpha = continuous::continuous_alpha(alpha)?;
        self.interpolate_with(n, options).map(|colors| {
            colors
                .into_iter()
                .map(|color| color.with_alpha_u8(alpha))
                .collect()
        })
    }

    /// Interpolates `n` colors as `#RRGGBB` strings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotContinuousPalette`] for a discrete palette.
    pub fn interpolate_hex(&self, n: usize) -> Result<Vec<String>, Error> {
        self.interpolate_hex_with(n, ContinuousOptions::new())
    }

    /// Interpolates `n` colors as `#RRGGBB` strings with continuous options.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotContinuousPalette`] for a discrete palette.
    pub fn interpolate_hex_with(
        &self,
        n: usize,
        options: ContinuousOptions,
    ) -> Result<Vec<String>, Error> {
        self.interpolate_with(n, options)
            .map(|colors| colors.into_iter().map(Rgb::to_hex_string).collect())
    }

    /// Returns `n` colors using kind-aware palette semantics.
    ///
    /// Discrete palettes use [`Self::take`], while continuous palettes use
    /// [`Self::interpolate`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyColorsRequested`] when a discrete palette does
    /// not contain enough category colors.
    pub fn sample(&self, n: usize) -> Result<Vec<Rgb>, Error> {
        match self.kind {
            PaletteKind::Discrete => self.take(n),
            PaletteKind::Continuous => self.interpolate(n),
        }
    }

    /// Returns `n` kind-aware colors as `#RRGGBB` strings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyColorsRequested`] when a discrete palette does
    /// not contain enough category colors.
    pub fn sample_hex(&self, n: usize) -> Result<Vec<String>, Error> {
        match self.kind {
            PaletteKind::Discrete => self.take_hex(n),
            PaletteKind::Continuous => self.interpolate_hex(n),
        }
    }

    fn ensure_discrete(&self) -> Result<(), Error> {
        if self.is_discrete() {
            Ok(())
        } else {
            Err(Error::NotDiscretePalette {
                family: self.family,
                variant: self.variant,
            })
        }
    }

    fn ensure_continuous(&self) -> Result<(), Error> {
        if self.is_continuous() {
            Ok(())
        } else {
            Err(Error::NotContinuousPalette {
                family: self.family,
                variant: self.variant,
            })
        }
    }
}
