//! Scientific and sci-fi color palettes from the R package ggsci.
//!
//! This crate ships checked-in Rust source generated from upstream
//! `ggsci/R/palettes.R`. It has no runtime dependencies and does not require R
//! at build time.
//!
//! ```
//! use ggsci::{palette_by_spec, ContinuousOptions};
//!
//! let palette = palette_by_spec("material:blue-grey")?;
//! let colors = palette.interpolate(256)?;
//! let sampled = palette.sample(256)?;
//! let reversed = palette.interpolate_with(
//!     256,
//!     ContinuousOptions::new().with_reverse(true),
//! )?;
//!
//! assert_eq!(colors, sampled);
//! assert_eq!(colors.len(), 256);
//! assert_eq!(reversed.len(), 256);
//! # Ok::<(), ggsci::Error>(())
//! ```

mod color;
mod continuous;
mod error;
mod generated;
mod palette;

pub use color::{Rgb, Rgba};
pub use error::Error;
pub use palette::{ContinuousOptions, Palette, PaletteKind, PaletteSpec};

/// Returns all generated core palettes.
#[must_use]
pub fn palettes() -> &'static [Palette] {
    let palettes = generated::palettes::PALETTES;
    debug_assert_eq!(palettes.len(), generated::palettes::PALETTE_COUNT);
    debug_assert_eq!(
        palettes
            .iter()
            .filter(|palette| palette.is_discrete())
            .count(),
        generated::palettes::DISCRETE_PALETTE_COUNT
    );
    debug_assert_eq!(
        palettes
            .iter()
            .filter(|palette| palette.is_continuous())
            .count(),
        generated::palettes::CONTINUOUS_PALETTE_COUNT
    );
    debug_assert_eq!(
        palettes
            .iter()
            .filter(|palette| palette.is_discrete())
            .map(Palette::len)
            .sum::<usize>(),
        generated::palettes::DISCRETE_COLOR_COUNT
    );
    debug_assert_eq!(
        palettes
            .iter()
            .filter(|palette| palette.is_continuous())
            .map(Palette::len)
            .sum::<usize>(),
        generated::palettes::CONTINUOUS_ANCHOR_COLOR_COUNT
    );
    debug_assert_eq!(generated::palettes::DATA_SOURCE, "ggsci/R/palettes.R");
    palettes
}

/// Looks up a palette by family and variant.
///
/// Lookup is case-insensitive. Underscores, hyphens, and spaces are treated as
/// interchangeable separators.
///
/// # Errors
///
/// Returns [`Error::UnknownFamily`] if the family is not known, or
/// [`Error::UnknownVariant`] if the family exists but the variant does not.
pub fn palette(family: &str, variant: &str) -> Result<&'static Palette, Error> {
    let family_exists = palettes()
        .iter()
        .any(|palette| key_matches(palette.family(), family));

    if !family_exists {
        return Err(Error::UnknownFamily {
            family: family.to_owned(),
        });
    }

    palettes()
        .iter()
        .find(|palette| {
            key_matches(palette.family(), family) && key_matches(palette.variant(), variant)
        })
        .ok_or_else(|| Error::UnknownVariant {
            family: family.to_owned(),
            variant: variant.to_owned(),
        })
}

/// Looks up a palette by a `family:variant` specification.
///
/// # Errors
///
/// Returns [`Error::InvalidPaletteSpec`] if `spec` is not exactly
/// `family:variant`. Otherwise returns the same errors as [`palette`].
pub fn palette_by_spec(spec: &str) -> Result<&'static Palette, Error> {
    let trimmed = spec.trim();
    let Some((family, variant)) = trimmed.split_once(':') else {
        return Err(Error::InvalidPaletteSpec {
            spec: spec.to_owned(),
        });
    };

    if family.trim().is_empty() || variant.trim().is_empty() || variant.contains(':') {
        return Err(Error::InvalidPaletteSpec {
            spec: spec.to_owned(),
        });
    }

    palette(family.trim(), variant.trim())
}

/// Returns canonical `(family, variant)` names for all palettes.
pub fn palette_names() -> impl Iterator<Item = (&'static str, &'static str)> {
    palettes()
        .iter()
        .map(|palette| (palette.family(), palette.variant()))
}

