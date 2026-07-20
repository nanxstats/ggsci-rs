use ggsci::{
    palette, palettes, palettes_by_kind, total_color_count, ContinuousOptions, Error, PaletteKind,
    Rgb, Rgba,
};

#[allow(clippy::unreadable_literal)]
mod generated {
    include!("generated/continuous_fixtures.rs");
}

#[test]
fn generated_registry_has_expected_kind_counts() {
    assert_eq!(palettes().len(), 86);
    assert_eq!(total_color_count(), 946);
    assert_eq!(palettes_by_kind(PaletteKind::Discrete).count(), 33);
    assert_eq!(palettes_by_kind(PaletteKind::Continuous).count(), 53);

    let discrete_colors = palettes_by_kind(PaletteKind::Discrete)
        .map(ggsci::Palette::len)
        .sum::<usize>();
    let continuous_anchors = palettes_by_kind(PaletteKind::Continuous)
        .map(ggsci::Palette::len)
        .sum::<usize>();
    assert_eq!(discrete_colors, 403);
    assert_eq!(continuous_anchors, 543);
}

#[test]
fn representative_palettes_have_scale_semantic_kinds() {
    assert_eq!(palette("npg", "nrc").unwrap().kind(), PaletteKind::Discrete);
    assert_eq!(
        palette("gsea", "default").unwrap().kind(),
        PaletteKind::Continuous
    );
    assert_eq!(
        palette("bs5", "blue").unwrap().kind(),
        PaletteKind::Continuous
    );
    assert_eq!(
        palette("material", "blue-grey").unwrap().kind(),
        PaletteKind::Continuous
    );
    assert_eq!(
        palette("tw3", "slate").unwrap().kind(),
        PaletteKind::Continuous
    );
}

#[test]
fn palette_kind_has_canonical_names() {
    assert_eq!(PaletteKind::Discrete.as_str(), "discrete");
    assert_eq!(PaletteKind::Continuous.as_str(), "continuous");
    assert_eq!(PaletteKind::Discrete.to_string(), "discrete");
    assert_eq!(PaletteKind::Continuous.to_string(), "continuous");
}

#[test]
fn every_continuous_variant_matches_r_golden_fixtures() {
    assert_eq!(generated::CONTINUOUS_FIXTURES.len(), 53 * 7);
    for fixture in generated::CONTINUOUS_FIXTURES {
        let palette = palette(fixture.family, fixture.variant).unwrap();
        let expected = fixture
            .forward
            .iter()
            .copied()
            .map(Rgb::from_hex)
            .collect::<Vec<_>>();
        let actual = palette.interpolate(fixture.n).unwrap();
        assert_eq!(
            actual, expected,
            "forward mismatch for {}:{}, n={}",
            fixture.family, fixture.variant, fixture.n
        );

        let expected_reverse = fixture
            .reverse
            .iter()
            .copied()
            .map(Rgb::from_hex)
            .collect::<Vec<_>>();
        let actual_reverse = palette
            .interpolate_with(fixture.n, ContinuousOptions::new().with_reverse(true))
            .unwrap();
        assert_eq!(
            actual_reverse, expected_reverse,
            "reverse mismatch for {}:{}, n={}",
            fixture.family, fixture.variant, fixture.n
        );
    }
}

#[test]
fn representative_alpha_outputs_match_r_golden_fixtures() {
    assert_eq!(generated::ALPHA_FIXTURES.len(), 4 * 2);
    for fixture in generated::ALPHA_FIXTURES {
        let expected = fixture
            .colors
            .iter()
            .copied()
            .map(Rgba::from_hex)
            .collect::<Vec<_>>();
        let actual = palette(fixture.family, fixture.variant)
            .unwrap()
            .interpolate_rgba_with(
                7,
                0.6,
                ContinuousOptions::new().with_reverse(fixture.reverse),
            )
            .unwrap();
        assert_eq!(
            actual, expected,
            "alpha mismatch for {}:{}, reverse={}",
            fixture.family, fixture.variant, fixture.reverse
        );
    }
}

