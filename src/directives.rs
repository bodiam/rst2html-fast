use crate::html_utils::{dedent, escape_html, slugify};
use crate::inline::process_inline;
use crate::tables;
use std::collections::HashMap;

/// Parsed directive information.
pub struct DirectiveInfo<'a> {
    pub name: &'a str,
    pub arguments: &'a str,
    pub options: HashMap<String, String>,
    pub content: String,
}

/// Render a directive to HTML.
pub fn render_directive(info: &DirectiveInfo, state: &mut crate::converter::ConverterState) -> String {
    match info.name {
        // Code blocks
        "code-block" | "code" | "sourcecode" => render_code_block(info),
        "highlight" => render_code_block(info),
        "parsed-literal" => render_parsed_literal(info),

        // Admonitions
        "note" | "warning" | "tip" | "caution" | "danger" | "attention" | "error" | "hint"
        | "important" => render_admonition(info),
        "admonition" => render_generic_admonition(info),
        "seealso" => render_seealso(info),

        // Images
        "image" => render_image(info),
        "figure" => render_figure(info),

        // Body elements
        "topic" => render_topic(info),
        "sidebar" => render_sidebar(info),
        "rubric" => render_rubric(info),
        "centered" => render_centered(info),
        "epigraph" | "highlights" | "pull-quote" => render_blockquote(info),
        "compound" => render_compound(info),
        "container" => render_container(info),
        "contents" => render_contents(info, state),

        // Tables
        "table" => render_table_directive(info),
        "csv-table" => render_csv_table(info),
        "list-table" => render_list_table(info),

        // Special
        "raw" => render_raw(info),
        "include" => render_include(info),
        "class" => render_class(info),
        "meta" => render_meta(info),
        "math" => render_math(info),

        // Sphinx
        "toctree" => render_toctree(info),
        "versionadded" => render_version(info, "New in version"),
        "versionchanged" => render_version(info, "Changed in version"),
        "deprecated" => render_deprecated(info),
        "glossary" => render_glossary(info),
        "productionlist" => render_productionlist(info),

        // Code documentation
        "doctest" => render_doctest(info),
        "testcode" => render_testcode(info),
        "testoutput" => render_testoutput(info),

        // Unicode/Date/Role (substitution directives)
        "unicode" => render_unicode(info),
        "date" => render_date(info),
        "replace" => render_replace(info),
        "role" => String::new(),

        // Document parts (collected in first pass, not rendered)
        "sectnum" | "header" | "footer" => String::new(),

        // Target notes
        "target-notes" => render_target_notes(state),

        _ => render_unknown(info),
    }
}

fn render_code_block(info: &DirectiveInfo) -> String {
    let lang = info.arguments.trim();
    let code = dedent(&info.content);
    let lang_class = if !lang.is_empty() {
        format!(" class=\"language-{}\"", escape_html(lang))
    } else {
        String::new()
    };

    let mut html = String::new();

    // Caption
    if let Some(caption) = info.options.get("caption") {
        html.push_str(&format!(
            "<div class=\"code-block-caption\">{}</div>",
            escape_html(caption)
        ));
    }

    let linenos_class = if info.options.contains_key("linenos") {
        " linenos"
    } else {
        ""
    };

    html.push_str(&format!(
        "<pre class=\"code-block{}\"{}><code{}>{}</code></pre>",
        linenos_class,
        "",
        lang_class,
        escape_html(code.trim_end())
    ));
    html
}

fn render_parsed_literal(info: &DirectiveInfo) -> String {
    format!(
        "<pre class=\"parsed-literal\">{}</pre>",
        process_inline(&info.content)
    )
}

fn render_admonition(info: &DirectiveInfo) -> String {
    let title = capitalize_first(info.name);
    let content = render_directive_content(&info.content);
    format!(
        "<div class=\"admonition {}\">\n<p class=\"admonition-title\">{}</p>\n{}\n</div>",
        info.name, title, content
    )
}

