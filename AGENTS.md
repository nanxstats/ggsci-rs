# AGENTS.md

- Generated palette files are checked in. Do not edit
  `crates/ggsci/src/generated/palettes.rs` by hand.
- Run `cargo xtask update-palettes` to refresh palette data from upstream
  `ggsci/R/palettes.R`.
- The published `ggsci` crate must not require R, Python, NumPy, matplotlib,
  jsonlite, or network access at build time.
- Do not add feature flags for now. The first release is a packaged deal crate.
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
