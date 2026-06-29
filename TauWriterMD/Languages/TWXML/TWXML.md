# TauWriter XML (TWXML)

An XML-structured document language. Designed to be simple, but powerful. TWXML serves as the raw format for all prose in TauWriter, providing a controlled vocabulary purpose-built for narrative writing workflows with embedded knowledge-graph references.

## The Root Skeleton
Every valid TWXML document **must** conform to the following streamlined root skeleton[cite: 1]:

```xml
<document>
  <meta name="author" content="J.R.R. Tolkien" />
  <meta name="status" content="draft" />
  <body>
    <!-- Structural content: sections, headings, paragraphs, etc. -->
  </body>
</document>
```

### `<document>` - Root Tag

- The mandatory root element that wraps the entire document.
- Every `.twxml` file must have exactly one `<document>` root.
- Children are ordered sequentially: Zero or more `<meta />` declarations followed by exactly one `<body>` block.

### `<meta>` — Document Metadata
A single key-value metadata entry. Must be placed as an immediate child of `<document>` and precede the `<body>` element.
  Tags: `<meta />`   
  Tag Attributes:   
  - name: The metadata key.   
  - content: The metadata value.

### `<body>` — Document Body Block

- A single block-level element placed immediately under `<document>`, after any `<meta />` declarations.
- All prose, structural elements, and graph references must be nested within the `<body>` or its descendants.
- Nothing other than valid TWXML structural tags is permitted inside `<body>`.

## Presentation Attributes Enforced by tauwriter-fmt

- `align`: Valid values are strictly bounded to `left` | `center` | `right` | `justify`. Supported on structural block text, tables, and media elements.
- `width`: Restricted strictly to percentage values (e.g., `width="75%"`), ensuring liquid layout compatibility across responsive views.

## Nesting Rules

TWXML enforces a strict nesting hierarchy. The following rules apply:

### Headings (`<heading>`)

- A `<heading>` **must** be nested directly within a `<section>` or the `<body>`.
- Heading levels are derived from nesting depth, not explicit attributes:
  - Depth 1 (inside `<body>`): H1 — document title
  - Depth 2 (inside `<section>`): H2 — section heading
  - Depth 3+ (deeper nesting): H3+ — subsection headings

```xml
<body>
  <heading>The Fellowship of the Ring</heading>
  <section alias="prologue">
    <heading>Prologue</heading>
    <paragraph>Long ago...</paragraph>
  </section>
</body>
```

### Sections (`<section>`)

- A `<section>` **must** be nested within the `<body>` or another `<section>`.
- Sections carry an `alias` attribute that serves as a stable identifier for cross-referencing.
- Nesting sections creates a hierarchical structure: parts → chapters → scenes, etc.

```xml
<body>
  <section alias="part-one">
    <heading>Part One</heading>
    <section alias="chapter-one">
      <heading>The Journey Begins</heading>
      <paragraph>...</paragraph>
    </section>
  </section>
</body>
```

### Paragraphs (`<paragraph>`)

- A `<paragraph>` is the fundamental prose unit. It may contain inline formatting tags and graph references.
- Inline elements include: `<bold>`, `<italic>`, `<underline>`.
- Graph links are embedded via `<hubref id="<hub-ref>">`.

```xml
<paragraph>
  <bold><hubref id="aragorn">Aragorn</hubref></bold> drew his sword
  and looked toward <hubref id="mordor">Mordor</hubref>.
</paragraph>
```

## Complete Document Example

```xml
<document>
    <meta name="author" content="J.R.R. Tolkien" />
    <meta name="tags" content="fantasy, lotr" />
    <meta name="status" content="draft" />
  <body>
    <heading>The Lord of the Rings</heading>
    <section alias="The Fellowship">
      <heading>Departure</heading>
      <paragraph>
        <bold><hubref id="aragorn">Aragorn</hubref></bold> drew his sword
        and looked across the field toward <hubref id="mordor">Mordor</hubref>.
      </paragraph>
    </section>
  </body>
</document>
```

## Key Design Principles

- **Heading levels by depth**: No explicit `<h1>`–`<h6>` needed. Structure emerges from nesting.
- **Metadata before content**: `<meta />` tags live inside `<document>`, always preceding `<body>`.
- **Prose in structural containers**: All text lives inside `<paragraph>`, `<section>`, or other block elements.
- **Graph links via `<hubref>`**: References to knowledge-graph entities use the `id` attribute, not inline data attributes.
- **Formatting wraps content**: Styling tags (`<bold>`, `<italic>`) wrap inner content including references — never the reverse.
- **Multiple hubs per paragraph**: Any number of `<hubref>` elements can appear within a single paragraph or phrase.

## Formatting Rules

The `tauwriter-fmt` LSP module normalizes TWXML on-save or on-demand. Below is the complete, deterministic rule set.

### Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `MAX_LINE_LEN` | 100 | Hard-wrap budget for prose and inline-expand triggers. Excludes indentation prefix. |

