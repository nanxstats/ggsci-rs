use std::{collections::HashSet, str::FromStr};

use ggsci::{
    iterm_palette, iterm_palette_count, iterm_palette_names, iterm_palettes,
    iterm_total_color_count, palettes, palettes_by_kind, total_color_count, Error, ItermChannel,
    ItermVariant, PaletteKind, ITERM_CHANNELS,
};

const ROSE_PINE: [&str; 6] = [
    "#9CCFD8", "#F6C177", "#EB6F92", "#EBBCBA", "#31748F", "#C4A7E7",
];

const DRACULA_NORMAL: [&str; 6] = [
    "#BD93F9", "#F1FA8C", "#FF5555", "#8BE9FD", "#50FA7B", "#FF79C6",
];

const DRACULA_BRIGHT: [&str; 6] = [
    "#D6ACFF", "#FFFFA5", "#FF6E6E", "#A4FFFF", "#69FF94", "#FF92DF",
];

#[test]
fn generated_iterm_registry_has_expected_counts_and_shape() {
    assert_eq!(iterm_palette_count(), 551);
    assert_eq!(iterm_palettes().len(), 551);
    assert_eq!(iterm_total_color_count(), 6_612);

    for palette in iterm_palettes() {
        assert_eq!(palette.kind(), PaletteKind::Discrete);
        assert_eq!(palette.colors(ItermVariant::Normal).len(), 6);
        assert_eq!(palette.colors(ItermVariant::Bright).len(), 6);
    }
}

#[test]
fn channels_have_canonical_order_names_and_indices() {
    assert_eq!(
        ITERM_CHANNELS,
        [
            ItermChannel::Blue,
            ItermChannel::Yellow,
            ItermChannel::Red,
            ItermChannel::Cyan,
            ItermChannel::Green,
            ItermChannel::Magenta,
        ]
    );

    let names = ITERM_CHANNELS.map(ItermChannel::as_str);
    assert_eq!(names, ["Blue", "Yellow", "Red", "Cyan", "Green", "Magenta"]);
    for (index, channel) in ITERM_CHANNELS.into_iter().enumerate() {
        assert_eq!(channel.index(), index);
    }
}

#[test]
fn canonical_names_are_unique_and_keep_upstream_order() {
    let names = iterm_palette_names().collect::<Vec<_>>();
    let unique = names.iter().copied().collect::<HashSet<_>>();
    assert_eq!(unique.len(), names.len());
    assert_eq!(names.first(), Some(&"0x96f"));
    assert_eq!(names.last(), Some(&"Zenwritten Light"));
}

#[test]
fn lookup_normalizes_case_and_separators_but_preserves_punctuation() {
    for name in ["Rose Pine", "rose-pine", "ROSE_PINE", "rose\tpine"] {
        assert_eq!(iterm_palette(name).unwrap().name(), "Rose Pine");
    }
    assert_eq!(iterm_palette("0x96f").unwrap().name(), "0x96f");

    let dracula = iterm_palette("Dracula").unwrap();
    let dracula_plus = iterm_palette("Dracula+").unwrap();
    assert_eq!(dracula.name(), "Dracula");
    assert_eq!(dracula_plus.name(), "Dracula+");
    assert_ne!(dracula, dracula_plus);
}

#[test]
fn rose_pine_matches_r_for_both_variants() {
    let rose_pine = iterm_palette("Rose Pine").unwrap();
    assert_eq!(
        rose_pine.take_hex(ItermVariant::Normal, 6).unwrap(),
        ROSE_PINE
    );
    assert_eq!(
        rose_pine.take_hex(ItermVariant::Bright, 6).unwrap(),
        ROSE_PINE
    );

    for channel in ITERM_CHANNELS {
        assert_eq!(
            rose_pine.color(ItermVariant::Normal, channel),
            rose_pine.colors(ItermVariant::Normal)[channel.index()]
        );
    }
}

