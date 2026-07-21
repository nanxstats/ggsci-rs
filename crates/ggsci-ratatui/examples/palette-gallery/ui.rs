use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs},
};

use crate::{app::App, palette_card::PaletteCard};

pub const CARD_MIN_WIDTH: u16 = 32;
pub const CARD_HEIGHT: u16 = 5;
pub const GRID_GAP: u16 = 1;
pub const MIN_TERMINAL_WIDTH: u16 = 48;
pub const MIN_TERMINAL_HEIGHT: u16 = 12;
const SCROLLBAR_WIDTH: u16 = 1;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GridMetrics {
    pub columns: usize,
    pub visible_rows: usize,
    pub total_rows: usize,
    pub max_scroll_row: usize,
}

impl GridMetrics {
    pub fn new(width: u16, height: u16, item_count: usize) -> Self {
        if width == 0 || height == 0 {
            return Self::default();
        }

        let columns = usize::from(
            width
                .saturating_add(GRID_GAP)
                .checked_div(CARD_MIN_WIDTH.saturating_add(GRID_GAP))
                .unwrap_or(0)
                .max(1),
        );
        let visible_rows = usize::from(
            height
                .saturating_add(GRID_GAP)
                .checked_div(CARD_HEIGHT.saturating_add(GRID_GAP))
                .unwrap_or(0)
                .max(1),
        );
        let total_rows = item_count.saturating_add(columns.saturating_sub(1)) / columns;
        let max_scroll_row = total_rows.saturating_sub(visible_rows);

        Self {
            columns,
            visible_rows,
            total_rows,
            max_scroll_row,
        }
    }

