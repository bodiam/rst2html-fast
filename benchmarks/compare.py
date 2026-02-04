#!/usr/bin/env python3
"""
Benchmark comparison: rst2html-fast vs docutils vs Sphinx.

Measures the time each tool takes to convert the same RST document to HTML.

Requirements:
    pip install docutils sphinx

Usage:
    python3 benchmarks/compare.py
"""

import os
import subprocess
import sys
import time
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
SAMPLE_RST = SCRIPT_DIR / "sample.rst"
ITERATIONS = 100
WARMUP = 5


def find_rst2html_binary():
    """Find the rst2html-fast binary."""
    project_root = SCRIPT_DIR.parent
    release_bin = project_root / "target" / "release" / "rst2html"
    debug_bin = project_root / "target" / "debug" / "rst2html"

    if release_bin.exists():
        return str(release_bin)
    if debug_bin.exists():
        print("WARNING: Using debug build. Run 'cargo build --release' for accurate benchmarks.\n")
        return str(debug_bin)
    return None


def bench_rst2html_fast(rst_content, binary_path, iterations):
    """Benchmark rst2html-fast via subprocess."""
    # Warmup
    for _ in range(WARMUP):
        proc = subprocess.run(
            [binary_path],
            input=rst_content,
            capture_output=True,
            text=True,
        )

    start = time.perf_counter()
    for _ in range(iterations):
        proc = subprocess.run(
            [binary_path],
            input=rst_content,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            print(f"rst2html-fast error: {proc.stderr}", file=sys.stderr)
            return None
    elapsed = time.perf_counter() - start
    return elapsed / iterations


def bench_docutils(rst_content, iterations):
    """Benchmark docutils rst2html conversion."""
    try:
        from docutils.core import publish_parts
    except ImportError:
        return None

    # Warmup
    for _ in range(WARMUP):
        publish_parts(rst_content, writer_name="html")

    start = time.perf_counter()
    for _ in range(iterations):
        publish_parts(rst_content, writer_name="html")
    elapsed = time.perf_counter() - start
    return elapsed / iterations


def bench_sphinx(rst_content, iterations):
    """Benchmark Sphinx conversion via subprocess."""
    try:
        import sphinx  # noqa: F401
    except ImportError:
        return None

    import tempfile
    import shutil

    tmpdir = tempfile.mkdtemp(prefix="rst2html_bench_")
    srcdir = os.path.join(tmpdir, "source")
    outdir = os.path.join(tmpdir, "build")
    os.makedirs(srcdir)

    # Write minimal conf.py
    with open(os.path.join(srcdir, "conf.py"), "w") as f:
        f.write("project = 'bench'\nextensions = []\n")

    # Write the RST content as index.rst
    with open(os.path.join(srcdir, "index.rst"), "w") as f:
        f.write(rst_content)

    try:
        # Warmup
        for _ in range(min(WARMUP, 2)):
            if os.path.exists(outdir):
                shutil.rmtree(outdir)
            subprocess.run(
                [sys.executable, "-m", "sphinx", "-b", "html", "-q", srcdir, outdir],
                capture_output=True,
                text=True,
            )

        # Fewer iterations for Sphinx since it's much slower
        sphinx_iterations = max(1, iterations // 10)
        start = time.perf_counter()
        for _ in range(sphinx_iterations):
            if os.path.exists(outdir):
                shutil.rmtree(outdir)
            proc = subprocess.run(
                [sys.executable, "-m", "sphinx", "-b", "html", "-q", srcdir, outdir],
                capture_output=True,
                text=True,
            )
            if proc.returncode != 0:
                print(f"Sphinx error: {proc.stderr}", file=sys.stderr)
                shutil.rmtree(tmpdir, ignore_errors=True)
                return None
        elapsed = time.perf_counter() - start
        return elapsed / sphinx_iterations
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def format_time(seconds):
    """Format time in appropriate units."""
    if seconds is None:
        return "N/A"
    if seconds < 0.001:
        return f"{seconds * 1_000_000:.1f} µs"
    if seconds < 1:
        return f"{seconds * 1_000:.2f} ms"
    return f"{seconds:.2f} s"


def main():
    print("=" * 65)
    print("  rst2html-fast benchmark comparison")
    print("=" * 65)
    print()

    if not SAMPLE_RST.exists():
        print(f"Error: sample document not found at {SAMPLE_RST}", file=sys.stderr)
        sys.exit(1)

    rst_content = SAMPLE_RST.read_text()
    lines = len(rst_content.splitlines())
    print(f"Document: {SAMPLE_RST.name} ({lines} lines, {len(rst_content)} bytes)")
    print(f"Iterations: {ITERATIONS} (Sphinx: {max(1, ITERATIONS // 10)})")
    print()

    # Find rst2html-fast binary
    binary = find_rst2html_binary()
    if binary is None:
        print("WARNING: rst2html-fast binary not found.")
        print("         Run 'cargo build --release' first.")
        print()

    # Run benchmarks
    results = {}

    if binary:
        print("Benchmarking rst2html-fast...", end=" ", flush=True)
        t = bench_rst2html_fast(rst_content, binary, ITERATIONS)
        results["rst2html-fast"] = t
        print(format_time(t))

    print("Benchmarking docutils...", end=" ", flush=True)
    t = bench_docutils(rst_content, ITERATIONS)
    if t is None:
        print("not installed (pip install docutils)")
    else:
        results["docutils"] = t
        print(format_time(t))

    print("Benchmarking Sphinx...", end=" ", flush=True)
    t = bench_sphinx(rst_content, ITERATIONS)
    if t is None:
        print("not installed (pip install sphinx)")
    else:
        results["Sphinx"] = t
        print(format_time(t))

    print()

    # Print comparison table
    if not results:
        print("No benchmarks were run.")
        return

    baseline = results.get("rst2html-fast")

    print("-" * 50)
    print(f"{'Tool':<20} {'Time':>12} {'Relative':>12}")
    print("-" * 50)

    for name in ["rst2html-fast", "docutils", "Sphinx"]:
        if name in results:
            t = results[name]
            time_str = format_time(t)
            if baseline and name != "rst2html-fast":
                ratio = t / baseline
                rel_str = f"{ratio:.0f}x slower"
            else:
                rel_str = "baseline" if baseline else ""
            print(f"{name:<20} {time_str:>12} {rel_str:>12}")

    print("-" * 50)
    print()

    if baseline and "docutils" in results:
        ratio = results["docutils"] / baseline
        print(f"rst2html-fast is {ratio:.0f}x faster than docutils")
    if baseline and "Sphinx" in results:
        ratio = results["Sphinx"] / baseline
        print(f"rst2html-fast is {ratio:.0f}x faster than Sphinx")


if __name__ == "__main__":
    main()