fn render_generic_admonition(info: &DirectiveInfo) -> String {
    let title = info.arguments.trim();
    let content = render_directive_content(&info.content);
    format!(
        "<div class=\"admonition\">\n<p class=\"admonition-title\">{}</p>\n{}\n</div>",
        escape_html(title),
        content
    )
}

fn render_seealso(info: &DirectiveInfo) -> String {
    let content = render_directive_content(&info.content);
    format!(
        "<div class=\"admonition seealso\">\n<p class=\"admonition-title\">See Also</p>\n{}\n</div>",
        content
    )
}

fn render_image(info: &DirectiveInfo) -> String {
    let src = info.arguments.trim();
    let mut attrs = format!("src=\"{}\"", escape_html(src));

    if let Some(alt) = info.options.get("alt") {
        attrs.push_str(&format!(" alt=\"{}\"", escape_html(alt)));
    }

    let mut style = String::new();
    if let Some(width) = info.options.get("width") {
        style.push_str(&format!("width: {};", width));
    }
    if let Some(height) = info.options.get("height") {
        style.push_str(&format!("height: {};", height));
    }
    if !style.is_empty() {
        attrs.push_str(&format!(" style=\"{}\"", style));
    }

    let img = format!("<img {}>", attrs);

    if let Some(target) = info.options.get("target") {
        format!("<a href=\"{}\">{}</a>", escape_html(target), img)
    } else {
        img
    }
}

fn render_figure(info: &DirectiveInfo) -> String {
    let src = info.arguments.trim();
    let mut attrs = format!("src=\"{}\"", escape_html(src));

    if let Some(alt) = info.options.get("alt") {
        attrs.push_str(&format!(" alt=\"{}\"", escape_html(alt)));
    }

    let mut style = String::new();
    if let Some(width) = info.options.get("width") {
        style.push_str(&format!("width: {};", width));
    }
    if !style.is_empty() {
        attrs.push_str(&format!(" style=\"{}\"", style));
    }

    // Figure class and alignment
    let mut fig_class = String::from("figure");
    if let Some(figclass) = info.options.get("figclass") {
        fig_class.push(' ');
        fig_class.push_str(figclass);
    }
    if let Some(align) = info.options.get("align") {
        fig_class.push_str(&format!(" align-{}", align));
    }

    let mut html = format!("<figure class=\"{}\">\n<img {}>\n", fig_class, attrs);

    // Caption and legend
    if !info.content.is_empty() {
        let content = info.content.trim();
        // Split into caption (first paragraph) and legend (rest)
        let parts: Vec<&str> = content.splitn(2, "\n\n").collect();
        let caption = parts[0];

        html.push_str(&format!("<figcaption>{}</figcaption>\n", process_inline(caption)));

        if parts.len() > 1 {
            let legend = parts[1];
            html.push_str(&format!(
                "<div class=\"legend\">{}</div>\n",
                render_directive_content(legend)
            ));
        }
    }

    html.push_str("</figure>");
    html
}

fn render_topic(info: &DirectiveInfo) -> String {
    let title = info.arguments.trim();
    let content = render_directive_content(&info.content);
    format!(
        "<div class=\"topic\">\n<p class=\"topic-title\">{}</p>\n{}\n</div>",
        escape_html(title),
        content
    )
}

fn render_sidebar(info: &DirectiveInfo) -> String {
    let title = info.arguments.trim();
    let mut html = format!(
        "<aside class=\"sidebar\">\n<p class=\"sidebar-title\">{}</p>\n",
        escape_html(title)
    );

    if let Some(subtitle) = info.options.get("subtitle") {
        html.push_str(&format!(
            "<p class=\"sidebar-subtitle\">{}</p>\n",
            escape_html(subtitle)
        ));
    }

    html.push_str(&render_directive_content(&info.content));
    html.push_str("\n</aside>");
    html
}

fn render_rubric(info: &DirectiveInfo) -> String {
    format!(
        "<p class=\"rubric\">{}</p>",
        process_inline(info.arguments.trim())
    )
}

fn render_centered(info: &DirectiveInfo) -> String {
    format!(
        "<p class=\"centered\" style=\"text-align: center\">{}</p>",
        process_inline(info.arguments.trim())
    )
}

