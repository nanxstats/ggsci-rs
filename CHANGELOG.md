# Changelog

## ggsci-rs 0.6.0

This release synchronizes `ggsci`, `ggsci-ratatui`, and `ggsci-ggsql` at
version 0.6.0 (#17).

### New features

- Publish `ggsci-ggsql` for the first time, targeting ggsql 0.4.1 without its
  default database reader, output, spatial, or built-in data features.
- Add the kind-preserving `GgsqlPalette` wrapper with core discrete and
  continuous resolution, explicit reverse options, fixed discrete iTerm
  conversion, and seeded or unseeded generative discrete Gephi conversion.
- Add typed `ggsql::plot::scale::OutputRange::Array` conversion and textual
  explicit-array `SCALE` generation. Preserve the discrete-only
  `color_array()` and `scale_discrete()` scaffold APIs.
- Add adapter-level `ScaleKind::{Discrete, Continuous, Binned, Ordinal}` with
  default mapping from `ggsci::PaletteKind` and support for explicit valid
  combinations such as continuous palette samples on binned scales.
- Add trimmed ASCII aesthetic validation and convenience functions for typed
  output, continuous scales, fixed iTerm themes, and reproducible Gephi output.

### Testing, documentation, and maintenance

- Add ggsql parser integration tests for generated discrete and 256-color
  continuous clauses, typed-array conversion tests, all four scale keywords,
  invalid SQL identifiers, reversed gradients, iTerm themes, and deterministic
  Gephi generation.
- Add discrete and continuous examples plus crates.io documentation explaining
  the current explicit-array integration. ggsql has no stable third-party
  provider or registry API; the adapter can adopt a future official trait when
  one becomes available.
- Keep `ggsci` at Rust 1.85 and `ggsci-ratatui` at Rust 1.88. Give `ggsci-ggsql`
  the Rust 1.86 MSRV and Rust 2021 edition inherited by ggsql 0.4.1,
  with a separate CI package check.

## ggsci-rs 0.5.0

### New features

- Publish `ggsci-ratatui` for the first time at version 0.5.0, using
  `ratatui-core` directly for its public `Color` and `Style` API (#15).
- Add palette kind aware core palette conversion, explicit reversed continuous
  conversion, fixed discrete iTerm conversion, and seeded or unseeded
  generative discrete Gephi conversion.
- Add 24-bit truecolor and deterministic xterm ANSI-256 modes, a local
  `ToRatatuiColor` extension trait, and foreground/background style helpers.
- Add explicit source-over RGBA compositing against an opaque background so
  alpha is resolved before terminal color conversion.

### Maintenance

- Set the `ggsci` MSRV to Rust 1.85, the first stable toolchain whose Cargo
  supports the workspace's Rust 2024 manifests and which also supports the
  core interpolation code's floating point `const fn`. Give `ggsci-ratatui`
  the Rust 1.88 MSRV and Rust 2024 edition required by `ratatui-core` 0.1.2.
- Add adapter documentation, a compiling swatch example, package metadata,
  package content validation, and separate CI checks for both crate MSRVs.

## ggsci-rs 0.4.1

### Documentation

- Add readme badge and improve code examples (#13).

## ggsci-rs 0.4.0

### New features

- Added all 17 generative discrete Gephi palettes through a dedicated filter
  registry and pure Rust palette engine. Every Gephi generator reports
  `PaletteKind::Discrete` (#9).
- Added normalized Gephi lookup and listing APIs, arbitrary-length RGB and RGBA
  generation, reproducible seeded generation, and convenient nondeterministic
  generation from fresh operating-system randomness.
- Defined stable seeded output with `ChaCha8Rng`, explicit SplitMix64 seed
  expansion, and a crate-owned 53-bit uniform floating-point mapping.
  Golden tests lock the design; Rust seeds are intentionally not promised
  to match R or NumPy streams.

### Maintenance

- Ported rejection sampling, normalized Lab-like filtering, k-means center
  replacement, free-sample removal, farthest-first ordering, color conversion,
  clipping, transfer, and rounding from the canonical R implementation.
- Added a thread-safe indexed cache for the valid 9,261-point sample grids.
  Generated palettes are not cached, and the upstream quality schedule reduces
  iteration counts as requested palette sizes grow.
- Added deterministic Gephi metadata generation to `cargo xtask update-palettes`.
  Builds and tests use the checked-in 17-filter registry and require no
  R, Python, NumPy, or network access.
- Kept Gephi definitions separate from the core registry because they require
  an algorithm and random state.

## ggsci-rs 0.3.0

### New features

- Added 551 fixed discrete iTerm palettes through a dedicated typed registry,
  with 6,612 checked-in colors across normal and bright variants (#7).
- Added typed iTerm variants and terminal channels, normalized theme lookup,
  fixed-length RGB/hex/RGBA access, explicit cycling, and registry count/name
  APIs. Every iTerm palette reports `PaletteKind::Discrete`.

### Data model and maintenance

- Kept iTerm themes separate from the 86-record core `Palette` registry so
  their paired normal/bright variants and fixed Blue, Yellow, Red, Cyan, Green,
  Magenta channel ordering remain explicit. Normal/bright is an
  `ItermVariant`, not a palette kind.
- Added deterministic iTerm Rust generation to `cargo xtask update-palettes`;
  builds and tests use checked-in data and do not require R or vendored sources.
  iTerm data ships without feature flags.
- Left the core registry unchanged at 86 records and 946 stored source colors:
  33 discrete records and 53 continuous records.

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