/// Filters the core records returned by [`palettes`] by scale semantics.
///
/// Future dedicated iTerm and Gephi registries are not flattened into this
/// concrete [`Palette`] registry.
pub fn palettes_by_kind(kind: PaletteKind) -> impl Iterator<Item = &'static Palette> {
    palettes()
        .iter()
        .filter(move |palette| palette.kind() == kind)
}

/// Returns the number of stored source colors across the core registry.
#[must_use]
pub const fn total_color_count() -> usize {
    generated::palettes::COLOR_COUNT
}

fn key_matches(canonical: &str, requested: &str) -> bool {
    normalize_key(canonical) == normalize_key(requested)
}

fn normalize_key(input: &str) -> String {
    input
        .trim()
        .chars()
        .map(|ch| match ch {
            '_' | '-' => ' ',
            ch if ch.is_whitespace() => ' ',
            ch => ch.to_ascii_lowercase(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{palette, palette_by_spec, palettes, total_color_count, Error, Rgb};

    const CINNABAR: u32 = 0x00E6_4B35;

    #[test]
    fn parses_hex_rgb() {
        let color = Rgb::parse_hex("#E64B35").unwrap();
        assert_eq!(color.r(), 0xE6);
        assert_eq!(color.g(), 0x4B);
        assert_eq!(color.b(), 0x35);
        assert_eq!(color.to_u32(), CINNABAR);
    }

    #[test]
    fn formats_hex_rgb() {
        assert_eq!(Rgb::from_hex(CINNABAR).to_hex_string(), "#E64B35");
    }

    #[test]
    fn applies_alpha() {
        let color = Rgb::from_hex(CINNABAR).with_alpha(0.5).unwrap();
        assert_eq!(color.to_hex_string(), "#E64B3580");

        assert!(matches!(
            Rgb::from_hex(CINNABAR).with_alpha(-0.1),
            Err(Error::InvalidAlpha { .. })
        ));
        assert!(matches!(
            Rgb::from_hex(CINNABAR).with_alpha(1.1),
            Err(Error::InvalidAlpha { .. })
        ));
        assert!(matches!(
            Rgb::from_hex(CINNABAR).with_alpha(f32::NAN),
            Err(Error::InvalidAlpha { .. })
        ));
    }

    #[test]
    fn looks_up_npg_palette() {
        let npg = palette("npg", "nrc").unwrap();
        assert_eq!(npg.family(), "npg");
        assert_eq!(npg.variant(), "nrc");

        let colors = npg.take_hex(3).unwrap();
        assert_eq!(colors, ["#E64B35", "#4DBBD5", "#00A087"]);
    }

    #[test]
    fn looks_up_palette_spec_with_hyphen() {
        let palette = palette_by_spec("material:blue-grey").unwrap();
        assert_eq!(palette.family(), "material");
        assert_eq!(palette.variant(), "blue-grey");
    }

    #[test]
    fn normalizes_underscores_and_hyphens() {
        let palette = palette_by_spec("cosmic:signature-substitutions").unwrap();
        assert_eq!(palette.family(), "cosmic");
        assert_eq!(palette.variant(), "signature_substitutions");
    }

    #[test]
    fn generated_counts_match_upstream_subset() {
        assert_eq!(palettes().len(), 86);
        assert_eq!(total_color_count(), 946);
    }

    #[test]
    fn take_respects_palette_length() {
        let npg = palette("npg", "nrc").unwrap();

        assert!(npg.take(0).unwrap().is_empty());
        assert_eq!(npg.take(npg.len()).unwrap(), npg.colors());
        assert!(matches!(
            npg.take(npg.len() + 1),
            Err(Error::TooManyColorsRequested { .. })
        ));
    }

    #[test]
    fn reports_lookup_errors() {
        assert!(matches!(
            palette("missing", "nrc"),
            Err(Error::UnknownFamily { .. })
        ));
        assert!(matches!(
            palette("npg", "missing"),
            Err(Error::UnknownVariant { .. })
        ));
        assert!(matches!(
            palette_by_spec("npg"),
            Err(Error::InvalidPaletteSpec { .. })
        ));
    }
}
