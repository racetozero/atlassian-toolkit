//! Atlassian query languages (JQL, CQL, Bitbucket's filter syntax).

/// A string literal: `"..."` with `\` and `"` escaped.
pub(crate) fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
