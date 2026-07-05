//! Bridge helpers for using ggsci palettes in ggsql text.
//!
//! This private scaffold intentionally does not depend on the ggsql crate yet.

/// Emits a ggsql color array for the first `n` colors in `spec`.
///
/// # Errors
///
/// Returns any palette lookup or length error reported by ggsci.
pub fn color_array(spec: &str, n: usize) -> Result<String, ggsci::Error> {
    let colors = ggsci::palette_by_spec(spec)?.take_hex(n)?;
    let quoted = colors
        .iter()
        .map(|color| format!("'{color}'"))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!("[{quoted}]"))
}

/// Emits a ggsql discrete scale clause using colors from `spec`.
///
/// # Errors
///
/// Returns any palette lookup or length error reported by ggsci.
pub fn scale_discrete(aesthetic: &str, spec: &str, n: usize) -> Result<String, ggsci::Error> {
    Ok(format!(
        "SCALE DISCRETE {} TO {}",
        aesthetic.trim(),
        color_array(spec, n)?
    ))
}

#[cfg(test)]
mod tests {
    use super::{color_array, scale_discrete};

    #[test]
    fn emits_color_array() {
        assert_eq!(
            color_array("npg:nrc", 3).unwrap(),
            "['#E64B35', '#4DBBD5', '#00A087']"
        );
    }

    #[test]
    fn emits_discrete_scale() {
        assert_eq!(
            scale_discrete("color", "npg:nrc", 3).unwrap(),
            "SCALE DISCRETE color TO ['#E64B35', '#4DBBD5', '#00A087']"
        );
    }
}
