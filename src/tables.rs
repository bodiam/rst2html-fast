use crate::html_utils::escape_html;
use crate::inline::process_inline;

/// Check if text is a simple RST table (uses = and - borders).
pub fn is_simple_table(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 3 {
        return false;
    }
    let first = lines[0].trim();
    let last = lines[lines.len() - 1].trim();
    is_simple_border(first) && is_simple_border(last)
}

fn is_simple_border(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| c == '=' || c == ' ')
        && line.contains('=')
}

fn is_dash_separator(line: &str) -> bool {
    !line.is_empty() && line.chars().all(|c| c == '-' || c == ' ' || c == '=')
        && (line.contains('-') || line.contains('='))
}

/// Parse column boundaries from a border line.
fn parse_column_boundaries(border: &str) -> Vec<(usize, usize)> {
    let mut cols = Vec::new();
    let bytes = border.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Find start of column (first =)
        while i < len && bytes[i] == b' ' {
            i += 1;
        }
        if i >= len {
            break;
        }
        let start = i;
        // Find end of column (last = before space or end)
        while i < len && bytes[i] == b'=' {
            i += 1;
        }
        if i > start {
            cols.push((start, i));
        }
    }
    cols
}

/// Extract cell content from a line using column boundaries.
fn extract_cells(line: &str, boundaries: &[(usize, usize)]) -> Vec<String> {
    let mut cells = Vec::with_capacity(boundaries.len());
    let bytes = line.as_bytes();
    let len = bytes.len();

    for &(start, end) in boundaries {
        let s = start.min(len);
        let e = end.min(len);
        if s < len {
            let cell = &line[s..e];
            cells.push(cell.trim().to_string());
        } else {
            cells.push(String::new());
        }
    }
    cells
}

/// Convert a simple RST table to HTML.
pub fn convert_simple_table(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 3 {
        return format!("<p>{}</p>", escape_html(text));
    }

    let border = lines[0].trim();
    let boundaries = parse_column_boundaries(border);
    if boundaries.len() < 1 {
        return format!("<p>{}</p>", escape_html(text));
    }

    let mut html = String::with_capacity(text.len() * 3);
    html.push_str("<table class=\"simple-table\">\n");

    // Determine structure: 2 borders = no header, 3+ borders = has header
    let mut border_positions: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if is_simple_border(line.trim()) {
            border_positions.push(i);
        }
    }

    let has_header = border_positions.len() >= 3;
    let mut in_header = has_header;
    let mut header_row_count = 0;

    if has_header {
        html.push_str("<thead>\n");
    }

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip border lines
        if is_simple_border(trimmed) {
            if has_header && i == border_positions[1] && header_row_count > 0 {
                // Second border: end of header
                html.push_str("</thead>\n<tbody>\n");
                in_header = false;
            }
            continue;
        }

        // Skip dash separator lines (used in spanning headers)
        if is_dash_separator(trimmed)
            && trimmed.contains('-')
            && !trimmed.chars().any(|c| c.is_alphanumeric())
        {
            continue;
        }

        let cells = extract_cells(line, &boundaries);
        let tag = if in_header { "th" } else { "td" };

        html.push_str("<tr>");
        for cell in &cells {
            let processed = if cell.contains("http://") || cell.contains("https://") {
                // Auto-link URLs in table cells
                if cell.starts_with("http://") || cell.starts_with("https://") {
                    let url = cell.trim();
                    format!("<a href=\"{}\">{}</a>", escape_html(url), escape_html(url))
                } else {
                    process_inline_with_escapes(cell)
                }
            } else {
                process_inline_with_escapes(cell)
            };
            html.push_str(&format!("<{tag}>{processed}</{tag}>"));
        }
        html.push_str("</tr>\n");

        if in_header {
            header_row_count += 1;
        }
    }

    if has_header {
        html.push_str("</tbody>\n");
    }

    html.push_str("</table>");
    html
}

