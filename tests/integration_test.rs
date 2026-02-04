use rst2html::convert;

// ========== Basic Formatting Tests ==========

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
fn test_bold_italic_text() {
    let html = convert("This has ***bold italic*** text.");
    assert!(
        html.contains("<strong><em>bold italic</em></strong>")
            || html.contains("<em><strong>bold italic</strong></em>")
    );
}

// ========== Section Tests ==========

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

// ========== List Tests ==========

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
fn test_nested_bullet_list() {
    let html = convert("- Level 1 item A\n\n  - Level 2 item A.1\n  - Level 2 item A.2\n\n- Level 1 item B");
    assert!(html.contains("<ul>"));
    assert!(html.contains("Level 1 item A"));
    assert!(html.contains("Level 2 item A.1"));
    assert!(html.contains("Level 1 item B"));
}

#[test]
fn test_definition_list_basic() {
    let html = convert("Term 1\n    Definition for term 1.\n\nTerm 2\n    Definition for term 2.");
    assert!(html.contains("<dl"));
    assert!(html.contains("<dt"));
    assert!(html.contains("<dd"));
    assert!(html.contains("Term 1"));
    assert!(html.contains("Definition for term 1"));
}

#[test]
fn test_definition_list_with_classifier() {
    let html = convert("Term : classifier\n    Definition with a classifier.");
    assert!(html.contains("<dl"));
    assert!(html.contains("Term"));
    assert!(html.contains("classifier"));
}

// ========== Option List Tests ==========

#[test]
fn test_option_list_basic() {
    let html = convert("-a            Simple short option.\n-b FILE       Short option with argument.\n--verbose     Long option only.");
    assert!(
        html.contains("option-list") || html.contains("<dl"),
        "HTML: {}",
        html
    );
    assert!(html.contains("-a"));
    assert!(html.contains("Simple short option"));
    assert!(html.contains("--verbose"));
}

// ========== Directive Tests ==========

#[test]
fn test_code_block_directive() {
    let html = convert(".. code-block:: python\n\n   print(\"Hello\")");
    assert!(html.contains("<pre"));
    assert!(html.contains("<code"));
    assert!(html.contains("language-python"));
}

#[test]
fn test_code_block_multiline() {
    let html = convert(".. code-block:: python\n\n   def hello():\n       print(\"Hello\")");
    assert!(html.contains("<pre"));
    assert!(html.contains("language-python"));
    assert!(html.contains("def hello"));
}

