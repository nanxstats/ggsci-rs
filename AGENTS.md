# AGENTS.md

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
- `PaletteKind::{Discrete, Continuous}` describes scale semantics, not whether
  colors are stored or generated. The `gsea`, `bs5`, `material`, and `tw3`
  families are continuous; the other current core families are discrete.
- Continuous interpolation must remain compatible with R's CIE Lab/FMM spline
  behavior at the final RGB channel level. R is a maintainer dependency only.
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
- The published `ggsci` crate must not require R, Python, NumPy, matplotlib,
  jsonlite, or network access at build time.
- `ggsci` has a Rust 1.85 MSRV, the first stable toolchain whose Cargo supports
  the workspace's Rust 2024 manifests. This also supports floating-point
  arithmetic in `const fn`. The publishable `ggsci-ratatui` adapter matches the
  higher Rust 1.88 MSRV of its `ratatui-core` dependency.
- `ggsci-ratatui` depends on the registry version of `ratatui-core`, not the
  vendored path. Its APIs adapt core, fixed iTerm, and generative Gephi output;
  they do not introduce another palette kind.
- `ggsci-ggsql` remains unpublished.
- Do not add feature flags for now. The crate remains a packaged deal.
- Before publishing, run:

  ```bash
  cargo xtask update-palettes
  cargo fmt --all --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --all-targets
  cargo doc -p ggsci --no-deps
  cargo doc -p ggsci-ratatui --no-deps
  cargo package -p ggsci --list
  cargo package -p ggsci-ratatui --list
  cargo publish -p ggsci --dry-run
  cargo publish -p ggsci-ratatui --dry-run
  ```

  The adapter dry-run can fail until the matching `ggsci` release has reached
  the registry. Do not weaken its registry version requirement to bypass the
  release order constraint.
