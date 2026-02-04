use crate::html_utils::{escape_html, slugify};

/// Render an RST role to HTML.
/// This is a hot path - uses match dispatch for zero-overhead.
pub fn render_role(role: &str, content: &str) -> String {
    match role {
        "emphasis" => format!("<em>{}</em>", escape_html(content)),
        "strong" => format!("<strong>{}</strong>", escape_html(content)),
        "literal" | "code" => format!("<code>{}</code>", escape_html(content)),
        "subscript" | "sub" => format!("<sub>{}</sub>", escape_html(content)),
        "superscript" | "sup" => format!("<sup>{}</sup>", escape_html(content)),
        "title-reference" | "title" | "t" => format!("<cite>{}</cite>", escape_html(content)),
        "kbd" => format!("<kbd>{}</kbd>", escape_html(content)),
        "dfn" => format!("<dfn>{}</dfn>", escape_html(content)),
        "samp" => format!("<samp>{}</samp>", escape_html(content)),
        "guilabel" => format!("<span class=\"guilabel\">{}</span>", escape_html(content)),
        "menuselection" => {
            format!(
                "<span class=\"menuselection\">{}</span>",
                escape_html(content)
            )
        }
        "file" => format!("<code class=\"file\">{}</code>", escape_html(content)),
        "command" => format!(
            "<strong class=\"command\">{}</strong>",
            escape_html(content)
        ),
        "program" => format!(
            "<strong class=\"program\">{}</strong>",
            escape_html(content)
        ),
        "option" => format!("<code class=\"option\">{}</code>", escape_html(content)),
        "envvar" => format!("<code class=\"envvar\">{}</code>", escape_html(content)),
        "makevar" => format!("<code class=\"makevar\">{}</code>", escape_html(content)),
        "math" => format!(
            "<span class=\"math-inline\">{}</span>",
            escape_html(content)
        ),
        "ref" => render_ref_role(content),
        "doc" => render_doc_role(content),
        "term" => render_term_role(content),
        "abbr" | "abbreviation" => render_abbr_role(content),
        "pep" => format!(
            "<a href=\"https://peps.python.org/pep-{}/\">PEP {}</a>",
            content, content
        ),
        "rfc" => format!(
            "<a href=\"https://datatracker.ietf.org/doc/html/rfc{}\">RFC {}</a>",
            content, content
        ),
        // Sphinx cross-reference roles
        "class" | "func" | "meth" | "mod" | "attr" | "exc" | "obj" | "data" | "const"
        | "type" => {
            format!(
                "<code class=\"xref\">{}</code>",
                escape_html(content)
            )
        }
        _ => format!(
            "<span class=\"role-{}\">{}</span>",
            escape_html(role),
            escape_html(content)
        ),
    }
}

/// Parse display text and target from role content like "Display Text <target>"
fn parse_display_and_target(content: &str) -> Option<(&str, &str)> {
    if let Some(angle_start) = content.rfind('<') {
        if content.ends_with('>') {
            let display = content[..angle_start].trim();
            let target = &content[angle_start + 1..content.len() - 1];
            if !display.is_empty() {
                return Some((display, target));
            }
        }
    }
    None
}

fn render_ref_role(content: &str) -> String {
    if let Some((display, target)) = parse_display_and_target(content) {
        format!(
            "<a href=\"#{}\" class=\"reference internal\">{}</a>",
            escape_html(target),
            escape_html(display)
        )
    } else {
        format!(
            "<a href=\"#{}\" class=\"reference internal\">{}</a>",
            escape_html(content),
            escape_html(content)
        )
    }
}

fn render_doc_role(content: &str) -> String {
    if let Some((display, target)) = parse_display_and_target(content) {
        let href = if target.ends_with(".html") {
            target.to_string()
        } else {
            format!("{}.html", target)
        };
        format!(
            "<a href=\"{}\" class=\"reference internal\">{}</a>",
            escape_html(&href),
            escape_html(display)
        )
    } else {
        let href = format!("{}.html", content);
        format!(
            "<a href=\"{}\" class=\"reference internal\">{}</a>",
            escape_html(&href),
            escape_html(content)
        )
    }
}