### Rules

| # | Rule | Applies To |
|---|------|-----------|
| R1 | Container blocks always expand multiline when they have children (even just whitespace that normalizes to a non-empty text node) | `document`, `body`, `section`, `paragraph`, `aside`, `blockquote`, `codeblock`, `ul`, `ol`, `dl`, `details`, `table`, `footnote`, `review` |
| R2 | Leaf blocks expand multiline if tag + content > `MAX_LINE_LEN`. This measure does **not** include indentation prefix. | `heading`, `li`, `dt`, `dd`, `summary`, inline elements with content (`hubref`, `link`, `code`) |
| R3 | Nested content indented +2 spaces per level | All elements |
| R4 | Self-closing tags (including explicitly defined `<self_closing_element>` nodes like `<meta />`) get their own indented line in block context | `hr`, `image`, `audio`, `video`, `meta` |
| R5 | `<br/>` splits inline content into separate text chunks, each chunk on its own indented line inside the parent | Inside any container |
| R6 | Empty containers stay compact: `<tag></tag>` or `<tag attr="val"></tag>` | All elements |
| R7 | Attributes expand one-per-line if opening tag + attributes > `MAX_LINE_LEN`. Closing `>` on its own line. Attributes indented +2 from tag name column. | All elements |
| R8 | Prose text is hard-wrapped at `MAX_LINE_LEN` for source readability. Internal whitespace (e.g., intentional double spaces) and word boundaries around inline formatting (e.g., `<bold>L</bold>etter`) are strictly preserved. Raw newline characters (`\n`) are treated as non-semantic soft wraps and ignored by the renderer; semantic line breaks require the `<br/>` tag. Indentation prefix is not counted in the line budget. | Block-level text content |
| R9 | Text styling elements expand multiline when wrapping content > `MAX_LINE_LEN` (same rule as leaf blocks) | `bold`, `italic`, `underline`, `strikethrough`, `super`, `sub` |
| R10 | `<td>`/`<th>` follow 100-char rule, not forced-expand. `<table>` and `<tr>` always expand. | Table elements |
| R11 | Comments always on own indented line | All contexts |
| R12 | `<codeblock>` content preserved verbatim. Only the first line is padded to match nesting indentation; internal whitespace untouched, no hard-wrap applied. | Code blocks |
| R13 | Parse errors: return original text unchanged. Do not risk corrupting malformed input. | Fallback |
| R14 | `<fr/>` stays inline with adjacent text, even in expanded blocks. Not given its own line. | Expanded blocks |

### Rule Exceptions Matrix

| Element | Forced-expand? | 100-char expand? | Hard-wrap prose? | Preserve whitespace? |
|---------|---------------|-----------------|------------------|----------------------|
| `document`, `body`, `section` | Yes | — | No (structural) | No |
| `paragraph`, `aside`, `blockquote` | Yes | — | **Yes** | **Yes (Internal)** |
| `codeblock` | Yes | — | No | **Yes (Absolute)** |
| `ul`, `ol`, `dl`, `details`, `table`, `footnote`, `review` | Yes | — | No (structural) | No |
| `tr` | Yes | — | No | No |
| `heading`, `li`, `dt`, `dd`, `summary` | No | **Yes** | Yes | **Yes (Internal)** |
| `td`, `th` | No | **Yes** | Yes | **Yes (Internal)** |
| `hubref`, `link`, `code` | No | **Yes** | Yes | **Yes (Internal)** |
| `bold`, `italic`, `underline`, `strikethrough`, `super`, `sub` | No | **Yes** | Yes | **Yes (Internal)** |
| `hr`, `image`, `audio`, `video`, `meta` | Self-closing, own line | — | — | — |
| `br` | Splits chunks | — | — | — |
| `fr` | Inline, never expands | — | — | — |
| Comment | Own line | — | — | — |

### R8 Implementation Note: Whitespace and Soft Wraps

Unlike traditional HTML or Markdown rendering that eagerly collapses all spaces, TWXML's formatting preserves the author's internal spacing intent. 

* **Inline Word Boundaries:** Formatting tags do not implicitly break words. A string like `<bold>L</bold>etter` remains a single contiguous word during rendering. 
* **Soft vs. Hard Wraps:** The formatter injects newline characters (`\n`) strictly to keep the raw TWXML source code within the `MAX_LINE_LEN` budget. The rendering engine ignores these raw newlines, treating them as soft wraps. To enforce a structural line break in the final rendered output, authors must use the explicit `<br/>` tag.

#### Whitespace-only children after normalization (R8 + R14)

After collapsing whitespace, text nodes that become empty are **dropped**. Example:

```xml
<!-- Before -->
<paragraph>   <bold>Hello</bold>   </paragraph>
<!-- After -->
<paragraph>
  <bold>Hello</bold>
</paragraph>
```

Leading/trailing space-only text inside `<paragraph>` normalizes to nothing and is removed. Without this, spurious blank lines appear in expanded blocks.
