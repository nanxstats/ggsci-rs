# ggsci

Scientific and sci-fi color palettes from the R package
[ggsci](https://github.com/nanxstats/ggsci), packaged as static Rust data.

This crate includes all non-Gephi core palettes and all 551 iTerm themes
generated from upstream. `PaletteKind::Discrete` maps categories to individual
colors, while `PaletteKind::Continuous` maps a continuous domain through an
interpolated gradient. These are scale semantics: whether palette data is
stored or generated is an orthogonal implementation detail.

The `gsea`, `bs5`, `material`, and `tw3` families are continuous. Their
arbitrary-length output reproduces ggsci for R's
`colorRamp(..., space = "Lab", interpolate = "spline")`, including its FMM
cubic spline, gamut handling, rounding, and endpoint behavior.

Use `take()` for discrete category colors:

```rust
let palette = ggsci::palette("npg", "nrc")?;
let colors = palette.take_hex(3)?;

assert_eq!(colors, ["#E64B35", "#4DBBD5", "#00A087"]);
# Ok::<(), ggsci::Error>(())
```

Use `interpolate()` for a continuous gradient, or `sample()` when accepting
either kind:

```rust
use ggsci::{palette_by_spec, ContinuousOptions};

let palette = palette_by_spec("material:blue-grey")?;

let colors = palette.interpolate(256)?;
let sampled = palette.sample(256)?;
let reversed = palette.interpolate_with(
    256,
    ContinuousOptions::new().with_reverse(true),
)?;
let translucent = palette.interpolate_rgba(256, 0.6)?;

assert_eq!(colors, sampled);
assert_eq!(colors.len(), 256);
assert_eq!(reversed.len(), 256);
assert_eq!(translucent.len(), 256);
# Ok::<(), ggsci::Error>(())
```

`Palette::colors()` returns canonical source colors: category colors for a
discrete palette and interpolation anchors for a continuous palette. Its
length therefore does not limit how many colors a continuous palette can
produce. Reverse is applied after interpolation, matching R. The continuous
RGBA methods accept finite alpha values in `(0.0, 1.0]`.

Lookup is case-insensitive and accepts `_`, `-`, and spaces interchangeably:

```rust
let palette = ggsci::palette_by_spec("material:blue-grey")?;
assert_eq!(palette.family(), "material");
assert_eq!(palette.variant(), "blue-grey");
# Ok::<(), ggsci::Error>(())
```

## iTerm palettes

iTerm is a fixed discrete palette family exposed through a dedicated typed
registry:

```rust
use ggsci::{iterm_palette, ItermVariant};

let rose_pine = iterm_palette("Rose Pine")?;
let colors = rose_pine.take_hex(ItermVariant::Normal, 6)?;

assert_eq!(colors.len(), 6);
# Ok::<(), ggsci::Error>(())
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

The complete core and iTerm data is included without feature flags. The crate
has no runtime dependencies and no build script. Generated Rust data and
R-generated exact-channel fixtures are checked in and run as ordinary Rust
integration tests. R is only a maintainer dependency. The single
`cargo xtask update-palettes` command regenerates the core registry, continuous
fixtures, and iTerm registry, then formats the workspace.