fn render_term_role(content: &str) -> String {
    if let Some((display, target)) = parse_display_and_target(content) {
        let slug = slugify(target);
        format!(
            "<a href=\"#term-{}\" class=\"reference internal\">{}</a>",
            slug,
            escape_html(display)
        )
    } else {
        let slug = slugify(content);
        format!(
            "<a href=\"#term-{}\" class=\"reference internal\">{}</a>",
            slug,
            escape_html(content)
        )
    }
}

fn render_abbr_role(content: &str) -> String {
    // Parse "ABBR (Expansion)" pattern
    if let Some(paren_start) = content.find('(') {
        if content.ends_with(')') {
            let abbr = content[..paren_start].trim();
            let expansion = &content[paren_start + 1..content.len() - 1];
            return format!(
                "<abbr title=\"{}\">{}</abbr>",
                escape_html(expansion),
                escape_html(abbr)
            );
        }
    }
    format!("<abbr>{}</abbr>", escape_html(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emphasis_role() {
        assert_eq!(
            render_role("emphasis", "emphasized text"),
            "<em>emphasized text</em>"
        );
    }

    #[test]
    fn test_strong_role() {
        assert_eq!(
            render_role("strong", "strong text"),
            "<strong>strong text</strong>"
        );
    }

    #[test]
    fn test_literal_role() {
        assert_eq!(
            render_role("literal", "literal text"),
            "<code>literal text</code>"
        );
    }

    #[test]
    fn test_code_role() {
        assert_eq!(
            render_role("code", "inline code"),
            "<code>inline code</code>"
        );
    }

    #[test]
    fn test_subscript_role() {
        assert_eq!(render_role("subscript", "2"), "<sub>2</sub>");
    }

    #[test]
    fn test_sub_role() {
        assert_eq!(render_role("sub", "2"), "<sub>2</sub>");
    }

    #[test]
    fn test_superscript_role() {
        assert_eq!(render_role("superscript", "2"), "<sup>2</sup>");
    }

    #[test]
    fn test_sup_role() {
        assert_eq!(render_role("sup", "2"), "<sup>2</sup>");
    }

    #[test]
    fn test_title_reference_role() {
        assert_eq!(
            render_role("title-reference", "Book Title"),
            "<cite>Book Title</cite>"
        );
    }

    #[test]
    fn test_title_role() {
        assert_eq!(
            render_role("title", "Book Title"),
            "<cite>Book Title</cite>"
        );
    }

    #[test]
    fn test_t_role() {
        assert_eq!(render_role("t", "Book Title"), "<cite>Book Title</cite>");
    }

    #[test]
    fn test_ref_role() {
        assert_eq!(
            render_role("ref", "my-label"),
            "<a href=\"#my-label\" class=\"reference internal\">my-label</a>"
        );
    }

    #[test]
    fn test_ref_role_with_display_text() {
        assert_eq!(
            render_role("ref", "My Section <section-label>"),
            "<a href=\"#section-label\" class=\"reference internal\">My Section</a>"
        );
    }

    #[test]
    fn test_doc_role() {
        assert_eq!(
            render_role("doc", "other-doc"),
            "<a href=\"other-doc.html\" class=\"reference internal\">other-doc</a>"
        );
    }

    #[test]
    fn test_doc_role_with_display_text() {
        assert_eq!(
            render_role("doc", "Visual diff </visual-diff>"),
            "<a href=\"/visual-diff.html\" class=\"reference internal\">Visual diff</a>"
        );
    }

    #[test]
    fn test_doc_role_with_path() {
        assert_eq!(
            render_role("doc", "Link previews </link-previews>"),
            "<a href=\"/link-previews.html\" class=\"reference internal\">Link previews</a>"
        );
    }

    #[test]
    fn test_term_role() {
        assert_eq!(
            render_role("term", "RST"),
            "<a href=\"#term-rst\" class=\"reference internal\">RST</a>"
        );
    }

    #[test]
    fn test_term_role_with_spaces() {
        assert_eq!(
            render_role("term", "Python Language"),
            "<a href=\"#term-python-language\" class=\"reference internal\">Python Language</a>"
        );
    }

    #[test]
    fn test_term_role_with_display_text() {
        assert_eq!(
            render_role("term", "reStructuredText <RST>"),
            "<a href=\"#term-rst\" class=\"reference internal\">reStructuredText</a>"
        );
    }

    #[test]
    fn test_kbd_role() {
        assert_eq!(render_role("kbd", "Ctrl+C"), "<kbd>Ctrl+C</kbd>");
    }

    #[test]
    fn test_guilabel_role() {
        assert_eq!(
            render_role("guilabel", "OK"),
            "<span class=\"guilabel\">OK</span>"
        );
    }

    #[test]
    fn test_menuselection_role() {
        assert_eq!(
            render_role("menuselection", "File --> Open"),
            "<span class=\"menuselection\">File --&gt; Open</span>"
        );
    }

    #[test]
    fn test_file_role() {
        assert_eq!(
            render_role("file", "/etc/passwd"),
            "<code class=\"file\">/etc/passwd</code>"
        );
    }

    #[test]
    fn test_command_role() {
        assert_eq!(
            render_role("command", "ls"),
            "<strong class=\"command\">ls</strong>"
        );
    }

    #[test]
    fn test_program_role() {
        assert_eq!(
            render_role("program", "python"),
            "<strong class=\"program\">python</strong>"
        );
    }

    #[test]
    fn test_option_role() {
        assert_eq!(
            render_role("option", "--verbose"),
            "<code class=\"option\">--verbose</code>"
        );
    }

    #[test]
    fn test_envvar_role() {
        assert_eq!(
            render_role("envvar", "PATH"),
            "<code class=\"envvar\">PATH</code>"
        );
    }

    #[test]
    fn test_makevar_role() {
        assert_eq!(
            render_role("makevar", "CC"),
            "<code class=\"makevar\">CC</code>"
        );
    }

    #[test]
    fn test_samp_role() {
        assert_eq!(
            render_role("samp", "sample text"),
            "<samp>sample text</samp>"
        );
    }

    #[test]
    fn test_abbr_role_with_expansion() {
        assert_eq!(
            render_role("abbr", "RST (reStructuredText)"),
            "<abbr title=\"reStructuredText\">RST</abbr>"
        );
    }

    #[test]
    fn test_abbr_role_without_expansion() {
        assert_eq!(render_role("abbr", "HTML"), "<abbr>HTML</abbr>");
    }

    #[test]
    fn test_abbreviation_role() {
        assert_eq!(
            render_role("abbreviation", "API (Application Programming Interface)"),
            "<abbr title=\"Application Programming Interface\">API</abbr>"
        );
    }

    #[test]
    fn test_dfn_role() {
        assert_eq!(render_role("dfn", "term"), "<dfn>term</dfn>");
    }

    #[test]
    fn test_math_role() {
        assert_eq!(
            render_role("math", "E = mc^2"),
            "<span class=\"math-inline\">E = mc^2</span>"
        );
    }

    #[test]
    fn test_pep_role() {
        assert_eq!(
            render_role("pep", "8"),
            "<a href=\"https://peps.python.org/pep-8/\">PEP 8</a>"
        );
    }

    #[test]
    fn test_rfc_role() {
        assert_eq!(
            render_role("rfc", "2616"),
            "<a href=\"https://datatracker.ietf.org/doc/html/rfc2616\">RFC 2616</a>"
        );
    }

    #[test]
    fn test_unknown_role() {
        assert_eq!(
            render_role("custom-role", "content"),
            "<span class=\"role-custom-role\">content</span>"
        );
    }
}
