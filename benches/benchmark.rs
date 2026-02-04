use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rst2html::convert;

fn generate_large_changelog() -> String {
    let mut lines = String::new();
    lines.push_str("Changelog\n");
    lines.push_str("=========\n\n");

    for i in 1..=1000 {
        lines.push_str(&format!(
            "* `@user{i} <https://github.com/user{i}>`__: Description of change (`#{i} <https://github.com/org/repo/pull/{i}>`__)\n"
        ));
    }
    lines
}

fn load_sample_document() -> String {
    include_str!("../benchmarks/sample.rst").to_string()
}

fn bench_simple_paragraph(c: &mut Criterion) {
    let input = "This is a simple paragraph with **bold** and *italic* text.";
    c.bench_function("simple_paragraph", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_sections_and_lists(c: &mut Criterion) {
    let input = r#"Title
=====

Subtitle
--------

- Item 1
- Item 2
- Item 3

1. First
2. Second
3. Third
"#;
    c.bench_function("sections_and_lists", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_code_blocks(c: &mut Criterion) {
    let input = r#".. code-block:: python

   def hello():
       print("Hello, World!")
       for i in range(10):
           print(i)

.. code-block:: javascript

   function hello() {
       console.log("Hello");
   }
"#;
    c.bench_function("code_blocks", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_large_changelog(c: &mut Criterion) {
    let input = generate_large_changelog();
    c.bench_function("large_changelog_1000_entries", |b| {
        b.iter(|| convert(black_box(&input)))
    });
}

fn bench_inline_markup(c: &mut Criterion) {
    let input = "This has **bold**, *italic*, ``code``, :ref:`label`, :doc:`doc`, and `link <https://example.com>`_ text.";
    c.bench_function("inline_markup", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_tables(c: &mut Criterion) {
    let input = r#"=====  =====  =======
  A      B    A and B
=====  =====  =======
False  False  False
True   False  False
False  True   False
True   True   True
=====  =====  =======
"#;
    c.bench_function("simple_table", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_full_document(c: &mut Criterion) {
    let input = load_sample_document();
    c.bench_function("full_document", |b| {
        b.iter(|| convert(black_box(&input)))
    });
}

fn bench_grid_table(c: &mut Criterion) {
    let input = r#"+---------------+---------------+---------------+
| Header 1      | Header 2      | Header 3      |
+===============+===============+===============+
| Row 1, Col 1  | Row 1, Col 2  | Row 1, Col 3  |
+---------------+---------------+---------------+
| Row 2, Col 1  | Row 2 spans two columns       |
+---------------+---------------+---------------+
| Row 3 spans   | Row 3, Col 2  | Row 3, Col 3  |
| two lines     |               |               |
+---------------+---------------+---------------+
| Row 4, Col 1  | Row 4, Col 2  | Row 4, Col 3  |
+---------------+               +---------------+
| Row 5, Col 1  |               | Row 5, Col 3  |
+---------------+---------------+---------------+
"#;
    c.bench_function("grid_table", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_many_directives(c: &mut Criterion) {
    let input = r#"Title
=====

.. note::

   This is a note about something important.

.. warning::

   Be careful with this operation.

.. tip::

   Here's a helpful tip for users.

.. important::

   Do not skip this step.

.. caution::

   Proceed with caution when modifying these settings.

.. danger::

   This operation cannot be undone!

.. error::

   An error occurred during processing.

.. hint::

   Try using the ``--verbose`` flag for more details.

.. attention::

   This section requires your attention.

.. code-block:: python

   def process_data(items):
       for item in items:
           validate(item)
           transform(item)
           store(item)

.. code-block:: rust

   fn main() {
       let data = vec![1, 2, 3, 4, 5];
       let sum: i32 = data.iter().sum();
       println!("Sum: {}", sum);
   }

.. code-block:: javascript

   const express = require('express');
   const app = express();
   app.get('/', (req, res) => res.send('Hello'));
   app.listen(3000);

.. admonition:: Custom title

   This is a custom admonition with arbitrary content.

.. topic:: Summary

   This topic summarizes the key points discussed above.

.. sidebar:: Related

   See also the advanced configuration guide.
"#;
    c.bench_function("many_directives", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_many_roles(c: &mut Criterion) {
    let mut lines = Vec::new();
    lines.push("Document with many roles\n========================\n".to_string());
    for i in 0..50 {
        lines.push(format!(
            "Line {i}: :code:`code_{i}` and :emphasis:`emph_{i}` and :strong:`strong_{i}` \
             and :literal:`lit_{i}` and :title-reference:`title_{i}` and :math:`x^{i}` \
             and :sub:`sub_{i}` and :sup:`sup_{i}`."
        ));
        lines.push(String::new());
    }
    let input = lines.join("\n");
    c.bench_function("many_roles", |b| {
        b.iter(|| convert(black_box(&input)))
    });
}

fn bench_nested_lists(c: &mut Criterion) {
    let input = r#"Nested lists
============

- Level 1, item 1

  - Level 2, item 1

    - Level 3, item 1

      - Level 4, item 1
      - Level 4, item 2
      - Level 4, item 3

    - Level 3, item 2
    - Level 3, item 3

  - Level 2, item 2

    1. Enum level 3, item 1
    2. Enum level 3, item 2

       a. Enum level 4, item 1
       b. Enum level 4, item 2
       c. Enum level 4, item 3

    3. Enum level 3, item 3

  - Level 2, item 3

- Level 1, item 2

  - Level 2, item 1
  - Level 2, item 2
  - Level 2, item 3

    - Level 3, item 1
    - Level 3, item 2

- Level 1, item 3

  1. First
  2. Second

     - Sub-bullet 1
     - Sub-bullet 2

  3. Third
"#;
    c.bench_function("nested_lists", |b| {
        b.iter(|| convert(black_box(input)))
    });
}

fn bench_large_document(c: &mut Criterion) {
    let base = load_sample_document();
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str(&base);
        input.push('\n');
    }
    c.bench_function("large_document_10x", |b| {
        b.iter(|| convert(black_box(&input)))
    });
}

criterion_group!(
    benches,
    bench_simple_paragraph,
    bench_sections_and_lists,
    bench_code_blocks,
    bench_large_changelog,
    bench_inline_markup,
    bench_tables,
    bench_full_document,
    bench_grid_table,
    bench_many_directives,
    bench_many_roles,
    bench_nested_lists,
    bench_large_document,
);
criterion_main!(benches);
