use std::collections::HashMap;

use crate::directives::{self, DirectiveInfo};
use crate::html_utils::{dedent, escape_html, slugify};
use crate::inline::process_inline;
use crate::lists;
use crate::parser::{self, LineType};
use crate::tables;

/// Conversion options.
#[derive(Default)]
pub struct ConvertOptions {
    pub add_data_lines: bool,
}

/// State collected during conversion.
pub struct ConverterState {
    pub section_titles: Vec<(u8, String)>,
    pub section_chars: Vec<char>,
    pub substitutions: HashMap<String, String>,
    pub target_references: Vec<(String, String)>,
    pub header_content: Option<String>,
    pub footer_content: Option<String>,
    pub sectnum: bool,
    pub sectnum_depth: usize,
    pub sectnum_prefix: String,
    pub sectnum_suffix: String,
}

impl ConverterState {
    pub fn new() -> Self {
        Self {
            section_titles: Vec::new(),
            section_chars: Vec::new(),
            substitutions: HashMap::new(),
            target_references: Vec::new(),
            header_content: None,
            footer_content: None,
            sectnum: false,
            sectnum_depth: 6,
            sectnum_prefix: String::new(),
            sectnum_suffix: String::new(),
        }
    }
}

/// Convert RST to HTML with default options.
pub fn convert(rst: &str) -> String {
    convert_with_options(rst, &ConvertOptions::default())
}

/// Convert RST to HTML with options.
pub fn convert_with_options(rst: &str, _options: &ConvertOptions) -> String {
    let lines: Vec<&str> = rst.lines().collect();
    let mut state = ConverterState::new();

    // First pass: collect metadata
    first_pass(&lines, &mut state);

    // Second pass: convert to HTML
    let html = second_pass(&lines, &mut state);

    // Post-processing: resolve substitutions
    resolve_substitutions(&html, &state)
}

/// Convert RST content (used for directive content, nested blocks).
pub fn convert_rst_content(content: &str) -> String {
    if content.trim().is_empty() {
        return String::new();
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut state = ConverterState::new();
    second_pass(&lines, &mut state)
}

/// First pass: collect section titles, substitutions, targets, directives.
fn first_pass(lines: &[&str], state: &mut ConverterState) {
    let len = lines.len();
    let mut i = 0;

    while i < len {
        let trimmed = lines[i].trim();

        // Section title detection: text followed by adornment
        if i + 1 < len && !trimmed.is_empty() {
            let next = lines[i + 1].trim();
            if is_section_adornment(next) && next.len() >= trimmed.len() {
                let ch = next.chars().next().unwrap();
                let level = get_or_assign_level(ch, &mut state.section_chars);
                state.section_titles.push((level, trimmed.to_string()));
                i += 2;
                continue;
            }
        }

        // Overlined section: adornment, text, adornment
        if i + 2 < len && is_section_adornment(trimmed) {
            let text = lines[i + 1].trim();
            let next_adorn = lines[i + 2].trim();
            if !text.is_empty()
                && is_section_adornment(next_adorn)
                && trimmed.chars().next() == next_adorn.chars().next()
            {
                let ch = trimmed.chars().next().unwrap();
                let level = get_or_assign_level(ch, &mut state.section_chars);
                state.section_titles.push((level, text.to_string()));
                i += 3;
                continue;
            }
        }

        // Directive detection for metadata
        if trimmed.starts_with(".. ") {
            let rest = &trimmed[3..];

            // Substitution definitions
            if rest.starts_with('|') {
                if let Some(pipe_end) = rest[1..].find('|') {
                    let name = &rest[1..pipe_end + 1];
                    let after = rest[pipe_end + 2..].trim();
                    if let Some(colon_pos) = after.find("::") {
                        let dir_type = after[..colon_pos].trim();
                        let args = after[colon_pos + 2..].trim();
                        // Collect the substitution
                        let mut j = i + 1;
                        while j < len {
                            let indent_line = lines[j];
                            if indent_line.trim().is_empty() && j > i + 1 {
                                break;
                            }
                            if !indent_line.trim().is_empty()
                                && indent_line.len() > indent_line.trim_start().len()
                            {
                                // Collect options
                                j += 1;
                                continue;
                            }
                            if j > i + 1 {
                                break;
                            }
                            j += 1;
                        }

                        // Process the substitution
                        let sub_html = process_substitution(dir_type, args, &state);
                        state.substitutions.insert(name.to_string(), sub_html);
                    }
                }
            }

            // Target definitions
            if rest.starts_with('_') {
                let target_rest = &rest[1..];
                if let Some(colon_pos) = target_rest.find(": ") {
                    let name = target_rest[..colon_pos].trim();
                    let url = target_rest[colon_pos + 2..].trim();
                    state
                        .target_references
                        .push((name.to_string(), url.to_string()));
                }
            }

            // Sectnum directive
            if rest.starts_with("sectnum::") {
                state.sectnum = true;
                // Parse options
                let mut j = i + 1;
                while j < len {
                    let opt_line = lines[j].trim();
                    if opt_line.starts_with(':') {
                        if let Some(end) = opt_line[1..].find(':') {
                            let name = &opt_line[1..end + 1];
                            let value = opt_line[end + 2..].trim();
                            match name {
                                "depth" => {
                                    state.sectnum_depth =
                                        value.parse().unwrap_or(6);
                                }
                                "prefix" => {
                                    state.sectnum_prefix = value.to_string();
                                }
                                "suffix" => {
                                    state.sectnum_suffix = value.to_string();
                                }
                                _ => {}
                            }
                        }
                        j += 1;
                    } else {
                        break;
                    }
                }
            }

            // Header directive
            if rest.starts_with("header::") {
                let mut content = String::new();
                let mut j = i + 1;
                while j < len {
                    let cl = lines[j];
                    if cl.trim().is_empty() && j > i + 1 {
                        break;
                    }
                    if cl.len() > cl.trim_start().len() || cl.trim().is_empty() {
                        if !content.is_empty() {
                            content.push('\n');
                        }
                        content.push_str(cl.trim());
                        j += 1;
                    } else if j > i + 1 {
                        break;
                    } else {
                        j += 1;
                    }
                }
                state.header_content = Some(content);
            }

            // Footer directive
            if rest.starts_with("footer::") {
                let mut content = String::new();
                let mut j = i + 1;
                while j < len {
                    let cl = lines[j];
                    if cl.trim().is_empty() && j > i + 1 {
                        break;
                    }
                    if cl.len() > cl.trim_start().len() || cl.trim().is_empty() {
                        if !content.is_empty() {
                            content.push('\n');
                        }
                        content.push_str(cl.trim());
                        j += 1;
                    } else if j > i + 1 {
                        break;
                    } else {
                        j += 1;
                    }
                }
                state.footer_content = Some(content);
            }
        }

        i += 1;
    }
}

fn process_substitution(dir_type: &str, args: &str, _state: &ConverterState) -> String {
    match dir_type {
        "image" => format!("<img src=\"{}\" alt=\"\">", escape_html(args)),
        "replace" => process_inline(args),
        "date" => {
            let now = chrono::Local::now();
            if args.is_empty() {
                now.format("%Y-%m-%d").to_string()
            } else {
                now.format(args).to_string()
            }
        }
        "unicode" => {
            let mut result = String::new();
            for part in args.split_whitespace() {
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
                }
            }
            result
        }
        _ => args.to_string(),
    }
}

