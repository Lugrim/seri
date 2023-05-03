//!Small templating engine

#[must_use]
/// Replace the `{{ key }}` template string by the `replacement` in the `source`
pub fn replace(source: &str, key: &str, replacement: &str) -> String {
    let full_key = "{{ ".to_owned() + key + " }}";
    source.replace(&full_key, replacement)
}
