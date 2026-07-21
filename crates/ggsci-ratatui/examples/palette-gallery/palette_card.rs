use ggsci_ratatui::ColorMode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::catalog::PaletteEntry;

pub struct PaletteCard<'a> {
    entry: &'a PaletteEntry,
    mode: ColorMode,
}

impl<'a> PaletteCard<'a> {
    pub const fn new(entry: &'a PaletteEntry, mode: ColorMode) -> Self {
        Self { entry, mode }
    }
}

impl Widget for PaletteCard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::DarkGray))
            .title(self.entry.title.as_str())
            .title_style(Style::new().fg(Color::White));
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.is_empty() {
            return;
        }

        let detail_y = inner.bottom().saturating_sub(1);
        buf.set_stringn(
            inner.x,
            detail_y,
            self.entry.detail.as_str(),
            usize::from(inner.width),
            Style::new().fg(Color::Gray),
        );

        let available_strip_rows = inner.height.saturating_sub(1);
        if available_strip_rows == 0 || self.entry.strips.is_empty() {
            return;
        }

        if self.entry.strips.len() == 1 {
            let rows = available_strip_rows.min(2);
            for row in 0..rows {
                render_strip(
                    &self.entry.strips[0],
                    self.mode,
                    inner.x,
                    inner.y.saturating_add(row),
                    inner.width,
                    buf,
                );
            }
        } else {
            for (row, strip) in self
                .entry
                .strips
                .iter()
                .take(usize::from(available_strip_rows))
                .enumerate()
            {
                let row = u16::try_from(row).unwrap_or(u16::MAX);
                render_strip(
                    strip,
                    self.mode,
                    inner.x,
                    inner.y.saturating_add(row),
                    inner.width,
                    buf,
                );
            }
        }
    }
}

fn render_strip(
    strip: &crate::catalog::ColorStrip,
    mode: ColorMode,
    x: u16,
    y: u16,
    width: u16,
    buf: &mut Buffer,
) {
    let label_width = if let Some(label) = strip.label {
        buf.set_stringn(
            x,
            y,
            format!("{label} "),
            usize::from(width.min(2)),
            Style::new().fg(Color::White),
        );
        width.min(2)
    } else {
        0
    };
    let swatch_x = x.saturating_add(label_width);
    let swatch_width = width.saturating_sub(label_width);
    let colors = strip.colors(mode);
    if swatch_width == 0 || colors.is_empty() {
        return;
    }

    for offset in 0..swatch_width {
        let color_index = usize::from(offset) * colors.len() / usize::from(swatch_width);
        if let Some(cell) = buf.cell_mut((swatch_x.saturating_add(offset), y)) {
            cell.set_symbol(" ").set_bg(colors[color_index]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ColorStrip, PaletteEntry};

    fn entry(strips: Vec<ColorStrip>) -> PaletteEntry {
        PaletteEntry {
            title: "family:variant".to_owned(),
            detail: "discrete · 4 colors".to_owned(),
            strips,
        }
    }

    fn strip(label: Option<&'static str>) -> ColorStrip {
        ColorStrip {
            label,
            truecolor: vec![Color::Red, Color::Green, Color::Blue, Color::Yellow],
            ansi256: vec![
                Color::Indexed(196),
                Color::Indexed(46),
                Color::Indexed(21),
                Color::Indexed(226),
            ],
        }
    }

    fn buffer_text(buffer: &Buffer) -> String {
        let mut text = String::new();
        for y in buffer.area.top()..buffer.area.bottom() {
            for x in buffer.area.left()..buffer.area.right() {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn title_detail_and_every_discrete_color_render() {
        let entry = entry(vec![strip(None)]);
        let area = Rect::new(0, 0, 24, 5);
        let mut buffer = Buffer::empty(area);
        PaletteCard::new(&entry, ColorMode::TrueColor).render(area, &mut buffer);

        let text = buffer_text(&buffer);
        assert!(text.contains("family:variant"));
        assert!(text.contains("discrete · 4 colors"));
        for color in [Color::Red, Color::Green, Color::Blue, Color::Yellow] {
            assert!(buffer.content.iter().any(|cell| cell.bg == color));
        }
    }

    #[test]
    fn iterm_card_renders_both_labels() {
        let entry = entry(vec![strip(Some("N")), strip(Some("B"))]);
        let area = Rect::new(0, 0, 24, 5);
        let mut buffer = Buffer::empty(area);
        PaletteCard::new(&entry, ColorMode::TrueColor).render(area, &mut buffer);
        let text = buffer_text(&buffer);
        assert!(text.contains("N "));
        assert!(text.contains("B "));
    }

    #[test]
    fn zero_width_and_tiny_areas_do_not_panic() {
        let entry = entry(vec![strip(None)]);
        for area in [Rect::ZERO, Rect::new(0, 0, 1, 1), Rect::new(0, 0, 2, 2)] {
            let mut buffer = Buffer::empty(area);
            PaletteCard::new(&entry, ColorMode::TrueColor).render(area, &mut buffer);
        }
    }
}
