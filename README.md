# ggsci-rs

[![CI tests](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/ggsci)](https://docs.rs/ggsci)
[![crates.io](https://img.shields.io/crates/v/ggsci.svg)](https://crates.io/crates/ggsci)

Rust workspace for scientific and sci-fi color palettes from
[ggsci](https://github.com/nanxstats/ggsci).

The `ggsci` crate provides a core registry of 33 discrete palette variants
and 53 continuous variants, all 551 fixed discrete iTerm palettes,
and all 17 Gephi generative discrete palettes.

The `ggsci-ratatui` crate converts that output to ratatui colors and styles
in truecolor or ANSI-256 mode.

The `ggsci-ggsql` crate converts palettes to typed ggsql output ranges or
explicit-array SQL scale clauses.

## Core palettes

The core registry contains the stored discrete palettes and the continuous
palettes represented by canonical interpolation anchors.

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

`PaletteKind::{Discrete, Continuous}` describes scale semantics. Whether data
is stored or generated is an orthogonal implementation detail. Accordingly,
`Palette::colors()` contains category colors for discrete palettes and the
canonical interpolation anchors for continuous palettes.

## iTerm palettes

iTerm themes use their own API because each theme has normal and bright
variants plus the fixed terminal-channel order Blue, Yellow, Red, Cyan, Green,
Magenta:

```rust
use ggsci::{iterm_palette, ItermVariant};

fn main() -> Result<(), ggsci::Error> {
    let rose_pine = iterm_palette("Rose Pine")?;
    let colors = rose_pine.take_hex(ItermVariant::Normal, 6)?;

    assert_eq!(colors.len(), 6);
    Ok(())
}
```

Theme lookup is case-insensitive and treats `_`, `-`, and whitespace as
interchangeable separators while preserving other punctuation. Variant parsing
is also case-insensitive. Every `ItermPalette` reports
`PaletteKind::Discrete`; normal/bright is an `ItermVariant`, not a palette
kind. The iTerm records are not flattened into `palettes()` or
`palettes_by_kind()` because the core `Palette` model cannot preserve their two
variants and terminal-channel ordering. The complete iTerm data is included in
the normal crate package without feature flags.

## Gephi palettes

Gephi palettes generate visually distinct category colors for a discrete
scale. Every `GephiPalette` reports `PaletteKind::Discrete`; generation is its
mechanism, not a third palette kind.

```rust
use ggsci::gephi_palette;

fn main() -> Result<(), ggsci::Error> {
    let gephi = gephi_palette("fancy-light")?;
    let colors = gephi.generate_with_seed(20, 42)?;

    assert_eq!(colors.len(), 20);
    Ok(())
}
```

Available canonical names are `default`, `fancy_light`, `fancy_dark`,
`shades`, `tarnish`, `pastel`, `pimp`, `intense`, `fluo`, `red_roses`,
`ochre_sand`, `yellow_lime`, `green_mint`, `ice_cube`, `blue_ocean`,
`indigo_night`, and `purple_wine`. Lookup is case-insensitive and treats `_`,
`-`, and whitespace as interchangeable separators.

Use `generate_with_seed()` for reproducible output. It uses `ChaCha8Rng` with
an explicitly defined SplitMix64 `u64`-to-32-byte seed expansion and a stable
53-bit floating-point sampling rule. Golden tests lock this crate's seeded
output, but seeds are not cross-language compatible with R or NumPy. Use
`generate()` for fresh nondeterministic output from an independent RNG seeded
from the operating system; it does not mutate an application RNG. The RGBA
methods accept finite alpha in `(0.0, 1.0]` and apply it after RGB generation.

Generation performs rejection sampling followed by filtered k-means and
farthest-first ordering. Its cost grows with the requested color count; as in
R, quality drops from 50 iterations at up to 50 colors to 25, 10, 5, and 2 at
the 50/100/200/300 boundaries. The deterministic 9,261-point candidate grid is
filtered once into a thread-safe indexed cache for all 17 generated filters;
generated palettes themselves are never cached.

Gephi has a dedicated generator registry because each result requires an
algorithm and random state. Its definitions are not duplicated in the stored
core `palettes()` or `palettes_by_kind()` registry even though both Gephi and
stored categorical palettes have discrete scale semantics.

## Workspace

The workspace has three publishable crates, all released at the same version:

- `ggsci` contains palette data, interpolation, and generation.
- `ggsci-ratatui` depends on `ratatui-core` and provides color conversion,
  alpha compositing, palette adapters, and foreground/background style helpers.
- `ggsci-ggsql` depends on ggsql without its default features and provides
  typed `OutputRange` conversion plus validated textual scale clauses.

The `ggsci` crate requires Rust 1.85, the first stable release whose Cargo can
load the workspace's Rust 2024 manifests.
`ggsci-ratatui` requires Rust 1.88 to match `ratatui-core` 0.1.2.
`ggsci-ggsql` requires Rust 1.86 to match ggsql 0.4.1.
CI checks each public crate at its own MSRV in addition to checking the
workspace on stable, beta, and nightly.

For example:

```rust
use ggsci_ratatui::{colors, foreground_styles, ColorMode};

fn main() -> Result<(), ggsci::Error> {
    let colors = colors("observable:observable10", 5, ColorMode::TrueColor)?;
    let styles = foreground_styles(&colors);

    assert_eq!(colors.len(), 5);
    assert_eq!(styles.len(), 5);
    Ok(())
}
```

See the [ggsci-ratatui README](crates/ggsci-ratatui/README.md) for ANSI-256,
continuous, iTerm, Gephi, and RGBA conversion details.

ggsql does not currently expose a third-party palette registry, so
`ggsci-ggsql` emits explicit color arrays instead of globally registering
ggsci names. It preserves source `PaletteKind` separately from its ggsql
`ScaleKind`, supports typed `OutputRange::Array` conversion, and provides
discrete, continuous, fixed iTerm, and seeded Gephi SQL helpers. See the
[ggsci-ggsql README](crates/ggsci-ggsql/README.md) for examples and the ggsql
parser build note.

## Interactive palette gallery

Run the complete interactive Ratatui gallery from the workspace root:

```bash
cargo run -p ggsci-ratatui --example palette-gallery
```

The responsive gallery browses every core, iTerm, and Gephi definition in
TrueColor or ANSI-256 mode. It shows a realistic terminal application with tabs,
palette cards, scrolling, resize handling, and keyboard controls.

![Interactive palette gallery cycling through the discrete, continuous, iTerm, and Gephi tabs](https://github.com/user-attachments/assets/0451593e-cf48-44e7-907e-8a798a78730c)

## Maintenance

Core, iTerm, and Gephi metadata plus R-generated continuous golden fixtures are
checked in. To refresh all of them from the vendored upstream source during
development:

```bash
cargo xtask update-palettes
```

The command regenerates the core registry, continuous fixtures, dedicated
iTerm registry, and Gephi filter registry, then formats the workspace. It
requires R. Building, testing, documenting, and using the published crate do
not require R, Python, NumPy, or the vendor sources.

## Related work

- [ggsci for R](https://github.com/nanxstats/ggsci)
- [ggsci for Python](https://github.com/nanxstats/py-ggsci)
