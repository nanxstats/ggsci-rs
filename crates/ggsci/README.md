# ggsci

[![CI tests](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/ggsci)](https://docs.rs/ggsci)
[![crates.io](https://img.shields.io/crates/v/ggsci.svg)](https://crates.io/crates/ggsci)

Scientific and sci-fi color palettes from the R package
[ggsci](https://github.com/nanxstats/ggsci), packaged as Rust data and native
palette generation algorithms.

This crate includes all 86 core palettes, all 551 iTerm themes, and all 17
Gephi generators from upstream. `PaletteKind::Discrete` maps categories to
individual colors, while `PaletteKind::Continuous` maps a continuous domain
through an interpolated gradient. These are scale semantics: whether palette
data is stored or generated is an orthogonal implementation detail.

## Core palettes

Use `take()` for discrete category colors:

```rust
use ggsci::palette_by_spec;

fn main() -> Result<(), ggsci::Error> {
    let palette = palette_by_spec("observable:observable10")?;
    let colors = palette.take_hex(3)?;

    assert_eq!(colors, ["#4269D0", "#EFB118", "#FF725C"]);
    Ok(())
}
```

The `gsea`, `bs5`, `material`, and `tw3` families are continuous. Their
arbitrary-length output reproduces ggsci for R's
`colorRamp(..., space = "Lab", interpolate = "spline")`, including its FMM
cubic spline, gamut handling, rounding, and endpoint behavior.

Use `interpolate()` for continuous gradient samples:

```rust
use ggsci::{palette_by_spec, ContinuousOptions};

fn main() -> Result<(), ggsci::Error> {
    let palette = palette_by_spec("material:blue-grey")?;
    let colors = palette.interpolate(256)?;
    let reversed = palette.interpolate_with(
        256,
        ContinuousOptions::new().with_reverse(true),
    )?;

    assert_eq!(reversed, colors.into_iter().rev().collect::<Vec<_>>());
    Ok(())
}
```

Use `sample()` for kind-aware dispatch when accepting either palette kind.

`Palette::colors()` returns canonical source colors: category colors for a
discrete palette and interpolation anchors for a continuous palette.
Its length therefore does not limit how many colors a continuous palette can
produce. Reverse is applied after interpolation, matching R.
The continuous RGBA methods accept finite alpha values in `(0.0, 1.0]`.

Lookup is case-insensitive and accepts `_`, `-`, and spaces interchangeably.

## iTerm palettes

iTerm is a fixed discrete palette family exposed through a dedicated typed
registry:

```rust
use ggsci::{iterm_palette, ItermVariant};

fn main() -> Result<(), ggsci::Error> {
    let rose_pine = iterm_palette("Rose Pine")?;
    let colors = rose_pine.take_hex(ItermVariant::Normal, 6)?;

    assert_eq!(colors.len(), 6);
    Ok(())
}
```

Use `iterm_palettes()` to traverse the registry,
`iterm_palette_names()` to list canonical names, and `iterm_palette()` for
case-insensitive theme lookup. Theme lookup treats `_`, `-`, and whitespace as
interchangeable separators while preserving punctuation such as the `+` that
distinguishes `Dracula+` from `Dracula`. `ItermVariant::parse()` accepts normal
and bright case-insensitively.

Every `ItermPalette` reports `PaletteKind::Discrete`. Normal and bright are
theme variants represented by `ItermVariant`, not palette kinds. Within each
variant, the six colors have the fixed channel ordering Blue, Yellow, Red,
Cyan, Green, Magenta, also exposed as `ITERM_CHANNELS`.

iTerm records are deliberately not flattened into the core `palettes()` or
`palettes_by_kind()` registry. Although they share discrete scale semantics,
the core `Palette` data model cannot preserve a theme's paired normal/bright
variants and fixed terminal-channel ordering. The dedicated registry keeps
that structure explicit.

## Gephi palettes

Gephi palettes are generative discrete palettes ported from the palette engine
in Gephi via the canonical implementation in `ggsci/R/discrete-gephi.R`.
Every `GephiPalette` reports `PaletteKind::Discrete`. Generative describes
how its colors are produced, while discrete describes how the result maps to
category values.

```rust
use ggsci::gephi_palette;

fn main() -> Result<(), ggsci::Error> {
    let gephi = gephi_palette("fancy-light")?;
    let colors = gephi.generate_with_seed(20, 42)?;

    assert_eq!(colors.len(), 20);
    Ok(())
}
```

Use `gephi_palettes()` to inspect the dedicated registry,
`gephi_palette_names()` to list canonical names, and `gephi_palette()` for
normalized lookup. Available names are `default`, `fancy_light`, `fancy_dark`,
`shades`, `tarnish`, `pastel`, `pimp`, `intense`, `fluo`, `red_roses`,
`ochre_sand`, `yellow_lime`, `green_mint`, `ice_cube`, `blue_ocean`,
`indigo_night`, and `purple_wine`. Lookup is case-insensitive and treats `_`,
`-`, and whitespace as interchangeable.

`generate_with_seed()` and `generate_rgba_with_seed()` are reproducible within
this crate. Seeded generation uses `ChaCha8Rng`, four SplitMix64 outputs in
little-endian order to expand a `u64` seed, and the high 53 bits of each
`next_u64()` output for uniform floating-point values. Golden tests lock this
design against accidental patch-release changes. It does not promise R or
NumPy seed compatibility. `generate()` and `generate_rgba()` seed a private RNG
from fresh operating-system randomness and do not mutate an application RNG.
RGBA alpha is applied after RGB generation and must be finite and in
`(0.0, 1.0]`.

The algorithm uses rejection sampling, filtered k-means, and farthest-first
ordering. Quality uses 50 iterations through 50 colors, then 25 through 100,
10 through 200, 5 through 300, and 2 above 300. A thread-safe indexed cache
filters the deterministic 9,261-point Lab-like sample grid once for all 17
filters. Output palettes are not cached, so generation time still grows with
the requested color count.

Gephi definitions stay out of `palettes()` and `palettes_by_kind()` because
they require an algorithm and random state instead of stored color records.
That dedicated API reflects their generation mechanism, not a different scale
kind.

## Packaging and maintenance

The minimum supported Rust version is 1.85. This is the first stable release
whose Cargo supports the workspace's Rust 2024 manifests, and it supports the
floating-point `const fn` used by continuous interpolation.

The complete core, iTerm, and Gephi metadata is included without feature flags.
The crate has no build script. Generated Rust data and R-generated exact-channel
fixtures are checked in and run as ordinary Rust integration tests.

R is only a maintainer dependency; builds do not require R, Python, NumPy,
matplotlib, jsonlite, vendor sources, or network access. The single
`cargo xtask update-palettes` command regenerates the core registry, continuous
fixtures, iTerm registry, and Gephi filter registry, then formats the workspace.

## Adapters

The separately published [`ggsci-ratatui`](https://crates.io/crates/ggsci-ratatui)
crate converts core, iTerm, and Gephi output to `ratatui_core::style::Color`.
It provides truecolor and deterministic ANSI-256 modes, explicit RGBA compositing,
and foreground or background `Style` helpers without depending on the full
ratatui application crate.

The separately published [`ggsci-ggsql`](https://crates.io/crates/ggsci-ggsql)
crate converts palettes to ggsql 0.4.1 explicit color arrays. It supports typed
`OutputRange` conversion and textual `SCALE` clauses for core, iTerm, and
Gephi palettes without enabling ggsql's database reader or output features.
ggsql does not yet expose a third-party palette registry, so the adapter does
not register named ggsci palettes globally.