/// Second pass: convert lines to HTML.
fn second_pass(lines: &[&str], state: &mut ConverterState) -> String {
    let mut html = String::with_capacity(lines.len() * 80);
    let len = lines.len();
    let mut i = 0;

    // Add header if present
    if let Some(ref header) = state.header_content.clone() {
        html.push_str(&format!("<header>{}</header>\n", process_inline(header)));
    }

    // Section numbering counters
    let mut section_counters: Vec<usize> = vec![0; 7];

    while i < len {
        let line = lines[i];
        let trimmed = line.trim();

        // Empty line: skip
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Section title: text followed by adornment
        if i + 1 < len && !trimmed.is_empty() && !trimmed.starts_with("..") {
            let next = lines[i + 1].trim();
            if is_section_adornment(next) && next.len() >= trimmed.len() {
                let ch = next.chars().next().unwrap();
                let level = get_or_assign_level(ch, &mut state.section_chars);
                let slug = slugify(trimmed);

                // Section numbering
                let prefix = if state.sectnum && (level as usize) <= state.sectnum_depth {
                    let lvl = level as usize;
                    section_counters[lvl] += 1;
                    // Reset deeper counters
                    for c in section_counters.iter_mut().skip(lvl + 1) {
                        *c = 0;
                    }
                    let nums: Vec<String> = section_counters[1..=lvl]
                        .iter()
                        .map(|n| n.to_string())
                        .collect();
                    format!(
                        "{}{}{} ",
                        state.sectnum_prefix,
                        nums.join("."),
                        state.sectnum_suffix
                    )
                } else {
                    String::new()
                };

                html.push_str(&format!(
                    "<h{level} id=\"{slug}\">{prefix}{title}</h{level}>\n",
                    level = level,
                    slug = slug,
                    prefix = prefix,
                    title = process_inline(trimmed),
                ));
                i += 2;
                continue;
            }
        }

        // Overlined section: adornment, text, adornment
        if i + 2 < len && is_section_adornment(trimmed) {
            let text = lines[i + 1].trim();
            let next_adorn = lines[i + 2].trim();
            if !text.is_empty()
                && is_section_adornment(next_adorn)
                && trimmed.chars().next() == next_adorn.chars().next()
            {
                let ch = trimmed.chars().next().unwrap();
                let level = get_or_assign_level(ch, &mut state.section_chars);
                let slug = slugify(text);
                html.push_str(&format!(
                    "<h{level} id=\"{slug}\">{title}</h{level}>\n",
                    level = level,
                    slug = slug,
                    title = process_inline(text),
                ));
                i += 3;
                continue;
            }
        }

        // Transition: 4+ adornment chars on their own (no adjacent title)
        if trimmed.len() >= 4 && is_section_adornment(trimmed) {
            // Check it's not a section underline (previous line must be blank or start of doc)
            let prev_blank = i == 0 || lines[i - 1].trim().is_empty();
            let next_blank = i + 1 >= len || lines[i + 1].trim().is_empty();
            if prev_blank && next_blank {
                html.push_str("<hr>\n");
                i += 1;
                continue;
            }
        }

        // Simple table
        if is_simple_table_border(trimmed) {
            let (table_text, end) = collect_simple_table(lines, i);
            if !table_text.is_empty() {
                html.push_str(&tables::convert_simple_table(&table_text));
                html.push('\n');
                i = end;
                continue;
            }
        }

        // Grid table
        if trimmed.starts_with('+') && (trimmed.contains('-') || trimmed.contains('=')) && trimmed.ends_with('+') {
            let (table_text, end) = collect_grid_table(lines, i);
            if tables::is_grid_table(&table_text) {
                html.push_str(&tables::convert_grid_table(&table_text));
                html.push('\n');
                i = end;
                continue;
            }
        }

        // Directive
        if trimmed.starts_with(".. ") {
            let rest = &trimmed[3..];

            // Skip substitution definitions (handled in first pass)
            if rest.starts_with('|') && rest[1..].contains('|') {
                i = skip_indented_block(lines, i);
                continue;
            }

            // Skip targets (handled in first pass)
            if rest.starts_with('_') {
                i += 1;
                continue;
            }

            // Parse directive
            if let Some(colon_pos) = rest.find("::") {
                let name = rest[..colon_pos].trim();
                if !name.is_empty() && !name.contains(' ') {
                    let args = rest[colon_pos + 2..].trim();
                    let (options, content, end) = collect_directive(lines, i + 1);

                    // Skip directives already handled in first pass
                    let _skip = matches!(name, "sectnum" | "header" | "footer");

                    let info = DirectiveInfo {
                        name,
                        arguments: args,
                        options,
                        content,
                    };

                    let directive_html = directives::render_directive(&info, state);
                    if !directive_html.is_empty() {
                        html.push_str(&directive_html);
                        html.push('\n');
                    }
                    i = end;
                    continue;
                }
            }

            // Comment
            i = skip_indented_block(lines, i);
            continue;
        }

        // Standalone :: literal block marker
        if trimmed == "::" {
            // Collect the indented literal block
            let (content, end) = collect_indented_block(lines, i + 1);
            if !content.is_empty() {
                let code = dedent(&content);
                html.push_str(&format!(
                    "<pre class=\"nohighlight\"><code>{}</code></pre>\n",
                    escape_html(code.trim())
                ));
            }
            i = end;
            continue;
        }

        // Doctest block
        if trimmed.starts_with(">>>") {
            let (block, end) = collect_doctest_block(lines, i);
            html.push_str(&format!(
                "<pre class=\"doctest\"><code class=\"language-python\">{}</code></pre>\n",
                escape_html(&block)
            ));
            i = end;
            continue;
        }

        // Line block
        if trimmed.starts_with("| ") || trimmed == "|" {
            let (block_html, end) = convert_line_block(lines, i);
            html.push_str(&block_html);
            html.push('\n');
            i = end;
            continue;
        }

        // Bullet list
        if (trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ "))
            && trimmed.len() > 2
        {
            let (list_html, end) = convert_bullet_list(lines, i);
            html.push_str(&list_html);
            html.push('\n');
            i = end;
            continue;
        }

        // Enumerated list
        if is_enumerated_item(trimmed) {
            let (list_html, end) = convert_enumerated_list(lines, i);
            html.push_str(&list_html);
            html.push('\n');
            i = end;
            continue;
        }

        // Field list (but not roles like :role:`content`)
        if trimmed.starts_with(':') && !trimmed.starts_with("::") {
            if let Some(second_colon) = trimmed[1..].find(':') {
                let name = &trimmed[1..second_colon + 1];
                // Check it's not a role (roles have :`content` after the second colon)
                let after_colon = &trimmed[second_colon + 2..];
                let is_role = after_colon.starts_with('`');
                if !name.is_empty() && !name.contains('\n') && !is_role {
                    let (dl_html, end) = convert_field_list(lines, i);
                    html.push_str(&dl_html);
                    html.push('\n');
                    i = end;
                    continue;
                }
            }
        }

        // Definition list: text line followed by indented definition
        if !trimmed.is_empty()
            && line.len() == line.trim_start().len()
            && i + 1 < len
        {
            let next_line = lines[i + 1];
            let next_indent = next_line.len() - next_line.trim_start().len();
            if next_indent > 0 && !next_line.trim().is_empty() && !next_line.trim().starts_with("..") {
                // Check if this is really a definition list (not followed by adornment)
                if !is_section_adornment(next_line.trim()) {
                    let (dl_html, end) = convert_definition_list(lines, i);
                    html.push_str(&dl_html);
                    html.push('\n');
                    i = end;
                    continue;
                }
            }
        }

        // Option list
        if lists::is_option_line(trimmed) {
            let (list_text, end) = collect_option_list(lines, i);
            if lists::is_option_list(&list_text) {
                html.push_str(&lists::convert_option_list(&list_text, i, false));
                html.push('\n');
                i = end;
                continue;
            }
        }

        // Paragraph (default)
        let (para, end) = collect_paragraph(lines, i);
        if !para.is_empty() {
            // Check for literal block marker (paragraph ending with ::)
            if para.ends_with("::") {
                // Remove trailing : and render paragraph
                let para_text = if para == "::" {
                    String::new()
                } else {
                    let trimmed_para = para.trim_end_matches(':');
                    format!(
                        "<p>{}:</p>\n",
                        process_inline(trimmed_para)
                    )
                };
                html.push_str(&para_text);

                // Collect literal block
                let (content, literal_end) = collect_indented_block(lines, end);
                if !content.is_empty() {
                    let code = dedent(&content);
                    html.push_str(&format!(
                        "<pre class=\"nohighlight\"><code>{}</code></pre>\n",
                        escape_html(code.trim())
                    ));
                }
                i = literal_end;
            } else {
                html.push_str(&format!("<p>{}</p>\n", process_inline(&para)));
                i = end;
            }
        } else {
            i = end;
        }
    }

    // Add footer if present
    if let Some(ref footer) = state.footer_content.clone() {
        html.push_str(&format!("<footer>{}</footer>\n", process_inline(footer)));
    }

    html
}

/// Resolve substitution references |name| in the final HTML.
fn resolve_substitutions(html: &str, state: &ConverterState) -> String {
    if state.substitutions.is_empty() {
        return html.to_string();
    }

    let mut result = html.to_string();
    for (name, replacement) in &state.substitutions {
        let pattern = format!("|{}|", name);
        result = result.replace(&pattern, replacement);
    }
    result
}

fn is_section_adornment(line: &str) -> bool {
    if line.len() < 4 {
        return false;
    }
    let first = line.chars().next().unwrap();
    parser::classify_line(line) == LineType::SectionAdornment(first)
        || (line.chars().all(|c| c == first) && is_adornment_char(first))
}

fn is_adornment_char(c: char) -> bool {
    matches!(
        c,
        '=' | '-' | '~' | '`' | ':' | '\'' | '"' | '^' | '_' | '*' | '+' | '#' | '<' | '>'
    )
}

fn get_or_assign_level(ch: char, chars: &mut Vec<char>) -> u8 {
    if let Some(pos) = chars.iter().position(|&c| c == ch) {
        (pos + 1) as u8
    } else {
        chars.push(ch);
        chars.len() as u8
    }
}

fn is_simple_table_border(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && trimmed.chars().all(|c| c == '=' || c == ' ')
        && trimmed.contains('=')
        && trimmed.contains(' ')
}

fn collect_simple_table(lines: &[&str], start: usize) -> (String, usize) {
    let mut end = start + 1;
    let len = lines.len();
    let mut border_count = 1;

    while end < len {
        let trimmed = lines[end].trim();
        if is_simple_table_border(trimmed) {
            border_count += 1;
            if border_count >= 2 {
                // Check if there's another border (3-border table with header)
                let mut j = end + 1;
                while j < len {
                    let jt = lines[j].trim();
                    if jt.is_empty() {
                        break;
                    }
                    if is_simple_table_border(jt) {
                        end = j + 1;
                        let text: String = lines[start..end]
                            .iter()
                            .map(|l| *l)
                            .collect::<Vec<_>>()
                            .join("\n");
                        return (text, end);
                    }
                    j += 1;
                }
                end += 1;
                let text: String = lines[start..end]
                    .iter()
                    .map(|l| *l)
                    .collect::<Vec<_>>()
                    .join("\n");
                return (text, end);
            }
        } else if trimmed.is_empty() {
            break;
        }
        end += 1;
    }

    let text: String = lines[start..end]
        .iter()
        .map(|l| *l)
        .collect::<Vec<_>>()
        .join("\n");
    (text, end)
}

fn collect_grid_table(lines: &[&str], start: usize) -> (String, usize) {
    let mut end = start + 1;
    let len = lines.len();

    while end < len {
        let trimmed = lines[end].trim();
        if trimmed.is_empty() {
            break;
        }
        if !trimmed.starts_with('+') && !trimmed.starts_with('|') {
            break;
        }
        end += 1;
    }

    let text: String = lines[start..end]
        .iter()
        .map(|l| *l)
        .collect::<Vec<_>>()
        .join("\n");
    (text, end)
}

fn collect_directive(
    lines: &[&str],
    start: usize,
) -> (HashMap<String, String>, String, usize) {
    let len = lines.len();
    let mut options = HashMap::new();
    let mut content_lines: Vec<String> = Vec::new();
    let mut i = start;

    // Skip empty line after directive start
    if i < len && lines[i].trim().is_empty() {
        i += 1;
    }

    // Collect options (indented lines starting with :)
    while i < len {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            break;
        }
        let indent = lines[i].len() - lines[i].trim_start().len();
        if indent > 0 && trimmed.starts_with(':') {
            if let Some(end_colon) = trimmed[1..].find(':') {
                let name = &trimmed[1..end_colon + 1];
                let value = trimmed[end_colon + 2..].trim();
                options.insert(name.to_string(), value.to_string());
                i += 1;
                continue;
            }
        }
        break;
    }

    // Skip blank line between options and content
    if i < len && lines[i].trim().is_empty() {
        i += 1;
    }

    // Collect content (indented block)
    while i < len {
        let line = lines[i];
        let indent = line.len() - line.trim_start().len();
        if line.trim().is_empty() {
            // Empty line within content is OK if followed by more indented content
            if i + 1 < len {
                let next_indent = lines[i + 1].len() - lines[i + 1].trim_start().len();
                if next_indent > 0 && !lines[i + 1].trim().is_empty() {
                    content_lines.push(String::new());
                    i += 1;
                    continue;
                }
            }
            break;
        }
        if indent == 0 {
            break;
        }
        content_lines.push(line.to_string());
        i += 1;
    }

    let content = if content_lines.is_empty() {
        String::new()
    } else {
        dedent(&content_lines.join("\n"))
    };

    (options, content, i)
}

