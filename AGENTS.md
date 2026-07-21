# AGENTS.md

## Maintenance

- Generated core, iTerm, and Gephi palette metadata and continuous fixtures are
  checked in. Do not edit
  `crates/ggsci/src/generated/palettes.rs` or
  `crates/ggsci/src/generated/iterm.rs` or
  `crates/ggsci/src/generated/gephi.rs` or
  `crates/ggsci/tests/generated/continuous_fixtures.rs` by hand.
- Run `cargo xtask update-palettes` to refresh palette data and R golden
  fixtures from the vendored ggsci source. The command generates the core
  database, generates continuous fixtures, generates the dedicated iTerm
  registry, generates Gephi filter metadata, and runs `cargo fmt --all`.

## Core palettes

- `PaletteKind::{Discrete, Continuous}` describes scale semantics, not whether
  colors are stored or generated. The `gsea`, `bs5`, `material`, and `tw3`
  families are continuous; the other current core families are discrete.
- Continuous interpolation must remain compatible with R's CIE Lab/FMM spline
  behavior at the final RGB channel level. R is a maintainer dependency only.

## iTerm and Gephi palettes

- iTerm themes are fixed discrete palettes in a dedicated typed registry. Every
  `ItermPalette` reports `PaletteKind::Discrete`, while normal/bright is an
  `ItermVariant`. Do not flatten iTerm themes into the core `palettes()` slice.
- Gephi palettes are generative discrete palettes in a dedicated filter
  registry. Every `GephiPalette` reports `PaletteKind::Discrete`; generation
  mechanism and scale semantics are orthogonal. Do not flatten Gephi
  definitions into the core `palettes()` slice.
- Seeded Gephi output uses `ChaCha8Rng`, explicit SplitMix64 seed expansion,
  and a crate-owned 53-bit `f64` mapping. Golden tests lock this design. The
  filter-valid sample grids are held in one thread-safe indexed cache; do not
  cache generated output by seed and size.

## Adapter crates

- `ggsci-ratatui` depends on the registry version of `ratatui-core`, not the
  vendored path. Its APIs adapt core, fixed iTerm, and generative Gephi output;
  they do not introduce another palette kind.
- Full Ratatui and Crossterm are development dependencies for application
  examples only. The adapter's normal dependency graph must remain limited to
  `ratatui-core` plus `ggsci`.
- The interactive palette gallery builds its catalog from the public core,
  iTerm, and Gephi registries and public adapter functions, never generated
  implementation modules. Its Gephi cards use fixed seed 42 so startup output,
  tests, screenshots, and recordings are deterministic.
- `ggsci-ggsql` is publishable and matches ggsql 0.4.1's Rust 1.86 MSRV and
  Rust 2021 edition. It depends on the registry ggsql crate with default
  features disabled, never on `vendor/ggsql`.
- ggsql does not currently expose a third-party palette provider or registry.
  Integrate ggsci by emitting explicit color arrays, preserve source
  `PaletteKind` separately from adapter `ScaleKind`, and do not patch or modify
  `vendor/ggsql`.
- `tree-sitter-ggsql` 0.4.1 regenerates its parser unless
  `GGSQL_SKIP_GENERATE=1` is set. Workspace configuration and CI use that
  upstream-supported setting to compile its packaged generated `src/parser.c`
  without `tree-sitter-cli`.

## Publishing

- The published `ggsci` crate must not require R, Python, NumPy, matplotlib,
  jsonlite, or network access at build time.
- `ggsci` has a Rust 1.85 MSRV, the first stable toolchain whose Cargo supports
  the workspace's Rust 2024 manifests. This also supports floating-point
  arithmetic in `const fn`. The publishable `ggsci-ratatui` adapter matches the
  higher Rust 1.88 MSRV of its `ratatui-core` dependency.
- Do not add feature flags for now. The crate remains a packaged deal.
- Before publishing, run:

  ```bash
  cargo xtask update-palettes
  cargo fmt --all --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --all-targets
  cargo doc -p ggsci --no-deps
  cargo doc -p ggsci-ratatui --no-deps
  cargo doc -p ggsci-ggsql --no-deps
  cargo package -p ggsci --list
  cargo package -p ggsci-ratatui --list
  cargo package -p ggsci-ggsql --list
  cargo publish -p ggsci --dry-run
  cargo publish -p ggsci-ratatui --dry-run
  cargo publish -p ggsci-ggsql --dry-run
  ```

  The adapter dry-runs can fail until the matching `ggsci` release has reached
  the registry. Do not weaken their registry version requirements to bypass
  the release order constraint.
