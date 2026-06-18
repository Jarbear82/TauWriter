### `test/corpus/1_structure.txt`

This tests the top-level comma separation and import blocks.

```Plaintext
====================================
Top-Level Sections and Imports
====================================
IMPORTS [
    [Character, Location] FROM "core.hubgs",
    [MagicSystem] FROM "../magic.hubgs"
],
DEFINITIONS [
    ENUMS [
        Status { ACTIVE, INACTIVE }
    ]
]
---
(document
  (imports_section
    (import_statement
      (identifier)
      (identifier)
      (string))
    (import_statement
      (identifier)
      (string)))
  (definitions_section
    (enums_block
      (enum_definition
        (identifier)
        (identifier)
        (identifier)))))
```

### `test/corpus/2_definitions.txt`

This tests fields, decorators, and the specific syntax for all four role directions.

```Plaintext
====================================
Hub Definitions and Roles
====================================
DEFINITIONS [
    HUBS [
        Person {
            first_name,
            age = @default(18),
            home -> (1) ALLOWS [Location],
            friends <-> (0..*) ALLOWS [Person],
            killed_by <- (0..1) ALLOWS [Person],
            allies - (*) ALLOWS [Faction]
        }
    ]
]
---
(document
  (definitions_section
    (hubs_block
      (hub_definition
        (identifier)
        (hub_field
          (identifier))
        (hub_field
          (identifier)
          (decorator
            (number)))
        (hub_role
          (identifier)
          (role_direction)
          (multiplicity
            (number))
          (identifier))
        (hub_role
          (identifier)
          (role_direction)
          (multiplicity
            (number)
            (number))
          (identifier))
        (hub_role
          (identifier)
          (role_direction)
          (multiplicity
            (number)
            (number))
          (identifier))
        (hub_role
          (identifier)
          (role_direction)
          (multiplicity)
          (identifier))))))
```

### `test/corpus/3_expressions.txt`

This is the critical "torture test" file to ensure math precedence, member access, and unary operators group correctly without deforming the tree.


```Plaintext
====================================
Operator Precedence and Member Access
====================================
INSTANCES [
    math_test:Test {
        val1 = a + b * c,
        val2 = (a + b) * c,
        val3 = !this.is_active || count == 0
    }
]
---
(document
  (instances_section
    (instance_block
      ref: (identifier)
      type: (identifier)
      (instance_assignment
        (identifier)
        (binary_expression
          left: (identifier)
          right: (binary_expression
            left: (identifier)
            right: (identifier))))
      (instance_assignment
        (identifier)
        (binary_expression
          left: (parenthesized_expression
            (binary_expression
              left: (identifier)
              right: (identifier)))
          right: (identifier)))
      (instance_assignment
        (identifier)
        (binary_expression
          left: (unary_expression
            argument: (member_expression
              object: (identifier)
              property: (identifier)))
          right: (binary_expression
            left: (identifier)
            right: (number)))))))
```

### `test/corpus/4_instances.txt`

This tests data assignments, arrays, template strings with interpolation, and the `@metadata` block.

```Plaintext
====================================
Instances, Arrays, and Strings
====================================
INSTANCES [
    aragorn:Person {
        name = "Aragorn",
        background_color = 0xFFD700,
        title = `King ${name}`,
        stats = [10, 0x1A, 0b101],
        is_active = true,
        @metadata {
            display = name,
            background = background_color
        }
    }
]
---
(document
  (instances_section
    (instance_block
      ref: (identifier)
      type: (identifier)
      (instance_assignment
        (identifier)
        (string))
      (instance_assignment
        (identifier)
        (number))
      (instance_assignment
        (identifier)
        (template_string
          (identifier)))
      (instance_assignment
        (identifier)
        (array
          (number)
          (number)
          (number)))
      (instance_assignment
        (identifier)
        (boolean))
      (metadata_block
        (identifier)
        (identifier)
        (identifier)
        (identifier)))))
```