fn skip_indented_block(lines: &[&str], start: usize) -> usize {
    let len = lines.len();
    let mut i = start + 1;
    while i < len {
        let line = lines[i];
        if line.trim().is_empty() {
            i += 1;
            // Check if next line is still indented
            if i < len
                && !lines[i].trim().is_empty()
                && lines[i].len() > lines[i].trim_start().len()
            {
                continue;
            }
            break;
        }
        if line.len() == line.trim_start().len() {
            break;
        }
        i += 1;
    }
    i
}

fn collect_indented_block(lines: &[&str], start: usize) -> (String, usize) {
    let len = lines.len();
    let mut i = start;

    // Skip blank lines
    while i < len && lines[i].trim().is_empty() {
        i += 1;
    }

    let mut content_lines: Vec<&str> = Vec::new();
    while i < len {
        let line = lines[i];
        if line.trim().is_empty() {
            // Check if more indented content follows
            if i + 1 < len
                && !lines[i + 1].trim().is_empty()
                && lines[i + 1].len() > lines[i + 1].trim_start().len()
            {
                content_lines.push("");
                i += 1;
                continue;
            }
            break;
        }
        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            break;
        }
        content_lines.push(line);
        i += 1;
    }

    (content_lines.join("\n"), i)
}

fn collect_doctest_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut end = start;
    let len = lines.len();

    while end < len {
        let trimmed = lines[end].trim();
        if trimmed.is_empty() {
            break;
        }
        end += 1;
    }

    let block: String = lines[start..end]
        .iter()
        .map(|l| *l)
        .collect::<Vec<_>>()
        .join("\n");
    (block, end)
}

