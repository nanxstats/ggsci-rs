# ggsci-rs

Rust workspace for scientific and sci-fi color palettes from
[ggsci](https://github.com/nanxstats/ggsci).

The `ggsci` 0.2.0 release provides 33 discrete palette variants and 53
continuous variants from the `gsea`, `bs5`, `material`, and `tw3` families.
Continuous colors reproduce ggsci for R's CIE Lab/FMM spline interpolation.
Gephi palettes, iTerm palettes, ratatui conversions, and native ggsql extension
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

`crates/ggsci` is the only publishable crate in the workspace. The ratatui and
ggsql crates are private scaffolds for later integrations.

## Maintenance

Palette data and R-generated continuous golden fixtures are checked in.
To refresh both from the vendored upstream source during development:

```bash
cargo xtask update-palettes
```

This maintainer command requires R. Building, testing, documenting, and using
the published crate do not require R or the vendor sources.

## Related work

- [ggsci for R](https://github.com/nanxstats/ggsci)
- [ggsci for Python](https://github.com/nanxstats/py-ggsci)