fn render_blockquote(info: &DirectiveInfo) -> String {
    let content = render_directive_content(&info.content);
    format!(
        "<blockquote class=\"{}\">\n{}\n</blockquote>",
        info.name, content
    )
}

fn render_compound(info: &DirectiveInfo) -> String {
    let content = render_directive_content(&info.content);
    format!("<div class=\"compound\">\n{}\n</div>", content)
}

fn render_container(info: &DirectiveInfo) -> String {
    let class = info.arguments.trim();
    let content = render_directive_content(&info.content);
    format!("<div class=\"{}\">\n{}\n</div>", escape_html(class), content)
}

fn render_contents(info: &DirectiveInfo, state: &crate::converter::ConverterState) -> String {
    let title = if !info.arguments.is_empty() {
        info.arguments.trim()
    } else {
        "Contents"
    };

    let mut html = format!(
        "<div class=\"contents\">\n<p class=\"topic-title\">{}</p>\n",
        escape_html(title)
    );

    // Generate TOC from collected section titles (exclude h1)
    if !state.section_titles.is_empty() {
        html.push_str("<ul class=\"toc\">\n");
        for (level, title) in &state.section_titles {
            if *level > 1 {
                let slug = slugify(title);
                let indent = "  ".repeat((*level as usize).saturating_sub(2));
                html.push_str(&format!(
                    "{}<li><a href=\"#{}\">{}</a></li>\n",
                    indent,
                    slug,
                    escape_html(title)
                ));
            }
        }
        html.push_str("</ul>\n");
    }

    html.push_str("</div>");
    html
}

fn render_table_directive(info: &DirectiveInfo) -> String {
    let title = info.arguments.trim();
    let mut html = String::from("<table>\n");

    if !title.is_empty() {
        html.push_str(&format!("<caption>{}</caption>\n", escape_html(title)));
    }

    // Content could be a simple or grid table
    if !info.content.is_empty() {
        let content = info.content.trim();
        if tables::is_simple_table(content) {
            return format!(
                "<table>\n<caption>{}</caption>\n</table>",
                escape_html(title)
            );
        }
    }

    html.push_str("</table>");
    html
}

fn render_csv_table(info: &DirectiveInfo) -> String {
    tables::convert_csv_table(
        info.arguments.trim(),
        info.options.get("header").map(|s| s.as_str()),
        info.options.get("widths").map(|s| s.as_str()),
        info.options.get("align").map(|s| s.as_str()),
        &info.content,
    )
}

fn render_list_table(info: &DirectiveInfo) -> String {
    let header_rows = info
        .options
        .get("header-rows")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let stub_columns = info
        .options
        .get("stub-columns")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    tables::convert_list_table(
        info.arguments.trim(),
        header_rows,
        stub_columns,
        info.options.get("widths").map(|s| s.as_str()),
        info.options.get("align").map(|s| s.as_str()),
        &info.content,
    )
}

fn render_raw(info: &DirectiveInfo) -> String {
    match info.arguments.trim() {
        "html" => info.content.clone(),
        format => format!("<!-- {}: {} -->", capitalize_first(format), escape_html(&info.content)),
    }
}

fn render_include(info: &DirectiveInfo) -> String {
    format!(
        "<p class=\"include\">Include: {}</p>",
        escape_html(info.arguments.trim())
    )
}

fn render_class(info: &DirectiveInfo) -> String {
    let class = info.arguments.trim();
    let content = render_directive_content(&info.content);
    format!("<div class=\"{}\">\n{}\n</div>", escape_html(class), content)
}

fn render_meta(info: &DirectiveInfo) -> String {
    format!("<!-- meta: {} -->", escape_html(&info.content))
}

fn render_math(info: &DirectiveInfo) -> String {
    let mut html = String::new();

    if let Some(label) = info.options.get("label") {
        html.push_str(&format!(
            "<div class=\"math-block\" id=\"equation-{}\">\n",
            escape_html(label)
        ));
    } else {
        html.push_str("<div class=\"math-block\">\n");
    }

    html.push_str(&escape_html(info.content.trim()));
    html.push_str("\n</div>");
    html
}

