(element
  (start_tag
    name: (tag_name) @_tag_name
    (attribute
      name: (attribute_name) @_attr_name
      value: (attribute_value) @injection.language))
  (text) @injection.content
  (#eq? @_tag_name "codeblock")
  (#eq? @_attr_name "language"))
