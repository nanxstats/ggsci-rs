use std::{borrow::Cow, sync::OnceLock};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{continuous, generated, normalize::key_matches, Error, PaletteKind, Rgb, Rgba};

type Lab = [f64; 3];

const MAX_RANDOM_CENTER_ATTEMPTS: usize = 1_000_000;
const SPLITMIX64_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX64_MULTIPLIER_1: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX64_MULTIPLIER_2: u64 = 0x94D0_49BB_1331_11EB;
const UNIT_F64_SCALE: f64 = 1.0 / 9_007_199_254_740_992.0;

// This string documents every stability-sensitive layer of seeded generation.
// Keeping the f64 mapping local avoids depending on rand's StandardUniform
// implementation for golden output.
const SEEDED_RNG_ALGORITHM: &str =
    "ChaCha8Rng; four SplitMix64 outputs in little-endian order; next_u64 high 53 bits / 2^53";

static VALID_SAMPLE_CACHE: OnceLock<Vec<Vec<Lab>>> = OnceLock::new();

/// A Gephi hue/chroma/luminance filter in the engine's normalized color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GephiFilter {
    h_min: f64,
    h_max: f64,
    c_min: f64,
    c_max: f64,
    l_min: f64,
    l_max: f64,
}

impl GephiFilter {
    /// Creates a Gephi filter from its inclusive bounds.
    #[must_use]
    pub const fn new(
        h_min: f64,
        h_max: f64,
        c_min: f64,
        c_max: f64,
        l_min: f64,
        l_max: f64,
    ) -> Self {
        Self {
            h_min,
            h_max,
            c_min,
            c_max,
            l_min,
            l_max,
        }
    }

    /// Returns the inclusive lower hue bound in degrees.
    #[must_use]
    pub const fn h_min(self) -> f64 {
        self.h_min
    }

    /// Returns the inclusive upper hue bound in degrees.
    #[must_use]
    pub const fn h_max(self) -> f64 {
        self.h_max
    }

    /// Returns the inclusive lower normalized chroma bound.
    #[must_use]
    pub const fn c_min(self) -> f64 {
        self.c_min
    }

    /// Returns the inclusive upper normalized chroma bound.
    #[must_use]
    pub const fn c_max(self) -> f64 {
        self.c_max
    }

    /// Returns the inclusive lower normalized luminance bound.
    #[must_use]
    pub const fn l_min(self) -> f64 {
        self.l_min
    }

    /// Returns the inclusive upper normalized luminance bound.
    #[must_use]
    pub const fn l_max(self) -> f64 {
        self.l_max
    }

    fn is_finite(self) -> bool {
        [
            self.h_min, self.h_max, self.c_min, self.c_max, self.l_min, self.l_max,
        ]
        .into_iter()
        .all(f64::is_finite)
    }
}

/// A named Gephi generative discrete palette definition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GephiPalette {
    name: &'static str,
    filter: GephiFilter,
    index: usize,
}

impl GephiPalette {
    /// Creates a Gephi palette definition.
    #[must_use]
    pub const fn new(index: usize, name: &'static str, filter: GephiFilter) -> Self {
        Self {
            name,
            filter,
            index,
        }
    }

    /// Returns the canonical upstream palette name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the scale semantics of this Gephi palette.
    #[must_use]
    pub const fn kind(&self) -> PaletteKind {
        PaletteKind::Discrete
    }

    /// Returns the generated hue/chroma/luminance filter metadata.
    #[must_use]
    pub const fn filter(&self) -> GephiFilter {
        self.filter
    }

    /// Generates `n` category colors from fresh operating-system randomness.
    ///
    /// This method creates an independent [`ChaCha8Rng`] internally and does
    /// not use or mutate an application RNG. Use [`Self::generate_with_seed`]
    /// for reproducible output.
    ///
    /// # Errors
    ///
    /// Returns [`Error::GephiGenerationFailed`] if fresh randomness is
    /// unavailable or the filter cannot produce valid colors.
    pub fn generate(&self, n: usize) -> Result<Vec<Rgb>, Error> {
        if n == 0 {
            return Ok(Vec::new());
        }

        let mut rng = ChaCha8Rng::try_from_os_rng().map_err(|error| {
            self.generation_error(format!(
                "could not obtain operating-system randomness: {error}"
            ))
        })?;
        self.generate_with_rng(n, &mut rng)
    }

