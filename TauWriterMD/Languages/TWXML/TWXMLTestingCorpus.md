# TWXML Testing Corpus
Standard tree-sitter corpus tests for validating the TWXML grammar.

### `test/corpus/1_basic_structure.txt`

```Plaintext
====================================
Basic Document and Paragraphs
====================================
<document>
  <paragraph>Hello World</paragraph>
</document>
---
(document
  (element
    (start_tag (tag_name))
    (element
      (start_tag (tag_name))
      (text)
      (end_tag (tag_name)))
    (end_tag (tag_name))))
```

### `test/corpus/2_metadata_and_attributes.txt`

```Plaintext
====================================
Metadata and Attributes
====================================
<meta name="author" content="Aragorn" />
<section alias="Scene 1">
  <heading>The Departure</heading>
</section>
---
(document
  (self_closing_element
    (tag_name)
    (attribute (attribute_name) (attribute_value))
    (attribute (attribute_name) (attribute_value)))
  (element
    (start_tag
      (tag_name)
      (attribute (attribute_name) (attribute_value)))
    (element
      (start_tag (tag_name))
      (text)
      (end_tag (tag_name)))
    (end_tag (tag_name))))
```

### `test/corpus/3_inline_references.txt`

```Plaintext
====================================
Inline Hub References and Styling
====================================
<paragraph>
  Beside <bold><hubref id="gandalf">Gandalf</hubref></bold> stood the King.
</paragraph>
---
(document
  (element
    (start_tag (tag_name))
    (text)
    (element
      (start_tag (tag_name))
      (element
        (start_tag
          (tag_name)
          (attribute (attribute_name) (attribute_value)))
        (text)
        (end_tag (tag_name)))
      (end_tag (tag_name)))
    (text)
    (end_tag (tag_name))))
```

### `test/corpus/4_complex_layouts.txt`

```Plaintext
====================================
Sidebars and Disclosures
====================================
<aside>
  <details open="true">
    <summary>Monster Stats</summary>
    <paragraph>HP: 50</paragraph>
  </details>
</aside>
---
(document
  (element
    (start_tag (tag_name))
    (element
      (start_tag
        (tag_name)
        (attribute (attribute_name) (attribute_value)))
      (element
        (start_tag (tag_name))
        (text)
        (end_tag (tag_name)))
      (element
        (start_tag (tag_name))
        (text)
        (end_tag (tag_name)))
      (end_tag (tag_name)))
    (end_tag (tag_name))))
```