fn convert_line_block(lines: &[&str], start: usize) -> (String, usize) {
    let mut html = String::from("<div class=\"line-block\">\n");
    let len = lines.len();
    let mut i = start;

    while i < len {
        let trimmed = lines[i].trim();

        if trimmed == "|" {
            // Empty line in block
            html.push_str("<div class=\"line\"><br></div>\n");
            i += 1;
            continue;
        }

        if trimmed.starts_with("| ") {
            let content = &trimmed[2..];
            // Check for indentation
            let leading_spaces = content.len() - content.trim_start().len();
            if leading_spaces > 0 {
                html.push_str(&format!(
                    "<div class=\"line\" style=\"margin-left: {}em\">{}</div>\n",
                    leading_spaces,
                    process_inline(content.trim())
                ));
            } else {
                html.push_str(&format!(
                    "<div class=\"line\">{}</div>\n",
                    process_inline(content)
                ));
            }
            i += 1;
            continue;
        }

        break;
    }

    html.push_str("</div>");
    (html, i)
}

fn convert_bullet_list(lines: &[&str], start: usize) -> (String, usize) {
    let mut html = String::from("<ul>\n");
    let len = lines.len();
    let mut i = start;

    while i < len {
        let trimmed = lines[i].trim();

        if trimmed.is_empty() {
            // Check if next line is a continuation (indented) or new list item
            if i + 1 < len {
                let next = lines[i + 1].trim();
                if next.starts_with("- ") || next.starts_with("* ") || next.starts_with("+ ") {
                    i += 1;
                    continue;
                }
                let next_indent = lines[i + 1].len() - lines[i + 1].trim_start().len();
                if next_indent > 0 && !lines[i + 1].trim().is_empty() {
                    i += 1;
                    continue;
                }
            }
            break;
        }

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            // Collect the full item (including continuation lines and nested content)
            let item_content = lists::strip_bullet_marker(trimmed);
            let (full_content, end) = collect_list_item_content(lines, i, &item_content);

            // Check for nested structures
            let rendered = render_list_item_content(&full_content);
            html.push_str(&format!("<li>{}</li>\n", rendered));
            i = end;
        } else {
            break;
        }
    }

    html.push_str("</ul>");
    (html, i)
}

