#!/usr/bin/env python3
"""
Benchmark comparison: rst2html-fast vs 5 other RST-to-HTML converters.

Compares:
  1. docutils     (Python)  - the reference implementation
  2. Pandoc       (Haskell) - universal document converter
  3. Sphinx       (Python)  - documentation system
  4. Nim rst2html (Nim/C)   - compiled language alternative
  5. Gregwar/RST  (PHP)     - mature PHP implementation

Setup:
    cargo build --release
    python3 -m venv benchmarks/.venv
    benchmarks/.venv/bin/pip install docutils sphinx
    brew install pandoc
    brew install nim
    cd benchmarks && composer require gregwar/rst

Usage:
    benchmarks/.venv/bin/python benchmarks/compare.py
"""

import os
import subprocess
import shutil
import sys
import tempfile
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent
SAMPLE_RST = SCRIPT_DIR / "sample.rst"
ITERATIONS = 100
WARMUP = 5

# Ordered list of all tools for display
TOOL_ORDER = ["rst2html-fast", "docutils", "Pandoc", "Sphinx", "Nim rst2html", "Gregwar/RST"]

# Gregwar/RST only supports basic RST constructs (no admonitions, limited roles).
# We benchmark it with a simpler document of comparable length using supported features.
GREGWAR_SAMPLE = """\
==================================
rst2html-fast benchmark document
==================================

Introduction
============

This is a realistic reStructuredText document used for benchmarking RST-to-HTML
converters. It exercises a range of RST features including sections,
inline markup, lists, tables, code blocks, and more.

This paragraph contains **bold text**, *italic text*, ``inline code``,
and a `hyperlink <https://example.com>`_. Another paragraph follows
with more **bold**, *italic*, and ``code`` content.

Getting started
===============

Installation
------------

You can install the package using pip::

   pip install rst2html-fast
   rst2html --version

Or build from source::

   git clone https://github.com/bodiam/rst2html-fast.git
   cd rst2html-fast
   cargo build --release

Configuration
-------------

The configuration file supports the following settings. Each setting
can be overridden via environment variables.

API reference
=============

Core module
-----------

The ``convert`` function
^^^^^^^^^^^^^^^^^^^^^^^^

The main entry point for converting RST to HTML::

   use rst2html::convert;

   let html = convert("**Hello** *world*");
   assert_eq!(html, "<p><strong>Hello</strong> <em>world</em></p>");

The ``convert_with_options`` function
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For more control over the conversion::

   use rst2html::{convert_with_options, ConvertOptions};

   let options = ConvertOptions {
       standalone: true,
       ..Default::default()
   };
   let html = convert_with_options("Hello", &options);

Data types
==========

Supported formats
-----------------

=============  ===========  ========  ===========
Format         Extension    Binary    Text
=============  ===========  ========  ===========
RST            ``.rst``     No        Yes
Markdown       ``.md``      No        Yes
HTML           ``.html``    No        Yes
PDF            ``.pdf``     Yes       No
Word           ``.docx``    Yes       No
=============  ===========  ========  ===========

Examples
========

Lists
-----

Bullet lists:

- First item with **bold**
- Second item with *italic*
- Third item with ``code``

  - Nested item 1
  - Nested item 2

    - Deeply nested item A
    - Deeply nested item B

  - Nested item 3

- Fourth item

Enumerated lists:

1. Step one: prepare the environment
2. Step two: install dependencies
3. Step three: run the build

   a. Sub-step: configure options
   b. Sub-step: compile sources
   c. Sub-step: run tests

4. Step four: deploy

Code examples
-------------

Python example::

   import asyncio

   async def fetch_data(url):
       async with aiohttp.ClientSession() as session:
           async with session.get(url) as response:
               return await response.json()

   async def main():
       urls = [
           "https://api.example.com/users",
           "https://api.example.com/posts",
           "https://api.example.com/comments",
       ]
       tasks = [fetch_data(url) for url in urls]
       results = await asyncio.gather(*tasks)
       for result in results:
           print(result)

   asyncio.run(main())

JavaScript example::

   class EventEmitter {
       constructor() {
           this.listeners = new Map();
       }

       on(event, callback) {
           if (!this.listeners.has(event)) {
               this.listeners.set(event, []);
           }
           this.listeners.get(event).push(callback);
           return this;
       }

       emit(event, ...args) {
           const callbacks = this.listeners.get(event) || [];
           callbacks.forEach(cb => cb(...args));
           return this;
       }
   }

Block quotes and attributions:

   "The best way to predict the future is to invent it."

   -- Alan Kay

Advanced features
=================

Hyperlink targets
-----------------

See the `installation`_ section for setup instructions.

Visit the `project homepage`_ for more information.

.. _project homepage: https://github.com/bodiam/rst2html-fast

Changelog
=========

Version 0.1.0
--------------

* Initial release
* Support for basic RST constructs
* CLI with file and stdin input
* Library API with ``convert()`` and ``convert_with_options()``

Version 0.0.1
--------------

* `@bodiam <https://github.com/bodiam>`__: Initial prototype
* `@contributor <https://github.com/contributor>`__: Added table support
* `@helper <https://github.com/helper>`__: Fixed inline markup parsing
* `@reviewer <https://github.com/reviewer>`__: Improved error handling
* `@tester <https://github.com/tester>`__: Added integration tests

License
=======

This project is licensed under the MIT License. See the ``LICENSE`` file
for details.
"""


