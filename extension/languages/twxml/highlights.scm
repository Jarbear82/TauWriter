; Tags
(tag_name) @tag

; Attributes
(attribute_name) @attribute
(attribute_value) @string

; Brackets
["<" ">" "</" "/>"] @punctuation.bracket

; Comments
(comment) @comment

; Specialized Tags (Contextual)
((tag_name) @keyword.control (#match? @keyword.control "^(section|document)$"))
((tag_name) @keyword (#match? @keyword "^(hubref)$"))
((tag_name) @markup.bold (#match? @markup.bold "^(bold)$"))
((tag_name) @markup.italic (#match? @markup.italic "^(italic)$"))
((tag_name) @keyword.exception (#match? @keyword.exception "^(review)$"))
