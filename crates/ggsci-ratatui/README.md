# ggsci-ratatui

[![CI tests](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nanxstats/ggsci-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/ggsci-ratatui)](https://docs.rs/ggsci-ratatui)
[![crates.io](https://img.shields.io/crates/v/ggsci-ratatui.svg)](https://crates.io/crates/ggsci-ratatui)

Use [ggsci](https://crates.io/crates/ggsci) color palettes in
[ratatui](https://ratatui.rs) applications.
The crate converts palette output to `ratatui_core::style::Color` and builds
lightweight foreground or background `Style` values.

The dependency is deliberately `ratatui-core`, not the full `ratatui`
application crate. Color and style primitives live in the core package, so
libraries can use this adapter without selecting a terminal backend or pulling
in widget and application-level APIs.

The minimum supported Rust version is 1.88, matching `ratatui-core` 0.1.2.
The core `ggsci` crate has a separate Rust 1.85 MSRV.

## Category colors and styles

Use `colors()` when the palette specification may be discrete or continuous.
It uses ggsci's kind-aware `Palette::sample()` dispatch: discrete palettes
produce category colors, while continuous palettes produce interpolated
gradient samples.

```rust
use ggsci_ratatui::{colors, foreground_styles, ColorMode};

fn main() -> Result<(), ggsci::Error> {
    let colors = colors("observable:observable10", 5, ColorMode::TrueColor)?;
    let styles = foreground_styles(&colors);

    assert_eq!(colors.len(), 5);
    assert_eq!(styles.len(), 5);
    Ok(())
}
```

`foreground_styles()` and `background_styles()` return one `Style` per input
color. This release intentionally provides conversion and style primitives,
not widget abstractions.

## Discrete and continuous semantics

A discrete palette maps categories to individual colors. A continuous palette
maps a continuous domain through a gradient. For continuous palettes,
`Palette::colors()` contains interpolation anchors, not the final gradient, so
the number of requested samples is not limited by the anchor count.

Use `continuous_colors()` when reverse handling should be explicit:

```rust
use ggsci::ContinuousOptions;
use ggsci_ratatui::{continuous_colors, ColorMode};

fn main() -> Result<(), ggsci::Error> {
    let colors = continuous_colors(
        "material:blue-grey",
        256,
        ContinuousOptions::new().with_reverse(true),
        ColorMode::TrueColor,
    )?;

    assert_eq!(colors.len(), 256);
    Ok(())
}
```

## Terminal color modes

`ColorMode::TrueColor` preserves each palette color as
`Color::Rgb(red, green, blue)`. The terminal and selected ratatui backend must
support 24-bit color for faithful output. Unsupported or partially supported
terminals may approximate colors, reset them, or render them unpredictably;
application code should not treat terminal capability as guaranteed.

`ColorMode::Ansi256` quantizes to xterm's 216-color RGB cube and 24-step
grayscale ramp. It never selects indices 0 through 15 because terminal themes
commonly redefine those entries. Selection uses squared Euclidean distance in
8-bit sRGB channel space, and exact ties choose the lower index.

Individual `ggsci::Rgb` values can be converted with `color()` or the local
`ToRatatuiColor` extension trait. `ansi256_index()` exposes the deterministic
index calculation.

## iTerm and Gephi palettes

`iterm_colors()` adapts ggsci's fixed discrete iTerm themes while preserving
their normal/bright variant parameter. `gephi_colors()` generates fresh
discrete category colors, and `gephi_colors_with_seed()` provides reproducible
generation.

Both `ItermPalette` and `GephiPalette` report `PaletteKind::Discrete`.
Their dedicated helpers do not imply additional palette kinds: iTerm needs
a theme variant and fixed terminal-channel order, while Gephi needs a generation
algorithm and optional seed. Those are data-access and generation concerns,
orthogonal to scale semantics.

## Alpha compositing

Ratatui colors do not have an alpha channel. `rgba_over()` therefore performs
standard source-over compositing against an explicit opaque `ggsci::Rgb`
background and rounds each resulting channel to the nearest 8-bit value.
In ANSI-256 mode, quantization happens after compositing. Alpha is never
silently discarded.
