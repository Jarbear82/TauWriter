; Structural block boundaries (match the literal string tokens)
"<document>" @keyword.control
"</document>" @keyword.control
"<body>" @keyword.control
"</body>" @keyword.control

; Tags (generic elements inside body/metadata)
(tag_name) @tag

; Attributes
(attribute_name) @attribute
(attribute_value) @string

; Brackets (for generic inner elements)
["<" ">" "</" "/>"] @punctuation.bracket

; Comments
(comment) @comment

; Specialized Tags (Contextual) - still applies to tag_name nodes
((tag_name) @keyword.control (#match? @keyword.control "^(meta|section)$"))
((tag_name) @keyword (#match? @keyword "^(hubref)$"))
((tag_name) @markup.bold (#match? @markup.bold "^(bold)$"))
((tag_name) @markup.italic (#match? @markup.italic "^(italic)$"))
((tag_name) @keyword.exception (#match? @keyword.exception "^(review)$"))
