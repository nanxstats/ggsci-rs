# AGENTS.md

- Generated palette data and continuous fixtures are checked in. Do not edit
  `crates/ggsci/src/generated/palettes.rs` or
  `crates/ggsci/tests/generated/continuous_fixtures.rs` by hand.
- Run `cargo xtask update-palettes` to refresh palette data and R golden
  fixtures from the vendored ggsci source. The command generates the core
  database, generates continuous fixtures, and runs `cargo fmt --all`.
- `PaletteKind::{Discrete, Continuous}` describes scale semantics, not whether
  colors are stored or generated. The `gsea`, `bs5`, `material`, and `tw3`
  families are continuous; the other current core families are discrete.
- Continuous interpolation must remain compatible with R's CIE Lab/FMM spline
  behavior at the final RGB channel level. R is a maintainer dependency only.
- The published `ggsci` crate must not require R, Python, NumPy, matplotlib,
  jsonlite, or network access at build time.
- Do not add feature flags for now. The crate remains a packaged deal.
- Before publishing, run:

  ```bash
  cargo xtask update-palettes
  cargo fmt --all --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --all-targets
  cargo doc -p ggsci --no-deps
  cargo package -p ggsci --list
  cargo publish -p ggsci --dry-run
  ```