    /// Generates `n` reproducible category colors from a `u64` seed.
    ///
    /// Seed expansion uses four `SplitMix64` outputs, serialized in
    /// little-endian order, to initialize `ChaCha8Rng`. Uniform `f64` samples
    /// use the high 53 bits of each `next_u64()` output divided by 2^53. This
    /// design is stable within this crate and locked by golden tests; seeds are
    /// not compatible with R or `NumPy` random streams.
    ///
    /// # Errors
    ///
    /// Returns [`Error::GephiGenerationFailed`] if the filter cannot produce
    /// valid colors.
    pub fn generate_with_seed(&self, n: usize, seed: u64) -> Result<Vec<Rgb>, Error> {
        if n == 0 {
            return Ok(Vec::new());
        }

        let mut rng = seeded_rng(seed);
        self.generate_with_rng(n, &mut rng)
    }

    /// Generates `n` category colors and applies an alpha channel.
    ///
    /// Alpha is applied after RGB generation and must be finite and in
    /// `(0.0, 1.0]`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAlpha`] for invalid alpha, or
    /// [`Error::GephiGenerationFailed`] if colors cannot be generated.
    pub fn generate_rgba(&self, n: usize, alpha: f32) -> Result<Vec<Rgba>, Error> {
        let alpha = continuous::continuous_alpha(alpha)?;
        self.generate(n).map(|colors| apply_alpha(colors, alpha))
    }

    /// Generates reproducible category colors and applies an alpha channel.
    ///
    /// Alpha is applied after RGB generation and must be finite and in
    /// `(0.0, 1.0]`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidAlpha`] for invalid alpha, or
    /// [`Error::GephiGenerationFailed`] if colors cannot be generated.
    pub fn generate_rgba_with_seed(
        &self,
        n: usize,
        alpha: f32,
        seed: u64,
    ) -> Result<Vec<Rgba>, Error> {
        let alpha = continuous::continuous_alpha(alpha)?;
        self.generate_with_seed(n, seed)
            .map(|colors| apply_alpha(colors, alpha))
    }

    fn generate_with_rng(&self, n: usize, rng: &mut impl RngCore) -> Result<Vec<Rgb>, Error> {
        if !self.filter.is_finite() {
            return Err(self.generation_error("filter contains non-finite values"));
        }

        let samples = self.valid_samples();
        if samples.is_empty() {
            return Err(self.generation_error("filter has no valid samples on the Gephi grid"));
        }

        let mut centers = random_centers(n, self.filter, rng).ok_or_else(|| {
            self.generation_error(format!(
                "rejection sampling exceeded {MAX_RANDOM_CENTER_ATTEMPTS} attempts for one center"
            ))
        })?;
        refine_centers(&mut centers, &samples, self.filter, quality(n));
        let ordered = farthest_first(&centers);

        ordered
            .into_iter()
            .map(|lab| {
                lab_to_rgb_color(lab).ok_or_else(|| {
                    self.generation_error("a generated center converted to invalid RGB channels")
                })
            })
            .collect()
    }

    fn valid_samples(&self) -> Cow<'static, [Lab]> {
        let registered = generated::gephi::GEPHI_PALETTES
            .get(self.index)
            .is_some_and(|palette| palette.name == self.name && palette.filter == self.filter);

        if registered {
            let cache = VALID_SAMPLE_CACHE.get_or_init(|| {
                generated::gephi::GEPHI_PALETTES
                    .iter()
                    .map(|palette| color_samples(palette.filter))
                    .collect()
            });
            if let Some(samples) = cache.get(self.index) {
                return Cow::Borrowed(samples);
            }
        }

