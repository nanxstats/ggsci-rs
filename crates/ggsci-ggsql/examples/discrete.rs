use ggsci_ggsql::GgsqlPalette;

fn main() -> Result<(), ggsci_ggsql::Error> {
    let palette = GgsqlPalette::from_spec("observable:observable10", 3)?;

    assert_eq!(palette.to_sql_array(), "['#4269D0', '#EFB118', '#FF725C']");
    println!("{}", palette.to_default_scale_clause("color")?);

    Ok(())
}
