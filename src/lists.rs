use crate::html_utils::escape_html;
use crate::inline::process_inline;

/// Check if a line is an option line (starts with -, --, or /).
pub fn is_option_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    let bytes = trimmed.as_bytes();
    if bytes[0] == b'-' {
        // Short option: -x or long option: --xxx
        if bytes.len() >= 2 {
            if bytes[1] == b'-' {
                // Long option: --something (at least 3 chars)
                return bytes.len() >= 3 && bytes[2].is_ascii_alphanumeric();
            }
            // Short option: -x where x is alphanumeric
            return bytes[1].is_ascii_alphanumeric();
        }
        return false;
    }

    if bytes[0] == b'/' {
        // DOS/VMS style: /V
        return bytes.len() >= 2 && bytes[1].is_ascii_alphanumeric();
    }

    false
}

/// Check if text represents an option list.
pub fn is_option_list(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return false;
    }

    // At least 50% of non-empty lines should be option lines
    let option_count = lines.iter().filter(|l| is_option_line(l.trim())).count();
    option_count > 0 && option_count * 2 >= lines.len()
}

/// Parse an option line into (option, description).
pub fn parse_option_line(line: &str) -> (String, String) {
    let trimmed = line.trim();

    // Find the split point: two or more spaces between option and description
    if let Some(pos) = find_description_start(trimmed) {
        let option = trimmed[..pos].trim().to_string();
        let description = trimmed[pos..].trim().to_string();
        (option, description)
    } else {
        (trimmed.to_string(), String::new())
    }
}