    fn visible_range(self, scroll_row: usize, item_count: usize) -> std::ops::Range<usize> {
        let start = scroll_row.saturating_mul(self.columns).min(item_count);
        let capacity = self.visible_rows.saturating_mul(self.columns);
        start..start.saturating_add(capacity).min(item_count)
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let [header_area, tabs_area, grid_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    render_header(frame, header_area, app);
    render_tabs(frame, tabs_area, app);
    render_footer(frame, footer_area);

    if terminal_too_small(area) {
        render_too_small(frame, grid_area);
        return;
    }

    render_grid(frame, grid_area, app);
}

const fn terminal_too_small(area: Rect) -> bool {
    area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.color_mode {
        ggsci_ratatui::ColorMode::TrueColor => "TrueColor",
        ggsci_ratatui::ColorMode::Ansi256 => "ANSI-256",
    };
    let text = format!(
        " ggsci-rs palette gallery  ·  {mode}  ·  {} palettes",
        app.current_entries().len()
    );
    frame.render_widget(
        Paragraph::new(text).style(Style::new().fg(Color::White).add_modifier(Modifier::BOLD)),
        area,
    );
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let counts = app.catalog.counts();
    let titles = [
        format!("Discrete ({})", counts[0]),
        format!("Continuous ({})", counts[1]),
        format!("iTerm ({})", counts[2]),
        format!("Gephi ({})", counts[3]),
    ];
    let tabs = Tabs::new(titles)
        .select(app.selected_tab.index())
        .divider(" ")
        .style(Style::new().fg(Color::Gray))
        .highlight_style(
            Style::new()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let help = if area.width >= 105 {
        " q/Esc quit · ←/→ or h/l tabs · 1–4 select · ↑/↓ or j/k scroll · PgUp/PgDn page · g/G first/last · m mode"
    } else if area.width >= 70 {
        " q quit · ←/→ tabs · 1–4 select · ↑/↓ scroll · PgUp/PgDn · g/G · m mode"
    } else {
        " q quit · ←/→ tabs · ↑/↓ scroll · m mode"
    };
    frame.render_widget(
        Paragraph::new(help).style(Style::new().fg(Color::Gray)),
        area,
    );
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let message_width = area.width.min(44);
    let message_height = area.height.min(5);
    let message_area = Rect::new(
        area.x
            .saturating_add(area.width.saturating_sub(message_width) / 2),
        area.y
            .saturating_add(area.height.saturating_sub(message_height) / 2),
        message_width,
        message_height,
    );
    let message = Paragraph::new(vec![
        Line::from(Span::styled(
            "Terminal too small",
            Style::new()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Use at least {MIN_TERMINAL_WIDTH} × {MIN_TERMINAL_HEIGHT} cells"
        )),
    ])
    .centered()
    .block(
        Block::new()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(Color::LightCyan)),
    );
    frame.render_widget(message, message_area);
}

fn render_grid(frame: &mut Frame, area: Rect, app: &mut App) {
    let card_area = Rect {
        width: area.width.saturating_sub(SCROLLBAR_WIDTH),
        ..area
    };
    let item_count = app.current_entries().len();
    let metrics = GridMetrics::new(card_area.width, card_area.height, item_count);
    app.update_grid(metrics);
    let scroll_row = app.scroll_row();
    let visible_range = metrics.visible_range(scroll_row, item_count);

    for (visible_index, entry) in app.current_entries()[visible_range].iter().enumerate() {
        let row = visible_index / metrics.columns;
        let column = visible_index % metrics.columns;
        let card_rect = grid_cell(card_area, metrics.columns, row, column);
        if !card_rect.is_empty() {
            frame.render_widget(PaletteCard::new(entry, app.color_mode), card_rect);
        }
    }

    if metrics.total_rows > metrics.visible_rows {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .track_style(Style::new().fg(Color::DarkGray))
            .thumb_symbol("█")
            .thumb_style(Style::new().fg(Color::LightCyan));
        let mut state = ScrollbarState::new(metrics.total_rows)
            .position(scroll_row)
            .viewport_content_length(metrics.visible_rows);
        frame.render_stateful_widget(scrollbar, area, &mut state);
    }
}

fn grid_cell(area: Rect, columns: usize, row: usize, column: usize) -> Rect {
    if columns == 0 || column >= columns {
        return Rect::ZERO;
    }
    let column_count = u16::try_from(columns).unwrap_or(u16::MAX);
    let column_index = u16::try_from(column).unwrap_or(u16::MAX);
    let row_u16 = u16::try_from(row).unwrap_or(u16::MAX);
    let gap_width = GRID_GAP.saturating_mul(column_count.saturating_sub(1));
    let usable_width = area.width.saturating_sub(gap_width);
    let base_width = usable_width.checked_div(column_count).unwrap_or(0);
    let extra = usable_width.checked_rem(column_count).unwrap_or(0);
    let width = base_width.saturating_add(u16::from(column_index < extra));
    let preceding_extra = column_index.min(extra);
    let x = area.x.saturating_add(
        column_index
            .saturating_mul(base_width.saturating_add(GRID_GAP))
            .saturating_add(preceding_extra),
    );
    let y = area
        .y
        .saturating_add(row_u16.saturating_mul(CARD_HEIGHT.saturating_add(GRID_GAP)));

    Rect::new(
        x,
        y,
        width.min(area.right().saturating_sub(x)),
        CARD_HEIGHT.min(area.bottom().saturating_sub(y)),
    )
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;
    use crate::{app::GalleryTab, catalog::Catalog};

    #[test]
    fn grid_math_handles_narrow_multi_column_and_exact_boundaries() {
        let narrow = GridMetrics::new(CARD_MIN_WIDTH, CARD_HEIGHT, 3);
        assert_eq!(narrow.columns, 1);
        assert_eq!(narrow.visible_rows, 1);
        assert_eq!(narrow.total_rows, 3);

        let exact_two =
            GridMetrics::new(CARD_MIN_WIDTH * 2 + GRID_GAP, CARD_HEIGHT * 2 + GRID_GAP, 7);
        assert_eq!(exact_two.columns, 2);
        assert_eq!(exact_two.visible_rows, 2);
        assert_eq!(exact_two.total_rows, 4);
        assert_eq!(exact_two.max_scroll_row, 2);

        let below_two = GridMetrics::new(CARD_MIN_WIDTH * 2, CARD_HEIGHT, 2);
        assert_eq!(below_two.columns, 1);
        let wide = GridMetrics::new(CARD_MIN_WIDTH * 3 + GRID_GAP * 2, CARD_HEIGHT, 9);
        assert_eq!(wide.columns, 3);
    }

    #[test]
    fn grid_math_handles_empty_partial_full_and_tall_content() {
        let empty = GridMetrics::new(100, 20, 0);
        assert_eq!(empty.total_rows, 0);
        assert_eq!(empty.max_scroll_row, 0);

        let partial = GridMetrics::new(CARD_MIN_WIDTH * 2 + GRID_GAP, CARD_HEIGHT, 3);
        assert_eq!(partial.total_rows, 2);
        let full = GridMetrics::new(CARD_MIN_WIDTH * 2 + GRID_GAP, CARD_HEIGHT, 4);
        assert_eq!(full.total_rows, 2);

        let taller_than_content = GridMetrics::new(CARD_MIN_WIDTH, 100, 2);
        assert!(taller_than_content.visible_rows > taller_than_content.total_rows);
        assert_eq!(taller_than_content.max_scroll_row, 0);
        assert_eq!(GridMetrics::new(0, 0, 4), GridMetrics::default());
    }

    #[test]
    fn minimum_size_constants_gate_the_grid() {
        assert!(!terminal_too_small(Rect::new(
            0,
            0,
            MIN_TERMINAL_WIDTH,
            MIN_TERMINAL_HEIGHT
        )));
        assert!(terminal_too_small(Rect::new(
            0,
            0,
            MIN_TERMINAL_WIDTH - 1,
            MIN_TERMINAL_HEIGHT
        )));
        assert!(terminal_too_small(Rect::new(
            0,
            0,
            MIN_TERMINAL_WIDTH,
            MIN_TERMINAL_HEIGHT - 1
        )));
    }

    #[test]
    fn headless_render_smoke_tests_every_tab_and_mode() {
        let catalog = Catalog::new().unwrap();
        for tab in GalleryTab::ALL {
            for mode in [
                ggsci_ratatui::ColorMode::TrueColor,
                ggsci_ratatui::ColorMode::Ansi256,
            ] {
                let mut app = App::with_catalog(catalog.clone());
                app.selected_tab = tab;
                app.color_mode = mode;
                draw_once(&mut app, 120, 36);
            }
        }
    }

    #[test]
    fn headless_render_smokes_scrolled_iterm_and_too_small_views() {
        let mut app = App::with_catalog(Catalog::new().unwrap());
        app.selected_tab = GalleryTab::Iterm;
        app.scroll_rows[GalleryTab::Iterm.index()] = 4;
        draw_once(&mut app, 120, 36);
        assert!(app.scroll_row() > 0);

        let buffer = draw_once(&mut app, 40, 10);
        let text = buffer
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(text.contains("Terminal too small"));
    }

    fn draw_once(app: &mut App, width: u16, height: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();
        terminal.backend().buffer().clone()
    }
}