fn collect_list_item_content(lines: &[&str], start: usize, first_line: &str) -> (String, usize) {
    let mut content = first_line.to_string();
    let len = lines.len();
    let mut i = start + 1;

    while i < len {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if trimmed.is_empty() {
            // Blank line - check if continuation follows
            if i + 1 < len {
                let next_indent = lines[i + 1].len() - lines[i + 1].trim_start().len();
                let next_trimmed = lines[i + 1].trim();
                if next_indent > 0 && !next_trimmed.is_empty() {
                    content.push('\n');
                    i += 1;
                    continue;
                }
            }
            break;
        }

        if indent == 0 {
            break;
        }

        // Check if this is a nested list item
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            content.push_str("\n_NESTED_BULLET_");
            content.push_str(trimmed);
            i += 1;
            continue;
        }

        // Check for directives within list items
        if trimmed.starts_with(".. ") {
            content.push_str("\n_NESTED_DIRECTIVE_");
            content.push_str(trimmed);
            // Collect the rest of the directive
            i += 1;
            while i < len {
                let dl = lines[i];
                if dl.trim().is_empty() {
                    if i + 1 < len && lines[i + 1].len() > lines[i + 1].trim_start().len() {
                        content.push('\n');
                        content.push_str(dl);
                        i += 1;
                        continue;
                    }
                    break;
                }
                if dl.len() == dl.trim_start().len() {
                    break;
                }
                content.push('\n');
                content.push_str(dl);
                i += 1;
            }
            continue;
        }

        // Continuation line
        content.push(' ');
        content.push_str(trimmed);
        i += 1;
    }

    (content, i)
}

