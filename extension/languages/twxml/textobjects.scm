; Section / Body text objects
(element
  (start_tag name: (tag_name) @_tag (#match? @_tag "^(section|body)$"))
  (end_tag name: (tag_name))
) @class.around

; Elements as functions/objects
(element) @function.around

; Comments
(comment) @comment.around
