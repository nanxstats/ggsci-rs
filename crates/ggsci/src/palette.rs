use crate::{Error, Rgb};

/// The implementation kind for a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    /// A static palette backed by checked-in color data.
    Static,
}

/// A canonical static palette specification.
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

/// A generated ggsci color palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    family: &'static str,
    variant: &'static str,
    kind: PaletteKind,
    colors: &'static [Rgb],
}

impl Palette {
    /// Creates a palette from canonical metadata and static colors.
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

    /// Returns this palette's implementation kind.
    #[must_use]
    pub const fn kind(&self) -> PaletteKind {
        self.kind
    }

    /// Returns this palette's canonical specification.
    #[must_use]
    pub const fn spec(&self) -> PaletteSpec {
        PaletteSpec::new(self.family, self.variant)
    }

    /// Returns all colors in the palette.
    #[must_use]
    pub const fn colors(&self) -> &'static [Rgb] {
        self.colors
    }

    /// Returns the number of colors in the palette.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns `true` if the palette has no colors.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    /// Returns the first `n` colors.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyColorsRequested`] if `n` exceeds the palette
    /// length. This method does not cycle colors.
    pub fn take(&self, n: usize) -> Result<Vec<Rgb>, Error> {
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

    /// Returns the first `n` colors as `#RRGGBB` strings.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyColorsRequested`] if `n` exceeds the palette
    /// length. This method does not cycle colors.
    pub fn take_hex(&self, n: usize) -> Result<Vec<String>, Error> {
        self.take(n)
            .map(|colors| colors.into_iter().map(Rgb::to_hex_string).collect())
    }

    /// Returns an explicit cycling iterator over the palette colors.
    pub fn cycle(&self) -> impl Iterator<Item = Rgb> + '_ {
        self.colors.iter().copied().cycle()
    }
}
