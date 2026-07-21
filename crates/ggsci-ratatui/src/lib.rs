//! Adapt [ggsci] palettes to [`ratatui_core`] colors and styles.
//!
//! Core palettes can be sampled with [`colors`], while the dedicated
//! [`iterm_colors`] and [`gephi_colors`] helpers preserve the distinct inputs
//! required by fixed iTerm themes and generative Gephi palettes. All three
//! sources still use discrete or continuous scale semantics from [ggsci].

use ratatui_core::style::{Color, Style};

const ANSI_CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Selects how RGB values are represented in a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorMode {
    /// Preserve the original channels as a 24-bit terminal color.
    #[default]
    TrueColor,
    /// Quantize to the xterm 256-color cube and grayscale ramp.
    Ansi256,
}

/// Converts a ggsci color to a ratatui color.
pub trait ToRatatuiColor {
    /// Converts to a 24-bit ratatui color.
    #[must_use]
    fn to_ratatui_color(self) -> Color;

    /// Converts using the requested terminal color mode.
    #[must_use]
    fn to_ratatui_color_with(self, mode: ColorMode) -> Color;
}

impl ToRatatuiColor for ggsci::Rgb {
    fn to_ratatui_color(self) -> Color {
        color(self, ColorMode::TrueColor)
    }

    fn to_ratatui_color_with(self, mode: ColorMode) -> Color {
        color(self, mode)
    }
}

/// Converts an RGB value to a ratatui color.
#[must_use]
pub fn color(rgb: ggsci::Rgb, mode: ColorMode) -> Color {
    match mode {
        ColorMode::TrueColor => Color::Rgb(rgb.r(), rgb.g(), rgb.b()),
        ColorMode::Ansi256 => Color::Indexed(ansi256_index(rgb)),
    }
}

/// Returns the nearest xterm 256-color index for an RGB value.
///
/// Only indices 16 through 255 are considered, avoiding the 16 colors that a
/// terminal theme can redefine. Distance is squared Euclidean distance in
/// 8-bit sRGB space, with the lower index winning an exact tie.
#[must_use]
pub fn ansi256_index(rgb: ggsci::Rgb) -> u8 {
    nearest_ansi256(rgb)
}

/// Composites an RGBA source over an opaque background and converts it.
///
/// Source-over compositing is performed before any ANSI-256 quantization.
#[must_use]
pub fn rgba_over(rgba: ggsci::Rgba, background: ggsci::Rgb, mode: ColorMode) -> Color {
    let alpha = u32::from(rgba.a());
    let inverse_alpha = 255 - alpha;
    let composite_channel = |source: u8, destination: u8| {
        let numerator = u32::from(source) * alpha + u32::from(destination) * inverse_alpha + 127;
        u8::try_from(numerator / 255).unwrap_or(u8::MAX)
    };
    let red = composite_channel(rgba.r(), background.r());
    let green = composite_channel(rgba.g(), background.g());
    let blue = composite_channel(rgba.b(), background.b());
    let rgb =
        ggsci::Rgb::from_hex((u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue));

    color(rgb, mode)
}

/// Samples a core palette and converts its colors.
///
/// [`ggsci::Palette::sample`] performs the authoritative kind-aware dispatch:
/// discrete palettes return category colors and continuous palettes return
/// interpolated gradient samples.
///
/// # Errors
///
/// Returns lookup errors for an invalid palette specification, or
/// [`ggsci::Error::TooManyColorsRequested`] if a discrete palette is too
/// short.
pub fn colors(spec: &str, n: usize, mode: ColorMode) -> Result<Vec<Color>, ggsci::Error> {
    ggsci::palette_by_spec(spec)?
        .sample(n)
        .map(|colors| convert_colors(colors, mode))
}

/// Interpolates and converts a continuous core palette.
///
/// # Errors
///
/// Returns lookup errors for an invalid palette specification, or
/// [`ggsci::Error::NotContinuousPalette`] when the requested palette is
/// discrete.
pub fn continuous_colors(
    spec: &str,
    n: usize,
    options: ggsci::ContinuousOptions,
    mode: ColorMode,
) -> Result<Vec<Color>, ggsci::Error> {
    ggsci::palette_by_spec(spec)?
        .interpolate_with(n, options)
        .map(|colors| convert_colors(colors, mode))
}

/// Takes and converts colors from a fixed discrete iTerm theme variant.
///
/// # Errors
///
/// Returns [`ggsci::Error::UnknownItermPalette`] when lookup fails, or
/// [`ggsci::Error::TooManyItermColorsRequested`] when `n` exceeds the fixed
/// variant length.
pub fn iterm_colors(
    palette: &str,
    variant: ggsci::ItermVariant,
    n: usize,
    mode: ColorMode,
) -> Result<Vec<Color>, ggsci::Error> {
    ggsci::iterm_palette(palette)?
        .take(variant, n)
        .map(|colors| convert_colors(colors, mode))
}

