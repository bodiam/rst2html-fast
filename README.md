# rst2html-fast

The world's fastest reStructuredText to HTML converter. Written in Rust for maximum performance.

## Performance

rst2html-fast is **~200x faster** than docutils and **~400x faster** than Sphinx for typical RST documents.

| Tool | Time (347-line doc) | Relative |
|---|---|---|
| **rst2html-fast** | ~0.1 ms | 1x |
| docutils | ~20 ms | ~200x slower |
| Sphinx | ~40 ms | ~400x slower |

Run your own benchmarks with `python3 benchmarks/compare.py`.

## Installation

### From source

```bash
cargo install --path .
```

### Build manually

```bash
cargo build --release
# Binary is at ./target/release/rst2html
```

## Usage

### Convert a file

```bash
rst2html document.rst > output.html
```

### Convert from stdin

```bash
echo "**Hello** *world*" | rst2html
```

### Use as a library

```rust
use rst2html::convert;

let html = convert("**Hello** *world*");
assert_eq!(html, "<p><strong>Hello</strong> <em>world</em></p>\n");
```

## IntelliJ RST plugin integration

rst2html-fast can be used as an external rendering engine for the [IntelliJ reStructuredText plugin](https://github.com/Jetplugins/intellij-rst-plugin). The plugin invokes the binary as an external process (similar to how it uses Sphinx):

1. Build or install `rst2html`
2. The plugin sends RST content via stdin
3. HTML is returned on stdout

This is the same pattern used by the plugin's Sphinx integration, making rst2html-fast a drop-in alternative for fast preview rendering.

## Supported features

- Paragraphs and inline markup (bold, italic, inline code)
- Sections with underline/overline characters
- Bullet lists and enumerated lists (including nested)
- Simple tables and grid tables
- Code blocks with language annotation (`.. code-block::`)
- Directives: note, warning, tip, important, caution, danger, error, hint, attention, admonition, image, figure, topic, sidebar, container, rubric, epigraph, highlights, pull-quote, compound, math, raw, include, contents, replace
- Roles: ref, doc, code, math, sub, sup, title-reference, abbreviation, emphasis, strong, literal, and more
- Field lists, definition lists, option lists
- Line blocks
- Block quotes
- Transitions
- Footnotes and citations
- Hyperlink references and targets
- Substitution references
- Comments

## Running benchmarks

### Rust benchmarks (Criterion)

```bash
cargo bench
```

### Comparison with docutils and Sphinx

Requires Python 3 with `docutils` and `sphinx` installed:

```bash
pip install docutils sphinx
python3 benchmarks/compare.py
```

## Running tests

```bash
cargo test
```

## License

MIT
