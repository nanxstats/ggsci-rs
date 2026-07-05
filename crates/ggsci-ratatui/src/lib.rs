//! Future ratatui integration for ggsci.
//!
//! This crate is intentionally minimal for the first workspace release. It is
//! not published and does not depend on ratatui yet.

/// Returns the current scaffold status.
#[must_use]
pub const fn status() -> &'static str {
    "ratatui conversions are planned for a future release"
}