fn process_inline_with_escapes(text: &str) -> String {
    // Process RST escapes then inline markup
    use crate::html_utils::process_rst_escapes;
    let unescaped = process_rst_escapes(text);
    // If the text was changed by rst escape processing, we need to be careful
    // about double-escaping. The process_rst_escapes already converts \< to &lt;
    // so we need process_inline to not re-escape those.
    if text.contains('\\') {
        // Has escape sequences: use the pre-processed text
        // But process_inline will do HTML escaping, so we need to handle this carefully
        process_inline_already_escaped(&unescaped)
    } else {
        process_inline(text)
    }
}

/// Process inline markup on text that already has some HTML entities from escape processing.
fn process_inline_already_escaped(text: &str) -> String {
    // Simple approach: process the text as-is, but don't double-escape existing entities
    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Preserve existing &...; entities
        if chars[i] == '&' {
            let mut j = i + 1;
            while j < len && j < i + 10 && chars[j] != ';' && !chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && chars[j] == ';' {
                let entity: String = chars[i..=j].iter().collect();
                result.push_str(&entity);
                i = j + 1;
                continue;
            }
            result.push_str("&amp;");
            i += 1;
            continue;
        }

        // Inline code: ``...``
        if i + 1 < len && chars[i] == '`' && chars[i + 1] == '`' {
            if let Some(end) = find_double_backtick_end(&chars, i + 2) {
                let code: String = chars[i + 2..end].iter().collect();
                result.push_str("<code>");
                result.push_str(&escape_html(&code));
                result.push_str("</code>");
                i = end + 2;
                continue;
            }
        }

        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_star_end(&chars, i + 2) {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str("<strong>");
                result.push_str(&escape_html(&inner));
                result.push_str("</strong>");
                i = end + 2;
                continue;
            }
        }

        // Italic: *text*
        if chars[i] == '*' {
            if let Some(end) = find_single_star_end(&chars, i + 1) {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str("<em>");
                result.push_str(&escape_html(&inner));
                result.push_str("</em>");
                i = end + 1;
                continue;
            }
        }

        match chars[i] {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            c => result.push(c),
        }
        i += 1;
    }

    result
}

