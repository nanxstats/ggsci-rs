use ggsci::{ContinuousOptions, ItermVariant, PaletteKind};
use ggsci_ratatui::{ColorMode, colors, continuous_colors, gephi_colors_with_seed, iterm_colors};
use ratatui::style::Color;

pub const CONTINUOUS_SAMPLE_COUNT: usize = 32;
pub const GEPHI_SAMPLE_COUNT: usize = 12;
pub const GEPHI_SEED: u64 = 42;
const ITERM_COLOR_COUNT: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorStrip {
    pub label: Option<&'static str>,
    pub truecolor: Vec<Color>,
    pub ansi256: Vec<Color>,
}

impl ColorStrip {
    pub fn colors(&self, mode: ColorMode) -> &[Color] {
        match mode {
            ColorMode::TrueColor => &self.truecolor,
            ColorMode::Ansi256 => &self.ansi256,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteEntry {
    pub title: String,
    pub detail: String,
    pub strips: Vec<ColorStrip>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    pub discrete: Vec<PaletteEntry>,
    pub continuous: Vec<PaletteEntry>,
    pub iterm: Vec<PaletteEntry>,
    pub gephi: Vec<PaletteEntry>,
}

impl Catalog {
    pub fn new() -> Result<Self, ggsci::Error> {
        let discrete = ggsci::palettes_by_kind(PaletteKind::Discrete)
            .map(discrete_entry)
            .collect::<Result<_, _>>()?;
        let continuous = ggsci::palettes_by_kind(PaletteKind::Continuous)
            .map(continuous_entry)
            .collect::<Result<_, _>>()?;
        let iterm = ggsci::iterm_palettes()
            .iter()
            .map(iterm_entry)
            .collect::<Result<_, _>>()?;
        let gephi = ggsci::gephi_palettes()
            .iter()
            .map(gephi_entry)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            discrete,
            continuous,
            iterm,
            gephi,
        })
    }

    pub fn entries(&self, tab_index: usize) -> &[PaletteEntry] {
        match tab_index {
            0 => &self.discrete,
            1 => &self.continuous,
            2 => &self.iterm,
            3 => &self.gephi,
            _ => &[],
        }
    }

    pub fn counts(&self) -> [usize; 4] {
        [
            self.discrete.len(),
            self.continuous.len(),
            self.iterm.len(),
            self.gephi.len(),
        ]
    }

    #[cfg(test)]
    pub fn test_fixture() -> Self {
        fn entry(index: usize, strips: usize) -> PaletteEntry {
            let source = [
                Color::Rgb(230, 75, 53),
                Color::Rgb(77, 187, 213),
                Color::Rgb(0, 160, 135),
                Color::Rgb(60, 84, 136),
                Color::Rgb(243, 155, 127),
                Color::Rgb(126, 92, 150),
            ];
            let quantized = [
                Color::Indexed(167),
                Color::Indexed(74),
                Color::Indexed(36),
                Color::Indexed(60),
                Color::Indexed(216),
                Color::Indexed(97),
            ];
            let strips = (0..strips)
                .map(|strip| ColorStrip {
                    label: match (strips, strip) {
                        (2, 0) => Some("N"),
                        (2, 1) => Some("B"),
                        _ => None,
                    },
                    truecolor: source.to_vec(),
                    ansi256: quantized.to_vec(),
                })
                .collect();
            PaletteEntry {
                title: format!("fixture:{index}"),
                detail: "test palette".to_owned(),
                strips,
            }
        }

        Self {
            discrete: (0..8).map(|index| entry(index, 1)).collect(),
            continuous: (0..8).map(|index| entry(index, 1)).collect(),
            iterm: (0..40).map(|index| entry(index, 2)).collect(),
            gephi: (0..8).map(|index| entry(index, 1)).collect(),
        }
    }
}

fn discrete_entry(palette: &ggsci::Palette) -> Result<PaletteEntry, ggsci::Error> {
    let spec = format!("{}:{}", palette.family(), palette.variant());
    let count = palette.len();
    Ok(PaletteEntry {
        title: spec.clone(),
        detail: format!("discrete · {count} colors"),
        strips: vec![dual_mode_strip(None, |mode| colors(&spec, count, mode))?],
    })
}

fn continuous_entry(palette: &ggsci::Palette) -> Result<PaletteEntry, ggsci::Error> {
    let spec = format!("{}:{}", palette.family(), palette.variant());
    Ok(PaletteEntry {
        title: spec.clone(),
        detail: format!("continuous · {CONTINUOUS_SAMPLE_COUNT} samples"),
        strips: vec![dual_mode_strip(None, |mode| {
            continuous_colors(
                &spec,
                CONTINUOUS_SAMPLE_COUNT,
                ContinuousOptions::new(),
                mode,
            )
        })?],
    })
}

fn iterm_entry(palette: &ggsci::ItermPalette) -> Result<PaletteEntry, ggsci::Error> {
    let name = palette.name();
    let normal = dual_mode_strip(Some("N"), |mode| {
        iterm_colors(name, ItermVariant::Normal, ITERM_COLOR_COUNT, mode)
    })?;
    let bright = dual_mode_strip(Some("B"), |mode| {
        iterm_colors(name, ItermVariant::Bright, ITERM_COLOR_COUNT, mode)
    })?;
    Ok(PaletteEntry {
        title: name.to_owned(),
        detail: "iTerm · normal + bright".to_owned(),
        strips: vec![normal, bright],
    })
}

fn gephi_entry(palette: &ggsci::GephiPalette) -> Result<PaletteEntry, ggsci::Error> {
    let name = palette.name();
    Ok(PaletteEntry {
        title: name.to_owned(),
        detail: format!("Gephi · {GEPHI_SAMPLE_COUNT} colors · seed {GEPHI_SEED}"),
        strips: vec![dual_mode_strip(None, |mode| {
            gephi_colors_with_seed(name, GEPHI_SAMPLE_COUNT, GEPHI_SEED, mode)
        })?],
    })
}

fn dual_mode_strip<F>(label: Option<&'static str>, mut build: F) -> Result<ColorStrip, ggsci::Error>
where
    F: FnMut(ColorMode) -> Result<Vec<Color>, ggsci::Error>,
{
    Ok(ColorStrip {
        label,
        truecolor: build(ColorMode::TrueColor)?,
        ansi256: build(ColorMode::Ansi256)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{CONTINUOUS_SAMPLE_COUNT, Catalog, GEPHI_SAMPLE_COUNT, gephi_entry};
    use ggsci_ratatui::ColorMode;

    #[test]
    fn catalog_has_every_registry_entry_with_expected_strip_shapes() {
        let catalog = Catalog::new().unwrap();

        assert_eq!(catalog.counts(), [33, 53, 551, 17]);
        assert!(catalog.discrete.iter().all(|entry| {
            entry.strips.len() == 1 && !entry.strips[0].colors(ColorMode::TrueColor).is_empty()
        }));
        assert!(catalog.continuous.iter().all(|entry| {
            entry.strips.len() == 1
                && entry.strips[0].colors(ColorMode::TrueColor).len() == CONTINUOUS_SAMPLE_COUNT
        }));
        assert!(catalog.iterm.iter().all(|entry| {
            entry.strips.len() == 2
                && entry.strips[0].label == Some("N")
                && entry.strips[1].label == Some("B")
                && entry
                    .strips
                    .iter()
                    .all(|strip| strip.colors(ColorMode::TrueColor).len() == 6)
        }));
        assert!(catalog.gephi.iter().all(|entry| {
            entry.strips.len() == 1
                && entry.strips[0].colors(ColorMode::TrueColor).len() == GEPHI_SAMPLE_COUNT
        }));
    }

    #[test]
    fn both_color_modes_share_metadata_and_strip_lengths() {
        let catalog = Catalog::new().unwrap();

        for tab in 0..4 {
            for entry in catalog.entries(tab) {
                assert!(!entry.title.is_empty());
                assert!(!entry.detail.is_empty());
                for strip in &entry.strips {
                    assert_eq!(
                        strip.colors(ColorMode::TrueColor).len(),
                        strip.colors(ColorMode::Ansi256).len()
                    );
                }
            }
        }
    }

    #[test]
    fn gephi_construction_is_deterministic_for_seed_42() {
        let palette = &ggsci::gephi_palettes()[0];
        assert_eq!(gephi_entry(palette).unwrap(), gephi_entry(palette).unwrap());
    }
}
