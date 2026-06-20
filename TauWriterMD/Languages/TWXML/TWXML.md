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