        Cow::Owned(color_samples(self.filter))
    }

    fn generation_error(&self, reason: impl Into<String>) -> Error {
        Error::GephiGenerationFailed {
            palette: self.name,
            reason: reason.into(),
        }
    }
}

/// Returns the dedicated Gephi generator registry.
///
/// Every definition has [`PaletteKind::Discrete`] scale semantics. Gephi
/// definitions are not included in [`crate::palettes`] or
/// [`crate::palettes_by_kind`] because producing colors requires an algorithm
/// and random state, not because Gephi has a different palette kind.
#[must_use]
pub fn gephi_palettes() -> &'static [GephiPalette] {
    let palettes = generated::gephi::GEPHI_PALETTES;
    debug_assert_eq!(palettes.len(), generated::gephi::GEPHI_PALETTE_COUNT);
    debug_assert_eq!(generated::gephi::GEPHI_DATA_SOURCE, "ggsci/R/palettes.R");
    palettes
}

/// Looks up a Gephi generator by name.
///
/// Lookup is case-insensitive. Underscores, hyphens, and whitespace are
/// interchangeable separators.
///
/// # Errors
///
/// Returns [`Error::UnknownGephiPalette`] when the name is not known.
pub fn gephi_palette(name: &str) -> Result<&'static GephiPalette, Error> {
    gephi_palettes()
        .iter()
        .find(|palette| key_matches(palette.name(), name))
        .ok_or_else(|| Error::UnknownGephiPalette {
            palette: name.to_owned(),
        })
}

/// Returns all canonical Gephi palette names in upstream order.
pub fn gephi_palette_names() -> impl Iterator<Item = &'static str> {
    gephi_palettes().iter().map(GephiPalette::name)
}

/// Returns the number of Gephi generators in the dedicated registry.
#[must_use]
pub const fn gephi_palette_count() -> usize {
    generated::gephi::GEPHI_PALETTE_COUNT
}

fn apply_alpha(colors: Vec<Rgb>, alpha: u8) -> Vec<Rgba> {
    colors
        .into_iter()
        .map(|color| color.with_alpha_u8(alpha))
        .collect()
}

fn seeded_rng(seed: u64) -> ChaCha8Rng {
    debug_assert!(!SEEDED_RNG_ALGORITHM.is_empty());
    ChaCha8Rng::from_seed(expand_seed(seed))
}

fn expand_seed(seed: u64) -> [u8; 32] {
    let mut state = seed;
    let mut expanded = [0_u8; 32];

    for chunk in expanded.chunks_exact_mut(8) {
        state = state.wrapping_add(SPLITMIX64_INCREMENT);
        let mut mixed = state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLITMIX64_MULTIPLIER_1);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLITMIX64_MULTIPLIER_2);
        mixed ^= mixed >> 31;
        chunk.copy_from_slice(&mixed.to_le_bytes());
    }

    expanded
}

fn random_unit_f64(rng: &mut impl RngCore) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let numerator = (rng.next_u64() >> 11) as f64;
    numerator * UNIT_F64_SCALE
}

fn random_centers(n: usize, filter: GephiFilter, rng: &mut impl RngCore) -> Option<Vec<Lab>> {
    let mut centers = Vec::with_capacity(n);

    for _ in 0..n {
        let mut accepted = None;
        for _ in 0..MAX_RANDOM_CENTER_ATTEMPTS {
            let lab = [
                random_unit_f64(rng),
                2.0 * random_unit_f64(rng) - 1.0,
                2.0 * random_unit_f64(rng) - 1.0,
            ];
            if check_color(lab, filter) {
                accepted = Some(lab);
                break;
            }
        }
        centers.push(accepted?);
    }

    Some(centers)
}

const fn quality(colors_count: usize) -> usize {
    if colors_count > 300 {
        2
    } else if colors_count > 200 {
        5
    } else if colors_count > 100 {
        10
    } else if colors_count > 50 {
        25
    } else {
        50
    }
}

