# ggsci-rs

Rust workspace for scientific and sci-fi color palettes from
[ggsci](https://github.com/nanxstats/ggsci).

The `ggsci` 0.3.0 release provides a core registry of 33 discrete palette
variants and 53 continuous variants from the `gsea`, `bs5`, `material`, and
`tw3` families, plus all 551 fixed discrete iTerm palettes through a dedicated
typed registry. Continuous colors reproduce ggsci for R's CIE Lab/FMM spline
interpolation. Gephi palettes, ratatui conversions, and native ggsql extension
APIs remain future work.

## Usage

```rust
let palette = ggsci::palette("npg", "nrc")?;
let colors = palette.take_hex(3)?;
# Ok::<(), ggsci::Error>(())
```

Use `take()` for discrete category colors, `interpolate()` for continuous
gradient samples, or `sample()` for kind-aware dispatch:

```rust
use ggsci::{palette_by_spec, ContinuousOptions};

let palette = palette_by_spec("material:blue-grey")?;
let colors = palette.interpolate(256)?;
let sampled = palette.sample(256)?;
let reversed = palette.interpolate_with(
    256,
    ContinuousOptions::new().with_reverse(true),
)?;

assert_eq!(colors, sampled);
assert_eq!(reversed.len(), 256);
# Ok::<(), ggsci::Error>(())
```

`PaletteKind::{Discrete, Continuous}` describes scale semantics. Whether data
is stored or generated is an orthogonal implementation detail. Accordingly,
`Palette::colors()` contains category colors for discrete palettes and the
canonical interpolation anchors for continuous palettes.

iTerm themes use their own API because each theme has normal and bright
variants plus the fixed terminal-channel order Blue, Yellow, Red, Cyan, Green,
Magenta:

```rust
use ggsci::{iterm_palette, ItermVariant};

let rose_pine = iterm_palette("Rose Pine")?;
let colors = rose_pine.take_hex(ItermVariant::Normal, 6)?;

assert_eq!(colors.len(), 6);
# Ok::<(), ggsci::Error>(())
```

Theme lookup is case-insensitive and treats `_`, `-`, and whitespace as
interchangeable separators while preserving other punctuation. Variant parsing
is also case-insensitive. Every `ItermPalette` reports
`PaletteKind::Discrete`; normal/bright is an `ItermVariant`, not a palette
kind. The iTerm records are not flattened into `palettes()` or
`palettes_by_kind()` because the core `Palette` model cannot preserve their two
variants and terminal-channel ordering. The complete iTerm data is included in
the normal crate package without feature flags.

`crates/ggsci` is the only publishable crate in the workspace. The ratatui and
ggsql crates are private scaffolds for later integrations.

## Maintenance

Core and iTerm palette data and R-generated continuous golden fixtures are
checked in. To refresh all of them from the vendored upstream source during
development:

```bash
cargo xtask update-palettes
```

The command regenerates the core registry, continuous fixtures, and dedicated
iTerm registry, then formats the workspace. It requires R. Building, testing,
documenting, and using the published crate do not require R or the vendor
sources.

## Related work

- [ggsci for R](https://github.com/nanxstats/ggsci)
- [ggsci for Python](https://github.com/nanxstats/py-ggsci)