fn render_list_item_content(content: &str) -> String {
    if content.contains("_NESTED_BULLET_") {
        // Has nested bullet list
        let parts: Vec<&str> = content.splitn(2, "\n_NESTED_BULLET_").collect();
        let main_text = process_inline(parts[0].trim());

        let nested_text = parts.get(1).unwrap_or(&"");
        // Reconstruct the nested list
        let nested_lines: Vec<String> = nested_text
            .lines()
            .map(|l| l.trim_start_matches("_NESTED_BULLET_").to_string())
            .collect();
        let nested_rst = nested_lines.join("\n");
        let nested_html = convert_rst_content(&nested_rst);

        format!("{}\n{}", main_text, nested_html)
    } else if content.contains("_NESTED_DIRECTIVE_") {
        let parts: Vec<&str> = content.splitn(2, "\n_NESTED_DIRECTIVE_").collect();
        let main_text = process_inline(parts[0].trim());
        let directive_text = parts.get(1).unwrap_or(&"");
        let directive_html = convert_rst_content(&format!(".. {}", directive_text.trim_start_matches("_NESTED_DIRECTIVE_")));
        format!("{}\n{}", main_text, directive_html)
    } else {
        process_inline(content.trim())
    }
}

fn is_enumerated_item(s: &str) -> bool {
    // Check common patterns: 1. 2. a. b. #.
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }

    // (X) style
    if trimmed.starts_with('(') {
        if let Some(close) = trimmed.find(')') {
            let marker = &trimmed[1..close];
            return is_valid_enum(marker) && trimmed.get(close + 1..close + 2) == Some(" ");
        }
        return false;
    }

    // X. or X) style
    for (i, c) in trimmed.char_indices() {
        if c == '.' || c == ')' {
            if i > 0 {
                let marker = &trimmed[..i];
                return is_valid_enum(marker) && trimmed.get(i + 1..i + 2) == Some(" ");
            }
            return false;
        }
        if c == ' ' {
            return false;
        }
    }
    false
}

fn is_valid_enum(s: &str) -> bool {
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
    lower
        .chars()
        .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm'))
}

fn convert_enumerated_list(lines: &[&str], start: usize) -> (String, usize) {
    let mut html = String::from("<ol>\n");
    let len = lines.len();
    let mut i = start;

    while i < len {
        let trimmed = lines[i].trim();

        if trimmed.is_empty() {
            // Check if list continues
            if i + 1 < len && is_enumerated_item(lines[i + 1].trim()) {
                i += 1;
                continue;
            }
            // Check for continuation
            if i + 1 < len {
                let next_indent = lines[i + 1].len() - lines[i + 1].trim_start().len();
                if next_indent > 0 && !lines[i + 1].trim().is_empty() {
                    i += 1;
                    continue;
                }
            }
            break;
        }

        if is_enumerated_item(trimmed) {
            let item_text = lists::strip_enumerated_marker(trimmed);
            let (full_content, end) = collect_list_item_content(lines, i, &item_text);
            let rendered = render_list_item_content(&full_content);
            html.push_str(&format!("<li>{}</li>\n", rendered));
            i = end;
        } else {
            let indent = lines[i].len() - lines[i].trim_start().len();
            if indent > 0 {
                // Continuation of previous item - skip
                i += 1;
            } else {
                break;
            }
        }
    }

    html.push_str("</ol>");
    (html, i)
}

