# TauWriter XML (TWXML)

An XML-structured document language. Designed to be simple, but powerful. TWXML serves as the raw format for all prose in TauWriter, providing a controlled vocabulary purpose-built for narrative writing workflows with embedded knowledge-graph references.

## The Root Skeleton

Every valid TWXML document **must** conform to the following root skeleton:

```xml
<document>
  <metadata>
    <!-- Zero or more <meta /> declarations -->
  </metadata>
  <body>
    <!-- Structural content: sections, headings, paragraphs, etc. -->
  </body>
</document>
```

### `<document>` — Root Tag

- The **mandatory** root element that wraps the entire document.
- Every `.twxml` file must have exactly one `<document>` root. No siblings, no wrapper elements are permitted outside it.
- Only two valid immediate children: `<metadata>` and `<body>`.

### `<metadata>` — Document Metadata Block

- A single block-level element placed immediately under `<document>`, before `<body>`.
- Contains zero or more self-closing `<meta />` child declarations that define document-level properties.
- The `<metadata>` block itself is not self-closing; it wraps its children:

```xml
<metadata>
  <meta name="author" content="J.R.R. Tolkien" />
  <meta name="tags" content="fantasy, lotr, chapter-1" />
  <meta name="status" content="draft" />
</metadata>
```

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | Yes | The metadata key (e.g., `"author"`, `"tags"`, `"status"`) |
| `content` | Yes | The metadata value as a string |

### `<body>` — Document Body Block

- A single block-level element placed immediately under `<document>`, after `<metadata>`.
- All prose, structural elements, and graph references must be nested within the `<body>` or its descendants.
- Nothing other than valid TWXML structural tags is permitted inside `<body>`.

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
  <metadata>
    <meta name="author" content="J.R.R. Tolkien" />
    <meta name="tags" content="fantasy, lotr" />
    <meta name="status" content="draft" />
  </metadata>
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
- **Metadata before content**: `<meta />` tags live inside `<metadata>`, which always precedes `<body>`.
- **Prose in structural containers**: All text lives inside `<paragraph>`, `<section>`, or other block elements.
- **Graph links via `<hubref>`**: References to knowledge-graph entities use the `id` attribute, not inline data attributes.
- **Formatting wraps content**: Styling tags (`<bold>`, `<italic>`) wrap inner content including references — never the reverse.
- **Multiple hubs per paragraph**: Any number of `<hubref>` elements can appear within a single paragraph or phrase.

## Formatting Rules

The `tauwriter-fmt` LSP module normalizes TWXML on-save or on-demand. Below is the complete, deterministic rule set.

### Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `MAX_LINE_LEN` | 80 | Hard-wrap budget for prose and inline-expand triggers. Excludes indentation prefix. |

### Rules

| # | Rule | Applies To |
|---|------|-----------|
| R1 | Container blocks always expand multiline when they have children (even just whitespace that normalizes to a non-empty text node) | `document`, `metadata`, `body`, `section`, `paragraph`, `aside`, `blockquote`, `codeblock`, `ul`, `ol`, `dl`, `details`, `table`, `footnote`, `review` |
| R2 | Leaf blocks expand multiline if tag + content > `MAX_LINE_LEN`. This measure does **not** include indentation prefix. | `heading`, `li`, `dt`, `dd`, `summary`, inline elements with content (`hubref`, `link`, `code`) |
| R3 | Nested content indented +2 spaces per level | All elements |
| R4 | Self-closing block tags get their own indented line in block context | `hr`, `image`, `audio`, `video` |
| R5 | `<br/>` splits inline content into separate text chunks, each chunk on its own indented line inside the parent | Inside any container |
| R6 | Empty containers stay compact: `<tag></tag>` or `<tag attr="val"></tag>` | All elements |
| R7 | Attributes expand one-per-line if opening tag + attributes > `MAX_LINE_LEN`. Closing `>` on its own line. Attributes indented +2 from tag name column. | All elements |
| R8 | Prose text hard-wrapped at `MAX_LINE_LEN`. Whitespace normalized: leading/trailing stripped, internal runs collapsed to single space. Zero-length text nodes dropped after normalization. Indentation prefix not counted in budget. | Block-level text content |
| R9 | Text styling elements expand multiline when wrapping content > `MAX_LINE_LEN` (same rule as leaf blocks) | `bold`, `italic`, `underline`, `strikethrough`, `super`, `sub` |
| R10 | `<td>`/`<th>` follow 80-char rule, not forced-expand. `<table>` and `<tr>` always expand. | Table elements |
| R11 | Comments always on own indented line | All contexts |
| R12 | `<codeblock>` content preserved verbatim. Only the first line is padded to match nesting indentation; internal whitespace untouched, no hard-wrap applied. | Code blocks |
| R13 | Parse errors: return original text unchanged. Do not risk corrupting malformed input. | Fallback |
| R14 | `<fr/>` stays inline with adjacent text, even in expanded blocks. Not given its own line. | Expanded blocks |

### Rule Exceptions Matrix

| Element | Forced-expand? | 80-char expand? | Hard-wrap prose? | Preserve whitespace? |
|---------|---------------|-----------------|------------------|----------------------|
| `document`, `metadata`, `body`, `section` | Yes | — | No (structural) | No |
| `paragraph`, `aside`, `blockquote` | Yes | — | **Yes** | No |
| `codeblock` | Yes | — | No | **Yes** |
| `ul`, `ol`, `dl`, `details`, `table`, `footnote`, `review` | Yes | — | No (structural) | No |
| `tr` | Yes | — | No | No |
| `heading`, `li`, `dt`, `dd`, `summary` | No | **Yes** | Yes | No |
| `td`, `th` | No | **Yes** | Yes | No |
| `hubref`, `link`, `code` | No | **Yes** | Yes | No |
| `bold`, `italic`, `underline`, `strikethrough`, `super`, `sub` | No | **Yes** | Yes | No |
| `hr`, `image`, `audio`, `video` | Self-closing, own line | — | — | — |
| `br` | Splits chunks | — | — | — |
| `fr` | Inline, never expands | — | — | — |
| Comment | Own line | — | — | — |

### R8 Risk Note: Whitespace Normalization is Not Round-Trip Preserving

R8 collapses all internal whitespace into single spaces and strips leading/trailing whitespace inside text nodes. This means multi-word gaps, intentional spacing for alignment, and edge whitespace are all lost. Example:

```xml
<!-- Before -->
<paragraph>  Some text with   extra    spaces  </paragraph>
<!-- After -->
<paragraph>Some text with extra spaces</paragraph>
```

Semantically identical for rendering, but the transform is **not** a lossless identity. This is acceptable because the goal is readability, not parser-preserving fidelity.

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
