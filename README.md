# ggsci-rs

Rust workspace for scientific and sci-fi color palettes from
[ggsci](https://github.com/nanxstats/ggsci).

The `ggsci` 0.4.0 release provides a core registry of 33 discrete palette
variants and 53 continuous variants from the `gsea`, `bs5`, `material`, and
`tw3` families, all 551 fixed discrete iTerm palettes, and all 17 Gephi
generative discrete palettes. Continuous colors reproduce ggsci for R's CIE
Lab/FMM spline interpolation. The Gephi engine is a pure Rust port of the
canonical algorithm in `ggsci/R/discrete-gephi.R`.

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

## Gephi palettes

Gephi palettes generate visually distinct category colors for a discrete
scale. Every `GephiPalette` reports `PaletteKind::Discrete`; generation is its
mechanism, not a third palette kind.

```rust
use ggsci::gephi_palette;

let gephi = gephi_palette("fancy-light")?;

let colors = gephi.generate_with_seed(20, 42)?;

assert_eq!(colors.len(), 20);

# Ok::<(), ggsci::Error>(())
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