fn find_description_start(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let len = bytes.len();

    // Skip past option part (up to first double-space after non-space)
    let mut found_option = false;
    while i < len {
        if bytes[i] != b' ' {
            found_option = true;
        } else if found_option && i + 1 < len && bytes[i + 1] == b' ' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Convert an option list to HTML.
pub fn convert_option_list(text: &str, line_num: usize, add_data_line: bool) -> String {
    let mut html = String::with_capacity(text.len() * 3);

    let data_line = if add_data_line {
        format!(" data-line=\"{}\"", line_num)
    } else {
        String::new()
    };

    html.push_str(&format!(
        "<dl class=\"option-list\"{}>\n",
        data_line
    ));

    let mut current_option = String::new();
    let mut current_desc = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if is_option_line(trimmed) {
            // Flush previous entry
            if !current_option.is_empty() {
                html.push_str(&format!(
                    "<dt><code>{}</code></dt>\n<dd>{}</dd>\n",
                    escape_html(&current_option),
                    process_inline(current_desc.trim())
                ));
            }
            let (opt, desc) = parse_option_line(trimmed);
            current_option = opt;
            current_desc = desc;
        } else {
            // Continuation line
            if !current_desc.is_empty() {
                current_desc.push(' ');
            }
            current_desc.push_str(trimmed);
        }
    }

    // Flush last entry
    if !current_option.is_empty() {
        html.push_str(&format!(
            "<dt><code>{}</code></dt>\n<dd>{}</dd>\n",
            escape_html(&current_option),
            process_inline(current_desc.trim())
        ));
    }

    html.push_str("</dl>");
    html
}

/// Strip bullet marker from a line (-, *, +).
pub fn strip_bullet_marker(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
    {
        trimmed[2..].to_string()
    } else if trimmed.starts_with('-')
        || trimmed.starts_with('*')
        || trimmed.starts_with('+')
    {
        if trimmed.len() > 1 {
            trimmed[1..].trim_start().to_string()
        } else {
            String::new()
        }
    } else {
        line.to_string()
    }
}

/// Strip enumerated marker from a line (1., a., i., #., etc.).
pub fn strip_enumerated_marker(line: &str) -> String {
    let trimmed = line.trim_start();

    // (#) or (1) or (a) or (i) style
    if trimmed.starts_with('(') {
        if let Some(close) = trimmed.find(')') {
            let marker = &trimmed[1..close];
            if is_valid_enumerator(marker) {
                return trimmed[close + 1..].trim_start().to_string();
            }
        }
    }

    // 1. or a. or i. or #. style
    if let Some(dot_pos) = trimmed.find(". ") {
        let marker = &trimmed[..dot_pos];
        if is_valid_enumerator(marker) {
            return trimmed[dot_pos + 2..].to_string();
        }
    }

    // 1) or a) style
    if let Some(paren_pos) = trimmed.find(") ") {
        let marker = &trimmed[..paren_pos];
        if is_valid_enumerator(marker) {
            return trimmed[paren_pos + 2..].to_string();
        }
    }

    line.to_string()
}

fn is_valid_enumerator(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // #
    if s == "#" {
        return true;
    }
    // Numeric: 1, 2, 10, 123
    if s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    // Single alpha: a, b, A, B
    if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() {
        return true;
    }
    // Roman numerals: i, ii, iii, iv, v, vi, vii, viii, ix, x, etc.
    let lower = s.to_lowercase();
    lower
        .chars()
        .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_option_line_short() {
        assert!(is_option_line("-a"));
        assert!(is_option_line("-v FILE"));
        assert!(is_option_line("-1"));
    }

    #[test]
    fn test_is_option_line_long() {
        assert!(is_option_line("--verbose"));
        assert!(is_option_line("--output=FILE"));
        assert!(is_option_line("--config FILE"));
    }

    #[test]
    fn test_is_option_line_dos() {
        assert!(is_option_line("/V"));
        assert!(is_option_line("/W:warning"));
    }

    #[test]
    fn test_is_option_line_non_options() {
        assert!(!is_option_line(""));
        assert!(!is_option_line("Some text"));
        assert!(!is_option_line("-"));
        assert!(!is_option_line("- item"));
        assert!(!is_option_line("--"));
    }

    #[test]
    fn test_is_option_list_true() {
        let text = "-a            Simple short option.\n-b FILE       Short option with argument.";
        assert!(is_option_list(text));
    }

    #[test]
    fn test_is_option_list_false() {
        let text = "- First bullet item\n- Second bullet item";
        assert!(!is_option_list(text));
    }

    #[test]
    fn test_parse_option_line_with_two_spaces() {
        let (option, description) = parse_option_line("-a            Simple short option.");
        assert_eq!(option, "-a");
        assert_eq!(description, "Simple short option.");
    }

    #[test]
    fn test_parse_option_line_no_description() {
        let (option, description) = parse_option_line("--verbose");
        assert_eq!(option, "--verbose");
        assert_eq!(description, "");
    }

    #[test]
    fn test_parse_option_line_with_argument() {
        let (option, description) = parse_option_line("-b FILE       Short option with argument.");
        assert_eq!(option, "-b FILE");
        assert_eq!(description, "Short option with argument.");
    }

    #[test]
    fn test_strip_bullet_marker_dash() {
        assert_eq!(strip_bullet_marker("- Item text"), "Item text");
        assert_eq!(strip_bullet_marker("-Item text"), "Item text");
        assert_eq!(strip_bullet_marker("-  Item text"), " Item text");
    }

    #[test]
    fn test_strip_bullet_marker_asterisk() {
        assert_eq!(strip_bullet_marker("* Item text"), "Item text");
        assert_eq!(strip_bullet_marker("*Item text"), "Item text");
    }

    #[test]
    fn test_strip_bullet_marker_plus() {
        assert_eq!(strip_bullet_marker("+ Item text"), "Item text");
        assert_eq!(strip_bullet_marker("+Item text"), "Item text");
    }

    #[test]
    fn test_strip_bullet_marker_no_marker() {
        assert_eq!(strip_bullet_marker("No marker here"), "No marker here");
    }

    #[test]
    fn test_strip_enumerated_marker_numeric_dot() {
        assert_eq!(strip_enumerated_marker("1. Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("10. Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("123. Item text"), "Item text");
    }

    #[test]
    fn test_strip_enumerated_marker_numeric_paren() {
        assert_eq!(strip_enumerated_marker("1) Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("(1) Item text"), "Item text");
    }

    #[test]
    fn test_strip_enumerated_marker_alpha() {
        assert_eq!(strip_enumerated_marker("a. Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("A. Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("(a) Item text"), "Item text");
    }

    #[test]
    fn test_strip_enumerated_marker_roman() {
        assert_eq!(strip_enumerated_marker("i. Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("iv. Item text"), "Item text");
        assert_eq!(strip_enumerated_marker("(ii) Item text"), "Item text");
    }

    #[test]
    fn test_strip_enumerated_marker_auto_number() {
        assert_eq!(strip_enumerated_marker("#. Item text"), "Item text");
    }

    #[test]
    fn test_strip_enumerated_marker_no_marker() {
        assert_eq!(
            strip_enumerated_marker("No marker here"),
            "No marker here"
        );
    }
}