fn find_double_backtick_end(chars: &[char], start: usize) -> Option<usize> {
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

fn find_double_star_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i + 1 < len {
        if chars[i] == '*' && chars[i + 1] == '*' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_single_star_end(chars: &[char], start: usize) -> Option<usize> {
    let len = chars.len();
    let mut i = start;
    while i < len {
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Check if text is a grid table.
pub fn is_grid_table(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 3 {
        return false;
    }
    let first = lines[0].trim();
    first.starts_with('+') && first.ends_with('+') && (first.contains('-') || first.contains('='))
}

/// Convert a grid RST table to HTML.
pub fn convert_grid_table(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < 3 {
        return format!("<p>{}</p>", escape_html(text));
    }

    let mut html = String::with_capacity(text.len() * 3);
    html.push_str("<table class=\"grid-table\">\n");

    // Find column positions from first border line
    let col_positions = parse_grid_columns(lines[0]);
    if col_positions.is_empty() {
        return format!("<p>{}</p>", escape_html(text));
    }

    // Determine header separator (line with ====)
    let mut header_end = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('+') && trimmed.contains('=') && !trimmed.contains('-') {
            header_end = Some(i);
            break;
        }
    }

    // Parse rows
    let mut rows: Vec<Vec<GridCell>> = Vec::new();
    let mut current_row_cells: Vec<Vec<String>> = vec![Vec::new(); col_positions.len() - 1];
    let mut _row_start_line = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if is_grid_border(trimmed) {
            // This is a border line - flush current row if we have content
            if current_row_cells.iter().any(|c| !c.is_empty()) {
                let cells: Vec<GridCell> = current_row_cells
                    .iter()
                    .map(|lines| GridCell {
                        content: lines.join("\n").trim().to_string(),
                        colspan: 1,
                        rowspan: 1,
                    })
                    .collect();
                rows.push(cells);
                current_row_cells = vec![Vec::new(); col_positions.len() - 1];
            }
            _row_start_line = line_idx;
            continue;
        }

        if trimmed.starts_with('|') {
            // Content line - extract cell contents
            let cell_contents = extract_grid_cells(line, &col_positions);
            for (col, content) in cell_contents.iter().enumerate() {
                if col < current_row_cells.len() {
                    current_row_cells[col].push(content.clone());
                }
            }
        }
    }

    // Detect colspan and rowspan from the border structure
    let parsed_rows = detect_spans(&lines, &col_positions);

    // Render
    let mut in_header = header_end.is_some();
    if in_header {
        html.push_str("<thead>\n");
    }

    let header_end_row = if let Some(he) = header_end {
        // Count content rows before header end
        let mut count = 0;
        for (i, line) in lines.iter().enumerate() {
            if i >= he {
                break;
            }
            if is_grid_border(line.trim()) && i > 0 {
                count += 1;
            }
        }
        count
    } else {
        0
    };

    for (row_idx, row) in parsed_rows.iter().enumerate() {
        if in_header && row_idx >= header_end_row {
            html.push_str("</thead>\n<tbody>\n");
            in_header = false;
        }

        html.push_str("<tr>");
        let tag = if row_idx < header_end_row { "th" } else { "td" };

        for cell in row {
            if cell.skip {
                continue;
            }
            let mut attrs = String::new();
            if cell.colspan > 1 {
                attrs.push_str(&format!(" colspan=\"{}\"", cell.colspan));
            }
            if cell.rowspan > 1 {
                attrs.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
            }

            let content = process_grid_cell_content(&cell.content);
            html.push_str(&format!("<{tag}{attrs}>{content}</{tag}>"));
        }
        html.push_str("</tr>\n");
    }

    if !in_header && header_end.is_some() {
        html.push_str("</tbody>\n");
    }

    html.push_str("</table>");
    html
}

#[allow(dead_code)]
struct GridCell {
    content: String,
    colspan: usize,
    rowspan: usize,
}

struct SpannedCell {
    content: String,
    colspan: usize,
    rowspan: usize,
    skip: bool,
}

fn parse_grid_columns(border: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    for (i, c) in border.chars().enumerate() {
        if c == '+' {
            positions.push(i);
        }
    }
    positions
}

fn is_grid_border(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    line.starts_with('+')
        && line.ends_with('+')
        && line.chars().all(|c| matches!(c, '+' | '-' | '=' | ' '))
}

fn extract_grid_cells(line: &str, col_positions: &[usize]) -> Vec<String> {
    let mut cells = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();

    for i in 0..col_positions.len() - 1 {
        let start = (col_positions[i] + 1).min(len);
        let end = col_positions[i + 1].min(len);
        if start < end {
            let cell = &line[start..end];
            // Remove trailing |
            let cell = cell.trim_end_matches('|');
            cells.push(cell.trim().to_string());
        } else {
            cells.push(String::new());
        }
    }
    cells
}

fn detect_spans(lines: &[&str], col_positions: &[usize]) -> Vec<Vec<SpannedCell>> {
    let num_cols = if col_positions.len() > 1 {
        col_positions.len() - 1
    } else {
        return Vec::new();
    };

    // Find all border lines and content regions
    let mut border_indices: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if is_grid_border(line.trim()) {
            border_indices.push(i);
        }
    }

    if border_indices.len() < 2 {
        return Vec::new();
    }

    let mut all_rows: Vec<Vec<SpannedCell>> = Vec::new();

    // For each pair of consecutive borders, extract a row
    for bi in 0..border_indices.len() - 1 {
        let start = border_indices[bi];
        let end = border_indices[bi + 1];

        // Collect content lines between borders
        let content_lines: Vec<&&str> = lines[start + 1..end]
            .iter()
            .filter(|l| l.trim().starts_with('|'))
            .collect();

        if content_lines.is_empty() {
            continue;
        }

        // Build cells for this row
        let mut row_cells: Vec<SpannedCell> = Vec::new();

        // Detect colspan by examining the border line at 'start'
        let border = lines[start];
        let mut col = 0;
        while col < num_cols {
            // Check if the '+' at the next column boundary exists
            let next_pos = col_positions.get(col + 1);
            let mut span = 1;

            if let Some(&pos) = next_pos {
                // Check if border has '+' at this position
                let border_bytes = border.as_bytes();
                if pos < border_bytes.len() && border_bytes[pos] != b'+' {
                    // No '+' means this cell spans into the next column
                    span = 1;
                    let mut check_col = col + 1;
                    while check_col < num_cols {
                        if let Some(&cp) = col_positions.get(check_col + 1) {
                            if cp < border_bytes.len() && border_bytes[cp] == b'+' {
                                span = check_col - col + 1;
                                break;
                            }
                        }
                        check_col += 1;
                    }
                    if span == 1 {
                        span = num_cols - col;
                    }
                }
            }

            // Extract content for this cell across all content lines
            let cell_start = col_positions[col] + 1;
            let cell_end = col_positions.get(col + span).copied().unwrap_or(
                col_positions.last().copied().unwrap_or(cell_start)
            );

            let mut cell_lines: Vec<String> = Vec::new();
            for content_line in &content_lines {
                let line_bytes = content_line.as_bytes();
                let s = cell_start.min(line_bytes.len());
                let e = cell_end.min(line_bytes.len());
                if s < e {
                    let slice = &content_line[s..e];
                    let cleaned = slice.trim_end_matches('|').trim();
                    cell_lines.push(cleaned.to_string());
                }
            }

            let content = cell_lines.join("\n").trim().to_string();

            // Detect rowspan by checking if the border line at 'end' has no '+' at our positions
            let mut rowspan = 1;
            let next_border_idx = bi + 1;
            if next_border_idx < border_indices.len() {
                let next_border = lines[border_indices[next_border_idx]];
                let nb_bytes = next_border.as_bytes();
                // Check if our cell's left boundary has '+' in the next border
                let left_pos = col_positions[col];
                if left_pos < nb_bytes.len() && nb_bytes[left_pos] == b'+' {
                    // Check if horizontal line exists
                    let has_separator = if cell_start < nb_bytes.len() {
                        nb_bytes[cell_start] == b'-' || nb_bytes[cell_start] == b'='
                    } else {
                        false
                    };
                    if !has_separator {
                        // This cell spans rows
                        rowspan = 2; // Simplified: detect 2-row spans
                    }
                } else if left_pos < nb_bytes.len() && nb_bytes[left_pos] == b'|' {
                    // Cell continues from previous row
                    // Don't start a new cell
                }
            }

            row_cells.push(SpannedCell {
                content,
                colspan: span,
                rowspan,
                skip: false,
            });

            col += span;
        }

        all_rows.push(row_cells);
    }

    // Mark cells that should be skipped due to rowspan
    for row_idx in 0..all_rows.len() {
        for col_idx in 0..all_rows[row_idx].len() {
            let rs = all_rows[row_idx][col_idx].rowspan;
            if rs > 1 {
                // Mark subsequent rows' cells as skipped
                for r in 1..rs {
                    let target_row = row_idx + r;
                    if target_row < all_rows.len() && col_idx < all_rows[target_row].len() {
                        all_rows[target_row][col_idx].skip = true;
                    }
                }
            }
        }
    }

    all_rows
}

fn process_grid_cell_content(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Check for RST constructs within cell
    // Code blocks
    if trimmed.starts_with(".. code-block::") || trimmed.starts_with(".. code::") {
        return process_cell_code_block(trimmed);
    }

    // Bullet lists
    if trimmed.starts_with("- ") {
        return process_cell_list(trimmed);
    }

    // Multi-line content: join with space or <br>
    if trimmed.contains('\n') {
        let lines: Vec<&str> = trimmed.lines().collect();
        // Check if it's a list
        if lines.iter().all(|l| l.trim().starts_with("- ")) {
            return process_cell_list(trimmed);
        }
        // Regular multi-line: join
        let joined = lines
            .iter()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join(" ");
        return process_inline(&joined);
    }

    process_inline(trimmed)
}

fn process_cell_code_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    // Extract language
    let first = lines[0].trim();
    let lang = if let Some(pos) = first.rfind("::") {
        first[pos + 2..].trim()
    } else {
        ""
    };

    // Extract code (skip directive line and blank lines)
    let code_lines: Vec<&str> = lines[1..]
        .iter()
        .skip_while(|l| l.trim().is_empty())
        .copied()
        .collect();

    let code = crate::html_utils::dedent(&code_lines.join("\n"));
    let lang_class = if !lang.is_empty() {
        format!(" class=\"language-{}\"", escape_html(lang))
    } else {
        String::new()
    };

    format!(
        "<pre><code{}>{}</code></pre>",
        lang_class,
        escape_html(&code)
    )
}

