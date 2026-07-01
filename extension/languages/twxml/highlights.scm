; highlights.scm for TWXML (Zed)

; ----------------------------------------------------------------------
; Structural literal keywords — fixed, mandatory document shell
; ----------------------------------------------------------------------

"<document>" @keyword
"</document>" @keyword
"<body>" @keyword
"</body>" @keyword

; ----------------------------------------------------------------------
; Punctuation
; ----------------------------------------------------------------------

(start_tag ["<" ">"] @punctuation.bracket)
(end_tag ["</" ">"] @punctuation.bracket)
(self_closing_element ["<" "/>"] @punctuation.bracket)

"=" @operator

; ----------------------------------------------------------------------
; Tag names — generic fallback
; ----------------------------------------------------------------------

(start_tag name: (tag_name) @tag)
(end_tag name: (tag_name) @tag)
(self_closing_element name: (tag_name) @constructor)

; <meta /> tags (aliased node) — distinct from generic self-closing tags
(meta_tag name: (tag_name) @keyword)

; ----------------------------------------------------------------------
; Specialized tag names (contextual overrides by name)
; ----------------------------------------------------------------------

(
  (start_tag name: (tag_name) @title)
  (#eq? @title "heading")
)
(
  (end_tag name: (tag_name) @title)
  (#eq? @title "heading")
)

(
  (start_tag name: (tag_name) @keyword)
  (#eq? @keyword "section")
)
(
  (end_tag name: (tag_name) @keyword)
  (#eq? @keyword "section")
)

(
  (start_tag name: (tag_name) @label)
  (#eq? @label "hubref")
)
(
  (end_tag name: (tag_name) @label)
  (#eq? @label "hubref")
)

(
  (start_tag name: (tag_name) @emphasis.strong)
  (#eq? @emphasis.strong "bold")
)
(
  (end_tag name: (tag_name) @emphasis.strong)
  (#eq? @emphasis.strong "bold")
)

(
  (start_tag name: (tag_name) @emphasis)
  (#eq? @emphasis "italic")
)
(
  (end_tag name: (tag_name) @emphasis)
  (#eq? @emphasis "italic")
)

(
  (start_tag name: (tag_name) @hint)
  (#eq? @hint "review")
)
(
  (end_tag name: (tag_name) @hint)
  (#eq? @hint "review")
)

; ----------------------------------------------------------------------
; Specialized content — the text actually being formatted
; ----------------------------------------------------------------------

(element
  (start_tag name: (tag_name) @_tag)
  (text) @title
  (#eq? @_tag "heading")
)

(element
  (start_tag name: (tag_name) @_tag)
  (text) @emphasis.strong
  (#eq? @_tag "bold")
)

(element
  (start_tag name: (tag_name) @_tag)
  (text) @emphasis
  (#eq? @_tag "italic")
)

(element
  (start_tag name: (tag_name) @_tag)
  (text) @hint
  (#eq? @_tag "review")
)

(element
  (start_tag name: (tag_name) @_tag)
  (text) @link_text
  (#eq? @_tag "hubref")
)

; ----------------------------------------------------------------------
; Attributes
; ----------------------------------------------------------------------

(attribute name: (attribute_name) @attribute)
(attribute_value) @string

; ----------------------------------------------------------------------
; Comments
; ----------------------------------------------------------------------

(comment) @comment
