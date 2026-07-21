use ggsci::ContinuousOptions;
use ggsci_ggsql::{GgsqlPalette, ScaleKind};

fn main() -> Result<(), ggsci_ggsql::Error> {
    let palette = GgsqlPalette::from_continuous(
        "material:blue-grey",
        256,
        ContinuousOptions::new().with_reverse(true),
    )?;

    let clause = palette.to_default_scale_clause("fill")?;
    assert!(clause.starts_with("SCALE CONTINUOUS fill TO"));

    let binned = palette.to_scale_clause(ScaleKind::Binned, "fill")?;
    assert!(binned.starts_with("SCALE BINNED fill TO"));

    Ok(())
}