# ---------------------------------------------------------------------------
# Tool detection
# ---------------------------------------------------------------------------

def find_rst2html_binary():
    """Find the rst2html-fast binary."""
    release_bin = PROJECT_ROOT / "target" / "release" / "rst2html"
    debug_bin = PROJECT_ROOT / "target" / "debug" / "rst2html"

    if release_bin.exists():
        return str(release_bin)
    if debug_bin.exists():
        print("  WARNING: Using debug build. Run 'cargo build --release' for accurate benchmarks.")
        return str(debug_bin)
    return None


def find_pandoc():
    """Find the pandoc binary."""
    return shutil.which("pandoc")


def find_nim():
    """Find the nim binary."""
    return shutil.which("nim")


def find_php():
    """Find PHP and check if Gregwar/RST is available."""
    php = shutil.which("php")
    if not php:
        return None
    # Check if Gregwar/RST autoloader exists (installed via composer)
    autoloader = SCRIPT_DIR / "vendor" / "autoload.php"
    if not autoloader.exists():
        return None
    return php


# ---------------------------------------------------------------------------
# Benchmark functions
# ---------------------------------------------------------------------------

def bench_subprocess(cmd, input_text, iterations, warmup=WARMUP):
    """Generic subprocess benchmark. Returns average time per iteration."""
    for _ in range(warmup):
        subprocess.run(cmd, input=input_text, capture_output=True, text=True)

    start = time.perf_counter()
    for _ in range(iterations):
        proc = subprocess.run(cmd, input=input_text, capture_output=True, text=True)
        if proc.returncode != 0:
            return None, proc.stderr.strip()
    elapsed = time.perf_counter() - start
    return elapsed / iterations, None


def bench_rst2html_fast(rst_content, binary_path, iterations):
    """Benchmark rst2html-fast via subprocess."""
    return bench_subprocess([binary_path], rst_content, iterations)


def bench_docutils(rst_content, iterations):
    """Benchmark docutils rst2html conversion (in-process for accuracy)."""
    try:
        from docutils.core import publish_parts
    except ImportError:
        return None, "not installed"

    # Warmup
    for _ in range(WARMUP):
        publish_parts(rst_content, writer_name="html")

    start = time.perf_counter()
    for _ in range(iterations):
        publish_parts(rst_content, writer_name="html")
    elapsed = time.perf_counter() - start
    return elapsed / iterations, None


def bench_pandoc(rst_content, pandoc_path, iterations):
    """Benchmark Pandoc RST-to-HTML conversion."""
    return bench_subprocess(
        [pandoc_path, "-f", "rst", "-t", "html"],
        rst_content,
        iterations,
    )