fn convert_field_list(lines: &[&str], start: usize) -> (String, usize) {
    let mut html = String::from("<dl>\n");
    let len = lines.len();
    let mut i = start;

    while i < len {
        let trimmed = lines[i].trim();

        if trimmed.is_empty() {
            // Check for continuation
            if i + 1 < len {
                let next = lines[i + 1].trim();
                if next.starts_with(':') && !next.starts_with("::") {
                    i += 1;
                    continue;
                }
                let next_indent = lines[i + 1].len() - lines[i + 1].trim_start().len();
                if next_indent > 0 {
                    i += 1;
                    continue;
                }
            }
            break;
        }

        if trimmed.starts_with(':') && !trimmed.starts_with("::") {
            if let Some(end_colon) = trimmed[1..].find(':') {
                let name = &trimmed[1..end_colon + 1];
                let value = trimmed[end_colon + 2..].trim();

                html.push_str(&format!("<dt>{}</dt>\n", escape_html(name)));

                // Collect multi-line value
                let mut full_value = value.to_string();
                let mut j = i + 1;
                while j < len {
                    let vl = lines[j];
                    let vi = vl.len() - vl.trim_start().len();
                    if vl.trim().is_empty() {
                        if j + 1 < len {
                            let ni = lines[j + 1].len() - lines[j + 1].trim_start().len();
                            if ni > 0 && !lines[j + 1].trim().is_empty() {
                                full_value.push('\n');
                                j += 1;
                                continue;
                            }
                        }
                        break;
                    }
                    if vi > 0 {
                        full_value.push(' ');
                        full_value.push_str(vl.trim());
                        j += 1;
                    } else {
                        break;
                    }
                }

                if full_value.is_empty() {
                    html.push_str("<dd></dd>\n");
                } else {
                    let content = convert_rst_content(full_value.trim());
                    html.push_str(&format!("<dd>{}</dd>\n", content));
                }
                i = j;
                continue;
            }
        }

        break;
    }

    html.push_str("</dl>");
    (html, i)
}

fn convert_definition_list(lines: &[&str], start: usize) -> (String, usize) {
    let mut html = String::from("<dl>\n");
    let len = lines.len();
    let mut i = start;

    while i < len {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Check if more definitions follow
            if i + 1 < len && !lines[i + 1].trim().is_empty() {
                let next_indent = lines[i + 1].len() - lines[i + 1].trim_start().len();
                if next_indent == 0 && i + 2 < len {
                    let after = lines[i + 2];
                    let after_indent = after.len() - after.trim_start().len();
                    if after_indent > 0 && !after.trim().is_empty() && !is_section_adornment(after.trim()) {
                        i += 1;
                        continue;
                    }
                }
            }
            break;
        }

        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            // Term line - may contain classifiers separated by " : "
            let parts: Vec<&str> = trimmed.splitn(2, " : ").collect();
            let term = parts[0];
            html.push_str(&format!("<dt>{}", process_inline(term)));
            if parts.len() > 1 {
                // Handle classifiers
                let classifiers: Vec<&str> = parts[1].split(" : ").collect();
                for classifier in classifiers {
                    html.push_str(&format!(
                        " <span class=\"classifier\">{}</span>",
                        process_inline(classifier.trim())
                    ));
                }
            }
            html.push_str("</dt>\n");

            // Collect definition
            i += 1;
            let mut def_lines: Vec<String> = Vec::new();
            while i < len {
                let dl = lines[i];
                let di = dl.len() - dl.trim_start().len();
                if dl.trim().is_empty() {
                    // Check if next indented line follows
                    if i + 1 < len && lines[i + 1].len() > lines[i + 1].trim_start().len() {
                        def_lines.push(String::new());
                        i += 1;
                        continue;
                    }
                    break;
                }
                if di > 0 {
                    def_lines.push(dl.trim().to_string());
                    i += 1;
                } else {
                    break;
                }
            }

            let def = def_lines.join(" ");
            html.push_str(&format!("<dd>{}</dd>\n", process_inline(def.trim())));
        } else {
            i += 1;
        }
    }

    html.push_str("</dl>");
    (html, i)
}

fn collect_option_list(lines: &[&str], start: usize) -> (String, usize) {
    let mut end = start;
    let len = lines.len();
    let mut text_lines: Vec<&str> = Vec::new();

    while end < len {
        let trimmed = lines[end].trim();
        if trimmed.is_empty() {
            break;
        }
        text_lines.push(trimmed);
        end += 1;
    }

    (text_lines.join("\n"), end)
}

