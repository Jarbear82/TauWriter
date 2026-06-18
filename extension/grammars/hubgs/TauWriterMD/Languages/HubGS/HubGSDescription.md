	# HubGS: Hub Graph Script
A json-like file format for the Hub-graph model.

## File Structure
All definitions must be contained within a `DEFINITIONS` block, and instances within an `INSTANCES` block. Imports are handled in an `IMPORTS` block.

```hubgs
IMPORTS [ ... ]
DEFINITIONS [
    FIELDS [ ... ]
    ENUMS [ ... ]
    STRUCTS [ ... ]
    HUBS [ ... ]
]
INSTANCES [ ... ]
```

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