/// Generates and converts colors from a discrete Gephi palette.
///
/// # Errors
///
/// Returns [`ggsci::Error::UnknownGephiPalette`] when lookup fails, or
/// [`ggsci::Error::GephiGenerationFailed`] when generation fails.
pub fn gephi_colors(palette: &str, n: usize, mode: ColorMode) -> Result<Vec<Color>, ggsci::Error> {
    ggsci::gephi_palette(palette)?
        .generate(n)
        .map(|colors| convert_colors(colors, mode))
}

/// Generates reproducible colors from a discrete Gephi palette and converts
/// them.
///
/// # Errors
///
/// Returns [`ggsci::Error::UnknownGephiPalette`] when lookup fails, or
/// [`ggsci::Error::GephiGenerationFailed`] when generation fails.
pub fn gephi_colors_with_seed(
    palette: &str,
    n: usize,
    seed: u64,
    mode: ColorMode,
) -> Result<Vec<Color>, ggsci::Error> {
    ggsci::gephi_palette(palette)?
        .generate_with_seed(n, seed)
        .map(|colors| convert_colors(colors, mode))
}

/// Builds one foreground style per color.
#[must_use]
pub fn foreground_styles(colors: &[Color]) -> Vec<Style> {
    colors
        .iter()
        .copied()
        .map(|color| Style::new().fg(color))
        .collect()
}

/// Builds one background style per color.
#[must_use]
pub fn background_styles(colors: &[Color]) -> Vec<Style> {
    colors
        .iter()
        .copied()
        .map(|color| Style::new().bg(color))
        .collect()
}

fn convert_colors(colors: Vec<ggsci::Rgb>, mode: ColorMode) -> Vec<Color> {
    colors.into_iter().map(|rgb| color(rgb, mode)).collect()
}

fn nearest_ansi256(rgb: ggsci::Rgb) -> u8 {
    let mut closest_index = 16;
    let mut closest_distance = u32::MAX;

    for index in 16_u8..=u8::MAX {
        let candidate = ansi256_rgb(index);
        let distance = squared_distance(rgb, candidate);
        if distance < closest_distance {
            closest_index = index;
            closest_distance = distance;
        }
    }

    closest_index
}

fn ansi256_rgb(index: u8) -> [u8; 3] {
    if index <= 231 {
        let offset = index - 16;
        let red = offset / 36;
        let green = (offset % 36) / 6;
        let blue = offset % 6;
        [
            ANSI_CUBE_LEVELS[usize::from(red)],
            ANSI_CUBE_LEVELS[usize::from(green)],
            ANSI_CUBE_LEVELS[usize::from(blue)],
        ]
    } else {
        let level = 8 + 10 * (index - 232);
        [level, level, level]
    }
}

fn squared_distance(rgb: ggsci::Rgb, candidate: [u8; 3]) -> u32 {
    let red = i32::from(rgb.r()) - i32::from(candidate[0]);
    let green = i32::from(rgb.g()) - i32::from(candidate[1]);
    let blue = i32::from(rgb.b()) - i32::from(candidate[2]);

    u32::try_from(red * red + green * green + blue * blue)
        .expect("squared RGB distance is nonnegative")
}

#[cfg(test)]
mod tests {
    use ggsci::{ContinuousOptions, Error, ItermVariant, PaletteKind, Rgb, Rgba};
    use ratatui_core::style::{Color, Style};

    use super::{
        ColorMode, ToRatatuiColor, ansi256_index, background_styles, color, colors,
        continuous_colors, foreground_styles, gephi_colors, gephi_colors_with_seed, iterm_colors,
        rgba_over,
    };

    #[test]
    fn converts_rgb_to_truecolor() {
        let rgb = Rgb::from_hex(0x12_34_56);
        assert_eq!(
            color(rgb, ColorMode::TrueColor),
            Color::Rgb(0x12, 0x34, 0x56)
        );
    }

    #[test]
    fn extension_trait_converts_with_default_and_explicit_modes() {
        let rgb = Rgb::from_hex(0xFF_00_00);
        assert_eq!(rgb.to_ratatui_color(), Color::Rgb(255, 0, 0));
        assert_eq!(
            rgb.to_ratatui_color_with(ColorMode::Ansi256),
            Color::Indexed(196)
        );
    }

    #[test]
    fn maps_expected_ansi256_colors() {
        let cases = [
            (0x00_00_00, 16),
            (0xFF_FF_FF, 231),
            (0xFF_00_00, 196),
            (0x00_FF_00, 46),
            (0x00_00_FF, 21),
            (0x80_80_80, 244),
        ];

        for (hex, expected) in cases {
            assert_eq!(ansi256_index(Rgb::from_hex(hex)), expected);
        }
    }