fn collect_paragraph(lines: &[&str], start: usize) -> (String, usize) {
    let len = lines.len();
    let mut i = start;
    let mut para_lines: Vec<&str> = Vec::new();

    while i < len {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            break;
        }
        // Stop at block-level elements
        let indent = lines[i].len() - lines[i].trim_start().len();
        if indent > 0 && !para_lines.is_empty() {
            break;
        }
        if !para_lines.is_empty()
            && (trimmed.starts_with(".. ")
                || trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || is_enumerated_item(trimmed)
                || is_simple_table_border(trimmed)
                || (trimmed.starts_with('+') && trimmed.contains('-')))
        {
            break;
        }
        // Check for section adornment (means previous line was a title)
        if is_section_adornment(trimmed) && !para_lines.is_empty() {
            // The previous para_line is actually a section title, remove it
            let _title = para_lines.pop();
            if para_lines.is_empty() {
                // Let the section handler deal with this
                return (String::new(), start);
            }
            break;
        }
        para_lines.push(trimmed);
        i += 1;
    }

    (para_lines.join(" "), i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_paragraph() {
        let html = convert("This is a simple paragraph.");
        assert!(html.contains("<p>This is a simple paragraph.</p>"));
    }

    #[test]
    fn test_bold_text() {
        let html = convert("This has **bold** text.");
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_italic_text() {
        let html = convert("This has *italic* text.");
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_inline_code() {
        let html = convert("This has ``inline code`` in it.");
        assert!(html.contains("<code>inline code</code>"));
    }

    #[test]
    fn test_section_header() {
        let html = convert("Title\n=====");
        assert!(html.contains("<h1"));
        assert!(html.contains("Title"));
        assert!(html.contains("</h1>"));
    }

    #[test]
    fn test_multiple_section_levels() {
        let html = convert("Title\n=====\n\nSubtitle\n--------\n\nSubSubtitle\n~~~~~~~~~~~");
        assert!(html.contains("<h1"));
        assert!(html.contains("<h2"));
        assert!(html.contains("<h3"));
    }

    #[test]
    fn test_bullet_list() {
        let html = convert("- Item 1\n- Item 2\n- Item 3");
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
        assert!(html.contains("</ul>"));
    }

    #[test]
    fn test_enumerated_list() {
        let html = convert("1. First\n2. Second\n3. Third");
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>"));
        assert!(html.contains("</ol>"));
    }

    #[test]
    fn test_transition_dashes() {
        let html = convert("Some text before.\n\n----\n\nSome text after.");
        assert!(html.contains("<hr"));
    }

    #[test]
    fn test_transition_equals() {
        let html = convert("Before.\n\n====\n\nAfter.");
        assert!(html.contains("<hr"));
    }

    #[test]
    fn test_three_chars_not_transition() {
        let html = convert("Before.\n\n---\n\nAfter.");
        assert!(!html.contains("<hr"));
    }

    #[test]
    fn test_literal_block() {
        let html = convert("Example::\n\n    This is literal\n    text here");
        assert!(html.contains("<pre"));
        assert!(html.contains("nohighlight"));
        assert!(html.contains("This is literal"));
    }

    #[test]
    fn test_standalone_double_colon() {
        let html = convert("The double colon:\n\n::\n\n    Another literal block.");
        assert!(html.contains("<pre"));
        assert!(html.contains("Another literal block"));
        assert!(!html.contains("<p>::</p>"));
    }

    #[test]
    fn test_code_block_directive() {
        let html = convert(".. code-block:: python\n\n   print(\"Hello\")");
        assert!(html.contains("<pre"));
        assert!(html.contains("<code"));
        assert!(html.contains("language-python"));
    }

    #[test]
    fn test_note_admonition() {
        let html = convert(".. note::\n\n   This is a note.");
        assert!(html.contains("admonition"));
        assert!(html.contains("note"));
    }

    #[test]
    fn test_warning_admonition() {
        let html = convert(".. warning::\n\n   This is a warning.");
        assert!(html.contains("admonition"));
        assert!(html.contains("warning"));
    }

    #[test]
    fn test_escaped_html() {
        let html = convert("Use <script> and & characters safely.");
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn test_external_link() {
        let html = convert("Visit `Example <https://example.com>`_ for more.");
        assert!(html.contains("<a"));
        assert!(html.contains("https://example.com"));
    }

    #[test]
    fn test_ref_role() {
        let html = convert("See :ref:`my-label` for details.");
        assert!(html.contains("<a"));
        assert!(html.contains("href=\"#my-label\""));
    }

    #[test]
    fn test_doc_role() {
        let html = convert("See :doc:`other-doc` for details.");
        assert!(html.contains("<a"));
        assert!(html.contains("other-doc.html"));
    }

    #[test]
    fn test_image_directive() {
        let html = convert(".. image:: /path/to/image.png\n   :alt: Alt text");
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"/path/to/image.png\""));
    }

    #[test]
    fn test_figure_directive() {
        let html = convert(".. figure:: /path/to/figure.png\n   :alt: Figure alt\n\n   This is the caption.");
        assert!(html.contains("<figure"));
        assert!(html.contains("<figcaption>"));
    }

    #[test]
    fn test_field_list() {
        let html = convert(":Author: John Doe\n:Version: 1.0");
        assert!(html.contains("<dl"));
        assert!(html.contains("<dt>"));
        assert!(html.contains("<dd>"));
    }

    #[test]
    fn test_doctest_block() {
        let html = convert("Some text.\n\n>>> some_function()\n'result'\n\nMore text.");
        assert!(html.contains("doctest"));
        assert!(html.contains("some_function()"));
    }
}
