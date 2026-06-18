; keywords
(imports_section "IMPORTS" @keyword)
(definitions_section "DEFINITIONS" @keyword)
(instances_section "INSTANCES" @keyword)
(fields_block "FIELDS" @keyword)
(enums_block "ENUMS" @keyword)
(structs_block "STRUCTS" @keyword)
(hubs_block "HUBS" @keyword)
(import_statement "FROM" @keyword)
(hub_role "ALLOWS" @keyword)

; Literals
(string) @string
(template_string) @string
(number) @number
(boolean) @boolean

; Identifiers
(field_definition (identifier) @property)
(enum_definition (identifier) @type)
(struct_definition (identifier) @type)
(hub_definition (identifier) @type)
(instance_block ref: (identifier) @variable)
(instance_block type: (identifier) @type)
(instance_assignment (identifier) @property)
(hub_field (identifier) @property)
(hub_role (identifier) @function)

; Decorators
(decorator ["@computed" "@default"] @function.macro)
(metadata_block "@metadata" @function.macro)

; Operators & Punctuation
(role_direction) @operator
(binary_expression (operator) @operator)
(unary_expression (operator) @operator)
["[" "]" "{" "}" "(" ")" ":" "=" "," "." "->"] @punctuation.bracket

; Comments
(comment) @comment