fn render_toctree(info: &DirectiveInfo) -> String {
    let mut html = String::from("<nav class=\"toctree-wrapper\">\n");

    if let Some(caption) = info.options.get("caption") {
        html.push_str(&format!(
            "<p class=\"caption\">{}</p>\n",
            escape_html(caption)
        ));
    }

    html.push_str("<ul>\n");
    for line in info.content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with(':') {
            html.push_str(&format!(
                "<li><a href=\"{}.html\">{}</a></li>\n",
                escape_html(trimmed),
                escape_html(trimmed)
            ));
        }
    }
    html.push_str("</ul>\n</nav>");
    html
}

fn render_version(info: &DirectiveInfo, prefix: &str) -> String {
    let version = info.arguments.trim();
    let content = render_directive_content(&info.content);

    format!(
        "<div class=\"{}\">\n<span class=\"versionmodified\">{} {}: </span>{}\n</div>",
        info.name,
        prefix,
        escape_html(version),
        content
    )
}

fn render_deprecated(info: &DirectiveInfo) -> String {
    let version = info.arguments.trim();
    let content = render_directive_content(&info.content);

    format!(
        "<div class=\"deprecated\">\n<span class=\"versionmodified\">Deprecated since version {}: </span>{}\n</div>",
        escape_html(version),
        content
    )
}

fn render_glossary(info: &DirectiveInfo) -> String {
    let sorted = info.options.contains_key("sorted");

    let mut entries: Vec<(Vec<String>, String)> = Vec::new();
    let mut current_terms: Vec<String> = Vec::new();
    let mut current_def = String::new();

    for line in info.content.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            if !current_terms.is_empty() && !current_def.is_empty() {
                entries.push((current_terms.clone(), current_def.trim().to_string()));
                current_terms.clear();
                current_def.clear();
            }
            continue;
        }

        if trimmed.starts_with("   ") || trimmed.starts_with('\t') {
            // Definition line
            if !current_def.is_empty() {
                current_def.push(' ');
            }
            current_def.push_str(trimmed.trim());
        } else {
            // Term line
            if !current_def.is_empty() {
                entries.push((current_terms.clone(), current_def.trim().to_string()));
                current_terms.clear();
                current_def.clear();
            }
            current_terms.push(trimmed.to_string());
        }
    }
    if !current_terms.is_empty() {
        entries.push((current_terms, current_def.trim().to_string()));
    }

    if sorted {
        entries.sort_by(|a, b| {
            let a_term = a.0.first().map(|s| s.to_lowercase()).unwrap_or_default();
            let b_term = b.0.first().map(|s| s.to_lowercase()).unwrap_or_default();
            a_term.cmp(&b_term)
        });
    }

    let mut html = String::from("<dl class=\"glossary\">\n");
    for (terms, definition) in &entries {
        for term in terms {
            let id = slugify(term);
            html.push_str(&format!(
                "<dt id=\"term-{}\">{}</dt>\n",
                id,
                escape_html(term)
            ));
        }
        if !definition.is_empty() {
            html.push_str(&format!("<dd>{}</dd>\n", process_inline(definition)));
        }
    }
    html.push_str("</dl>");
    html
}

fn render_productionlist(info: &DirectiveInfo) -> String {
    let mut html = String::from("<pre class=\"productionlist\">\n");

    for line in info.content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let name = trimmed[..colon_pos].trim();
            let rule = trimmed[colon_pos + 1..].trim();

            // Process `references` in rule
            let processed_rule = process_production_refs(rule);

            html.push_str(&format!(
                "<strong id=\"grammar-token-{}\">{}</strong> ::= {}\n",
                escape_html(name),
                escape_html(name),
                processed_rule
            ));
        }
    }

    html.push_str("</pre>");
    html
}

