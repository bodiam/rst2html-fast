/// Fast HTML escaping using memchr for scanning.
/// Only allocates when escaping is actually needed.
#[inline]
pub fn escape_html(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len());
    let mut last = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let replacement = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            _ => continue,
        };
        result.push_str(&s[last..i]);
        result.push_str(replacement);
        last = i + 1;
    }

    if last == 0 {
        return s.to_string();
    }
    result.push_str(&s[last..]);
    result
}

/// Process RST backslash escapes.
/// `\*` -> `*`, `\\` -> `\`, `\ ` -> (removed), etc.
pub fn process_rst_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    ' ' => {
                        // Backslash-space: remove both
                        chars.next();
                    }
                    '\\' | '*' | '`' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+'
                    | '-' | '.' | '!' | '~' | '|' => {
                        // Escaped special char: output literal char
                        // For < and > we need HTML escaping later
                        chars.next();
                        result.push(next);
                    }
                    '<' => {
                        chars.next();
                        result.push_str("&lt;");
                    }
                    '>' => {
                        chars.next();
                        result.push_str("&gt;");
                    }
                    _ => {
                        result.push('\\');
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Generate a URL-safe slug from text.
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    let lower = text.to_lowercase();

    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch);
        } else if ch == ' ' || ch == '-' || ch == '_' {
            if !slug.ends_with('-') {
                slug.push('-');
            }
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// Dedent a block of text by removing common leading whitespace.
pub fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Find minimum indentation of non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    if min_indent == 0 {
        return text.to_string();
    }

    let mut result = String::with_capacity(text.len());
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if line.len() >= min_indent {
            result.push_str(&line[min_indent..]);
        } else {
            result.push_str(line);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_process_rst_escapes_backslash_space() {
        assert_eq!(process_rst_escapes("H\\ O"), "HO");
    }

    #[test]
    fn test_process_rst_escapes_double_backslash() {
        assert_eq!(process_rst_escapes("a \\\\ b"), "a \\ b");
    }

    #[test]
    fn test_process_rst_escapes_asterisk() {
        assert_eq!(process_rst_escapes("\\*not italic\\*"), "*not italic*");
    }

    #[test]
    fn test_process_rst_escapes_backtick() {
        assert_eq!(
            process_rst_escapes("\\`\\`not code\\`\\`"),
            "``not code``"
        );
    }

    #[test]
    fn test_process_rst_escapes_angle_brackets() {
        assert_eq!(process_rst_escapes("\\< \\>"), "&lt; &gt;");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Section 1.1"), "section-11");
        assert_eq!(slugify("RST"), "rst");
    }

    #[test]
    fn test_dedent() {
        assert_eq!(dedent("   a\n   b"), "a\nb");
        assert_eq!(dedent("  a\n    b"), "a\n  b");
        assert_eq!(dedent("a\nb"), "a\nb");
    }
}
