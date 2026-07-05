# ggsci

Scientific and sci-fi color palettes from the R package
[ggsci](https://github.com/nanxstats/ggsci), packaged as static Rust data.

This first release includes static non-Gephi, non-iTerm palettes generated from
upstream. Continuous seed palettes such as `gsea`, `bs5`, `material`, and `tw3`
are exposed as static color arrays only; interpolation is future work.

```rust
let palette = ggsci::palette("npg", "nrc")?;
let colors = palette.take_hex(3)?;

assert_eq!(colors, ["#E64B35", "#4DBBD5", "#00A087"]);
# Ok::<(), ggsci::Error>(())
```

Lookup is case-insensitive and accepts `_`, `-`, and spaces interchangeably:

```rust
let palette = ggsci::palette_by_spec("material:blue-grey")?;
assert_eq!(palette.family(), "material");
assert_eq!(palette.variant(), "blue-grey");
# Ok::<(), ggsci::Error>(())
```

The crate has no runtime dependencies, no feature flags, and no build script.
