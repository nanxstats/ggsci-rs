use ggsci_ratatui::{ColorMode, colors, foreground_styles};

fn main() -> Result<(), ggsci::Error> {
    let colors = colors("observable:observable10", 5, ColorMode::TrueColor)?;
    let styles = foreground_styles(&colors);

    assert_eq!(colors.len(), 5);
    assert_eq!(styles.len(), 5);

    Ok(())
}
