/// Line-based RST parser for fast conversion.
/// Instead of building an AST, we classify lines and process blocks directly.

use std::collections::HashMap;

/// Classification of a line for quick dispatch.
#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    Empty,
    SectionAdornment(char),
    BulletListItem,
    EnumeratedListItem(String), // the marker text
    FieldListItem(String, String), // name, body
    DirectiveStart(String, String), // name, arguments
    DirectiveOption(String, String), // option name, value
    Comment,
    SubstitutionDef(String, String, String), // name, directive_type, args
    Target(String, String), // name, url
    #[allow(dead_code)]
    Transition,
    LineBlockLine,
    LiteralBlockMarker,
    GridTableLine,
    SimpleTableBorder,
    DoctestBlock,
    IndentedLine(usize), // indent level
    TextLine,
}

/// Section adornment characters in RST.
const ADORNMENT_CHARS: &[char] = &[
    '=', '-', '~', '`', ':', '\'', '"', '^', '_', '*', '+', '#', '<', '>',
];

/// Classify a single line.
pub fn classify_line(line: &str) -> LineType {
    if line.trim().is_empty() {
        return LineType::Empty;
    }

    let trimmed = line.trim();

    // Doctest block: starts with >>>
    if trimmed.starts_with(">>>") {
        return LineType::DoctestBlock;
    }

    // Line block: starts with |
    if trimmed.starts_with("| ") || trimmed == "|" {
        return LineType::LineBlockLine;
    }

    // Standalone double colon
    if trimmed == "::" {
        return LineType::LiteralBlockMarker;
    }

    // Grid table line
    if trimmed.starts_with('+') && (trimmed.contains('-') || trimmed.contains('='))
        && trimmed.ends_with('+')
    {
        return LineType::GridTableLine;
    }
    if trimmed.starts_with('|') && trimmed.ends_with('|') {
        return LineType::GridTableLine;
    }

    // Simple table border (line of = with spaces)
    if trimmed.chars().all(|c| c == '=' || c == ' ') && trimmed.contains('=') && trimmed.contains(' ') {
        return LineType::SimpleTableBorder;
    }

    // Transition: 4+ repeated chars of adornment characters, alone on a line
    if trimmed.len() >= 4 {
        let first = trimmed.chars().next().unwrap();
        if ADORNMENT_CHARS.contains(&first) && trimmed.chars().all(|c| c == first) {
            // Could be transition or section adornment
            // We'll classify as adornment and let context determine
            return LineType::SectionAdornment(first);
        }
    }

    // Comment: starts with .. and no directive colon
    if trimmed.starts_with(".. ") {
        // Check for directive pattern: .. name:: or .. name:: args
        let rest = &trimmed[3..];

        // Substitution definition: .. |name| directive:: args
        if rest.starts_with('|') {
            if let Some(pipe_end) = rest[1..].find('|') {
                let name = &rest[1..pipe_end + 1];
                let after = rest[pipe_end + 2..].trim();
                if let Some(colon_pos) = after.find("::") {
                    let directive_type = after[..colon_pos].trim();
                    let args = after[colon_pos + 2..].trim();
                    return LineType::SubstitutionDef(
                        name.to_string(),
                        directive_type.to_string(),
                        args.to_string(),
                    );
                }
            }
        }

        // Target: .. _name: url
        if rest.starts_with('_') {
            let target_rest = &rest[1..];
            if let Some(colon_pos) = target_rest.find(": ") {
                let name = target_rest[..colon_pos].trim();
                let url = target_rest[colon_pos + 2..].trim();
                return LineType::Target(name.to_string(), url.to_string());
            }
            if target_rest.ends_with(':') {
                let name = target_rest[..target_rest.len()-1].trim();
                return LineType::Target(name.to_string(), String::new());
            }
        }

        // Directive
        if let Some(colon_pos) = rest.find("::") {
            let name = rest[..colon_pos].trim();
            if !name.is_empty() && !name.contains(' ') {
                let args = rest[colon_pos + 2..].trim();
                return LineType::DirectiveStart(name.to_string(), args.to_string());
            }
        }

        return LineType::Comment;
    }

    // Field list: :name: value
    if trimmed.starts_with(':') && !trimmed.starts_with("::") {
        if let Some(second_colon) = trimmed[1..].find(':') {
            let name = &trimmed[1..second_colon + 1];
            if !name.is_empty() && !name.contains('\n') {
                let value = trimmed[second_colon + 2..].trim();
                return LineType::FieldListItem(name.to_string(), value.to_string());
            }
        }
    }

    // Bullet list item: -, *, + followed by space
    if (trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ "))
        && trimmed.len() > 2
    {
        return LineType::BulletListItem;
    }

    // Enumerated list item
    if is_enumerated_list_item(trimmed) {
        let marker = extract_enum_marker(trimmed);
        return LineType::EnumeratedListItem(marker);
    }

    // Directive option (indented, starts with :)
    let indent = line.len() - line.trim_start().len();
    if indent > 0 && trimmed.starts_with(':') {
        if let Some(end_colon) = trimmed[1..].find(':') {
            let name = &trimmed[1..end_colon + 1];
            if !name.contains(' ') || name.len() < 30 {
                let value = trimmed[end_colon + 2..].trim();
                return LineType::DirectiveOption(name.to_string(), value.to_string());
            }
        }
    }

    // Indented line
    if indent > 0 {
        return LineType::IndentedLine(indent);
    }

    LineType::TextLine
}

fn is_enumerated_list_item(s: &str) -> bool {
    // Numeric: 1. or 1) or (1)
    // Alpha: a. or A. or a) or (a)
    // Roman: i. or I. or i) or (i)
    // Auto: #. or #)

    // (X) style
    if s.starts_with('(') {
        if let Some(close) = s.find(')') {
            let marker = &s[1..close];
            if is_valid_enum_marker(marker) && s.get(close + 1..close + 2) == Some(" ") {
                return true;
            }
        }
        return false;
    }

    // X. or X) style
    for (i, c) in s.char_indices() {
        if c == '.' || c == ')' {
            if i > 0 {
                let marker = &s[..i];
                if is_valid_enum_marker(marker) && s.get(i + 1..i + 2) == Some(" ") {
                    return true;
                }
            }
            return false;
        }
        if c == ' ' {
            return false;
        }
    }
    false
}

fn is_valid_enum_marker(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s == "#" {
        return true;
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() {
        return true;
    }
    let lower = s.to_lowercase();
    lower.chars().all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'))
}

fn extract_enum_marker(s: &str) -> String {
    if s.starts_with('(') {
        if let Some(close) = s.find(')') {
            return s[..close + 1].to_string();
        }
    }
    for (i, c) in s.char_indices() {
        if c == '.' || c == ')' {
            return s[..i + 1].to_string();
        }
    }
    String::new()
}

/// Parse directive options from indented lines following a directive.
#[allow(dead_code)]
pub fn parse_directive_options(lines: &[&str]) -> (HashMap<String, String>, usize) {
    let mut options = HashMap::new();
    let mut consumed = 0;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with(':') {
            if let Some(end_colon) = trimmed[1..].find(':') {
                let name = &trimmed[1..end_colon + 1];
                let value = trimmed[end_colon + 2..].trim();
                options.insert(name.to_string(), value.to_string());
                consumed += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    (options, consumed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_empty() {
        assert_eq!(classify_line(""), LineType::Empty);
        assert_eq!(classify_line("   "), LineType::Empty);
    }

    #[test]
    fn test_classify_section_adornment() {
        assert_eq!(classify_line("====="), LineType::SectionAdornment('='));
        assert_eq!(classify_line("-----"), LineType::SectionAdornment('-'));
        assert_eq!(classify_line("~~~~~"), LineType::SectionAdornment('~'));
    }

    #[test]
    fn test_classify_bullet_list() {
        assert_eq!(classify_line("- item"), LineType::BulletListItem);
        assert_eq!(classify_line("* item"), LineType::BulletListItem);
        assert_eq!(classify_line("+ item"), LineType::BulletListItem);
    }

    #[test]
    fn test_classify_directive() {
        match classify_line(".. code-block:: python") {
            LineType::DirectiveStart(name, args) => {
                assert_eq!(name, "code-block");
                assert_eq!(args, "python");
            }
            other => panic!("Expected DirectiveStart, got {:?}", other),
        }
    }

    #[test]
    fn test_classify_doctest() {
        assert_eq!(classify_line(">>> print('hello')"), LineType::DoctestBlock);
    }

    #[test]
    fn test_classify_line_block() {
        assert_eq!(classify_line("| John Smith"), LineType::LineBlockLine);
    }
}
