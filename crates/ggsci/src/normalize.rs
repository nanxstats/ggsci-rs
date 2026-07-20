pub(crate) fn key_matches(canonical: &str, requested: &str) -> bool {
    normalize_key(canonical) == normalize_key(requested)
}

pub(crate) fn normalize_key(input: &str) -> String {
    input
        .trim()
        .chars()
        .map(|ch| match ch {
            '_' | '-' => ' ',
            ch if ch.is_whitespace() => ' ',
            ch => ch.to_ascii_lowercase(),
        })
        .collect()
}