fn process_cell_list(content: &str) -> String {
    let mut html = String::from("<ul>");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") {
            html.push_str("<li>");
            html.push_str(&process_inline(&trimmed[2..]));
            html.push_str("</li>");
        }
    }
    html.push_str("</ul>");
    html
}

/// Convert a CSV table to HTML.
pub fn convert_csv_table(
    title: &str,
    headers: Option<&str>,
    widths: Option<&str>,
    align: Option<&str>,
    content: &str,
) -> String {
    let mut html = String::with_capacity(content.len() * 3);

    // Table opening with alignment
    let style = match align {
        Some("center") => " style=\"margin-left: auto; margin-right: auto;\"",
        Some("left") => " style=\"margin-right: auto;\"",
        Some("right") => " style=\"margin-left: auto;\"",
        _ => "",
    };
    html.push_str(&format!("<table class=\"csv-table\"{}>\n", style));

    // Caption
    if !title.is_empty() {
        html.push_str(&format!("<caption>{}</caption>\n", escape_html(title)));
    }

    // Column widths
    if let Some(w) = widths {
        html.push_str("<colgroup>\n");
        for width in w.split(',').map(|s| s.trim()) {
            if let Ok(n) = width.parse::<f64>() {
                let total: f64 = w
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f64>().ok())
                    .sum();
                let pct = (n / total) * 100.0;
                html.push_str(&format!("<col style=\"width: {:.1}%\">\n", pct));
            }
        }
        html.push_str("</colgroup>\n");
    }

    // Header row
    if let Some(h) = headers {
        html.push_str("<thead>\n<tr>");
        for cell in parse_csv_line(h) {
            html.push_str(&format!(
                "<th>{}</th>",
                process_inline(cell.trim())
            ));
        }
        html.push_str("</tr>\n</thead>\n");
    }

    // Body
    html.push_str("<tbody>\n");
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        html.push_str("<tr>");
        for cell in parse_csv_line(trimmed) {
            html.push_str(&format!(
                "<td>{}</td>",
                process_inline(cell.trim())
            ));
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</tbody>\n</table>");

    html
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if ch == ',' && !in_quotes {
            cells.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    cells.push(current);
    cells
}

/// Convert a list-table to HTML.
pub fn convert_list_table(
    title: &str,
    header_rows: usize,
    stub_columns: usize,
    widths: Option<&str>,
    align: Option<&str>,
    content: &str,
) -> String {
    let mut html = String::with_capacity(content.len() * 3);

    let style = match align {
        Some("center") => " style=\"margin-left: auto; margin-right: auto;\"",
        Some("left") => " style=\"margin-right: auto;\"",
        Some("right") => " style=\"margin-left: auto;\"",
        _ => "",
    };
    html.push_str(&format!("<table class=\"list-table\"{}>\n", style));

    if !title.is_empty() {
        html.push_str(&format!("<caption>{}</caption>\n", escape_html(title)));
    }

    // Column widths
    if let Some(w) = widths {
        html.push_str("<colgroup>\n");
        let parts: Vec<&str> = w.split_whitespace().collect();
        let total: f64 = parts.iter().filter_map(|s| s.parse::<f64>().ok()).sum();
        for part in &parts {
            if let Ok(n) = part.parse::<f64>() {
                let pct = (n / total) * 100.0;
                html.push_str(&format!("<col style=\"width: {:.1}%\">\n", pct));
            }
        }
        html.push_str("</colgroup>\n");
    }

    // Parse list-table rows
    let rows = parse_list_table_rows(content);

    for (row_idx, row) in rows.iter().enumerate() {
        if row_idx == 0 && header_rows > 0 {
            html.push_str("<thead>\n");
        }
        if row_idx == header_rows && header_rows > 0 {
            html.push_str("</thead>\n<tbody>\n");
        }

        html.push_str("<tr>");
        for (col_idx, cell) in row.iter().enumerate() {
            let is_header = row_idx < header_rows;
            let is_stub = col_idx < stub_columns;
            let tag = if is_header || is_stub { "th" } else { "td" };
            let class = if is_stub && !is_header {
                " class=\"stub\""
            } else if is_stub && is_header {
                " class=\"stub\""
            } else {
                ""
            };
            html.push_str(&format!(
                "<{tag}{class}>{}</{tag}>",
                process_inline(cell.trim())
            ));
        }
        html.push_str("</tr>\n");
    }

    if header_rows > 0 && !rows.is_empty() {
        html.push_str("</tbody>\n");
    }

    html.push_str("</table>");
    html
}

fn parse_list_table_rows(content: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();
    let mut in_row = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("* -") || trimmed.starts_with("* - ") {
            // Start of new row
            if in_row {
                if !current_cell.is_empty() {
                    current_row.push(current_cell.clone());
                    current_cell.clear();
                }
                rows.push(current_row.clone());
                current_row.clear();
            }
            in_row = true;
            let cell_content = trimmed.trim_start_matches("* -").trim_start_matches("* - ").trim();
            current_cell = cell_content.to_string();
        } else if trimmed.starts_with("- ") && in_row {
            // New cell in current row
            if !current_cell.is_empty() {
                current_row.push(current_cell.clone());
            }
            current_cell = trimmed[2..].trim().to_string();
        } else if in_row && !trimmed.is_empty() {
            // Continuation of current cell
            if !current_cell.is_empty() {
                current_cell.push(' ');
            }
            current_cell.push_str(trimmed);
        }
    }

    // Flush last row
    if in_row {
        if !current_cell.is_empty() {
            current_row.push(current_cell);
        }
        rows.push(current_row);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_simple_table() {
        let table = "=====  =====\nA      B\n=====  =====";
        assert!(is_simple_table(table));
    }

    #[test]
    fn test_is_simple_table_too_few_lines() {
        let table = "=====  =====\nA      B";
        assert!(!is_simple_table(table));
    }

    #[test]
    fn test_is_simple_table_no_borders() {
        let table = "A      B\nC      D\nE      F";
        assert!(!is_simple_table(table));
    }

    #[test]
    fn test_is_simple_table_empty() {
        assert!(!is_simple_table(""));
    }

    #[test]
    fn test_convert_simple_table_basic() {
        let table = "=====  =====\nA      B\n=====  =====";
        let html = convert_simple_table(table);
        assert!(html.contains("<table"));
        assert!(html.contains("simple-table"));
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
    }

    #[test]
    fn test_convert_simple_table_with_header() {
        let table = "=====  =====\nCol1   Col2\n=====  =====\nA      B\nC      D\n=====  =====";
        let html = convert_simple_table(table);
        assert!(html.contains("<thead>"));
        assert!(html.contains("<th>"));
        assert!(html.contains("<tbody>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_is_grid_table() {
        let table = "+------+------+\n| A    | B    |\n+======+======+\n| 1    | 2    |\n+------+------+";
        assert!(is_grid_table(table));
    }

    #[test]
    fn test_is_grid_table_invalid() {
        assert!(!is_grid_table("This is not a table\nJust some text"));
    }

    #[test]
    fn test_convert_grid_table_basic() {
        let table = "+------+------+\n| A    | B    |\n+======+======+\n| 1    | 2    |\n+------+------+";
        let html = convert_grid_table(table);
        assert!(html.contains("<table"));
        assert!(html.contains("grid-table"));
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
    }

    #[test]
    fn test_parse_csv_line() {
        let cells = parse_csv_line("\"Alice\", 28, \"New York\"");
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].trim(), "Alice");
        assert_eq!(cells[1].trim(), "28");
        assert_eq!(cells[2].trim(), "New York");
    }
}
