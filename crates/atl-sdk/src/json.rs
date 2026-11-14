//! Reading values out of Atlassian JSON responses.

use serde_json::Value;

/// The text at `pointer`: a non-empty string, or a number written as text
/// (some APIs send IDs as numbers, others as strings).
pub(crate) fn text(value: &Value, pointer: &str) -> Option<String> {
    match value.pointer(pointer)? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// The items of the array at `pointer` (`""` is the value itself); a
/// missing array has none.
pub(crate) fn items<'a>(value: &'a Value, pointer: &str) -> impl Iterator<Item = &'a Value> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// The strings in the array at `pointer`; non-string items are skipped.
pub(crate) fn strings(value: &Value, pointer: &str) -> Vec<String> {
    items(value, pointer)
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
