use ggsci::{ContinuousOptions, ItermVariant, PaletteKind};
use ggsci_ggsql::{
    color_array, output_range, scale_continuous, scale_discrete, scale_gephi_discrete_with_seed,
    scale_iterm_discrete, Error, GgsqlPalette, ScaleKind,
};
use ggsql::plot::{scale::OutputRange, ArrayElement, ScaleTypeKind};

#[test]
fn legacy_color_array_has_exact_output() {
    assert_eq!(
        color_array("npg:nrc", 3).unwrap(),
        "['#E64B35', '#4DBBD5', '#00A087']"
    );
}

#[test]
fn legacy_discrete_scale_has_exact_output() {
    assert_eq!(
        scale_discrete("color", "npg:nrc", 3).unwrap(),
        "SCALE DISCRETE color TO ['#E64B35', '#4DBBD5', '#00A087']"
    );
}

#[test]
fn legacy_helpers_reject_continuous_palettes() {
    assert!(matches!(
        color_array("material:blue-grey", 3),
        Err(ggsci::Error::NotDiscretePalette { .. })
    ));
    assert!(matches!(
        scale_discrete("color", "material:blue-grey", 3),
        Err(ggsci::Error::NotDiscretePalette { .. })
    ));
}

#[test]
fn from_spec_records_discrete_semantics() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    assert_eq!(palette.palette_kind(), PaletteKind::Discrete);
}

#[test]
fn from_spec_records_continuous_semantics() {
    let palette = GgsqlPalette::from_spec("material:blue-grey", 256).unwrap();
    assert_eq!(palette.palette_kind(), PaletteKind::Continuous);
    assert_eq!(palette.colors().len(), 256);
}

#[test]
fn from_spec_uses_kind_aware_continuous_sampling() {
    let core = ggsci::palette_by_spec("material:blue-grey").unwrap();
    let palette = GgsqlPalette::from_spec("material:blue-grey", 256).unwrap();

    assert!(core.colors().len() < palette.colors().len());
    assert_eq!(palette.colors(), core.sample(256).unwrap());
}

#[test]
fn continuous_output_supports_large_arrays() {
    assert_eq!(
        GgsqlPalette::from_spec("material:blue-grey", 256)
            .unwrap()
            .colors()
            .len(),
        256
    );
    assert_eq!(
        GgsqlPalette::from_spec("material:blue-grey", 512)
            .unwrap()
            .colors()
            .len(),
        512
    );
}

#[test]
fn continuous_output_can_be_reversed() {
    let forward =
        GgsqlPalette::from_continuous("material:blue-grey", 256, ContinuousOptions::new()).unwrap();
    let reversed = GgsqlPalette::from_continuous(
        "material:blue-grey",
        256,
        ContinuousOptions::new().with_reverse(true),
    )
    .unwrap();
    let expected = forward.colors().iter().rev().copied().collect::<Vec<_>>();

    assert_eq!(reversed.colors(), expected);
    assert_eq!(reversed.palette_kind(), PaletteKind::Continuous);
}

#[test]
fn rose_pine_normal_is_fixed_discrete_output() {
    let palette = GgsqlPalette::from_iterm("Rose Pine", ItermVariant::Normal, 6).unwrap();
    assert_eq!(palette.palette_kind(), PaletteKind::Discrete);
    assert_eq!(palette.colors().len(), 6);
}

#[test]
fn seeded_gephi_output_is_reproducible_and_discrete() {
    let first = GgsqlPalette::from_gephi_with_seed("fancy-light", 12, 42).unwrap();
    let second = GgsqlPalette::from_gephi_with_seed("fancy-light", 12, 42).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.palette_kind(), PaletteKind::Discrete);
}

#[test]
fn palette_kind_maps_to_default_scale_kind() {
    assert_eq!(ScaleKind::from(PaletteKind::Discrete), ScaleKind::Discrete);
    assert_eq!(
        ScaleKind::from(PaletteKind::Continuous),
        ScaleKind::Continuous
    );
}

#[test]
fn formats_sql_array() {
    let palette = GgsqlPalette::from_spec("observable:observable10", 3).unwrap();
    assert_eq!(palette.to_sql_array(), "['#4269D0', '#EFB118', '#FF725C']");
}

#[test]
fn converts_to_typed_output_range() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    let output = palette.to_output_range();

    let OutputRange::Array(elements) = output else {
        panic!("expected an explicit output array");
    };
    assert_eq!(elements.len(), 3);
    assert!(elements
        .iter()
        .all(|element| matches!(element, ArrayElement::String(_))));
}

#[test]
fn converts_owned_palette_into_output_range() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    let output: OutputRange = palette.into();

    assert!(matches!(output, OutputRange::Array(elements) if elements.len() == 3));
}

#[test]
fn converts_borrowed_palette_into_output_range() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    let output: OutputRange = (&palette).into();

    assert!(matches!(output, OutputRange::Array(elements) if elements.len() == 3));
    assert_eq!(palette.colors().len(), 3);
}

