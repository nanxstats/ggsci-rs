# Changelog

## ggsci-rs 0.2.0

### Breaking changes

- `PaletteKind::Static` was replaced with `PaletteKind::Discrete`.
  The old name described storage rather than scale semantics (#3).
- `Palette::take()`, `take_hex()`, and `cycle()` now apply only to
  discrete palettes. Use `interpolate()` for continuous palettes or
  `sample()` when accepting either kind.

### New features

- Added dependency-free continuous color generation for all 53 variants in
  the `gsea`, `bs5`, `material`, and `tw3` color scale families, compatible
  with ggsci for R's CIE Lab/FMM spline interpolation at the final RGB
  channel level.
- Added `ContinuousOptions`, continuous RGB/RGBA/hex interpolation methods,
  kind-aware `sample()` methods, kind predicates, and `palettes_by_kind()`.

### Testing and maintenance

- Added checked-in R golden fixtures covering every continuous variant at
  multiple output sizes in forward and reversed order. R remains a maintainer
  dependency only.
- Kept canonical category colors and interpolation anchors checked in: 86 core
  palettes and 946 stored source colors in total, split into 33/403 discrete
  palettes/colors and 53/543 continuous palettes/anchors.

## ggsci-rs 0.1.0

- Initial Rust workspace scaffold (#1).
- Added the publishable `ggsci` crate with static palettes
  (excluding Gephi and iTerm palettes) generated from upstream.
- Added private scaffold crates for future ratatui and ggsql integrations.
- Added `cargo xtask update-palettes` for deterministic palette regeneration.