fn process_production_refs(rule: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = rule.chars().collect();
    let len = chars.len();

    while i < len {
        if chars[i] == '`' {
            let start = i + 1;
            i += 1;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            if i < len {
                let ref_name: String = chars[start..i].iter().collect();
                result.push_str(&format!(
                    "<a href=\"#grammar-token-{}\" class=\"production-ref\">{}</a>",
                    escape_html(&ref_name),
                    escape_html(&ref_name)
                ));
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn render_doctest(info: &DirectiveInfo) -> String {
    // If the directive has a group name like "ignored", don't render
    if !info.arguments.trim().is_empty() {
        return String::new();
    }

    let code = dedent(&info.content);
    format!(
        "<pre class=\"doctest\"><code class=\"language-python\">{}</code></pre>",
        escape_html(code.trim())
    )
}

fn render_testcode(info: &DirectiveInfo) -> String {
    let code = dedent(&info.content);
    format!(
        "<pre class=\"testcode\"><code>{}</code></pre>",
        escape_html(code.trim())
    )
}

fn render_testoutput(info: &DirectiveInfo) -> String {
    let output = dedent(&info.content);
    format!(
        "<pre class=\"testoutput\"><samp>{}</samp></pre>",
        escape_html(output.trim())
    )
}

fn render_unicode(info: &DirectiveInfo) -> String {
    let mut result = String::new();
    for part in info.arguments.split_whitespace() {
        if let Some(hex) = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X")) {
            if let Ok(code) = u32::from_str_radix(hex, 16) {
                if let Some(ch) = char::from_u32(code) {
                    result.push(ch);
                }
            }
        } else if let Ok(code) = part.parse::<u32>() {
            if let Some(ch) = char::from_u32(code) {
                result.push(ch);
            }
        } else {
            // Literal text
            result.push_str(part);
        }
    }
    result
}

fn render_date(info: &DirectiveInfo) -> String {
    let format = info.arguments.trim();
    let now = chrono::Local::now();

    if format.is_empty() {
        now.format("%Y-%m-%d").to_string()
    } else {
        now.format(format).to_string()
    }
}

fn render_replace(info: &DirectiveInfo) -> String {
    process_inline(info.arguments.trim())
}

fn render_target_notes(state: &crate::converter::ConverterState) -> String {
    if state.target_references.is_empty() {
        return String::new();
    }

    let mut html = String::from("<div class=\"target-notes\">\n<ol>\n");
    for (name, url) in &state.target_references {
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>\n",
            escape_html(url),
            escape_html(name)
        ));
    }
    html.push_str("</ol>\n</div>");
    html
}

fn render_unknown(info: &DirectiveInfo) -> String {
    let mut html = format!("<div class=\"directive-{}\">\n", escape_html(info.name));

    if !info.arguments.is_empty() {
        html.push_str(&format!(
            "<div class=\"directive-arguments\">{}</div>\n",
            escape_html(info.arguments.trim())
        ));
    }

    if !info.content.is_empty() {
        html.push_str(&format!(
            "<div class=\"directive-content\">{}</div>\n",
            render_directive_content(&info.content)
        ));
    }

    html.push_str("</div>");
    html
}

/// Render directive content with inline markup and block elements.
fn render_directive_content(content: &str) -> String {
    crate::converter::convert_rst_content(content)
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut result = c.to_uppercase().to_string();
            result.extend(chars);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::ConverterState;

    fn make_info<'a>(
        name: &'a str,
        arguments: &'a str,
        options: HashMap<String, String>,
        content: &str,
    ) -> DirectiveInfo<'a> {
        DirectiveInfo {
            name,
            arguments,
            options,
            content: content.to_string(),
        }
    }

    fn empty_opts() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn test_render_code_block() {
        let mut state = ConverterState::new();
        let info = make_info("code-block", "python", empty_opts(), "print('hello')");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));
        assert!(html.contains("language-python"));
        assert!(html.contains("print(&#x27;hello&#x27;)") || html.contains("print('hello')"));
    }

    #[test]
    fn test_render_note() {
        let mut state = ConverterState::new();
        let info = make_info("note", "", empty_opts(), "This is a note.");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("admonition"));
        assert!(html.contains("note"));
        assert!(html.contains("Note"));
    }

    #[test]
    fn test_render_warning() {
        let mut state = ConverterState::new();
        let info = make_info("warning", "", empty_opts(), "Be careful!");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("warning"));
        assert!(html.contains("Warning"));
    }

    #[test]
    fn test_render_image() {
        let mut state = ConverterState::new();
        let info = make_info("image", "/path/to/image.png", empty_opts(), "");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"/path/to/image.png\""));
    }

    #[test]
    fn test_render_figure() {
        let mut state = ConverterState::new();
        let info = make_info(
            "figure",
            "/path/to/figure.png",
            empty_opts(),
            "This is the caption",
        );
        let html = render_directive(&info, &mut state);
        assert!(html.contains("<figure"));
        assert!(html.contains("<figcaption>"));
        assert!(html.contains("This is the caption"));
    }

    #[test]
    fn test_render_seealso() {
        let mut state = ConverterState::new();
        let info = make_info("seealso", "", empty_opts(), "Check docs");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("seealso"));
        assert!(html.contains("See Also"));
    }

    #[test]
    fn test_render_versionadded() {
        let mut state = ConverterState::new();
        let info = make_info("versionadded", "2.0", empty_opts(), "New feature");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("versionadded"));
        assert!(html.contains("2.0"));
        assert!(html.contains("New in version"));
    }

    #[test]
    fn test_render_deprecated() {
        let mut state = ConverterState::new();
        let info = make_info("deprecated", "4.0", empty_opts(), "Use X instead");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("deprecated"));
        assert!(html.contains("Deprecated since version"));
    }

    #[test]
    fn test_render_unicode_hex() {
        let mut state = ConverterState::new();
        let info = make_info("unicode", "0xA9", empty_opts(), "");
        let html = render_directive(&info, &mut state);
        assert_eq!(html, "\u{A9}");
    }

    #[test]
    fn test_render_unicode_decimal() {
        let mut state = ConverterState::new();
        let info = make_info("unicode", "169", empty_opts(), "");
        let html = render_directive(&info, &mut state);
        assert_eq!(html, "\u{A9}");
    }

    #[test]
    fn test_render_unicode_multiple() {
        let mut state = ConverterState::new();
        let info = make_info("unicode", "0x41 0x42 0x43", empty_opts(), "");
        let html = render_directive(&info, &mut state);
        assert_eq!(html, "ABC");
    }

    #[test]
    fn test_render_date_default() {
        let mut state = ConverterState::new();
        let info = make_info("date", "", empty_opts(), "");
        let html = render_directive(&info, &mut state);
        // Should match YYYY-MM-DD pattern
        assert!(html.len() == 10);
        assert!(html.chars().nth(4) == Some('-'));
    }

    #[test]
    fn test_render_unknown() {
        let mut state = ConverterState::new();
        let info = make_info("custom-directive", "arg", empty_opts(), "Content");
        let html = render_directive(&info, &mut state);
        assert!(html.contains("directive-custom-directive"));
        assert!(html.contains("directive-arguments"));
        assert!(html.contains("directive-content"));
    }

    #[test]
    fn test_render_glossary() {
        let mut state = ConverterState::new();
        let info = make_info(
            "glossary",
            "",
            empty_opts(),
            "RST\n   A plaintext markup language.",
        );
        let html = render_directive(&info, &mut state);
        assert!(html.contains("glossary"));
        assert!(html.contains("id=\"term-rst\""));
        assert!(html.contains("plaintext markup language"));
    }

    #[test]
    fn test_render_glossary_sorted() {
        let mut state = ConverterState::new();
        let mut opts = HashMap::new();
        opts.insert("sorted".to_string(), String::new());
        let info = make_info(
            "glossary",
            "",
            opts,
            "Zebra\n   The last animal.\n\nApple\n   A fruit.",
        );
        let html = render_directive(&info, &mut state);
        let apple_pos = html.find("Apple").unwrap();
        let zebra_pos = html.find("Zebra").unwrap();
        assert!(apple_pos < zebra_pos);
    }
}
