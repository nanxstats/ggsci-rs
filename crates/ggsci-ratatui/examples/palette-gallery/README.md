# Interactive palette gallery

This example is a complete Ratatui application that browses every palette
definition shipped by `ggsci` through the public `ggsci` registries and
`ggsci-ratatui` conversion functions. Run it from the workspace root:

```bash
cargo run -p ggsci-ratatui --example palette-gallery
```

The responsive card grid has four tabs: 33 core discrete palettes, 53 core
continuous palettes, 551 iTerm themes, and 17 Gephi generative palettes. iTerm
cards show all six normal and all six bright colors together. Continuous cards
show 32 interpolated samples, and Gephi cards use 12 deterministic colors from
the fixed seed `42`.

Press `m` to switch between 24-bit TrueColor and xterm ANSI-256 output.
TrueColor fidelity depends on support from the terminal and its backend.

## Controls

- `q` or `Esc`: quit
- `Left`, `h`, or `BackTab`: previous tab
- `Right`, `l`, or `Tab`: next tab
- `1`, `2`, `3`, or `4`: select a tab directly
- `Up` or `k`: scroll up one grid row
- `Down` or `j`: scroll down one grid row
- `PageUp` or `PageDown`: scroll one viewport
- `Home` or `g`: first row
- `End` or `G`: last row
- `m`: toggle TrueColor and ANSI-256

A terminal of at least 48 columns by 12 rows is required. A larger terminal is
recommended for multi-column browsing and media capture.