#[test]
fn discrete_operations_validate_kind_and_length() {
    let discrete = palette("npg", "nrc").unwrap();
    assert!(discrete.is_discrete());
    assert!(!discrete.is_continuous());
    assert!(discrete.take(0).unwrap().is_empty());
    assert_eq!(
        discrete.take_hex(3).unwrap(),
        ["#E64B35", "#4DBBD5", "#00A087"]
    );
    assert!(matches!(
        discrete.take(discrete.len() + 1),
        Err(Error::TooManyColorsRequested { .. })
    ));

    let continuous = palette("gsea", "default").unwrap();
    assert!(matches!(
        continuous.take(1),
        Err(Error::NotDiscretePalette { .. })
    ));
    assert!(matches!(
        continuous.take_hex(1),
        Err(Error::NotDiscretePalette { .. })
    ));
}

#[test]
fn cycle_is_infinite_only_for_discrete_palettes() {
    let discrete = palette("npg", "nrc").unwrap();
    let cycled = discrete
        .cycle()
        .unwrap()
        .take(discrete.len() + 2)
        .collect::<Vec<_>>();
    assert_eq!(cycled[0], discrete.colors()[0]);
    assert_eq!(cycled[discrete.len()], discrete.colors()[0]);
    assert_eq!(cycled[discrete.len() + 1], discrete.colors()[1]);

    let continuous = palette("gsea", "default").unwrap();
    assert!(matches!(
        continuous.cycle(),
        Err(Error::NotDiscretePalette { .. })
    ));
}

#[test]
fn continuous_operations_validate_kind_and_alpha() {
    let continuous = palette("material", "blue-grey").unwrap();
    assert!(continuous.is_continuous());
    assert!(!continuous.is_discrete());
    assert!(continuous.interpolate(0).unwrap().is_empty());

    let discrete = palette("npg", "nrc").unwrap();
    assert!(matches!(
        discrete.interpolate(3),
        Err(Error::NotContinuousPalette { .. })
    ));
    assert!(matches!(
        discrete.interpolate_hex(3),
        Err(Error::NotContinuousPalette { .. })
    ));

    for alpha in [0.0, -0.1, 1.1, f32::INFINITY, f32::NAN] {
        assert!(matches!(
            continuous.interpolate_rgba(3, alpha),
            Err(Error::InvalidAlpha { .. })
        ));
    }

    // R's rgb() truncates a scaled alpha channel. Rgb::with_alpha() retains
    // its existing round-to-nearest behavior independently.
    assert_eq!(continuous.interpolate_rgba(1, 0.5).unwrap()[0].a(), 127);
}

#[test]
fn sample_dispatches_by_palette_kind() {
    let discrete = palette("npg", "nrc").unwrap();
    assert_eq!(discrete.sample(3).unwrap(), discrete.take(3).unwrap());
    assert_eq!(
        discrete.sample_hex(3).unwrap(),
        discrete.take_hex(3).unwrap()
    );
    assert!(discrete.sample(0).unwrap().is_empty());

    let continuous = palette("material", "blue-grey").unwrap();
    let n = continuous.len() + 17;
    assert_eq!(
        continuous.sample(n).unwrap(),
        continuous.interpolate(n).unwrap()
    );
    assert_eq!(
        continuous.sample_hex(n).unwrap(),
        continuous.interpolate_hex(n).unwrap()
    );
    assert!(continuous.sample(0).unwrap().is_empty());
}

#[test]
fn interpolation_can_preserve_canonical_endpoints() {
    // Bootstrap orange is also an R round-trip case whose endpoint channels
    // remain exactly equal to its canonical anchors.
    let continuous = palette("bs5", "orange").unwrap();
    let endpoints = continuous.interpolate(2).unwrap();
    assert_eq!(endpoints[0], continuous.colors()[0]);
    assert_eq!(endpoints[1], continuous.colors()[continuous.len() - 1]);
}

#[test]
fn continuous_options_default_to_forward_order() {
    let options = ContinuousOptions::new();
    assert!(!options.reverse());
    assert_eq!(options, ContinuousOptions::default());
    assert!(options.with_reverse(true).reverse());
}
