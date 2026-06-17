# TWXML Elements

## Document Hierarchy
### Document
The root Element for any `*.twxml` file. Can wrap a complete document or a fragment of a larger stitched document.
Tags: `<document></document>`

### Meta
Defines document-level metadata (author, tags, status). Not rendered in the primary prose view but indexed by the LSP. 
Equivalent to Markdown Frontmatter (YAML).
Tags: `<meta />`
Tag Attributes:
- name: The metadata key.
- content: The metadata value.

### Section
A semantic divider element used to distinguish different sections and sub-sections within a document.
Useful for: Parts, Chapters, Scenes, etc.
Tags: `<section></section>`
Tag Attributes:
- alias: (ex: `<section alias="Scene 1">[CONTENT]</section>`)

### Heading
A tag used to distinguish the heading of a section. Heading level is automatically determined by its nested level in the document tree. 
Tags: `<heading></heading>`

---

## Block Elements
### Paragraph
The fundamental prose unit. Wraps standard blocks of narrative text.
Tags: `<paragraph></paragraph>`

### Aside
Used for sidebars, callout boxes, or flavor text. Distinguishes non-linear content from the main prose.
Maps to HTML `<aside>` or Markdown Admonitions/Blockquotes.
Tags: `<aside></aside>`

### Blockquote
Used to indicate that the enclosed text is an extended quotation from another source.
Tags: `<blockquote></blockquote>`

### Code Block
Represents preformatted text or source code blocks. Replaces Markdown's fenced code blocks (```).
Tags: `<codeblock></codeblock>`
Tag Attributes:
- language: Specifies the programming or scripting language for syntax highlighting (ex: `<codeblock language="javascript">`)

### Horizontal Rule
Represents a thematic break between paragraph-level elements. Equivalent to Markdown's `---`.
Tags: `<hr />`

### Line Break
Forces a hard line break within a block element without creating a new paragraph.
Tags: `<br />`

---

## Lists
Containers for bulleted or numbered lists.
Tags: `<ul></ul>` (Unordered), `<ol></ol>` (Ordered)

### List Item
Represents an individual item within a `<ul>` or `<ol>`.
Tags: `<li></li>`
Tag Attributes:
- checked: Adding this boolean attribute converts the list item into a task list item (ex: `<li checked="true">Buy potions</li>` or `<li checked="false">Slay dragon</li>`)

### Definition Lists
Used to define terms. Equivalent to Markdown's definition list syntax.
Tags: 
- `<dl></dl>`: The definition list container.
- `<dt></dt>`: The definition term.
- `<dd></dd>`: The definition description.

---

## Disclosure Elements
Used to manage information density by hiding content behind a clickable summary. Native 1:1 mapping to HTML.

### Details
A container element for content that can be toggled open or closed.
Tags: `<details></details>`
Tag Attributes:
- open: Boolean attribute indicating if the content is visible by default.

### Summary
Provides a visible label for a `<details>` element.
Tags: `<summary></summary>`

---

## Inline Elements
### HubRef
The tag that wraps text to link it to its HubGS graph reference.
Tags: `<hubref></hubref>`
Tag Attributes:
- id: The unique identifier for that hub. (ex: `<hubref id="aragorn">Aragorn</hubref>`)

### Link
Creates a standard hyperlink to an external URL or an internal heading/file anchor.
Tags: `<link></link>`
Tag Attributes:
- href: The URL or anchor destination (ex: `<link href="https://example.com">Website</link>`)

### Image
Embeds a visual asset into the document.
Tags: `<image />`
Tag Attributes:
- src: The file path or URL to the image.
- alt: Alternative text for accessibility and context.

### Audio
Embeds a auditory asset into the document.
Tags: `<audio />`
Tag Attributes:
- src: The file path or URL to the image.
- alt: Alternative text for accessibility and context.

### Vido
Embeds a visual asset into the document.
Tags: `<video />`
Tag Attributes:
- src: The file path or URL to the image.
- alt: Alternative text for accessibility and context.

### Inline Code
Displays its contents styled in a monospace font to indicate a short fragment of computer code.
Tags: `<code></code>`

### Footnote Reference
Places a superscript footnote marker in the text.
Tags: `<fr />`
Tag Attributes:
- id: Matches the ID of the defined footnote.

---

## Text Styling
Semantic tags for standard inline text formatting. These are applied via structural nesting around text or other inline elements.

- **Bold:** `<bold></bold>`
- **Italic:** `<italic></italic>`
- **Underline:** `<underline></underline>`
- **Strikethrough:** `<strikethrough></strikethrough>`
- **Superscript:** `<super></super>`
- **Subscript:** `<sub></sub>`

---

## Tables
Elements used to create and structure tabular data, directly mapping to Markdown's pipe-and-dash tables.

- `<table></table>`: The main container for the table.
- `<tr></tr>`: A table row.
- `<th></th>`: A table header cell (typically used in the first row).
- `<td></td>`: A standard table data cell.

---

## Footnote Definitions
### Footnote
Defines the content of a footnote referenced earlier in the text. Usually placed at the bottom of a `<section>` or `<document>`.
Tags: `<footnote></footnote>`
Tag Attributes:
- id: The unique identifier matching the `<fr>` tag.

---

## Usage Example

```xml
<document>
  <chapter id="ch1" title="A Shadow of the Past">
    <h2>Departure</h2>
    <p>
      <bold><hubref id="aragorn">Aragorn</hubref></bold> drew his sword 
      and looked across the field toward <hubref id="mordor">Mordor</hubref>.
    </p>
    
    <ul>
      <li checked="true">Pack lembas bread</li>
      <li checked="false">Sharpen sword</li>
    </ul>

    <blockquote>
      "Not all those who wander are lost."<br />
      <italic>- <hubref id="bilbo">Bilbo Baggins</hubref></italic>
    </blockquote>
  </chapter>
</document>
