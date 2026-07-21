//! Use ggsci color palettes in ggsql visualizations.
//!
//! ggsql does not currently expose a third-party palette provider or registry
//! API. This adapter therefore converts ggsci palettes to explicit color arrays,
//! either as typed [`ggsql::plot::scale::OutputRange`] values or as textual
//! `SCALE` clauses.

use std::fmt;

use ggsql::plot::{scale::OutputRange, ArrayElement};

/// An error produced while resolving a palette or building a ggsql clause.
#[derive(Debug)]
pub enum Error {
    /// An error reported by the core ggsci crate.
    Ggsci(ggsci::Error),
    /// An aesthetic was not a valid unquoted SQL identifier.
    InvalidAesthetic {
        /// The invalid aesthetic supplied by the caller.
        aesthetic: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ggsci(error) => error.fmt(f),
            Self::InvalidAesthetic { aesthetic } => write!(
                f,
                "invalid ggsql aesthetic `{aesthetic}`; expected an ASCII identifier matching [A-Za-z_][A-Za-z0-9_]*"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ggsci(error) => Some(error),
            Self::InvalidAesthetic { .. } => None,
        }
    }
}

impl From<ggsci::Error> for Error {
    fn from(error: ggsci::Error) -> Self {
        Self::Ggsci(error)
    }
}

/// The kind of ggsql `SCALE` clause to emit.
///
/// This is distinct from [`ggsci::PaletteKind`], which describes the semantic
/// domain of the source palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleKind {
    /// A categorical scale.
    Discrete,
    /// A continuous scale.
    Continuous,
    /// A scale that groups a continuous domain into bins.
    Binned,
    /// An ordered categorical scale.
    Ordinal,
}

impl ScaleKind {
    /// Returns the uppercase keyword used in a ggsql `SCALE` clause.
    #[must_use]
    pub const fn as_sql_keyword(self) -> &'static str {
        match self {
            Self::Discrete => "DISCRETE",
            Self::Continuous => "CONTINUOUS",
            Self::Binned => "BINNED",
            Self::Ordinal => "ORDINAL",
        }
    }
}

impl From<ggsci::PaletteKind> for ScaleKind {
    fn from(kind: ggsci::PaletteKind) -> Self {
        match kind {
            ggsci::PaletteKind::Discrete => Self::Discrete,
            ggsci::PaletteKind::Continuous => Self::Continuous,
        }
    }
}

/// A resolved ggsci palette ready for ggsql conversion.
///
/// The wrapper retains the source palette semantics until a caller selects a
/// ggsql scale kind or converts the colors to an untyped output range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GgsqlPalette {
    colors: Vec<ggsci::Rgb>,
    palette_kind: ggsci::PaletteKind,
}

impl GgsqlPalette {
    /// Creates a wrapper from already resolved colors and their source
    /// semantics.
    #[must_use]
    pub fn from_colors(colors: Vec<ggsci::Rgb>, palette_kind: ggsci::PaletteKind) -> Self {
        Self {
            colors,
            palette_kind,
        }
    }

    /// Resolves and samples a core palette using its discrete or continuous
    /// semantics.
    ///
    /// # Errors
    ///
    /// Returns an error when the specification is invalid, the palette is not
    /// known, or a discrete palette does not have `n` colors.
    pub fn from_spec(spec: &str, n: usize) -> Result<Self, Error> {
        sample_spec(spec, n).map_err(Error::from)
    }

    /// Resolves and interpolates a continuous core palette with explicit
    /// options.
    ///
    /// # Errors
    ///
    /// Returns an error when the palette cannot be found or is not continuous.
    pub fn from_continuous(
        spec: &str,
        n: usize,
        options: ggsci::ContinuousOptions,
    ) -> Result<Self, Error> {
        let palette = ggsci::palette_by_spec(spec)?;
        let colors = palette.interpolate_with(n, options)?;

        Ok(Self {
            colors,
            palette_kind: ggsci::PaletteKind::Continuous,
        })
    }