fn color_samples(filter: GephiFilter) -> Vec<Lab> {
    let mut samples = Vec::new();

    // R's expand.grid() varies its first argument fastest: l, then a, then b.
    for b_index in 0_u32..=20 {
        let b = grid_signed_value(b_index);
        for a_index in 0_u32..=20 {
            let a = grid_signed_value(a_index);
            for l_index in 0_u32..=20 {
                let l = if l_index == 20 {
                    1.0
                } else {
                    f64::from(l_index) * 0.05
                };
                let lab = [l, a, b];
                if check_color(lab, filter) {
                    samples.push(lab);
                }
            }
        }
    }

    samples
}

fn grid_signed_value(index: u32) -> f64 {
    if index == 20 {
        1.0
    } else {
        -1.0 + f64::from(index) * 0.1
    }
}

fn refine_centers(centers: &mut [Lab], samples: &[Lab], filter: GephiFilter, iterations: usize) {
    let mut samples_closest = vec![0_usize; samples.len()];
    let mut min_distance = vec![f64::INFINITY; samples.len()];
    let mut free_samples = Vec::with_capacity(samples.len());

    for _ in 0..iterations {
        min_distance.fill(f64::INFINITY);

        for (center_index, center) in centers.iter().enumerate() {
            for (sample_index, sample) in samples.iter().enumerate() {
                let distance = distance_sq(*sample, *center);
                if distance < min_distance[sample_index] {
                    min_distance[sample_index] = distance;
                    samples_closest[sample_index] = center_index;
                }
            }
        }

        free_samples.clear();
        free_samples.extend(0..samples.len());

        for (center_index, center) in centers.iter_mut().enumerate() {
            let mut sum = [0.0; 3];
            let mut assigned_count = 0_usize;
            for (sample_index, sample) in samples.iter().enumerate() {
                if samples_closest[sample_index] == center_index {
                    sum[0] += sample[0];
                    sum[1] += sample[1];
                    sum[2] += sample[2];
                    assigned_count += 1;
                }
            }

            let candidate = if assigned_count == 0 {
                [0.0; 3]
            } else {
                #[allow(clippy::cast_precision_loss)]
                let denominator = assigned_count as f64;
                [
                    sum[0] / denominator,
                    sum[1] / denominator,
                    sum[2] / denominator,
                ]
            };

            *center = if assigned_count > 0 && check_color(candidate, filter) {
                candidate
            } else if let Some(sample_index) =
                closest_indexed_sample(samples, free_samples.iter().copied(), candidate)
            {
                samples[sample_index]
            } else {
                samples[closest_sample(samples, candidate)]
            };

            // The R implementation removes rows by exact component equality.
            #[allow(clippy::float_cmp)]
            free_samples.retain(|&sample_index| samples[sample_index] != *center);
        }
    }
}

fn closest_indexed_sample(
    samples: &[Lab],
    indices: impl Iterator<Item = usize>,
    target: Lab,
) -> Option<usize> {
    let mut best = None;
    let mut best_distance = f64::INFINITY;
    for index in indices {
        let distance = distance_sq(samples[index], target);
        if distance < best_distance {
            best = Some(index);
            best_distance = distance;
        }
    }
    best
}

fn closest_sample(samples: &[Lab], target: Lab) -> usize {
    closest_indexed_sample(samples, 0..samples.len(), target).unwrap_or(0)
}

fn distance_sq(left: Lab, right: Lab) -> f64 {
    let l = left[0] - right[0];
    let a = left[1] - right[1];
    let b = left[2] - right[2];
    l * l + a * a + b * b
}

fn farthest_first(colors: &[Lab]) -> Vec<Lab> {
    if colors.len() <= 1 {
        return colors.to_vec();
    }

    let mut sorted = Vec::with_capacity(colors.len());
    sorted.push(0_usize);
    let mut remaining = (1..colors.len()).collect::<Vec<_>>();

    while !remaining.is_empty() {
        let mut best_position = 0_usize;
        let mut best_distance = f64::NEG_INFINITY;

        for (position, &candidate) in remaining.iter().enumerate() {
            let mut nearest_distance = f64::INFINITY;
            for &current in &sorted {
                nearest_distance =
                    nearest_distance.min(distance_sq(colors[candidate], colors[current]));
            }
            if nearest_distance > best_distance {
                best_position = position;
                best_distance = nearest_distance;
            }
        }

        sorted.push(remaining.remove(best_position));
    }

    sorted.into_iter().map(|index| colors[index]).collect()
}

