use crate::html_utils::escape_html;
use crate::roles::render_role;

/// Process inline RST markup within text and return HTML.
/// Handles: **bold**, *italic*, ``code``, :role:`text`, `link <url>`_, backslash escapes.
///
/// This is performance-critical - processes character by character with minimal allocation.
pub fn process_inline(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Backslash escape
        if chars[i] == '\\' && i + 1 < len {
            match chars[i + 1] {
                ' ' => {
                    // Backslash-space: remove both (join adjacent markup)
                    i += 2;
                    continue;
                }
                '\\' => {
                    result.push('\\');
                    i += 2;
                    continue;
                }
                '*' => {
                    result.push('*');
                    i += 2;
                    continue;
                }
                '`' => {
                    result.push('`');
                    i += 2;
                    continue;
                }
                '<' => {
                    result.push_str("&lt;");
                    i += 2;
                    continue;
                }
                '>' => {
                    result.push_str("&gt;");
                    i += 2;
                    continue;
                }
                c if is_rst_escapable(c) => {
                    result.push(c);
                    i += 2;
                    continue;
                }
                _ => {
                    result.push('\\');
                    i += 1;
                    continue;
                }
            }
        }

        // Inline code: ``...``
        if i + 1 < len && chars[i] == '`' && chars[i + 1] == '`' {
            if let Some(end) = find_inline_code_end(&chars, i + 2) {
                let code_text: String = chars[i + 2..end].iter().collect();
                result.push_str("<code>");
                result.push_str(&escape_html(&code_text));
                result.push_str("</code>");
                i = end + 2; // skip closing ``
                continue;
            }
        }

        // Role: :role:`content`
        if chars[i] == ':' {
            if let Some((role_name, content, end_pos)) = try_parse_role(&chars, i) {
                result.push_str(&render_role(&role_name, &content));
                i = end_pos;
                continue;
            }
        }

        // Bold+Italic: ***text***
        if i + 2 < len && chars[i] == '*' && chars[i + 1] == '*' && chars[i + 2] == '*' {
            if is_inline_start(i, &chars) {
                if let Some(end) = find_triple_star_end(&chars, i + 3) {
                    let inner: String = chars[i + 3..end].iter().collect();
                    result.push_str("<strong><em>");
                    result.push_str(&escape_html(&inner));
                    result.push_str("</em></strong>");
                    i = end + 3;
                    continue;
                }
            }
        }

        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if is_inline_start(i, &chars) {
                if let Some(end) = find_double_star_end(&chars, i + 2) {
                    let inner: String = chars[i + 2..end].iter().collect();
                    result.push_str("<strong>");
                    result.push_str(&process_inline(&inner));
                    result.push_str("</strong>");
                    i = end + 2;
                    continue;
                }
            }
        }

        // Italic: *text*
        if chars[i] == '*' {
            if is_inline_start(i, &chars) {
                if let Some(end) = find_single_star_end(&chars, i + 1) {
                    let inner: String = chars[i + 1..end].iter().collect();
                    result.push_str("<em>");
                    result.push_str(&process_inline(&inner));
                    result.push_str("</em>");
                    i = end + 1;
                    continue;
                }
            }
        }

        // External link: `text <url>`_
        if chars[i] == '`' {
            if let Some((text_part, url, end_pos)) = try_parse_external_link(&chars, i) {
                result.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(&url),
                    escape_html(&text_part)
                ));
                i = end_pos;
                continue;
            }
        }

        // Substitution reference: |name|
        if chars[i] == '|' {
            if let Some((name, end_pos)) = try_parse_substitution_ref(&chars, i) {
                // Return as a placeholder that the converter will resolve
                result.push_str(&format!("|{}|", name));
                i = end_pos;
                continue;
            }
        }

        // HTML escaping for plain characters
        match chars[i] {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            c => result.push(c),
        }
        i += 1;
    }

    result
}

fn is_rst_escapable(c: char) -> bool {
    matches!(
        c,
        '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '.' | '!' | '~' | '|'
    )
}

fn is_inline_start(pos: usize, chars: &[char]) -> bool {
    if pos == 0 {
        return true;
    }
    let prev = chars[pos - 1];
    prev.is_whitespace() || matches!(prev, '(' | '[' | '{' | '<' | '/' | '\'' | '"' | '-')
}

