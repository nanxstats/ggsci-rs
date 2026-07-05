# ggsci-rs

Rust workspace for scientific and sci-fi color palettes from
[ggsci](https://github.com/nanxstats/ggsci).

The first release contains static palettes generated from the upstream data.
Gephi palettes, iTerm palettes, ratatui conversions, and native ggsql extension
APIs are planned future work.

## Usage

```rust
let palette = ggsci::palette("npg", "nrc")?;
let colors = palette.take_hex(3)?;
# Ok::<(), ggsci::Error>(())
```

`crates/ggsci` is the only publishable crate in the workspace for `0.1.0`.
The ratatui and ggsql crates are private scaffolds for later integrations.

## Maintenance

Palette data is checked in as Rust source. To refresh it from the vendored
upstream source during development:

```bash
cargo xtask update-palettes
```

## Related work

- [ggsci for R](https://github.com/nanxstats/ggsci)
- [ggsci for Python](https://github.com/nanxstats/py-ggsci)