    #[test]
    fn ansi256_ties_choose_the_lower_index() {
        // Red 115 is equidistant from the cube levels 95 and 135.
        assert_eq!(ansi256_index(Rgb::from_hex(0x73_00_00)), 52);
    }

    #[test]
    fn composites_rgba_before_conversion() {
        let background = Rgb::from_hex(0x00_00_FF);
        assert_eq!(
            rgba_over(
                Rgba::from_hex(0xFF_00_00_00),
                background,
                ColorMode::TrueColor
            ),
            Color::Rgb(0, 0, 255)
        );
        assert_eq!(
            rgba_over(
                Rgba::from_hex(0xFF_00_00_80),
                background,
                ColorMode::TrueColor
            ),
            Color::Rgb(128, 0, 127)
        );
        assert_eq!(
            rgba_over(
                Rgba::from_hex(0xFF_00_00_FF),
                background,
                ColorMode::TrueColor
            ),
            Color::Rgb(255, 0, 0)
        );
        assert_eq!(
            rgba_over(
                Rgba::from_hex(0xFF_00_00_80),
                background,
                ColorMode::Ansi256
            ),
            color(Rgb::from_hex(0x80_00_7F), ColorMode::Ansi256)
        );
    }

    #[test]
    fn generic_colors_dispatches_discrete_palettes() {
        assert_eq!(
            colors("npg:nrc", 3, ColorMode::TrueColor).unwrap(),
            [
                Color::Rgb(0xE6, 0x4B, 0x35),
                Color::Rgb(0x4D, 0xBB, 0xD5),
                Color::Rgb(0x00, 0xA0, 0x87),
            ]
        );
    }

    #[test]
    fn generic_colors_dispatches_continuous_palettes_beyond_anchor_count() {
        let palette = ggsci::palette_by_spec("material:blue-grey").unwrap();
        assert_eq!(palette.kind(), PaletteKind::Continuous);
        let n = palette.len() + 7;
        let converted = colors("material:blue-grey", n, ColorMode::TrueColor).unwrap();
        let sampled = palette
            .sample(n)
            .unwrap()
            .into_iter()
            .map(ToRatatuiColor::to_ratatui_color)
            .collect::<Vec<_>>();
        assert_eq!(converted.len(), n);
        assert_eq!(converted, sampled);
        assert!(matches!(
            palette.take(1),
            Err(Error::NotDiscretePalette { .. })
        ));
    }

    #[test]
    fn reverses_explicit_continuous_output() {
        let forward = continuous_colors(
            "gsea:default",
            13,
            ContinuousOptions::new(),
            ColorMode::TrueColor,
        )
        .unwrap();
        let reversed = continuous_colors(
            "gsea:default",
            13,
            ContinuousOptions::new().with_reverse(true),
            ColorMode::TrueColor,
        )
        .unwrap();
        assert_eq!(reversed, forward.into_iter().rev().collect::<Vec<_>>());
    }

    #[test]
    fn converts_fixed_discrete_iterm_colors() {
        let palette = ggsci::iterm_palette("Rose Pine").unwrap();
        assert_eq!(palette.kind(), PaletteKind::Discrete);
        assert_eq!(
            iterm_colors("Rose Pine", ItermVariant::Normal, 3, ColorMode::TrueColor,).unwrap(),
            [
                Color::Rgb(0x9C, 0xCF, 0xD8),
                Color::Rgb(0xF6, 0xC1, 0x77),
                Color::Rgb(0xEB, 0x6F, 0x92),
            ]
        );
    }

    #[test]
    fn converts_generative_discrete_gephi_colors_deterministically() {
        let palette = ggsci::gephi_palette("fancy-light").unwrap();
        assert_eq!(palette.kind(), PaletteKind::Discrete);
        let first = gephi_colors_with_seed("fancy-light", 5, 42, ColorMode::TrueColor).unwrap();
        let second = gephi_colors_with_seed("fancy-light", 5, 42, ColorMode::TrueColor).unwrap();
        assert_eq!(first.len(), 5);
        assert_eq!(first, second);
    }

    #[test]
    fn unseeded_gephi_colors_have_the_requested_shape_and_representation() {
        let generated = gephi_colors("default", 3, ColorMode::TrueColor).unwrap();
        assert_eq!(generated.len(), 3);
        assert!(
            generated
                .into_iter()
                .all(|color| matches!(color, Color::Rgb(_, _, _)))
        );
    }

    #[test]
    fn builds_foreground_styles() {
        let colors = [Color::Rgb(1, 2, 3), Color::Indexed(42)];
        assert_eq!(
            foreground_styles(&colors),
            [Style::new().fg(colors[0]), Style::new().fg(colors[1])]
        );
    }

    #[test]
    fn builds_background_styles() {
        let colors = [Color::Rgb(1, 2, 3), Color::Indexed(42)];
        assert_eq!(
            background_styles(&colors),
            [Style::new().bg(colors[0]), Style::new().bg(colors[1])]
        );
    }
}