#[test]
fn dracula_matches_r_and_variants_differ() {
    let dracula = iterm_palette("Dracula").unwrap();
    assert_eq!(
        dracula.take_hex(ItermVariant::Normal, 6).unwrap(),
        DRACULA_NORMAL
    );
    assert_eq!(
        dracula.take_hex(ItermVariant::Bright, 6).unwrap(),
        DRACULA_BRIGHT
    );
    assert_ne!(
        dracula.colors(ItermVariant::Normal),
        dracula.colors(ItermVariant::Bright)
    );
}

#[test]
fn take_obeys_fixed_discrete_length_without_cycling() {
    let rose_pine = iterm_palette("Rose Pine").unwrap();
    assert!(rose_pine.take(ItermVariant::Normal, 0).unwrap().is_empty());
    assert_eq!(
        rose_pine.take(ItermVariant::Normal, 6).unwrap(),
        rose_pine.colors(ItermVariant::Normal)
    );
    assert_eq!(
        rose_pine.take(ItermVariant::Normal, 7),
        Err(Error::TooManyItermColorsRequested {
            palette: "Rose Pine",
            variant: "normal",
            requested: 7,
            available: 6,
        })
    );
}

#[test]
fn cycle_is_explicitly_infinite() {
    let palette = iterm_palette("0x96f").unwrap();
    let cycled = palette
        .cycle(ItermVariant::Bright)
        .take(8)
        .collect::<Vec<_>>();
    assert_eq!(cycled[..6], palette.colors(ItermVariant::Bright)[..]);
    assert_eq!(cycled[6], cycled[0]);
    assert_eq!(cycled[7], cycled[1]);
}

#[test]
fn variant_parsing_is_case_insensitive() {
    assert_eq!(ItermVariant::parse("normal").unwrap(), ItermVariant::Normal);
    assert_eq!(ItermVariant::parse("BRIGHT").unwrap(), ItermVariant::Bright);
    assert_eq!(
        ItermVariant::from_str(" Normal ").unwrap(),
        ItermVariant::Normal
    );
    assert_eq!(ItermVariant::Normal.as_str(), "normal");
    assert_eq!(ItermVariant::Bright.to_string(), "bright");
}

#[test]
fn lookup_and_variant_errors_are_specific() {
    assert_eq!(
        iterm_palette("missing"),
        Err(Error::UnknownItermPalette {
            palette: "missing".to_owned(),
        })
    );
    assert_eq!(
        ItermVariant::parse("dim"),
        Err(Error::UnknownItermVariant {
            variant: "dim".to_owned(),
        })
    );
}

#[test]
fn rgba_follows_r_alpha_semantics() {
    let palette = iterm_palette("Rose Pine").unwrap();
    let opaque = palette.take_rgba(ItermVariant::Normal, 6, 1.0).unwrap();
    assert!(opaque.iter().all(|color| color.a() == 255));

    let translucent = palette.take_rgba(ItermVariant::Bright, 6, 0.5).unwrap();
    assert!(translucent.iter().all(|color| color.a() == 127));

    for alpha in [0.0, -0.1, 1.1, f32::INFINITY, f32::NAN] {
        assert!(matches!(
            palette.take_rgba(ItermVariant::Normal, 1, alpha),
            Err(Error::InvalidAlpha { .. })
        ));
    }
}

#[test]
fn core_registry_counts_and_membership_remain_unchanged() {
    assert_eq!(palettes().len(), 86);
    assert_eq!(total_color_count(), 946);
    assert_eq!(palettes_by_kind(PaletteKind::Discrete).count(), 33);
    assert_eq!(palettes_by_kind(PaletteKind::Continuous).count(), 53);

    let iterm_names = iterm_palette_names().collect::<HashSet<_>>();
    assert!(palettes().iter().all(|palette| {
        palette.family() != "iterm"
            && !iterm_names.contains(palette.family())
            && !iterm_names.contains(palette.variant())
    }));
}