    /// Resolves colors from a fixed discrete iTerm theme variant.
    ///
    /// # Errors
    ///
    /// Returns an error when the theme is unknown or `n` exceeds the six
    /// colors in a variant.
    pub fn from_iterm(
        palette: &str,
        variant: ggsci::ItermVariant,
        n: usize,
    ) -> Result<Self, Error> {
        let colors = ggsci::iterm_palette(palette)?.take(variant, n)?;
        Ok(Self {
            colors,
            palette_kind: ggsci::PaletteKind::Discrete,
        })
    }

    /// Generates colors from a generative discrete Gephi palette.
    ///
    /// Use [`Self::from_gephi_with_seed`] when reproducibility matters.
    ///
    /// # Errors
    ///
    /// Returns an error when the palette is unknown or generation fails.
    pub fn from_gephi(palette: &str, n: usize) -> Result<Self, Error> {
        let colors = ggsci::gephi_palette(palette)?.generate(n)?;
        Ok(Self {
            colors,
            palette_kind: ggsci::PaletteKind::Discrete,
        })
    }

    /// Generates reproducible colors from a generative discrete Gephi palette.
    ///
    /// # Errors
    ///
    /// Returns an error when the palette is unknown or generation fails.
    pub fn from_gephi_with_seed(palette: &str, n: usize, seed: u64) -> Result<Self, Error> {
        let colors = ggsci::gephi_palette(palette)?.generate_with_seed(n, seed)?;
        Ok(Self {
            colors,
            palette_kind: ggsci::PaletteKind::Discrete,
        })
    }

    /// Returns the resolved colors.
    #[must_use]
    pub fn colors(&self) -> &[ggsci::Rgb] {
        &self.colors
    }

    /// Consumes the wrapper and returns its resolved colors.
    #[must_use]
    pub fn into_colors(self) -> Vec<ggsci::Rgb> {
        self.colors
    }

    /// Returns the source palette's semantic kind.
    #[must_use]
    pub const fn palette_kind(&self) -> ggsci::PaletteKind {
        self.palette_kind
    }

    /// Formats the colors as a ggsql array of uppercase `#RRGGBB` strings.
    #[must_use]
    pub fn to_sql_array(&self) -> String {
        format_sql_array(&self.colors)
    }

    /// Converts the colors to ggsql's typed explicit output range.
    ///
    /// The resulting value intentionally does not retain scale-kind metadata.
    #[must_use]
    pub fn to_output_range(&self) -> OutputRange {
        self.into()
    }

    /// Emits an explicit-array ggsql scale clause of the selected kind.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAesthetic`] unless `aesthetic`, after trimming,
    /// is an ASCII identifier matching `[A-Za-z_][A-Za-z0-9_]*`.
    pub fn to_scale_clause(&self, kind: ScaleKind, aesthetic: &str) -> Result<String, Error> {
        let aesthetic = validate_aesthetic(aesthetic)?;
        Ok(format_scale_clause(kind, aesthetic, &self.colors))
    }

    /// Emits an explicit-array ggsql scale clause whose kind defaults from the
    /// source palette semantics.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAesthetic`] unless `aesthetic`, after trimming,
    /// is a valid ASCII identifier.
    pub fn to_default_scale_clause(&self, aesthetic: &str) -> Result<String, Error> {
        self.to_scale_clause(ScaleKind::from(self.palette_kind()), aesthetic)
    }
}

impl From<GgsqlPalette> for OutputRange {
    fn from(palette: GgsqlPalette) -> Self {
        Self::Array(
            palette
                .colors
                .into_iter()
                .map(|color| ArrayElement::String(color.to_hex_string()))
                .collect(),
        )
    }
}

impl From<&GgsqlPalette> for OutputRange {
    fn from(palette: &GgsqlPalette) -> Self {
        Self::Array(
            palette
                .colors
                .iter()
                .map(|color| ArrayElement::String(color.to_hex_string()))
                .collect(),
        )
    }
}

/// Emits a ggsql color array for the first `n` colors in a discrete core
/// palette.
///
/// # Errors
///
/// Returns any palette lookup or length error reported by ggsci. In particular,
/// a continuous palette returns [`ggsci::Error::NotDiscretePalette`].
pub fn color_array(spec: &str, n: usize) -> Result<String, ggsci::Error> {
    resolve_discrete_hex(spec, n).map(|colors| format_sql_hex_array(&colors))
}