#[test]
fn default_scale_clause_uses_source_semantics() {
    let discrete = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    let continuous = GgsqlPalette::from_spec("material:blue-grey", 3).unwrap();

    assert!(discrete
        .to_default_scale_clause("color")
        .unwrap()
        .starts_with("SCALE DISCRETE color TO"));
    assert!(continuous
        .to_default_scale_clause("fill")
        .unwrap()
        .starts_with("SCALE CONTINUOUS fill TO"));
}

#[test]
fn explicit_scale_clause_supports_every_adapter_kind() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();

    for (kind, keyword) in [
        (ScaleKind::Discrete, "DISCRETE"),
        (ScaleKind::Continuous, "CONTINUOUS"),
        (ScaleKind::Binned, "BINNED"),
        (ScaleKind::Ordinal, "ORDINAL"),
    ] {
        let clause = palette.to_scale_clause(kind, "color").unwrap();
        assert!(clause.starts_with(&format!("SCALE {keyword} color TO")));
        assert_eq!(kind.as_sql_keyword(), keyword);
    }
}

#[test]
fn continuous_palette_can_use_explicit_binned_scale() {
    let palette = GgsqlPalette::from_spec("material:blue-grey", 12).unwrap();
    assert!(palette
        .to_scale_clause(ScaleKind::Binned, "fill")
        .unwrap()
        .starts_with("SCALE BINNED fill TO"));
}

#[test]
fn rejects_invalid_aesthetics() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    for aesthetic in [
        "",
        "   ",
        "two words",
        "color!",
        "color.fill",
        "'color'",
        "color; DROP TABLE data",
        "1color",
        "café",
    ] {
        assert!(matches!(
            palette.to_default_scale_clause(aesthetic),
            Err(Error::InvalidAesthetic { .. })
        ));
    }
}

#[test]
fn accepts_valid_aesthetics_and_trims_them() {
    let palette = GgsqlPalette::from_spec("npg:nrc", 3).unwrap();
    for aesthetic in ["color", "fill", "stroke", "pos1", "_custom"] {
        let clause = palette.to_default_scale_clause(aesthetic).unwrap();
        assert!(clause.starts_with(&format!("SCALE DISCRETE {aesthetic} TO")));
    }
    assert!(palette
        .to_default_scale_clause("  color\t")
        .unwrap()
        .starts_with("SCALE DISCRETE color TO"));
}

#[test]
fn convenience_functions_share_typed_resolution() {
    assert!(matches!(
        output_range("material:blue-grey", 256).unwrap(),
        OutputRange::Array(elements) if elements.len() == 256
    ));
    assert!(
        scale_continuous("fill", "material:blue-grey", 16, ContinuousOptions::new())
            .unwrap()
            .starts_with("SCALE CONTINUOUS fill TO")
    );
    assert!(
        scale_iterm_discrete("stroke", "Rose Pine", ItermVariant::Normal, 6)
            .unwrap()
            .starts_with("SCALE DISCRETE stroke TO")
    );
    assert_eq!(
        scale_gephi_discrete_with_seed("color", "fancy-light", 8, 42).unwrap(),
        scale_gephi_discrete_with_seed("color", "fancy-light", 8, 42).unwrap()
    );
}

#[test]
fn from_colors_accessors_preserve_values_and_kind() {
    let colors = ggsci::palette_by_spec("npg:nrc").unwrap().take(3).unwrap();
    let palette = GgsqlPalette::from_colors(colors.clone(), PaletteKind::Discrete);

    assert_eq!(palette.colors(), colors);
    assert_eq!(palette.into_colors(), colors);
}

#[test]
fn ggsql_parses_generated_discrete_clause() {
    let clause = GgsqlPalette::from_spec("observable:observable10", 3)
        .unwrap()
        .to_default_scale_clause("color")
        .unwrap();
    let query = format!("VISUALISE foo AS x, group AS color\nDRAW point\n{clause}");
    let plots = ggsql::parser::parse_query(&query).unwrap();

    assert_eq!(plots.len(), 1);
    assert_eq!(plots[0].scales.len(), 1);
    assert_eq!(
        plots[0].scales[0]
            .scale_type
            .as_ref()
            .unwrap()
            .scale_type_kind(),
        ScaleTypeKind::Discrete
    );
    assert!(matches!(
        plots[0].scales[0].output_range.as_ref(),
        Some(OutputRange::Array(elements)) if elements.len() == 3
    ));
}

#[test]
fn ggsql_parses_generated_continuous_clause() {
    let clause = GgsqlPalette::from_spec("material:blue-grey", 256)
        .unwrap()
        .to_default_scale_clause("color")
        .unwrap();
    let query = format!("VISUALISE foo AS x, value AS color\nDRAW point\n{clause}");
    let plots = ggsql::parser::parse_query(&query).unwrap();

    assert_eq!(plots.len(), 1);
    assert_eq!(plots[0].scales.len(), 1);
    assert_eq!(
        plots[0].scales[0]
            .scale_type
            .as_ref()
            .unwrap()
            .scale_type_kind(),
        ScaleTypeKind::Continuous
    );
    let Some(OutputRange::Array(elements)) = plots[0].scales[0].output_range.as_ref() else {
        panic!("expected parsed explicit output array");
    };
    assert_eq!(elements.len(), 256);
    assert!(elements
        .iter()
        .all(|element| matches!(element, ArrayElement::String(_))));
}
