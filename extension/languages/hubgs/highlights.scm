; Keywords
"IMPORTS" @keyword
"FROM" @keyword
"DEFINITIONS" @keyword
"FIELDS" @keyword
"ENUMS" @keyword
"STRUCTS" @keyword
"HUBS" @keyword
"ALLOWS" @keyword
"INSTANCES" @keyword

; Punctuations
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
":" @punctuation.delimiter
"," @punctuation.delimiter
"." @punctuation.delimiter

; Operators
"=" @operator
"->" @operator
"<-" @operator
"<->" @operator
"-" @operator
"&&" @operator
"||" @operator
"==" @operator
"!=" @operator
"+" @operator
"*" @operator
"/" @operator
"!" @operator
".." @operator

; Literals
(string) @string
(template_string) @string
(number) @number
(boolean) @constant.builtin.boolean

; Comments
(comment) @comment @spell

; Identifiers and Types
(identifier) @variable
(instance_block ref: (identifier) @variable.member)
(instance_block type: (identifier) @type)
(field_definition (identifier) @variable.field)
(enum_definition (identifier) @type)
(struct_definition (identifier) @type)
(hub_definition (identifier) @type)
(generic_type (identifier) @type)
(type (identifier) @type)

; Decorators and Metadata
"@computed" @function.builtin
"@default" @function.builtin
"@metadata" @keyword.directive
(decorator ["@computed" "@default"] @function.builtin)
