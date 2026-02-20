//! Utility functions for wrapper operations

/// Escape a string for safe use in JavaScript template literals
///
/// Escapes: backslashes, backticks, dollar signs, newlines, carriage returns
///
/// # Examples
///
/// ```
/// use kimi_lib::wrappers::utils::escape_js;
///
/// let escaped = escape_js("Hello `world`");
/// assert_eq!(escaped, "Hello \\`world\\`");
/// ```
pub fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Build a JavaScript template by replacing placeholders
///
/// Placeholders are in the format `{{key}}`
///
/// # Examples
///
/// ```
/// use kimi_lib::wrappers::utils::build_js;
///
/// let template = "Hello {{name}}!";
/// let result = build_js(template, &[("name", "World")]);
/// assert_eq!(result, "Hello World!");
/// ```
pub fn build_js(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |acc, (key, val)| {
        acc.replace(&format!("{{{{{}}}}}", key), val)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_escapes_backticks() {
        let escaped = escape_js("code `inline` here");
        assert!(escaped.contains(r"\`inline\`"));
        assert!(!escaped.contains("code `inline` here"));
    }

    #[test]
    fn test_escape_js_escapes_backslashes() {
        let escaped = escape_js(r"path\to\file");
        assert!(escaped.contains(r"path\\to\\file"));
    }

    #[test]
    fn test_escape_js_escapes_dollar_signs() {
        let escaped = escape_js("cost is $100");
        assert!(escaped.contains(r"cost is \$100"));
    }

    #[test]
    fn test_escape_js_escapes_newlines() {
        let escaped = escape_js("line1\nline2\rline3");
        assert!(escaped.contains(r"line1\nline2\rline3"));
    }

    #[test]
    fn test_escape_js_handles_empty_string() {
        let escaped = escape_js("");
        assert_eq!(escaped, "");
    }

    #[test]
    fn test_build_js_replaces_placeholders() {
        let template = "Hello {{name}}, you are {{age}} years old";
        let result = build_js(template, &[("name", "Alice"), ("age", "30")]);
        assert_eq!(result, "Hello Alice, you are 30 years old");
    }

    #[test]
    fn test_build_js_handles_missing_placeholders() {
        let template = "Hello {{name}}";
        let result = build_js(template, &[]);
        assert_eq!(result, "Hello {{name}}");
    }
}
