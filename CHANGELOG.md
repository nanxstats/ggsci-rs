# Changelog

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
