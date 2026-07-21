#![allow(clippy::float_cmp)]

use std::{collections::HashSet, thread};

use ggsci::{
    gephi_palette, gephi_palette_count, gephi_palette_names, gephi_palettes, palettes,
    palettes_by_kind, total_color_count, Error, GephiFilter, GephiPalette, PaletteKind, Rgb,
};

const GEPHI_NAMES: [&str; 17] = [
    "default",
    "fancy_light",
    "fancy_dark",
    "shades",
    "tarnish",
    "pastel",
    "pimp",
    "intense",
    "fluo",
    "red_roses",
    "ochre_sand",
    "yellow_lime",
    "green_mint",
    "ice_cube",
    "blue_ocean",
    "indigo_night",
    "purple_wine",
];

const DEFAULT_N1: [&str; 1] = ["#B1A38E"];
const DEFAULT_N5: [&str; 5] = ["#00CCF6", "#FF863B", "#FD88F0", "#64C938", "#766C5E"];
const DEFAULT_N10: [&str; 10] = [
    "#00DBFF", "#FF7739", "#63D400", "#FF83FF", "#786D38", "#00C994", "#FF6F90", "#87B5FF",
    "#E6A900", "#536F89",
];
const FANCY_LIGHT_N10: [&str; 10] = [
    "#DDD4FF", "#F8E568", "#77EDD0", "#FFB5BB", "#ACEA91", "#FFC384", "#FFC0F6", "#93E5FF",
    "#FBB7BC", "#EDC076",
];
const RED_ROSES_N10: [&str; 10] = [
    "#FFA674", "#B7655B", "#FF0C63", "#FF751F", "#FF9DA8", "#FF4614", "#E24D3B", "#FF6065",
    "#FF7CAE", "#FFBBA2",
];
const BLUE_OCEAN_N10: [&str; 10] = [
    "#062642", "#3077D6", "#66729C", "#0082CD", "#696A72", "#005C97", "#4359A2", "#30343E",
    "#24436A", "#387190",
];

#[test]
fn generated_registry_has_canonical_count_order_and_kind() {
    assert_eq!(gephi_palette_count(), 17);
    assert_eq!(gephi_palettes().len(), 17);
    assert_eq!(gephi_palette_names().collect::<Vec<_>>(), GEPHI_NAMES);
    assert!(gephi_palettes()
        .iter()
        .all(|palette| palette.kind() == PaletteKind::Discrete));

    let unique = gephi_palette_names().collect::<HashSet<_>>();
    assert_eq!(unique.len(), GEPHI_NAMES.len());
}

#[test]
fn representative_generated_filters_match_upstream() {
    assert_eq!(
        gephi_palette("default").unwrap().filter(),
        GephiFilter::new(0.0, 360.0, 0.0, 3.0, 0.0, 1.5)
    );
    assert_eq!(
        gephi_palette("fancy_light").unwrap().filter(),
        GephiFilter::new(0.0, 360.0, 0.4, 1.2, 1.0, 1.5)
    );
    assert_eq!(
        gephi_palette("red_roses").unwrap().filter(),
        GephiFilter::new(330.0, 20.0, 0.3, 3.0, 0.5, 1.5)
    );
    assert_eq!(
        gephi_palette("blue_ocean").unwrap().filter(),
        GephiFilter::new(220.0, 260.0, 0.2, 2.5, 0.0, 0.8)
    );

    let filter = gephi_palette("red_roses").unwrap().filter();
    assert_eq!(filter.h_min(), 330.0);
    assert_eq!(filter.h_max(), 20.0);
    assert_eq!(filter.c_min(), 0.3);
    assert_eq!(filter.c_max(), 3.0);
    assert_eq!(filter.l_min(), 0.5);
    assert_eq!(filter.l_max(), 1.5);
}

#[test]
fn normalized_lookup_preserves_canonical_names() {
    for name in ["fancy_light", "fancy-light", "FANCY LIGHT", "fancy\tlight"] {
        assert_eq!(gephi_palette(name).unwrap().name(), "fancy_light");
    }
    for name in ["red_roses", "red-roses", "RED ROSES"] {
        assert_eq!(gephi_palette(name).unwrap().name(), "red_roses");
    }
}

#[test]
fn unknown_lookup_has_specific_error() {
    assert_eq!(
        gephi_palette("missing"),
        Err(Error::UnknownGephiPalette {
            palette: "missing".to_owned(),
        })
    );
}

#[test]
fn zero_colors_returns_early_and_empty() {
    let palette = gephi_palette("default").unwrap();
    assert!(palette.generate(0).unwrap().is_empty());
    assert!(palette.generate_with_seed(0, 42).unwrap().is_empty());
    assert!(palette
        .generate_rgba_with_seed(0, 0.5, 42)
        .unwrap()
        .is_empty());

    let malformed = GephiPalette::new(
        usize::MAX,
        "malformed",
        GephiFilter::new(f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0),
    );
    assert!(malformed.generate_with_seed(0, 42).unwrap().is_empty());
}

#[test]
fn seeded_generation_is_reproducible_and_seed_sensitive() {
    let palette = gephi_palette("default").unwrap();
    let first = palette.generate_with_seed(10, 42).unwrap();
    let second = palette.generate_with_seed(10, 42).unwrap();
    let different = palette.generate_with_seed(10, 43).unwrap();
    assert_eq!(first, second);
    assert_ne!(first, different);
}