/// Emits a ggsql discrete scale clause using a discrete core palette.
///
/// This compatibility helper preserves its original ggsci-only error type. Use
/// [`GgsqlPalette::to_scale_clause`] when aesthetic validation is required.
///
/// # Errors
///
/// Returns any palette lookup or length error reported by ggsci. In particular,
/// a continuous palette returns [`ggsci::Error::NotDiscretePalette`].
pub fn scale_discrete(aesthetic: &str, spec: &str, n: usize) -> Result<String, ggsci::Error> {
    let colors = resolve_discrete_hex(spec, n)?;
    Ok(format_scale_clause_with_array(
        ScaleKind::Discrete,
        aesthetic.trim(),
        &format_sql_hex_array(&colors),
    ))
}

/// Resolves either kind of core palette as a typed ggsql output range.
///
/// # Errors
///
/// Returns any palette lookup, sampling, or length error reported by ggsci.
pub fn output_range(spec: &str, n: usize) -> Result<OutputRange, ggsci::Error> {
    sample_spec(spec, n).map(OutputRange::from)
}

/// Emits a continuous ggsql scale clause with explicit interpolation options.
///
/// # Errors
///
/// Returns an adapter error when palette resolution fails or the aesthetic is
/// invalid.
pub fn scale_continuous(
    aesthetic: &str,
    spec: &str,
    n: usize,
    options: ggsci::ContinuousOptions,
) -> Result<String, Error> {
    GgsqlPalette::from_continuous(spec, n, options)?
        .to_scale_clause(ScaleKind::Continuous, aesthetic)
}

/// Emits a discrete ggsql scale clause from a fixed iTerm theme variant.
///
/// # Errors
///
/// Returns an adapter error when palette resolution fails or the aesthetic is
/// invalid.
pub fn scale_iterm_discrete(
    aesthetic: &str,
    palette: &str,
    variant: ggsci::ItermVariant,
    n: usize,
) -> Result<String, Error> {
    GgsqlPalette::from_iterm(palette, variant, n)?.to_scale_clause(ScaleKind::Discrete, aesthetic)
}

/// Emits a reproducible discrete ggsql scale clause from a Gephi generator.
///
/// # Errors
///
/// Returns an adapter error when palette generation fails or the aesthetic is
/// invalid.
pub fn scale_gephi_discrete_with_seed(
    aesthetic: &str,
    palette: &str,
    n: usize,
    seed: u64,
) -> Result<String, Error> {
    GgsqlPalette::from_gephi_with_seed(palette, n, seed)?
        .to_scale_clause(ScaleKind::Discrete, aesthetic)
}

fn sample_spec(spec: &str, n: usize) -> Result<GgsqlPalette, ggsci::Error> {
    let palette = ggsci::palette_by_spec(spec)?;
    let colors = palette.sample(n)?;

    Ok(GgsqlPalette {
        colors,
        palette_kind: palette.kind(),
    })
}

fn resolve_discrete_hex(spec: &str, n: usize) -> Result<Vec<String>, ggsci::Error> {
    ggsci::palette_by_spec(spec)?.take_hex(n)
}

fn format_sql_array(colors: &[ggsci::Rgb]) -> String {
    let colors = colors
        .iter()
        .copied()
        .map(ggsci::Rgb::to_hex_string)
        .collect::<Vec<_>>();
    format_sql_hex_array(&colors)
}

fn format_sql_hex_array(colors: &[String]) -> String {
    let quoted = colors
        .iter()
        .map(|color| format!("'{color}'"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{quoted}]")
}

fn format_scale_clause(kind: ScaleKind, aesthetic: &str, colors: &[ggsci::Rgb]) -> String {
    format_scale_clause_with_array(kind, aesthetic, &format_sql_array(colors))
}

fn format_scale_clause_with_array(kind: ScaleKind, aesthetic: &str, array: &str) -> String {
    format!("SCALE {} {aesthetic} TO {array}", kind.as_sql_keyword())
}

fn validate_aesthetic(aesthetic: &str) -> Result<&str, Error> {
    let trimmed = aesthetic.trim();
    let mut bytes = trimmed.bytes();
    let valid = bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');

    if valid {
        Ok(trimmed)
    } else {
        Err(Error::InvalidAesthetic {
            aesthetic: aesthetic.to_owned(),
        })
    }
}
