==================================
rst2html-fast benchmark document
==================================

Introduction
============

This is a realistic reStructuredText document used for benchmarking RST-to-HTML
converters. It exercises a wide range of RST features including sections,
inline markup, lists, tables, code blocks, directives, roles, and more.

This paragraph contains **bold text**, *italic text*, ``inline code``,
and a `hyperlink <https://example.com>`_. It also has :code:`role-based code`,
:emphasis:`emphasized text`, and :strong:`strong text`.

Getting started
===============

Installation
------------

You can install the package using pip:

.. code-block:: bash

   pip install rst2html-fast
   rst2html --version

Or build from source:

.. code-block:: bash

   git clone https://github.com/bodiam/rst2html-fast.git
   cd rst2html-fast
   cargo build --release

Configuration
-------------

The following options are available:

:input: Path to the input RST file
:output: Path to the output HTML file
:theme: The theme to use for rendering (default: ``basic``)
:verbose: Enable verbose output

.. note::

   Configuration can also be provided via environment variables
   prefixed with ``RST2HTML_``.

.. warning::

   The ``--force`` flag will overwrite existing output files
   without confirmation.

API reference
=============

Core module
-----------

The ``convert`` function
^^^^^^^^^^^^^^^^^^^^^^^^

The main entry point for converting RST to HTML:

.. code-block:: rust

   use rst2html::convert;

   let html = convert("**Hello** *world*");
   assert_eq!(html, "<p><strong>Hello</strong> <em>world</em></p>\n");

The ``convert_with_options`` function
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For more control over the conversion:

.. code-block:: rust

   use rst2html::{convert_with_options, ConvertOptions};

   let options = ConvertOptions {
       standalone: true,
       ..Default::default()
   };
   let html = convert_with_options("Hello", &options);

Parser module
-------------

.. code-block:: python

   def parse_document(source):
       """Parse an RST document into a document tree.

       Args:
           source: The RST source text.

       Returns:
           A document tree that can be transformed to HTML.
       """
       parser = RSTParser()
       document = parser.parse(source)
       return document

Data types
==========

Supported formats
-----------------

The following table shows all supported input formats:

=============  ===========  ========  ===========
Format         Extension    Binary    Text
=============  ===========  ========  ===========
RST            ``.rst``     No        Yes
Markdown       ``.md``      No        Yes
HTML           ``.html``    No        Yes
PDF            ``.pdf``     Yes       No
Word           ``.docx``    Yes       No
=============  ===========  ========  ===========

Performance results
-------------------

+---------------+------------+----------+----------+
| Converter     | Time (ms)  | Memory   | Accuracy |
+===============+============+==========+==========+
| rst2html-fast | 0.1        | 2 MB     | 99%      |
+---------------+------------+----------+----------+
| docutils      | 20         | 45 MB    | 100%     |
+---------------+------------+----------+----------+
| Sphinx        | 40         | 120 MB   | 100%     |
+---------------+------------+----------+----------+
| pandoc        | 15         | 60 MB    | 95%      |
+---------------+------------+----------+----------+

Feature comparison
------------------

- **rst2html-fast**

  - Fastest converter available
  - Written in Rust
  - Zero-copy parsing where possible
  - Minimal memory allocation

- **docutils**

  - Reference implementation
  - Full RST spec compliance
  - Written in Python
  - Extensible with custom directives

- **Sphinx**

  - Full documentation system
  - Cross-referencing support
  - Multiple output formats
  - Extension ecosystem

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

      i. Unit tests
      ii. Integration tests
      iii. End-to-end tests

4. Step four: deploy

Definition list:

term 1
   Definition of term 1. This can span
   multiple lines.

term 2
   Definition of term 2.

term 3 : classifier
   Definition with a classifier.

Code examples
-------------

Python example:

.. code-block:: python

   import asyncio

   async def fetch_data(url: str) -> dict:
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

JavaScript example:

.. code-block:: javascript

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

Rust example:

.. code-block:: rust

   use std::collections::HashMap;

   fn word_count(text: &str) -> HashMap<&str, usize> {
       let mut counts = HashMap::new();
       for word in text.split_whitespace() {
           *counts.entry(word).or_insert(0) += 1;
       }
       counts
   }

Directives
----------

.. tip::

   Use ``rst2html-fast`` for the best performance when you don't need
   the full Sphinx feature set.

.. important::

   Always validate your RST syntax before converting. Invalid markup
   may produce unexpected HTML output.

.. caution::

   Grid tables with complex spanning cells may render differently
   than in docutils.

.. admonition:: Custom admonition

   This is a custom admonition with a user-defined title.
   It can contain any RST content including **bold**, *italic*,
   and ``code``.

.. topic:: Related resources

   For more information about reStructuredText, see:

   - `Docutils documentation <https://docutils.sourceforge.io/rst.html>`_
   - `Sphinx documentation <https://www.sphinx-doc.org/>`_

.. sidebar:: Quick reference

   The most common RST constructs:

   - ``**bold**`` for **bold**
   - ``*italic*`` for *italic*
   - ````code```` for ``code``

Block quotes and attributions:

   "The best way to predict the future is to invent it."

   -- Alan Kay

Line blocks:

| Roses are red,
| Violets are blue,
| rst2html-fast is fast,
| And so are you.

Transitions
-----------

Content before the transition.

----------

Content after the transition.

Advanced features
=================

Footnotes
---------

This text has a footnote [1]_ and another one [2]_.

.. [1] This is the first footnote.
.. [2] This is the second footnote with a longer
   explanation that spans multiple lines.

Citations
---------

According to the RST specification [RST]_, reStructuredText is a
plaintext markup syntax and parser component of Docutils.

.. [RST] reStructuredText Markup Specification,
   https://docutils.sourceforge.io/docs/ref/rst/restructuredtext.html

Substitutions
-------------

The |project| version |version| was released on |today|.

.. |project| replace:: rst2html-fast
.. |version| replace:: 0.1.0
.. |today| replace:: 2025-01-01

Comments
--------

.. This is a comment that should not appear in the output.

.. This is another comment.
   It spans multiple lines
   and should also be hidden.

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

* `@bodiam <https://github.com/bodiam>`__: Initial prototype (`#1 <https://github.com/bodiam/rst2html-fast/pull/1>`__)
* `@contributor <https://github.com/contributor>`__: Added table support (`#2 <https://github.com/bodiam/rst2html-fast/pull/2>`__)
* `@helper <https://github.com/helper>`__: Fixed inline markup parsing (`#3 <https://github.com/bodiam/rst2html-fast/pull/3>`__)
* `@reviewer <https://github.com/reviewer>`__: Improved error handling (`#4 <https://github.com/bodiam/rst2html-fast/pull/4>`__)
* `@tester <https://github.com/tester>`__: Added integration tests (`#5 <https://github.com/bodiam/rst2html-fast/pull/5>`__)

License
=======

This project is licensed under the MIT License. See the ``LICENSE`` file
for details.