#[test]
fn seeded_output_matches_locked_goldens() {
    assert_eq!(seeded_hex("default", 1), DEFAULT_N1);
    assert_eq!(seeded_hex("default", 5), DEFAULT_N5);
    assert_eq!(seeded_hex("default", 10), DEFAULT_N10);
    assert_eq!(seeded_hex("fancy_light", 10), FANCY_LIGHT_N10);
    assert_eq!(seeded_hex("red_roses", 10), RED_ROSES_N10);
    assert_eq!(seeded_hex("blue_ocean", 10), BLUE_OCEAN_N10);
}

#[test]
fn every_filter_generates_valid_rgb_channels() {
    for palette in gephi_palettes() {
        let colors = palette.generate_with_seed(10, 2026).unwrap();
        assert_eq!(colors.len(), 10, "{}", palette.name());
        for color in colors {
            assert!(color.to_u32() <= 0x00FF_FFFF);
            assert_eq!(
                color.to_u32(),
                (u32::from(color.r()) << 16) | (u32::from(color.g()) << 8) | u32::from(color.b())
            );
            assert_eq!(Rgb::from_hex(color.to_u32()), color);
        }
    }
}

#[test]
fn nondeterministic_generation_uses_the_convenience_api() {
    let colors = gephi_palette("fancy-dark").unwrap().generate(3).unwrap();
    assert_eq!(colors.len(), 3);
}

#[test]
fn rgba_applies_alpha_after_rgb_generation() {
    let palette = gephi_palette("default").unwrap();
    let rgb = palette.generate_with_seed(5, 42).unwrap();
    let opaque = palette.generate_rgba_with_seed(5, 1.0, 42).unwrap();
    let translucent = palette.generate_rgba_with_seed(5, 0.5, 42).unwrap();

    for ((rgb, opaque), translucent) in rgb.iter().zip(&opaque).zip(&translucent) {
        assert_eq!(
            (opaque.r(), opaque.g(), opaque.b()),
            (rgb.r(), rgb.g(), rgb.b())
        );
        assert_eq!(opaque.a(), 255);
        assert_eq!(
            (translucent.r(), translucent.g(), translucent.b()),
            (rgb.r(), rgb.g(), rgb.b())
        );
        assert_eq!(translucent.a(), 127);
    }
}

#[test]
fn rgba_rejects_invalid_alpha() {
    let palette = gephi_palette("default").unwrap();
    for alpha in [0.0, -0.1, 1.1, f32::INFINITY, f32::NAN] {
        assert!(matches!(
            palette.generate_rgba_with_seed(1, alpha, 42),
            Err(Error::InvalidAlpha { .. })
        ));
    }
}

#[test]
fn malformed_or_empty_filters_return_generation_errors() {
    let malformed = GephiPalette::new(
        usize::MAX,
        "malformed",
        GephiFilter::new(f64::NAN, 360.0, 0.0, 3.0, 0.0, 1.5),
    );
    assert!(matches!(
        malformed.generate_with_seed(1, 42),
        Err(Error::GephiGenerationFailed {
            palette: "malformed",
            ..
        })
    ));

    let empty = GephiPalette::new(
        usize::MAX,
        "empty",
        GephiFilter::new(0.0, 360.0, 100.0, 101.0, 0.0, 1.5),
    );
    assert!(matches!(
        empty.generate_with_seed(1, 42),
        Err(Error::GephiGenerationFailed {
            palette: "empty",
            ..
        })
    ));
}

#[test]
fn concurrent_seeded_generation_is_safe_and_deterministic() {
    let expected = gephi_palette("intense")
        .unwrap()
        .generate_with_seed(12, 7)
        .unwrap();
    let handles = (0..8)
        .map(|_| {
            thread::spawn(|| {
                gephi_palette("intense")
                    .unwrap()
                    .generate_with_seed(12, 7)
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        assert_eq!(handle.join().unwrap(), expected);
    }
}

#[test]
fn core_registry_counts_and_membership_remain_unchanged() {
    assert_eq!(palettes().len(), 86);
    assert_eq!(total_color_count(), 946);
    assert_eq!(palettes_by_kind(PaletteKind::Discrete).count(), 33);
    assert_eq!(palettes_by_kind(PaletteKind::Continuous).count(), 53);
    assert!(palettes().iter().all(|palette| palette.family() != "gephi"));
}

#[test]
fn palette_kind_has_no_storage_mechanism_variants() {
    let public_sources = [
        include_str!("../src/lib.rs"),
        include_str!("../src/palette.rs"),
        include_str!("../src/iterm.rs"),
        include_str!("../src/gephi.rs"),
    ];
    let generated_sources = [
        include_str!("../src/generated/palettes.rs"),
        include_str!("../src/generated/iterm.rs"),
        include_str!("../src/generated/gephi.rs"),
    ];
    let forbidden = [
        ["PaletteKind::", "Static"].concat(),
        ["PaletteKind::", "Generative"].concat(),
    ];

    for source in public_sources.into_iter().chain(generated_sources) {
        for variant in &forbidden {
            assert!(!source.contains(variant));
        }
    }
}

fn seeded_hex(name: &str, n: usize) -> Vec<String> {
    gephi_palette(name)
        .unwrap()
        .generate_with_seed(n, 42)
        .unwrap()
        .into_iter()
        .map(Rgb::to_hex_string)
        .collect()
}