fn find_inline_code_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i + 1 < len {
        if chars[i] == '`' && chars[i + 1] == '`' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_triple_star_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i + 2 < len {
        if chars[i] == '*' && chars[i + 1] == '*' && chars[i + 2] == '*' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_double_star_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i + 1 < len {
        if chars[i] == '*' && chars[i + 1] == '*' {
            // Make sure it's not *** (which would be bold+italic end)
            if i + 2 >= len || chars[i + 2] != '*' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn find_single_star_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i < len {
        if chars[i] == '*' {
            // Make sure it's not ** (which would be bold)
            if i + 1 >= len || chars[i + 1] != '*' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn try_parse_role(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    // :role:`content`
    let len = chars.len();
    if start >= len || chars[start] != ':' {
        return None;
    }

    // Find the role name (between first : and second :)
    let mut i = start + 1;
    while i < len && chars[i] != ':' && chars[i] != '`' && !chars[i].is_whitespace() {
        i += 1;
    }

    if i >= len || chars[i] != ':' {
        return None;
    }

    let role_name: String = chars[start + 1..i].iter().collect();
    if role_name.is_empty() {
        return None;
    }

    // Expect :`
    i += 1; // skip :
    if i >= len || chars[i] != '`' {
        return None;
    }
    i += 1; // skip `

    // Find closing `
    let content_start = i;
    while i < len && chars[i] != '`' {
        i += 1;
    }

    if i >= len {
        return None;
    }

    let content: String = chars[content_start..i].iter().collect();
    i += 1; // skip closing `

    Some((role_name, content, i))
}

fn try_parse_external_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    // `text <url>`_  or `text <url>`__
    let len = chars.len();
    if start >= len || chars[start] != '`' {
        return None;
    }

    let mut i = start + 1;
    // Find the closing `_ or `__
    while i < len {
        if chars[i] == '`' {
            // Check for `_ or `__
            if i + 1 < len && chars[i + 1] == '_' {
                let text: String = chars[start + 1..i].iter().collect();
                // Check for anonymous `__
                let end_pos = if i + 2 < len && chars[i + 2] == '_' {
                    i + 3
                } else {
                    i + 2
                };

                // Parse "text <url>" format
                if let Some(angle_start) = text.rfind('<') {
                    if text.ends_with('>') {
                        let display = text[..angle_start].trim().to_string();
                        let url = text[angle_start + 1..text.len() - 1].to_string();
                        return Some((display, url, end_pos));
                    }
                }
                return None;
            }
        }
        i += 1;
    }
    None
}

fn try_parse_substitution_ref(chars: &[char], start: usize) -> Option<(String, usize)> {
    let len = chars.len();
    if start >= len || chars[start] != '|' {
        return None;
    }

    let mut i = start + 1;
    while i < len && chars[i] != '|' && chars[i] != '\n' {
        i += 1;
    }

    if i >= len || chars[i] != '|' {
        return None;
    }

    let name: String = chars[start + 1..i].iter().collect();
    if name.is_empty() {
        return None;
    }

    Some((name, i + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold() {
        assert!(process_inline("This has **bold** text.").contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_italic() {
        assert!(process_inline("This has *italic* text.").contains("<em>italic</em>"));
    }

    #[test]
    fn test_inline_code() {
        assert!(
            process_inline("This has ``inline code`` in it.").contains("<code>inline code</code>")
        );
    }

    #[test]
    fn test_bold_italic() {
        let html = process_inline("This has ***bold italic*** text.");
        assert!(
            html.contains("<strong><em>bold italic</em></strong>")
                || html.contains("<em><strong>bold italic</strong></em>")
        );
    }

    #[test]
    fn test_role() {
        let html = process_inline("See :ref:`my-label` for details.");
        assert!(html.contains("<a"));
        assert!(html.contains("href=\"#my-label\""));
    }

    #[test]
    fn test_escaped_asterisk() {
        let html = process_inline("\\*not italic\\*");
        assert!(html.contains("*not italic*"));
        assert!(!html.contains("<em>"));
    }

    #[test]
    fn test_backslash_space_removal() {
        let html = process_inline("H\\ :subscript:`2`\\ O is water.");
        assert!(html.contains("H<sub>2</sub>O"));
    }

    #[test]
    fn test_double_backslash() {
        let html = process_inline("Use \\\\ for a literal backslash.");
        assert!(html.contains("Use \\ for"));
    }

    #[test]
    fn test_mixed_escapes_and_markup() {
        let html = process_inline("This is *italic* but \\*this\\* is not.");
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("*this*"));
    }

    #[test]
    fn test_external_link() {
        let html = process_inline("Visit `Example <https://example.com>`_ for more.");
        assert!(html.contains("<a"));
        assert!(html.contains("https://example.com"));
    }

    #[test]
    fn test_html_escaping() {
        let html = process_inline("Use <script> and & characters safely.");
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn test_escaped_angle_brackets() {
        let html = process_inline("Angle brackets: \\< \\>");
        assert!(html.contains("&lt; &gt;"));
        assert!(!html.contains("&amp;lt;"));
    }
}
