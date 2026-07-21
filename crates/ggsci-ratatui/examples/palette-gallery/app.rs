use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ggsci_ratatui::ColorMode;

use crate::{
    catalog::{Catalog, PaletteEntry},
    ui::{self, GridMetrics},
};

const TAB_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryTab {
    Discrete,
    Continuous,
    Iterm,
    Gephi,
}

impl GalleryTab {
    pub const ALL: [Self; TAB_COUNT] = [Self::Discrete, Self::Continuous, Self::Iterm, Self::Gephi];

    pub const fn index(self) -> usize {
        match self {
            Self::Discrete => 0,
            Self::Continuous => 1,
            Self::Iterm => 2,
            Self::Gephi => 3,
        }
    }

    fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % TAB_COUNT]
    }

    fn previous(self) -> Self {
        Self::ALL[(self.index() + TAB_COUNT - 1) % TAB_COUNT]
    }
}

pub struct App {
    pub catalog: Catalog,
    pub selected_tab: GalleryTab,
    pub color_mode: ColorMode,
    pub scroll_rows: [usize; TAB_COUNT],
    pub last_grid: GridMetrics,
    pub should_exit: bool,
}

impl App {
    pub fn new() -> Result<Self, ggsci::Error> {
        Catalog::new().map(Self::with_catalog)
    }

    pub fn with_catalog(catalog: Catalog) -> Self {
        Self {
            catalog,
            selected_tab: GalleryTab::Discrete,
            color_mode: ColorMode::TrueColor,
            scroll_rows: [0; TAB_COUNT],
            last_grid: GridMetrics::default(),
            should_exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| ui::render(frame, self))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key);
            }
        }
        Ok(())
    }

    pub fn current_entries(&self) -> &[PaletteEntry] {
        self.catalog.entries(self.selected_tab.index())
    }

    pub const fn scroll_row(&self) -> usize {
        self.scroll_rows[self.selected_tab.index()]
    }

    pub fn update_grid(&mut self, metrics: GridMetrics) {
        self.last_grid = metrics;
        let index = self.selected_tab.index();
        self.scroll_rows[index] = self.scroll_rows[index].min(metrics.max_scroll_row);
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.is_press() {
            self.handle_key(key.code);
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => {
                self.selected_tab = self.selected_tab.previous();
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                self.selected_tab = self.selected_tab.next();
            }
            KeyCode::Char('1') => self.selected_tab = GalleryTab::Discrete,
            KeyCode::Char('2') => self.selected_tab = GalleryTab::Continuous,
            KeyCode::Char('3') => self.selected_tab = GalleryTab::Iterm,
            KeyCode::Char('4') => self.selected_tab = GalleryTab::Gephi,
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(1),
            KeyCode::PageUp => {
                let page = self.last_grid.visible_rows.max(1);
                self.scroll_by(-usize_to_isize(page));
            }
            KeyCode::PageDown => {
                let page = self.last_grid.visible_rows.max(1);
                self.scroll_by(usize_to_isize(page));
            }
            KeyCode::Home | KeyCode::Char('g') => self.set_scroll_row(0),
            KeyCode::End | KeyCode::Char('G') => {
                self.set_scroll_row(self.last_grid.max_scroll_row);
            }
            KeyCode::Char('m') => {
                self.color_mode = match self.color_mode {
                    ColorMode::TrueColor => ColorMode::Ansi256,
                    ColorMode::Ansi256 => ColorMode::TrueColor,
                };
            }
            _ => {}
        }
    }

    fn scroll_by(&mut self, delta: isize) {
        let current = self.scroll_row();
        let target = if delta.is_negative() {
            current.saturating_sub(delta.unsigned_abs())
        } else {
            current.saturating_add(delta.unsigned_abs())
        };
        self.set_scroll_row(target);
    }

    fn set_scroll_row(&mut self, row: usize) {
        self.scroll_rows[self.selected_tab.index()] = row.min(self.last_grid.max_scroll_row);
    }
}

fn usize_to_isize(value: usize) -> isize {
    isize::try_from(value).unwrap_or(isize::MAX)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    use super::*;

    fn app() -> App {
        App::with_catalog(Catalog::test_fixture())
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn navigation_wraps_and_number_keys_select_tabs() {
        let mut app = app();
        app.handle_key(KeyCode::Left);
        assert_eq!(app.selected_tab, GalleryTab::Gephi);
        app.handle_key(KeyCode::Right);
        assert_eq!(app.selected_tab, GalleryTab::Discrete);

        for (key, tab) in [
            ('1', GalleryTab::Discrete),
            ('2', GalleryTab::Continuous),
            ('3', GalleryTab::Iterm),
            ('4', GalleryTab::Gephi),
        ] {
            app.handle_key(KeyCode::Char(key));
            assert_eq!(app.selected_tab, tab);
        }
    }

    #[test]
    fn color_mode_toggles_both_ways_and_release_is_ignored() {
        let mut app = app();
        app.handle_key(KeyCode::Char('m'));
        assert_eq!(app.color_mode, ColorMode::Ansi256);
        app.handle_key(KeyCode::Char('m'));
        assert_eq!(app.color_mode, ColorMode::TrueColor);

        app.handle_key_event(KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        });
        assert_eq!(app.color_mode, ColorMode::TrueColor);
        app.handle_key_event(press(KeyCode::Char('m')));
        assert_eq!(app.color_mode, ColorMode::Ansi256);
    }

    #[test]
    fn scrolling_saturates_and_home_end_use_logical_rows() {
        let mut app = app();
        app.update_grid(GridMetrics {
            columns: 2,
            visible_rows: 3,
            total_rows: 10,
            max_scroll_row: 7,
        });

        app.handle_key(KeyCode::Up);
        assert_eq!(app.scroll_row(), 0);
        app.handle_key(KeyCode::Down);
        assert_eq!(app.scroll_row(), 1);
        app.handle_key(KeyCode::PageDown);
        assert_eq!(app.scroll_row(), 4);
        app.handle_key(KeyCode::PageDown);
        assert_eq!(app.scroll_row(), 7);
        app.handle_key(KeyCode::Down);
        assert_eq!(app.scroll_row(), 7);
        app.handle_key(KeyCode::PageUp);
        assert_eq!(app.scroll_row(), 4);
        app.handle_key(KeyCode::Home);
        assert_eq!(app.scroll_row(), 0);
        app.handle_key(KeyCode::End);
        assert_eq!(app.scroll_row(), 7);
    }

    #[test]
    fn tabs_retain_independent_scroll_and_resize_clamps() {
        let mut app = app();
        app.update_grid(GridMetrics {
            columns: 1,
            visible_rows: 2,
            total_rows: 10,
            max_scroll_row: 8,
        });
        app.handle_key(KeyCode::End);
        app.handle_key(KeyCode::Char('2'));
        app.update_grid(GridMetrics {
            columns: 1,
            visible_rows: 2,
            total_rows: 6,
            max_scroll_row: 4,
        });
        app.handle_key(KeyCode::Down);
        assert_eq!(app.scroll_row(), 1);
        app.handle_key(KeyCode::Char('1'));
        assert_eq!(app.scroll_row(), 8);

        app.update_grid(GridMetrics {
            columns: 3,
            visible_rows: 4,
            total_rows: 5,
            max_scroll_row: 1,
        });
        assert_eq!(app.scroll_row(), 1);
        app.handle_key(KeyCode::Char('2'));
        assert_eq!(app.scroll_row(), 1);
    }
}
