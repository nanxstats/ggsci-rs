# ggsci-ggsql

[![CI tests](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/ggsci-ggsql)](https://docs.rs/ggsci-ggsql)
[![crates.io](https://img.shields.io/crates/v/ggsci-ggsql.svg)](https://crates.io/crates/ggsci-ggsql)

Use [ggsci](https://crates.io/crates/ggsci) color palettes in
[ggsql](https://ggsql.org) visualizations.

This release supports ggsql 0.4.1 and requires Rust 1.86, matching ggsql's MSRV.
The adapter disables ggsql's default features: palette conversion and
parser integration do not enable DuckDB, SQLite, ADBC, ODBC, Parquet,
Vega-Lite, spatial, or built-in data features.

## Explicit array integration

ggsql 0.4.1 resolves named palette identifiers in its own scale code and does
not expose a stable third-party palette provider or registry API. Consequently,
this adapter does not register ggsci names globally. It resolves a palette with
ggsci and passes ggsql an explicit array of uppercase `#RRGGBB` strings.

`GgsqlPalette` is the central wrapper. It keeps the resolved colors together
with their source `ggsci::PaletteKind` until the caller selects a textual scale
kind or converts to ggsql's typed `OutputRange`:

```rust
use ggsci_ggsql::GgsqlPalette;

fn main() -> Result<(), ggsci_ggsql::Error> {
    let palette = GgsqlPalette::from_spec("observable:observable10", 3)?;

    assert_eq!(
        palette.to_sql_array(),
        "['#4269D0', '#EFB118', '#FF725C']"
    );
    Ok(())
}
```

## Typed output ranges

Use `to_output_range()`, `output_range()`, or either `From<GgsqlPalette>`
implementation when building a ggsql plot programmatically:

```rust
use ggsci_ggsql::GgsqlPalette;
use ggsql::plot::{scale::OutputRange, ArrayElement};

fn main() -> Result<(), ggsci_ggsql::Error> {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3)?;
    let output: OutputRange = (&palette).into();

    let OutputRange::Array(colors) = output else {
        unreachable!("ggsci always emits an explicit array");
    };
    assert!(colors
        .iter()
        .all(|color| matches!(color, ArrayElement::String(_))));
    Ok(())
}
```

`OutputRange::Array` contains only the color values and intentionally loses
scale-kind metadata. Keep `GgsqlPalette` until the scale kind has been selected
when that distinction matters.

## Textual scale clauses

`to_default_scale_clause()` derives the ggsql keyword from the source palette:

```rust
use ggsci_ggsql::GgsqlPalette;

fn main() -> Result<(), ggsci_ggsql::Error> {
    let categories = GgsqlPalette::from_spec("npg:nrc", 3)?;
    let gradient = GgsqlPalette::from_spec("material:blue-grey", 256)?;

    assert!(categories
        .to_default_scale_clause("color")?
        .starts_with("SCALE DISCRETE color TO"));
    assert!(gradient
        .to_default_scale_clause("fill")?
        .starts_with("SCALE CONTINUOUS fill TO"));
    Ok(())
}
```

`ggsci::PaletteKind` describes the source palette's semantic domain.
`ggsci_ggsql::ScaleKind` describes the ggsql `SCALE` clause being emitted.
Their default mapping is discrete to `DISCRETE` and continuous to
`CONTINUOUS`, but the enums remain distinct so callers can make valid explicit
choices. For example, a sampled continuous palette can drive a binned scale,
and ordered category colors can drive an ordinal scale:

```rust
use ggsci_ggsql::{GgsqlPalette, ScaleKind};

fn main() -> Result<(), ggsci_ggsql::Error> {
    let gradient = GgsqlPalette::from_spec("material:blue-grey", 12)?;
    let clause = gradient.to_scale_clause(ScaleKind::Binned, "fill")?;

    assert!(clause.starts_with("SCALE BINNED fill TO"));
    Ok(())
}
```

Textual helpers validate aesthetics as trimmed ASCII identifiers matching
`[A-Za-z_][A-Za-z0-9_]*`. The original `color_array()` and
`scale_discrete()` compatibility helpers remain discrete-only and preserve
their `ggsci::Error` return type; continuous palettes return
`ggsci::Error::NotDiscretePalette`.

## Continuous palettes

`from_spec()` uses kind-aware `Palette::sample()`, so continuous records are
interpolated rather than truncated to their stored anchors. Use
`from_continuous()` when reverse behavior should be explicit:

```rust
use ggsci::ContinuousOptions;
use ggsci_ggsql::GgsqlPalette;

fn main() -> Result<(), ggsci_ggsql::Error> {
    let palette = GgsqlPalette::from_continuous(
        "material:blue-grey",
        512,
        ContinuousOptions::new().with_reverse(true),
    )?;

    assert_eq!(palette.colors().len(), 512);
    Ok(())
}
```

The `scale_continuous()` convenience function performs the same explicit
continuous resolution and emits a `SCALE CONTINUOUS` clause.

## iTerm palettes

iTerm themes are fixed discrete palettes with normal and bright variants:

```rust
use ggsci::ItermVariant;
use ggsci_ggsql::GgsqlPalette;

fn main() -> Result<(), ggsci_ggsql::Error> {
    let palette = GgsqlPalette::from_iterm("Rose Pine", ItermVariant::Normal, 6)?;
    let clause = palette.to_default_scale_clause("stroke")?;

    assert!(clause.starts_with("SCALE DISCRETE stroke TO"));
    Ok(())
}
```

## Gephi palettes

Gephi palettes generate discrete category colors. Prefer seeded generation for
reproducible SQL and tests:

```rust
use ggsci_ggsql::GgsqlPalette;

fn main() -> Result<(), ggsci_ggsql::Error> {
    let first = GgsqlPalette::from_gephi_with_seed("fancy-light", 20, 42)?;
    let second = GgsqlPalette::from_gephi_with_seed("fancy-light", 20, 42)?;

    assert_eq!(first, second);
    Ok(())
}
```

`scale_iterm_discrete()` and `scale_gephi_discrete_with_seed()` provide direct
textual equivalents. Unseeded `from_gephi()` is available when fresh random
output is desired.

## Parser build note

The published `tree-sitter-ggsql` 0.4.1 build script regenerates its parser by
default. Set `GGSQL_SKIP_GENERATE=1` to use the generated `src/parser.c` that is
already included in its crate package when `tree-sitter-cli` is not installed.
The ggsci-rs workspace and CI use this upstream supported mode automatically;
downstream workspaces without `tree-sitter-cli` should set the same variable.

## Future provider API

There is currently no ggsql plugin, palette registry, or provider trait for
third-party named palettes. If ggsql defines an official provider API in a
future release, this adapter intends to adopt it while retaining explicit
array conversion for portable, self-contained queries.