#[test]
fn test_code_block_with_language() {
    let html = convert(".. code-block:: javascript\n\n   console.log(\"Hello\");");
    assert!(html.contains("<pre"));
    assert!(html.contains("<code"));
    assert!(html.contains("language-javascript"));
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
fn test_attention_admonition() {
    let html = convert(".. attention::\n\n   Pay attention to this.");
    assert!(html.contains("admonition"));
    assert!(html.contains("attention"));
    assert!(html.contains("Attention"));
}

#[test]
fn test_caution_admonition() {
    let html = convert(".. caution::\n\n   Be careful here.");
    assert!(html.contains("caution"));
    assert!(html.contains("Caution"));
}

#[test]
fn test_danger_admonition() {
    let html = convert(".. danger::\n\n   This is dangerous!");
    assert!(html.contains("danger"));
    assert!(html.contains("Danger"));
}

#[test]
fn test_error_admonition() {
    let html = convert(".. error::\n\n   An error occurred.");
    assert!(html.contains("error"));
    assert!(html.contains("Error"));
}

#[test]
fn test_hint_admonition() {
    let html = convert(".. hint::\n\n   Here's a hint.");
    assert!(html.contains("hint"));
    assert!(html.contains("Hint"));
}

#[test]
fn test_important_admonition() {
    let html = convert(".. important::\n\n   This is important.");
    assert!(html.contains("important"));
    assert!(html.contains("Important"));
}

#[test]
fn test_tip_admonition() {
    let html = convert(".. tip::\n\n   Here's a tip.");
    assert!(html.contains("tip"));
    assert!(html.contains("Tip"));
}

#[test]
fn test_generic_admonition() {
    let html = convert(".. admonition:: Custom Title\n\n   This is a custom admonition.");
    assert!(html.contains("admonition"));
    assert!(html.contains("Custom Title"));
}

#[test]
fn test_seealso_admonition() {
    let html = convert(".. seealso::\n\n   Check the documentation.");
    assert!(html.contains("seealso"));
    assert!(html.contains("See Also"));
}

// ========== Role Tests ==========

#[test]
fn test_ref_role() {
    let html = convert("See :ref:`my-label` for details.");
    assert!(html.contains("<a"));
    assert!(html.contains("href=\"#my-label\""));
}

#[test]
fn test_emphasis_role() {
    let html = convert("This is :emphasis:`emphasized text` here.");
    assert!(html.contains("<em>emphasized text</em>"));
}

#[test]
fn test_strong_role() {
    let html = convert("This is :strong:`strong text` here.");
    assert!(html.contains("<strong>strong text</strong>"));
}

#[test]
fn test_literal_role() {
    let html = convert("This is :literal:`literal text` here.");
    assert!(html.contains("<code>literal text</code>"));
}

#[test]
fn test_code_role() {
    let html = convert("This is :code:`inline code` here.");
    assert!(html.contains("<code>inline code</code>"));
}

#[test]
fn test_subscript_role() {
    let html = convert("Text with :subscript:`2` here.");
    assert!(html.contains("<sub>2</sub>"));
}

#[test]
fn test_superscript_role() {
    let html = convert("Text with :superscript:`2` here.");
    assert!(html.contains("<sup>2</sup>"));
}

#[test]
fn test_title_reference_role() {
    let html = convert("Read :title-reference:`Book Title` for more.");
    assert!(html.contains("<cite>Book Title</cite>"));
}

#[test]
fn test_abbr_role_with_expansion() {
    let html = convert("The :abbr:`RST (reStructuredText)` format.");
    assert!(html.contains("<abbr title=\"reStructuredText\">RST</abbr>"));
}

#[test]
fn test_abbr_role_without_expansion() {
    let html = convert(":abbr:`HTML` is used.");
    assert!(html.contains("<abbr>HTML</abbr>"));
}

#[test]
fn test_doc_role() {
    let html = convert("See :doc:`other-doc` for details.");
    assert!(html.contains("<a"));
    assert!(html.contains("other-doc.html"));
}

#[test]
fn test_doc_role_with_display_text() {
    let html = convert("See :doc:`Visual diff </visual-diff>` for details.");
    assert!(html.contains(">Visual diff</a>"));
    assert!(html.contains("href=\"/visual-diff.html\""));
}

// ========== Escape Tests ==========

#[test]
fn test_backslash_space_removal() {
    let html = convert("H\\ :subscript:`2`\\ O is water.");
    assert!(
        html.contains("H<sub>2</sub>O"),
        "Expected H<sub>2</sub>O, got: {}",
        html
    );
}

#[test]
fn test_double_backslash() {
    let html = convert("Use \\\\ for a literal backslash.");
    assert!(html.contains("Use \\ for"));
}

#[test]
fn test_escaped_asterisk() {
    let html = convert("\\*not italic\\*");
    assert!(html.contains("*not italic*"));
    assert!(!html.contains("<em>"));
}

#[test]
fn test_escaped_backtick() {
    let html = convert("\\`\\`not code\\`\\`");
    assert!(html.contains("``not code``"));
    assert!(!html.contains("<code>not code</code>"));
}

#[test]
fn test_mixed_escapes_and_markup() {
    let html = convert("This is *italic* but \\*this\\* is not.");
    assert!(html.contains("<em>italic</em>"));
    assert!(html.contains("*this*"));
}

#[test]
fn test_escaped_angle_brackets() {
    let html = convert("Angle brackets: \\< \\>");
    assert!(
        html.contains("&lt; &gt;"),
        "Expected &lt; &gt;, got: {}",
        html
    );
    assert!(!html.contains("&amp;lt;"));
}

// ========== Line Block Tests ==========

#[test]
fn test_line_block_basic() {
    let html = convert("| John Smith\n| 123 Main Street\n| New York, NY 10001");
    assert!(html.contains("line-block"));
    assert!(html.contains("John Smith"));
    assert!(html.contains("123 Main Street"));
    assert!(html.contains("New York, NY 10001"));
    assert!(html.contains("class=\"line\""));
}

#[test]
fn test_line_block_with_indentation() {
    let html = convert("| Line one\n|     Indented line\n| Back to normal");
    assert!(html.contains("line-block"));
    assert!(html.contains("Line one"));
    assert!(html.contains("Indented line"));
    assert!(html.contains("margin-left"));
}

#[test]
fn test_line_block_with_empty_line() {
    let html = convert("| First line\n|\n| After empty line");
    assert!(html.contains("line-block"));
    assert!(html.contains("First line"));
    assert!(html.contains("After empty line"));
    assert!(html.contains("<br"));
}

// ========== Literal Block Tests ==========

#[test]
fn test_literal_block() {
    let html = convert("Example::\n\n    This is literal\n    text here");
    assert!(html.contains("<pre"));
    assert!(html.contains("nohighlight"));
    assert!(html.contains("This is literal"));
}

#[test]
fn test_standalone_double_colon_literal_block() {
    let html = convert(
        "The double colon can also be on its own line:\n\n::\n\n    Another literal block.\n    This one started with :: on its own line.",
    );
    assert!(html.contains("The double colon can also be on its own line"));
    assert!(html.contains("<pre"));
    assert!(html.contains("Another literal block"));
    assert!(!html.contains("<p>::</p>"));
}

// ========== Image Tests ==========

#[test]
fn test_image_basic() {
    let html = convert(".. image:: /path/to/image.png");
    assert!(html.contains("<img"));
    assert!(html.contains("src=\"/path/to/image.png\""));
}

#[test]
fn test_image_with_alt() {
    let html = convert(".. image:: /path/to/image.png\n   :alt: Alt text");
    assert!(html.contains("<img"));
    assert!(html.contains("src=\"/path/to/image.png\""));
}

#[test]
fn test_figure_basic() {
    let html = convert(".. figure:: /path/to/figure.png\n\n   This is the caption.");
    assert!(html.contains("<figure"));
    assert!(html.contains("<img"));
    assert!(html.contains("<figcaption>"));
    assert!(html.contains("This is the caption"));
}

// ========== Body Element Tests ==========

#[test]
fn test_topic_directive() {
    let html = convert(".. topic:: Topic Title\n\n   This is the topic content.");
    assert!(html.contains("topic"));
    assert!(html.contains("topic-title"));
    assert!(html.contains("Topic Title"));
}

#[test]
fn test_sidebar_directive() {
    let html = convert(".. sidebar:: Sidebar Title\n\n   This is sidebar content.");
    assert!(html.contains("<aside"));
    assert!(html.contains("sidebar"));
    assert!(html.contains("sidebar-title"));
    assert!(html.contains("Sidebar Title"));
}

#[test]
fn test_rubric_directive() {
    let html = convert(".. rubric:: Not in TOC\n");
    assert!(html.contains("rubric"));
    assert!(html.contains("Not in TOC"));
}

#[test]
fn test_epigraph_directive() {
    let html = convert(".. epigraph::\n\n   To be or not to be.");
    assert!(html.contains("<blockquote"));
    assert!(html.contains("epigraph"));
}

#[test]
fn test_highlights_directive() {
    let html = convert(".. highlights::\n\n   Key points here.");
    assert!(html.contains("<blockquote"));
    assert!(html.contains("highlights"));
}

#[test]
fn test_pull_quote_directive() {
    let html = convert(".. pull-quote::\n\n   An important quote.");
    assert!(html.contains("<blockquote"));
    assert!(html.contains("pull-quote"));
}

#[test]
fn test_compound_directive() {
    let html = convert(".. compound::\n\n   Compound paragraph content.");
    assert!(html.contains("compound"));
}

#[test]
fn test_container_directive() {
    let html = convert(".. container:: custom-class\n\n   Container content.");
    assert!(html.contains("custom-class"));
}

#[test]
fn test_contents_directive() {
    let html = convert(".. contents:: Table of Contents\n");
    assert!(html.contains("contents"));
    assert!(html.contains("Table of Contents"));
}

// ========== Table Tests ==========

#[test]
fn test_simple_table() {
    let html = convert(
        "==============  ==========================================================\nTravis          http://travis-ci.org/tony/pullv\nDocs            http://pullv.rtfd.org\n==============  ==========================================================",
    );
    assert!(html.contains("<table"));
    assert!(html.contains("<tr>"));
    assert!(html.contains("<td>"));
    assert!(html.contains("Travis"));
}

#[test]
fn test_simple_table_with_header() {
    let html = convert(
        "=====  =====  =======\nA      B      A and B\n=====  =====  =======\nFalse  False  False\nTrue   False  False\n=====  =====  =======",
    );
    assert!(html.contains("<table"));
    assert!(html.contains("<th>") || html.contains("<thead>"));
    assert!(html.contains("False"));
    assert!(html.contains("True"));
}

// ========== Special Directive Tests ==========

#[test]
fn test_raw_html_directive() {
    let html = convert(".. raw:: html\n\n   <div class=\"custom\">Custom HTML</div>");
    assert!(html.contains("<div class=\"custom\">Custom HTML</div>"));
}

#[test]
fn test_raw_latex_directive() {
    let html = convert(".. raw:: latex\n\n   \\textbf{LaTeX content}");
    assert!(html.contains("<!-- LaTeX:") || html.contains("<!-- Latex:"));
}

#[test]
fn test_include_directive() {
    let html = convert(".. include:: /path/to/file.rst\n");
    assert!(html.contains("Include:"));
    assert!(html.contains("/path/to/file.rst"));
}

#[test]
fn test_class_directive() {
    let html = convert(".. class:: special-class\n\n   Content with class");
    assert!(html.contains("special-class"));
}

// ========== Sphinx Tests ==========

#[test]
fn test_toctree_directive() {
    let html = convert(
        ".. toctree::\n   :maxdepth: 2\n   :caption: Contents\n\n   intro\n   tutorial\n   api",
    );
    assert!(html.contains("toctree-wrapper"));
    assert!(html.contains("<nav"));
    assert!(html.contains("<li>"));
}

#[test]
fn test_version_added_directive() {
    let html = convert(".. versionadded:: 2.0\n\n   This feature was added.");
    assert!(html.contains("versionadded"));
    assert!(html.contains("2.0"));
    assert!(html.contains("New in version"));
}

#[test]
fn test_version_changed_directive() {
    let html = convert(".. versionchanged:: 3.0\n\n   This behavior changed.");
    assert!(html.contains("versionchanged"));
    assert!(html.contains("3.0"));
    assert!(html.contains("Changed in version"));
}

#[test]
fn test_deprecated_directive() {
    let html = convert(".. deprecated:: 4.0\n\n   Use new_function instead.");
    assert!(html.contains("deprecated"));
    assert!(html.contains("4.0"));
    assert!(html.contains("Deprecated since version"));
}

#[test]
fn test_glossary_directive() {
    let html = convert(".. glossary::\n\n   term\n      Definition of term.");
    assert!(html.contains("glossary"));
    assert!(html.contains("<dl"));
}

#[test]
fn test_glossary_with_term_ids() {
    let html = convert(
        ".. glossary::\n\n   RST\n      A plaintext markup language.\n\n   Sphinx\n      A documentation generator.",
    );
    assert!(html.contains("<dl class=\"glossary\">"));
    assert!(html.contains("id=\"term-rst\""));
    assert!(html.contains("id=\"term-sphinx\""));
}

#[test]
fn test_glossary_sorted() {
    let html = convert(
        ".. glossary::\n   :sorted:\n\n   Zebra\n      The last animal.\n\n   Apple\n      A fruit.",
    );
    let apple_pos = html.find("Apple").unwrap();
    let zebra_pos = html.find("Zebra").unwrap();
    assert!(apple_pos < zebra_pos);
}

// ========== Code Documentation Tests ==========

#[test]
fn test_doctest_directive() {
    let html = convert(".. doctest::\n\n   >>> print(\"Hello\")\n   Hello");
    assert!(html.contains("doctest"));
    assert!(html.contains("<pre"));
    assert!(html.contains("language-python"));
}

#[test]
fn test_standalone_doctest_block() {
    let html = convert(
        "Some text before.\n\n>>> some_function()\n'result'\n\nSome text after.",
    );
    assert!(html.contains("doctest"));
    assert!(html.contains("<pre"));
    assert!(html.contains("some_function()"));
    assert!(html.contains("Some text before"));
    assert!(html.contains("Some text after"));
}

#[test]
fn test_testcode_directive() {
    let html = convert(".. testcode::\n\n   print(\"test\")");
    assert!(html.contains("testcode"));
    assert!(html.contains("<pre"));
}

#[test]
fn test_testoutput_directive() {
    let html = convert(".. testoutput::\n\n   test");
    assert!(html.contains("testoutput"));
    assert!(html.contains("<samp"));
}

// ========== Math Tests ==========

#[test]
fn test_math_directive() {
    let html = convert(".. math::\n\n   E = mc^2");
    assert!(html.contains("math-block"));
    assert!(html.contains("E = mc^2"));
}

// ========== Unknown Directive ==========

#[test]
fn test_unknown_directive() {
    let html = convert(
        ".. custom-directive:: argument\n   :option1: value1\n   :option2: value2\n\n   Directive content here.",
    );
    assert!(html.contains("directive-custom-directive"));
    assert!(html.contains("directive-arguments"));
    assert!(html.contains("directive-content"));
}

// ========== Transition Tests ==========

#[test]
fn test_transition_with_dashes() {
    let html = convert("Some text before.\n\n----\n\nSome text after.");
    assert!(html.contains("<hr"));
    assert!(html.contains("Some text before"));
    assert!(html.contains("Some text after"));
}

#[test]
fn test_transition_with_equals() {
    let html = convert("Before.\n\n====\n\nAfter.");
    assert!(html.contains("<hr"));
}

#[test]
fn test_transition_with_asterisks() {
    let html = convert("Before.\n\n****\n\nAfter.");
    assert!(html.contains("<hr"));
}

#[test]
fn test_transition_with_tildes() {
    let html = convert("Before.\n\n~~~~\n\nAfter.");
    assert!(html.contains("<hr"));
}

#[test]
fn test_transition_with_underscores() {
    let html = convert("Before.\n\n________\n\nAfter.");
    assert!(html.contains("<hr"));
}

#[test]
fn test_three_chars_is_not_transition() {
    let html = convert("Before.\n\n---\n\nAfter.");
    assert!(!html.contains("<hr"));
}

// ========== HTML Escaping ==========

#[test]
fn test_escaped_html() {
    let html = convert("Use <script> and & characters safely.");
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&amp;"));
}

// ========== External Links ==========

#[test]
fn test_external_link() {
    let html = convert("Visit `Example <https://example.com>`_ for more.");
    assert!(html.contains("<a"));
    assert!(html.contains("https://example.com"));
}

// ========== Field List Tests ==========

#[test]
fn test_field_list() {
    let html = convert(":Author: John Doe\n:Version: 1.0");
    assert!(html.contains("<dl"));
    assert!(html.contains("<dt>"));
    assert!(html.contains("<dd>"));
}

// ========== Code Block Caption Tests ==========

#[test]
fn test_code_block_with_caption() {
    let html = convert(
        ".. code-block:: javascript\n   :caption: my-script.js\n\n   console.log(\"Hello\");",
    );
    assert!(
        html.contains("code-block-caption"),
        "HTML: {}",
        html
    );
    assert!(html.contains("my-script.js"));
    assert!(html.contains("console.log"));
}

// ========== CSV Table Tests ==========

#[test]
fn test_csv_table_basic() {
    let html = convert(
        ".. csv-table:: CSV Data\n\n   Item1, 100, First item\n   Item2, 200, Second item",
    );
    assert!(html.contains("<table"));
    assert!(html.contains("csv-table"));
    assert!(html.contains("CSV Data"));
    assert!(html.contains("<td>"));
}

#[test]
fn test_csv_table_with_options() {
    let html = convert(
        ".. csv-table:: Sample CSV Table\n   :header: \"Name\", \"Age\", \"City\"\n   :widths: 20, 10, 20\n   :align: center\n\n   \"Alice Smith\", 28, \"New York\"\n   \"Bob Johnson\", 35, \"San Francisco\"",
    );
    assert!(html.contains("<thead>"));
    assert!(html.contains("Name"));
    assert!(html.contains("<colgroup>"));
    assert!(html.contains("margin-left: auto"));
    assert!(html.contains("Alice Smith"));
    assert!(!html.contains("\"Alice Smith\""));
}

// ========== List Table Tests ==========

#[test]
fn test_list_table_directive() {
    let html = convert(
        ".. list-table:: List Table\n   :header-rows: 1\n\n   * - Header 1\n     - Header 2\n   * - Cell 1\n     - Cell 2",
    );
    assert!(html.contains("<table"));
    assert!(html.contains("list-table"));
    assert!(html.contains("List Table"));
}

// ========== Performance Tests ==========

#[test]
fn test_large_file_with_many_inline_references() {
    let mut lines = String::new();
    lines.push_str("Changelog\n");
    lines.push_str("=========\n\n");

    for i in 1..=1000 {
        lines.push_str(&format!(
            "* `@user{i} <https://github.com/user{i}>`__: Description of change (`#{i} <https://github.com/org/repo/pull/{i}>`__)\n"
        ));
    }

    let start = std::time::Instant::now();
    let html = convert(&lines);
    let elapsed = start.elapsed();

    assert!(!html.is_empty());
    assert!(html.contains("<h1"));
    assert!(html.contains("<li"));

    // Should complete in reasonable time (under 10 seconds)
    assert!(
        elapsed.as_secs() < 10,
        "Conversion took {:?}, expected < 10s",
        elapsed
    );

    println!("Large changelog conversion took {:?}", elapsed);
}

// ========== Section Numbering Tests ==========

#[test]
fn test_section_numbering() {
    let html = convert(
        ".. sectnum::\n\nChapter 1\n=========\n\nSection 1.1\n-----------\n\nSection 1.2\n-----------\n\nChapter 2\n=========",
    );
    assert!(
        html.contains("1") && html.contains("Chapter 1"),
        "HTML: {}",
        html
    );
}

// ========== Contents Directive with Sections ==========

#[test]
fn test_contents_directive_with_sections() {
    let html = convert(
        "Title\n=====\n\n.. contents:: Table of Contents\n\nSection 1\n---------\n\nSome content.\n\nSection 2\n---------\n\nMore content.",
    );
    assert!(html.contains("contents"));
    assert!(html.contains("<ul class=\"toc\">"));
    assert!(html.contains("href=\"#section-1\""));
    assert!(html.contains("href=\"#section-2\""));
}

// ========== Escaped Angle Brackets in List ==========

#[test]
fn test_escaped_angle_brackets_in_list() {
    let html = convert("- Angle brackets: \\< \\>");
    assert!(
        html.contains("&lt; &gt;"),
        "Expected &lt; &gt; in: {}",
        html
    );
    assert!(!html.contains("&amp;lt;"));
}

// ========== Parsed Literal ==========

#[test]
fn test_parsed_literal_directive() {
    let html = convert(".. parsed-literal::\n\n   This is *parsed* literal text");
    assert!(html.contains("parsed-literal"));
    assert!(html.contains("<pre"));
}

// ========== Table Directive ==========

#[test]
fn test_table_directive() {
    let html = convert(".. table:: Table Caption\n   :width: 100%\n\n   Content here");
    assert!(html.contains("<table"));
    assert!(html.contains("<caption>"));
    assert!(html.contains("Table Caption"));
}
