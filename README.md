# rst2html-fast

The world's fastest reStructuredText to HTML converter. Written in Rust for maximum performance.

## Performance

Benchmarked on a 413-line RST document (Apple Silicon, 100 iterations):

| Tool | Language | Time | vs rst2html-fast |
|---|---|---|---|
| **rst2html-fast** | Rust | 4.27 ms | baseline |
| docutils | Python | 30.08 ms | 7x slower |
| Gregwar/RST | PHP | 60.50 ms | 14x slower |
| Nim rst2html | Nim | 61.73 ms | 14x slower |
| Pandoc | Haskell | 80.98 ms | 19x slower |
| Sphinx | Python | 469.06 ms | 110x slower |

Run your own benchmarks with `benchmarks/.venv/bin/python benchmarks/compare.py`.

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

### Comparison with other converters

Compares against docutils, Pandoc, Sphinx, Nim rst2html, and Gregwar/RST (PHP):

```bash
python3 -m venv benchmarks/.venv
benchmarks/.venv/bin/pip install docutils sphinx
benchmarks/.venv/bin/python benchmarks/compare.py
```

The script auto-detects installed tools. Optionally install more converters:

```bash
brew install pandoc nim
cd benchmarks && composer require gregwar/rst
```

## Running tests

```bash
cargo test
```

## License

[MIT](LICENSE)
