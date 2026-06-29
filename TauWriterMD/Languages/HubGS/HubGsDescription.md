# HubGS: Hub Graph Script
A structured, type-safe DSL for defining and instantiating knowledge graphs. HubGS powers the "Knowledge as a Codebase" methodology in TauWriter—treating worldbuilding data as strongly-typed code with compile-time validation.

## File Structure
All definitions must be contained within a `DEFINITIONS` block, and instances within an `INSTANCES` block. Imports are handled in an `IMPORTS` block.

```hubgs
IMPORTS [ ... ]
DEFINITIONS [
    FIELDS [ ... ],
    ENUMS [ ... ],
    STRUCTS [ ... ],
    HUBS [ ... ],
]
INSTANCES [ ... ]
```

## Implicit Module Namespaces
HubGS does not use explicit `MODULE` string declarations. Instead, the **filename and filepath act as the implicit module namespace**. 

When a file imports definitions from another schema, the LSP automatically namespaces those definitions behind the filename to prevent collisions.
- **Example:** If `campaign.hubgs` imports a `weight` field from `./core.hubgs`, it resolves as `core::weight`. 
- **Conflict Resolution:** If multiple files share the exact same implicit module namespace (e.g., both are named `core.hubgs` in different directories), they contribute to a unified namespace, and the LSP resolves collisions strictly on identical definitions.

## Block Dependencies

HubGS enforces conditional block requirements to ensure type safety:

- **`instances` → `definitions` or `imports`**: If an `INSTANCES` block is present, it is **mandatory** that the file also contains either:
  - A local `DEFINITIONS` block with all required Hub types declared in a `HUBS` sub-block, **or**
  - An `IMPORTS` statement that imports all referenced Hub types from other `.hubgs` files.
- This means you cannot instantiate a Hub without first declaring its type somewhere resolvable in the file's scope.

## Resolution Rules

Every instance must successfully resolve against a known definition, enforcing **strict referential integrity**:

1. **Type Resolution**: Each `instance:HubType` pair must map to a Hub type that is either locally defined or imported.
2. **Field Resolution**: Every field assigned in an instance must be declared on the resolved Hub type (either directly or inherited from global FIELDS).
3. **Role Target Resolution**: When assigning a role value (e.g., `resides_in = [rivendell]`), every referenced Hub ref must exist as a concrete instance within the unified graph.
4. **Import Chain Resolution**: Imported types transitively resolve their own dependencies—if File A imports from File B, and File B imports from File C, the resolution chain traverses all three.

If any of these resolutions fail, the parser emits a diagnostic error and the instance is marked invalid in the LSP.

## Tree-sitter Grammar Enforcement

The HubGS tree-sitter grammar **strictly enforces** block structure during the parsing phase:
- Top-level blocks (`IMPORTS`, `DEFINITIONS`, `INSTANCES`) must appear at the file root. No arbitrary code or declarations outside these containers are permitted.
- Within `DEFINITIONS`, sub-blocks (`FIELDS`, `ENUMS`, `HUBS`) are lexically validated and type-checked before any instance is evaluated.
- Parse errors in block structure produce immediate diagnostics—no partial evaluation of malformed files occurs.

---

## Types

### Enums
An enumeration type with a limited set of specific values. 
Example:
```hubgs
ENUMS [
    Aspect {
        Mind,
        Body,
        Spirit
    }
]
```

### Fields
Global scope defined. Named and typed properties to be used in hubs and structs.
Example:
```hubgs
FIELDS [
    name: Text,
    age: Number,
    is_hero: Boolean,
    tags: Array<Text>,
    meta: Map<Text, Text>
]
```

#### Primitive Types
*   **Number**: 64-bit float, supporting decimal, hex, binary, and octal.
*   **Text**: UTF-8 strings, supports triple-quotes and template interpolation.
*   **Boolean**: `true` or `false`.

#### Collections

##### Struct
Structs group existing global fields together. They are accessed using dot notation.
Example:
```hubgs
STRUCTS [
    Dimensions {
        width,
        height,
        depth
    }
]
```

##### Array
Ordered list of a single type. Syntax: `Array<Type>`. Comma separated list wrapped in `[]` square brackets.

##### Map
Key-value pairs. Syntax: `Map<KeyType, ValueType>`.

##### Set
Unique collection of types. Syntax: `Set<Type1, Type2, ...>`.

### Roles
Hub-specific relationships. Roles define how Hubs connect to each other.
Syntax:
`role_name <direction> (<multiplicity>) ALLOWS [<HubType>, ... ]`

*   **Direction**: `->` (outbound), `<-` (inbound), `<->` (bidirectional), `-` (unspecified).
*   **Multiplicity**: `(1)`, `(*)`, `(0..*)`, `(1..5)`, etc.

### Hubs
Hubs are the primary entities. They reference global fields and define local roles.
Example:
```hubgs
HUBS [
    Person {
        name,
        age = @default(18),
        home -> (1) ALLOWS [Location]
    }
]
```
Note: Hub fields **must** be defined globally in a `FIELDS` block before being used in a Hub.

## Type Inheritance (`EXTENDS`)

HubGS supports composite inheritance for Hub types using the `EXTENDS` keyword. This allows a child Hub type to inherit fields and roles from one or more parent Hub types.

### Syntax

```hubgs
HUBS [
    ParentTypeA { ... },
    ParentTypeB { ... },
    ChildType EXTENDS [ParentTypeA, ParentTypeB] {
        // Additional child-specific fields and roles
    }
]

```

---

### Semantic & Compilation Rules

#### 1. Composition via Set-Union

Because all properties must be defined in the global `FIELDS` block before being referenced in a Hub, there is no structural risk of field layout collision.

* If `ParentTypeA` and `ParentTypeB` both use the global field `description`, `ChildType` inherits it exactly once.
* No diamond-problem resolution strategies are required at the compiler level.

#### 2. Role Inheritance

The child Hub automatically participates in all relationship roles defined on its parents.

* **Allows List Polymorphism:** If a role anywhere in the system allows a parent type (e.g., `ALLOWS [ParentTypeA]`), instances of `ChildType` are automatically valid targets for that role.



#### 3. Decorator Precedence (`@default` and `@computed`)

When a child type overrides a field property inherited from a parent, configuration rules apply:

* **Overriding `@default`:** A child type can replace a parent's `@default()` value with a new local `@default()` expression.
* **Overriding `@computed`:** A field marked as `@computed()` in a parent type **cannot** be redefined or overridden by the child type, ensuring deterministic evaluation pipelines.

---

### Complete Blueprint Example

```hubgs
// implicit module namespace: entities.hubgs
DEFINITIONS [
    FIELDS [
        name: Text,
        description: Text,
        health: Number,
        mana: Number,
        is_aggressive: Boolean,
        faction: Text
    ],

    HUBS [
        // Parent Type 1
        Actor {
            name,
            description,
            health = @default(100)
        },

        // Parent Type 2
        FactionMember {
            faction = @default("Unaligned"),
            is_aggressive = @default(false)
        },

        // Composite Child Type inheriting from both parents
        Mage EXTENDS [Actor, FactionMember] {
            mana = @default(50),
            // Overriding a parent default value is permitted
            health = @default(75) 
        }
    ]
]

INSTANCES [
    // aragorn behaves polymorphically as an Actor and FactionMember
    jaina:Mage {
        name = "Jaina Proudmoore",
        description = "Archmage of the Kirin Tor.",
        mana = 120,
        faction = "Alliance"
        // health evaluates to its overridden default: 75
        // is_aggressive evaluates to its inherited default: false
    }
]

```