def bench_sphinx(rst_content, iterations):
    """Benchmark Sphinx conversion via subprocess."""
    try:
        import sphinx  # noqa: F401
    except ImportError:
        return None, "not installed"

    tmpdir = tempfile.mkdtemp(prefix="rst2html_bench_")
    srcdir = os.path.join(tmpdir, "source")
    outdir = os.path.join(tmpdir, "build")
    os.makedirs(srcdir)

    with open(os.path.join(srcdir, "conf.py"), "w") as f:
        f.write("project = 'bench'\nextensions = []\n")
    with open(os.path.join(srcdir, "index.rst"), "w") as f:
        f.write(rst_content)

    try:
        # Fewer iterations for Sphinx since it's much slower
        sphinx_iters = max(1, iterations // 10)

        # Warmup
        for _ in range(min(WARMUP, 2)):
            if os.path.exists(outdir):
                shutil.rmtree(outdir)
            subprocess.run(
                [sys.executable, "-m", "sphinx", "-b", "html", "-q", srcdir, outdir],
                capture_output=True, text=True,
            )

        start = time.perf_counter()
        for _ in range(sphinx_iters):
            if os.path.exists(outdir):
                shutil.rmtree(outdir)
            proc = subprocess.run(
                [sys.executable, "-m", "sphinx", "-b", "html", "-q", srcdir, outdir],
                capture_output=True, text=True,
            )
            if proc.returncode != 0:
                return None, proc.stderr.strip()[:200]
        elapsed = time.perf_counter() - start
        return elapsed / sphinx_iters, None
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def bench_nim(rst_content, nim_path, iterations):
    """Benchmark Nim's built-in rst2html via a temp file.

    Nim's RST parser has limited directive support (no grid tables, topic,
    sidebar, etc.), so we use the same basic sample as Gregwar.
    """
    nim_content = GREGWAR_SAMPLE

    tmpdir = tempfile.mkdtemp(prefix="rst2html_bench_nim_")
    rst_file = os.path.join(tmpdir, "input.rst")

    with open(rst_file, "w") as f:
        f.write(nim_content)

    try:
        # Nim rst2html reads from file, outputs .html alongside
        cmd = [nim_path, "rst2html", "--hints:off", rst_file]

        # Warmup
        for _ in range(WARMUP):
            subprocess.run(cmd, capture_output=True, text=True)

        start = time.perf_counter()
        for _ in range(iterations):
            proc = subprocess.run(cmd, capture_output=True, text=True)
            if proc.returncode != 0:
                return None, proc.stderr.strip()[:200]
        elapsed = time.perf_counter() - start
        return elapsed / iterations, None
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def bench_gregwar(rst_content, php_path, iterations):
    """Benchmark Gregwar/RST PHP parser.

    Gregwar/RST has limited directive/role support, so we use a basic RST
    document to get a fair parse-speed comparison on common constructs.
    """
    # Use a basic subset that Gregwar can handle
    simplified = GREGWAR_SAMPLE

    php_script = SCRIPT_DIR / "_bench_gregwar.php"
    php_code = """\
<?php
error_reporting(E_ALL & ~E_DEPRECATED);
require_once __DIR__ . '/vendor/autoload.php';
use Gregwar\\RST\\Parser;

$rst = file_get_contents('php://stdin');
$parser = new Parser();
$html = $parser->parse($rst);
echo $html;
"""
    php_script.write_text(php_code)
    try:
        return bench_subprocess(
            [php_path, str(php_script)],
            simplified,
            iterations,
        )
    finally:
        php_script.unlink(missing_ok=True)


# ---------------------------------------------------------------------------
# Formatting
# ---------------------------------------------------------------------------

def format_time(seconds):
    """Format time in human-readable units."""
    if seconds is None:
        return "N/A"
    if seconds < 0.001:
        return f"{seconds * 1_000_000:.1f} us"
    if seconds < 1:
        return f"{seconds * 1_000:.2f} ms"
    return f"{seconds:.2f} s"


def format_relative(seconds, baseline):
    """Format relative speed vs baseline."""
    if seconds is None or baseline is None:
        return ""
    if seconds == baseline:
        return "baseline"
    ratio = seconds / baseline
    if ratio >= 1.5:
        return f"{ratio:.0f}x slower"
    return f"{ratio:.1f}x slower"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    print("=" * 70)
    print("  rst2html-fast benchmark suite")
    print("=" * 70)
    print()

    if not SAMPLE_RST.exists():
        print(f"Error: sample document not found at {SAMPLE_RST}", file=sys.stderr)
        sys.exit(1)

    rst_content = SAMPLE_RST.read_text()
    lines = len(rst_content.splitlines())
    print(f"  Document: {SAMPLE_RST.name} ({lines} lines, {len(rst_content):,} bytes)")
    print(f"  Iterations: {ITERATIONS} (Sphinx: {max(1, ITERATIONS // 10)})")
    print()

    # Detect tools
    print("Detecting tools...")
    binary = find_rst2html_binary()
    pandoc = find_pandoc()
    nim = find_nim()
    php = find_php()

    tools_found = []
    tools_missing = []

    if binary:
        tools_found.append("rst2html-fast")
    else:
        tools_missing.append(("rst2html-fast", "cargo build --release"))

    # docutils - checked at runtime via import
    try:
        import docutils  # noqa: F401
        tools_found.append("docutils")
    except ImportError:
        tools_missing.append(("docutils", "pip install docutils"))

    if pandoc:
        tools_found.append("Pandoc")
    else:
        tools_missing.append(("Pandoc", "brew install pandoc"))

    try:
        import sphinx  # noqa: F401
        tools_found.append("Sphinx")
    except ImportError:
        tools_missing.append(("Sphinx", "pip install sphinx"))

    if nim:
        tools_found.append("Nim rst2html")
    else:
        tools_missing.append(("Nim rst2html", "brew install nim"))

    if php:
        tools_found.append("Gregwar/RST")
    else:
        tools_missing.append(("Gregwar/RST", "cd benchmarks && composer require gregwar/rst"))

    for name in tools_found:
        print(f"  [ok] {name}")
    for name, install_cmd in tools_missing:
        print(f"  [--] {name:<16} (install: {install_cmd})")

    if not tools_found:
        print("\nNo tools found. Install at least one to run benchmarks.")
        sys.exit(1)

    print()
    print("-" * 70)
    print("Running benchmarks...")
    print("-" * 70)
    print()

    results = {}

    # 1. rst2html-fast
    if binary:
        print("  rst2html-fast ...", end=" ", flush=True)
        t, err = bench_rst2html_fast(rst_content, binary, ITERATIONS)
        if err:
            print(f"error: {err}")
        else:
            results["rst2html-fast"] = t
            print(format_time(t))

    # 2. docutils
    if "docutils" in tools_found:
        print("  docutils      ...", end=" ", flush=True)
        t, err = bench_docutils(rst_content, ITERATIONS)
        if err:
            print(f"error: {err}")
        else:
            results["docutils"] = t
            print(format_time(t))

    # 3. Pandoc
    if pandoc:
        print("  Pandoc        ...", end=" ", flush=True)
        t, err = bench_pandoc(rst_content, pandoc, ITERATIONS)
        if err:
            print(f"error: {err}")
        else:
            results["Pandoc"] = t
            print(format_time(t))

    # 4. Sphinx
    if "Sphinx" in tools_found:
        print("  Sphinx        ...", end=" ", flush=True)
        t, err = bench_sphinx(rst_content, ITERATIONS)
        if err:
            print(f"error: {err}")
        else:
            results["Sphinx"] = t
            print(format_time(t))

    # 5. Nim rst2html
    if nim:
        print("  Nim rst2html  ...", end=" ", flush=True)
        t, err = bench_nim(rst_content, nim, ITERATIONS)
        if err:
            print(f"error: {err[:120]}")
        else:
            results["Nim rst2html"] = t
            print(f"{format_time(t)}  (simplified input*)")

    # 6. Gregwar/RST
    if php:
        print("  Gregwar/RST   ...", end=" ", flush=True)
        t, err = bench_gregwar(rst_content, php, ITERATIONS)
        if err:
            print(f"error: {err[:120]}")
        else:
            results["Gregwar/RST"] = t
            print(f"{format_time(t)}  (simplified input*)")

    print()

    if not results:
        print("No benchmarks completed successfully.")
        return

    # Results table
    baseline = results.get("rst2html-fast")

    print("=" * 70)
    print(f"  {'Tool':<20} {'Language':<12} {'Time':>12} {'vs rst2html-fast':>16}")
    print("=" * 70)

    tool_langs = {
        "rst2html-fast": "Rust",
        "docutils": "Python",
        "Pandoc": "Haskell",
        "Sphinx": "Python",
        "Nim rst2html": "Nim",
        "Gregwar/RST": "PHP",
    }

    for name in TOOL_ORDER:
        if name in results:
            t = results[name]
            lang = tool_langs[name]
            time_str = format_time(t)
            rel_str = format_relative(t, baseline) if name != "rst2html-fast" else "baseline"
            print(f"  {name:<20} {lang:<12} {time_str:>12} {rel_str:>16}")

    print("=" * 70)
    print()

    # Summary
    if baseline:
        print("Summary:")
        for name in TOOL_ORDER:
            if name in results and name != "rst2html-fast":
                ratio = results[name] / baseline
                print(f"  rst2html-fast is {ratio:.0f}x faster than {name}")

    has_footnotes = "Nim rst2html" in results or "Gregwar/RST" in results
    if has_footnotes:
        print()
    if "Nim rst2html" in results:
        print("  * Nim rst2html uses simplified input (no grid tables, admonitions, or topic/sidebar directives).")
    if "Gregwar/RST" in results:
        print("  * Gregwar/RST uses simplified input (roles/directives stripped) due to limited RST support.")
    print()


if __name__ == "__main__":
    main()