fn check_color(lab: Lab, filter: GephiFilter) -> bool {
    let rgb = lab_to_rgb(lab);
    let [hue, chroma, luminance] = lab_to_hcl(lab);
    let hue_ok = if filter.h_min < filter.h_max {
        hue >= filter.h_min && hue <= filter.h_max
    } else {
        hue >= filter.h_min || hue <= filter.h_max
    };

    rgb.iter()
        .all(|channel| channel.is_finite() && *channel >= 0.0 && *channel < 256.0)
        && hue_ok
        && chroma >= filter.c_min
        && chroma <= filter.c_max
        && luminance >= filter.l_min
        && luminance <= filter.l_max
}

fn lab_to_rgb_color(lab: Lab) -> Option<Rgb> {
    let [red, green, blue] = lab_to_rgb(lab);
    if ![red, green, blue]
        .into_iter()
        .all(|channel| channel.is_finite() && (0.0..256.0).contains(&channel))
    {
        return None;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let packed = ((red as u32) << 16) | ((green as u32) << 8) | blue as u32;
    Some(Rgb::from_hex(packed))
}

fn lab_to_rgb(lab: Lab) -> [f64; 3] {
    xyz_to_rgb(lab_to_xyz(lab))
}

fn lab_to_xyz(lab: Lab) -> [f64; 3] {
    let scaled_luminance = (lab[0] + 0.16) / 1.16;
    [
        0.964_21 * finv(scaled_luminance + lab[1] / 5.0),
        finv(scaled_luminance),
        0.825_19 * finv(scaled_luminance - lab[2] / 2.0),
    ]
}

fn xyz_to_rgb(xyz: [f64; 3]) -> [f64; 3] {
    let mut linear = [
        3.240_6 * xyz[0] - 1.537_2 * xyz[1] - 0.498_6 * xyz[2],
        -0.968_9 * xyz[0] + 1.875_8 * xyz[1] + 0.041_5 * xyz[2],
        0.055_7 * xyz[0] - 0.204 * xyz[1] + 1.057 * xyz[2],
    ];

    if linear.iter().any(|channel| *channel < -0.001)
        || linear.iter().any(|channel| *channel > 1.001)
    {
        linear = linear.map(|channel| channel.clamp(0.0, 1.0));
    }

    linear.map(|channel| round_ties_even(255.0 * correct_channel(channel)))
}

fn lab_to_hcl(lab: Lab) -> [f64; 3] {
    let luminance = (lab[0] - 0.09) / 0.61;
    let radius = (lab[1] * lab[1] + lab[2] * lab[2]).sqrt();
    let chroma = radius / (luminance * 0.311 + 0.125);
    let angle = lab[1].atan2(lab[2]);
    let mut hue = ((std::f64::consts::TAU / 6.0 - angle) / std::f64::consts::TAU) * 360.0;
    if hue < 0.0 {
        hue += 360.0;
    }
    [hue, chroma, luminance]
}

fn finv(value: f64) -> f64 {
    const THRESHOLD: f64 = 6.0 / 29.0;
    if value > THRESHOLD {
        value * value * value
    } else {
        3.0 * THRESHOLD * THRESHOLD * (value - 4.0 / 29.0)
    }
}

fn correct_channel(channel: f64) -> f64 {
    if channel <= 0.003_130_8 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

fn round_ties_even(value: f64) -> f64 {
    let floor = value.floor();
    let fraction = value - floor;
    if fraction < 0.5 {
        floor
    } else if fraction > 0.5 {
        floor + 1.0
    } else if floor.rem_euclid(2.0) == 0.0 {
        floor
    } else {
        floor + 1.0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::{
        check_color, color_samples, distance_sq, expand_seed, farthest_first, lab_to_hcl,
        lab_to_rgb, lab_to_xyz, quality, round_ties_even, GephiFilter, SEEDED_RNG_ALGORITHM,
    };
    use crate::generated;

    #[test]
    fn quality_thresholds_match_r() {
        for (n, expected) in [
            (0, 50),
            (1, 50),
            (50, 50),
            (51, 25),
            (100, 25),
            (101, 10),
            (200, 10),
            (201, 5),
            (300, 5),
            (301, 2),
        ] {
            assert_eq!(quality(n), expected, "n={n}");
        }
    }

    #[test]
    fn splitmix64_seed_expansion_is_locked() {
        assert_eq!(
            expand_seed(42),
            [
                149, 110, 235, 47, 38, 50, 215, 189, 3, 241, 102, 178, 51, 227, 239, 40, 82, 159,
                15, 19, 87, 103, 82, 71, 148, 227, 74, 14, 255, 225, 28, 88,
            ]
        );
        assert!(SEEDED_RNG_ALGORITHM.contains("ChaCha8Rng"));
        assert!(SEEDED_RNG_ALGORITHM.contains("SplitMix64"));
    }

    #[test]
    fn conversion_helpers_match_canonical_formulas() {
        let lab = [0.5, 0.1, -0.2];
        let xyz = lab_to_xyz(lab);
        assert!((xyz[0] - 0.196_988_664_866_108_53).abs() < 1e-15);
        assert!((xyz[1] - 0.184_186_518_512_444_16).abs() < 1e-15);
        assert!((xyz[2] - 0.247_038_790_559_678_53).abs() < 1e-15);
        assert_eq!(lab_to_rgb(lab), [132.0, 113.0, 133.0]);

        let hcl = lab_to_hcl(lab);
        assert!((hcl[0] - 266.565_051_177_078).abs() < 1e-12);
        assert!((hcl[1] - 0.669_415_717_645_696_7).abs() < 1e-15);
        assert!((hcl[2] - 0.672_131_147_540_983_7).abs() < 1e-15);
    }

    #[test]
    fn r_rounding_uses_ties_to_even() {
        assert_eq!(round_ties_even(0.5), 0.0);
        assert_eq!(round_ties_even(1.5), 2.0);
        assert_eq!(round_ties_even(2.5), 2.0);
        assert_eq!(round_ties_even(-0.5), 0.0);
    }

    #[test]
    fn red_roses_hue_range_wraps_zero() {
        let filter = GephiFilter::new(330.0, 20.0, 0.3, 3.0, 0.5, 1.5);
        let samples = color_samples(filter);
        assert!(samples.iter().any(|sample| lab_to_hcl(*sample)[0] >= 330.0));
        assert!(samples.iter().any(|sample| lab_to_hcl(*sample)[0] <= 20.0));

        let middle_hue = color_samples(GephiFilter::new(160.0, 200.0, 0.0, 3.0, 0.0, 1.5))
            .into_iter()
            .find(|sample| {
                let hue = lab_to_hcl(*sample)[0];
                (170.0..=190.0).contains(&hue)
            })
            .unwrap();
        assert!(!check_color(middle_hue, filter));
    }

    #[test]
    fn valid_sample_counts_match_r() {
        let expected = [
            5_708, 752, 160, 113, 65, 471, 1_957, 3_150, 1_508, 768, 259, 438, 923, 786, 147, 379,
            150,
        ];
        let actual = generated::gephi::GEPHI_PALETTES
            .iter()
            .map(|palette| color_samples(palette.filter()).len())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn distance_and_farthest_first_follow_r_ties() {
        assert_eq!(distance_sq([0.0; 3], [1.0, 2.0, 3.0]), 14.0);
        let colors = [[0.0; 3], [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        assert_eq!(
            farthest_first(&colors),
            vec![colors[0], colors[1], colors[2]]
        );
    }
}
