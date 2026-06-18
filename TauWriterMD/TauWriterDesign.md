# TauWriter

## Description
An IDE-style, graph-augmented word processor inspired by Obsidian and modern developer IDEs. Designed for **novel writers, worldbuilders, and D&D Dungeon Masters** who need to maintain structured knowledge alongside linear narrative text. Built with an eye toward future collaboration features (inspired by Zed's multiplayer model and Discord-like server organization).

The core idea: prose lives in a custom XML document format (TWXML), while structured data (characters, places, magic systems, mechanics, etc.) lives in a graph of typed nodes called **Hubs**. The two layers connect through references, with an LSP-powered editor providing real-time cross-referencing, autocomplete, and semantic underlining.

---

## Architecture Overview

```
+---------------------+          +----------------------+
|   Document Layer    |          |    Graph Layer       |
|   (TWXML files)     | <------> |   (HubGS Script)     |
+---------------------+   refs   +----------------------+
| - Linear prose      |          | - Structured data    |
| - Chapters, scenes  |          | - Relationships      |
| - Formatting        |          | - Validation         |
+---------------------+          +----------------------+
          |                                |
          v                                v
+---------------------+         +--------------------------------+
|       Editor        |         |   Graph Visualization          |
+---------------------+         +--------------------------------+
|   - WYSIWYG View    |         |   - Global Graph View          |
|   - Markdown View   |         |   - Outline Scoped View        |
|   - Raw TWXML View  |         |   - Node-Scoped View           |
|   - Document Tree   |         +--------------------------------+
+---------------------+         
```

The default UI layout is a split view: document editor on one side, HubGS graph visualization on the other. Editor views are switchable per tab and support split-pane mode (e.g., WYSIWYG alongside Raw TWXML).

---

## Knowledge Base Structure

A **Knowledge Base** is the top-level container — analogous to an Obsidian vault. It is a directory containing:

| Component | Format | Description |
|-----------|--------|-------------|
| Documents | `*.twxml` | One or more TWXML files, each representing a distinct document (novel chapter, wiki page, scratch notes, campaign log, etc.) |
| Graph Data | `*.hubgs` | One or more HubGS files defining schemas and instances. Multiple files are merged at runtime into a single unified graph. |

**Key design decisions:**
- A single Knowledge Base can contain multiple documents backed by the same graph. This allows one world bible to feed both a novel draft and a companion wiki simultaneously.
- Graph→Document back-references are **not persisted** in HubGS files. They are computed dynamically at runtime by the LSP, cached while open, and discarded on close. This prevents stale cross-links.
- Document→Graph references **are persisted** via `<hubref>` tags embedded in TWXML.

---

## Document Layer

### Format: TauWriter XML (TWXML)
The document uses a custom XML vocabulary called **TauWriter XML** as its raw format. Graph references are embedded using `<hubref>` tags for extensibility and standards compliance.

#### Why TWXML over HTML or Markdown?
- **No ambiguous syntax** — Markdown lacks consistent spec support across parsers; standard HTML is not designed for document fragmentation
- **Rich structure** — Native nesting, attributes, and extensibility like HTML, but with a controlled vocabulary purpose-built for writing workflows
- **Fragmentation resilience** — Documents can optionally be split into directories of multiple TWXML files that are stitched together at runtime. If a bug corrupts one fragment, the rest of the document survives. Standard HTML does not cleanly support this pattern.
- **Easy to render and parse** — XML parsers are universally available; can be rendered as Markdown in alternate view if desired
- **Human-readable and editable** — Like HTML, TWXML is plain text that writers and power users can read, version-control, and manually edit

#### TWXML Vocabulary
TWXML uses an **enforced tree hierarchy**: heading levels are derived from nesting depth, not explicit attributes. A `<heading>` directly under `<document>` is H1 (document title), one inside a `<section>` is H2, and deeper nesting produces H3+.

The vocabulary mirrors common structural elements used in writing:

| Element | Purpose |
|---------|---------|
| `<document>` | Root element wrapping a complete document or fragment |
| `<meta />` | Document-level metadata (author, tags, status); placed inside `<document>` before block content |
| `<section>` | Semantic divider for parts, chapters, scenes, etc.; carries `alias` attribute |
| `<heading>` | Section heading; level is determined by nesting depth |
| `<paragraph>` | Fundamental prose unit |
| `<hr />` | Thematic break between paragraph-level elements |
| `<hubref>` | Inline container for graph references; uses `id` attribute |
| `<bold>`, `<italic>`, `<underline>` | Text styling (applied via nesting, not attributes) |

Custom elements can be added as the system evolves. The vocabulary is intentionally minimal to keep files readable.

### Document Structure Example
```xml
<document>
  <meta name="author" content="J.R.R. Tolkien" />
  <section alias="The Fellowship">
    <heading>Departure</heading>
    <paragraph>
      <bold><hubref id="aragorn">Aragorn</hubref></bold> drew his sword 
      and looked across the field toward <hubref id="mordor">Mordor</hubref>.
    </paragraph>
  </section>
</document>
```

Key points:
- Heading levels are determined by nesting depth — no explicit `<h1>`–`<h6>` needed
- `<meta />` tags live inside `<document>` before any block content
- Prose lives inside standard structural tags (`<paragraph>`, `<section>`, etc.)
- Graph links use `<hubref id="<hub-reference>">`
- Formatting (bold, italic) wraps the reference tag — not the other way around
- Multiple hubs can be referenced within a single paragraph or even a single phrase

### Editor Views
The document side offers four switchable views. Each tab can independently select its active view, and split-pane mode allows two views side-by-side:

| View | Description |
|------|-------------|
| **WYSIWYG** | Primary editing mode — what you see is what you get, similar to a modern word processor (no page breaks; continuous scroll) |
| **Markdown** | Alternate view that renders TWXML as Markdown for users who prefer plain-text workflows |
| **Raw TWXML** | Shows the underlying XML with all `<hubref>` tags visible; precise and IDE-like, intended for power users |
| **Document Tree** | Graph visualization of the document's hierarchical structure. Nodes can represent chapters, sections (by heading level), or paragraphs. Granularity is adjustable — zoom from chapter-level nodes down to paragraph-level detail |

### Document Graph View Details
The Document Tree view treats the XML hierarchy as a graph since trees are a type of graph. Powered by standard XML parsers and language servers — not HubGS tools. The default UI resembles an IDE file explorer but can toggle to full 2D graph visualization. Adjusting granularity collapses or expands nodes at different heading levels, letting writers navigate large documents spatially.

---

## Graph Layer: HubGS

### Overview
HubGS (Hub Graph Script) is a custom DSL designed for defining typed, structured knowledge graphs optimized for **visualization**. It unifies nodes and hyperedges into a single vertex type called **Hubs**. Think of it as UML class diagrams meets data instances — with validation, computed fields, role-based relationships, and built-in visualization metadata.

- **File extension:** `.hubgs`
- **Tooling:** Custom parser + LSP for real-time validation, autocomplete, and cross-referencing with documents
- **Primary audience:** Worldbuilders who need to model characters, locations, magic systems, campaign mechanics, etc. However, HubGS is general-purpose — it can model any domain where structured knowledge benefits from visualization

### HubGS File Structure
A HubGS file has three optional top-level sections:

```hubgs
IMPORTS [ ... ]         // Optional — import schemas from other .hubgs files

DEFINITIONS [           // Schema-level type definitions
    FIELDS [...]
    ENUMS [...]
    HUBS [...]
]

INSTANCES [...]         // Concrete Hub objects with assigned values
```

| Section | Purpose |
|---------|---------|
| **IMPORTS** | Import schema definitions (Hubs, Fields, Enums) from other `.hubgs` files. Allows splitting large schemas across multiple files. |
| **FIELDS** | Typed scalar property definitions available to Hubs |
| **ENUMS** | Enumerated types used as field value constraints |
| **HUBS** | Hub type definitions that declare which fields they use and what roles they participate in |
| **INSTANCES** | Concrete Hub objects with assigned values |

### IMPORTS
Multiple `.hubgs` files are merged at runtime into a single unified graph. Each file can import schema definitions from others using:

```hubgs
IMPORTS [
    [Character, Location] FROM "./fantasy-core.hubgs",
    [MagicSystem]         FROM "../shared-mechanics.hubgs"
]
```

A HubGS file must define or import all schemas before using them in its own DEFINITIONS or INSTANCES sections. Path resolution is relative to the importing file's location within the Knowledge Base directory.

### FIELDS
Fields are typed property declarations. They define what scalar data a Hub can hold.

```hubgs
FIELDS [
    name: String,
    description: String,
    hit_points: Number,
    is_active: Boolean,
    realm_association: Realm,   // Enum type reference
    ids: Array<Number>          // Array of Numbers 
]
```

#### Field Types
| Type | Example | Description |
|------|---------|-------------|
| `String` | `"Aragorn"` | Free-form text |
| `Number` | `42`, `3.14` | Integer or floating-point |
| `Boolean` | `true`, `false` | True/false values |
| `Date` | `2024-01-15` | Timestamp/date values |
| `EnumName` | `PHYSICAL` | Any user-defined Enum (e.g., `Realm`) |
| `Array<T>` | `[1, 2, 3]` | Ordered collection of type T |
| `Set<T, U, ...>` | `{1, "hello"}` | Unordered collection mixing specified types |
| `Map<K, V>` | `{ key: value }` | Key-value pair collection of types K and V |

#### Computed Fields
Fields can be marked as computed using the `@computed()` decorator. These cannot be overridden at the instance level unless wrapped with a `@default()` decorator that provides a fallback value.

```hubgs
// In a Hub TYPE definition:
Person {
    first_name,
    last_name = @default("Anderson"),                     // Can be overridden in INSTANCES
    full_name = @computed(first_name + " " + last_name)  // Cannot be directly overridden
}
```

Computed fields support:
- String concatenation and simple formulas (current implementation)
- Arithmetic operations
- Cross-Hub access via roles (e.g., `this.resides_in.name`)
- **User-defined computed fields** — authors can define their own computed properties beyond system defaults. Full scripting support for power users is planned as a future extension.

### ENUMS
Enums are field-type constraints only. They cannot appear in role allows lists.

```hubgs
ENUMS [
    Realm {
        COGNITIVE,
        PHYSICAL,
        SPIRITUAL
    }
]
```

### HUBS (Type Definitions)
Hub types define the shape of data. Each Hub type declares:
- **Which fields** it uses for scalar properties
- **Which roles** it participates in for relationships with other Hubs

```hubgs
HUBS [
    Person {
        // Fields
        first_name,
        last_name = @default("Anderson"),
        full_name = @computed(first_name + " " + last_name),
        
        // Roles (relationships to other Hubs)
        resides_in -> (1) ALLOWS [World],          // Exactly 1 World, arrow points outward
        companions <-> (0..*) ALLOWS [Person],     // Bidirectional, any number of Person hubs
    },
    
    World {
        name,
        description,
        inhabitants <- (0..*) ALLOWS [Person]      // Arrow points inward: Persons "own" this link
    }
]
```

### Roles
Roles define relationships between Hubs. They are the edge-equivalent in the Hub model.

#### Role Syntax
```<role_name> <direction> (<multiplicity>) ALLOWS [<HubType>, ...]```

| Component | Description |
|-----------|-------------|
| **Role Name** | The key identifier for this relationship (e.g., `resides_in`, `companions`) |
| **Direction** | Controls semantic meaning and arrow rendering in visualization. See below. |
| **Multiplicity** | Constrains how many Hubs can fill this role. See below. |
| **Allows List** | Specifies which Hub types are valid values for this role |

#### Role Direction
Role direction is primarily about **semantic meaning** — it defines the relationship's intent and controls arrow rendering in visualization:

| Syntax | Name | Meaning | Visualization |
|:------:|------|---------|---------------|
| `->`  | Source | The hub containing the key is the source; the relationship flows outward | `(HubA)──[role]──>(HubB)` |
| `<-`  | Target | The hub containing the key is the target; the relationship flows inward from other hubs | `(HubA)<──[role]──(HubB)` |
| `<->` | Bidirectional | Mutual relationship rendered in both directions | `(HubA)<──[role]─>(HubB)` |
| `-`   | Undirected | Symmetric relationship with no directional semantics | `(HubA)────[role]────(HubB)` |

**Key insight:** Direction encodes semantic intent. `killed -> (1)` on Hub A means "A killed B" with the arrow pointing from A to B. Conversely, `killed_by <- (1)` means "A was killed by B" with the arrow pointing toward A. The same relationship can be expressed differently depending on which hub "owns" the perspective.

#### Role Multiplicity
Multiplicity constrains the cardinality of a role:

| Syntax | Meaning |
|--------|--------|
| `(1)`  | Exactly 1 Hub required |
| `(3)`  | Exactly 3 Hubs required |
| `(0..3)` | Between 0 and 3 Hubs (inclusive) |
| `(0..*)` | Optional many: zero or more |
| `(*)`  | Required many: at least 1, no upper bound |

**Validation:** An instance cannot be created if its role multiplicity constraints are not satisfied. If a linked Hub is deleted and would violate constraints, the deletion is **blocked** unless the user explicitly confirms relaxing the multiplicity (e.g., changing `(1..3)` to `(0..3)`).

#### Role Allows List
The allows list specifies which Hub types can be values of this role. Multiple types are permitted:

```hubgs
related_to -> (0..*) ALLOWS [Character, Location, Item]   // Any of these three types
```

### INSTANCES (Concrete Hubs)
The INSTANCES section creates actual Hub objects with data. Instances should be ordered so dependencies appear before dependents.

#### Hub Reference
Each Hub instance has a **reference** (ref) — a unique identifier used to locate the Hub in the graph and link it from documents.

```hubgs
// Syntax: <ref>:<HubType> {
//     <field> = <value>,
//     <role> = [<hub_ref1>, <hub_ref2>, ...]
// }
```

- **User-defined refs:** Human-readable identifiers like `aero`, `aragorn`, `mordor`
- **Auto-generated refs:** UUID-style identifiers when the system generates them automatically
- **Uniqueness:** Refs must be globally unique across all instances in a Knowledge Base. Two Hubs can have the same display name but MUST have different refs

#### Instance Example
```hubgs
INSTANCES [
    aragorn:Person {
        first_name = "Aragorn",
        last_name = "Elessar",
        // full_name is computed → "Aragorn Elessar"
        resides_in = [middle_earth],
        companions = [gandalf, legolas, gimli]
    },

    middle_earth:World {
        name = "Middle-earth",
        description = "The setting of the Lord of the Rings."
        // inhabitants is a target role (<-) populated by persons' resides_in
    }
]
```

### Hub Metadata (Visualization)
Hubs can include a metadata section for controlling visual appearance in the graph view. This includes properties like node labels, colors, and background styling:

```hubgs
aragorn:Person {
    first_name = "Aragorn",
    last_name = "Elessar",
    resides_in = [middle_earth],
    
    @metadata {
        display = "Aragorn Elessar",       // Must be stringifiable; shown as node label
        background = "#FFD700",            // Supported: color codes, image URLs of supported types
    }
}
```

---

## IDE-Style Features

### Cross-Referencing & Autocomplete
Powered by a custom LSP (Language Server Protocol) implementation:

1. **Semantic Underlining:** The editor detects text that may correspond to an existing Hub and underlines it similarly to spellcheck suggestions.
2. **Right-Click Context Menu:** User can view the suggested Hub link, inspect Hub details, or accept/reject the mapping.
3. **Accepting a Suggestion:** Maps the underlined text to the Hub via `<hubref id="<ref>">` in the document TWXML. This creates a live hyperlink-style reference — clicking it navigates to the Hub's detail panel in graph view (or opens a new graph tab scoped to that node if not currently visible).
4. **Autocomplete:** When typing Hub refs (e.g., in raw view or role assignments), the LSP provides predictive suggestions based on existing Hubs and valid types.

### Bidirectional Linking
- **Document → Graph:** Click a referenced word in prose → jump to the Hub detail panel in graph view. If the node is already within an open graph's scope, it highlights there; otherwise a new graph tab opens with that node as root.
- **Graph → Document:** Click a Hub → see all document locations where it's referenced (computed by LSP at query time, cached while open). The displayed text in documents acts like a hyperlink — it references the Hub but does not modify its data.

---

## Graph Visualization Views

The graph side of the split-view desktop app offers visualization modes for exploring structured knowledge:

| View | Description |
|------|-------------|
| **Global Graph** | Full view of every Hub and Role in the system. Best for high-level overview; can be overwhelming on large graphs. |
| **Outline View** | Shows only Hubs and Roles linked to the currently selected document element (e.g., "everything referenced in Chapter 1"). Most useful during active writing. |
| **Scoped View** | User selects a root Hub → displays all connected Hubs via roles. Filterable by multiplicity, direction, Hub type, etc. Best for deep-diving into specific entities. |

Filtering tools (by cardinality, direction, Hub type) are available in all views but are primarily used in Outline and Scoped views.

---

## Complete HubGS Example

```hubgs
// Notes As Code V8

IMPORTS [
]

DEFINITIONS [
    FIELDS [
        access_systems: Array<AccessSystem>,
        alt_name: String,
        description: String,
        essence_groups: Array<EssenceGroup>,
        essences: Array<Essence>,
        first_name: String,
        full_name: String,
        last_name: String,
        name: String,
        realm_association: Realm,
        worlds: Array<World>
    ],

    ENUMS [
        Realm {
            COGNITIVE,
            PHYSICAL,
            SPIRITUAL
        }
    ],

    HUBS [
        Universe {
            name,
            worlds -> (0..*) ALLOWS [World]
        },
        World {
            name,
            description,
            essence_groups -> (0..*) ALLOWS [EssenceGroup],
            access_systems -> (0..*) ALLOWS [AccessSystem]
        },
        Essence {
            name,
            alt_name,
            realm_association
        },
        EssenceGroup {
            name,
            essences -> (*) ALLOWS [Essence]    // At least 1 Essence required
        },
        AccessSystem {
            name
        }
    ]
];

INSTANCES [
    aero:Essence {
        name = "Aero",
        alt_name = "Air",
        realm_association = PHYSICAL
    },
    akvo:Essence {
        name = "Akvo",
        alt_name = "Water",
        realm_association = PHYSICAL
    },
    besto:Essence {
        name = "Besto",
        alt_name = "Animal",
        realm_association = COGNITIVE
    },
    drako:Essence {
        name = "Drako",
        alt_name = "Dragon",
        realm_association = SPIRITUAL
    },

    elementalEssences:EssenceGroup {
        name = "Elemental Essences",
        essences = [aero, akvo, besto, drako]
    },

    chakraSystem:AccessSystem {
        name = "Chakra System"
    },
    manifestation:AccessSystem {
        name = "Manifestation"
    },
    channeling:AccessSystem {
        name = "Channeling"
    },
    crystalCommerce:AccessSystem {
        name = "Crystal Commerce"
    },

    tera:World {
        name = "Tera",
        description = "The world of the elemental essences.",
        essence_groups = [elementalEssences],
        access_systems = [chakraSystem, manifestation, channeling]
    },
    atia:World {
        name = "Atia",
        description = "The world of the Attributes",
        essence_groups = [],
        access_systems = [crystalCommerce]
    },
    geo:World {
        name = "Geo",
        description = "The world of the Spells",
        essence_groups = [],
        access_systems = []
    },

    creationUniverse:Universe {
        name = "Creation Universe",
        worlds = [tera, atia, geo]
    }
];
```

---

## Persistence

### Knowledge Base Layout
A single directory acts as a Knowledge Base (similar to an Obsidian vault):

```
my-world-kb/
├── novel-chapter-1.twxml
├── novel-chapter-2.twxml
├── wiki-overview.twxml
├── scratch-notes.twxml
├── fantasy-core.hubgs
└── campaign-specific.hubgs
```

Each `.twxml` file is a complete document. Each `.hubgs` file contributes schema definitions and instances to the unified graph. Multiple documents share the same graph, allowing a single world bible to feed a novel draft, a wiki, scratch notes, and more simultaneously.

### Future (TBD)
Consider a compressed archive format (analogous to `.docx`) that bundles the directory contents into a single file for portability.

---

## Design Notes & Decisions

| Topic | Status | Notes |
|-------|--------|-------|
| **Modules** | ❌ Dropped in favor of Hubs | Grouping is achieved through container Hubs (e.g., a `Campaign` Hub with roles pointing to its Characters, Locations, Items). This solves the visualization grouping need without definition-scoping complexity. Schema-level modularity via IMPORTS may partially reintroduce this concern — see Open Questions below. |
| **Document format** | ✅ TWXML chosen | Custom XML vocabulary over standard HTML for fragmentation resilience, controlled extensibility, and cleaner multi-document support. Human-readable like HTML; still supports `data-*` attributes for graph references. Markdown view available as alternate rendering mode. |
| **Backlinks** | ✅ LSP-computed at query time | Document→Graph refs are persisted in TWXML. Graph→Document backrefs are computed dynamically and cached while open. Never stored in HubGS, so they cannot become stale. |
| **Hub deletion cascade** | ✅ Refs removed from document | When a Hub is deleted, all `<hubref>` tags pointing to it are automatically removed from documents. Prose text remains; only the link annotation is stripped. |
| **Role metadata (weighted edges)** | 🔍 Under consideration | Currently Roles only reference Hubs. Adding properties to roles (e.g., `owns { start: Date, end: Date }`) blurs into Labeled Property Graph territory. Noted for future evaluation. |
| **Target platform** | ✅ Desktop-first | Leaning toward Tauri or Rust + GPUI. Web and mobile are secondary targets. |
| **Hub Reference naming** | ✅ "Ref" chosen | Discarded: Key, Alias, Variable. Settled on "Reference" (shortened to "ref"). |
| **Computed field scope** | ✅ Type-level only | `@computed()` lives in Hub TYPE definitions within DEFINITIONS. Cannot be overridden at instance level. `@default()` is the mechanism for overridable fallbacks. |
| **Document-graph sync** | ✅ Graph is source of truth | Hub data lives exclusively in the graph layer. Documents reference hubs via hyperlinks — editing displayed text in a document does NOT modify the underlying hub. The graph instance remains the single source of truth. |

---

## Example: Integrated Document + Graph

### HubGS (graph side)
```hubgs
IMPORTS [
]

DEFINITIONS [
    FIELDS [
        name: String,
        description: String,
    ],

    HUBS [
        Character {
            name,
            description,
            companions <-> (0..*) ALLOWS [Character],
            resides_in -> (1) ALLOWS [Location]
        },
        Location {
            name,
            description
        }
    ]
];

INSTANCES [
    aragorn:Character {
        name = "Aragorn",
        description = "Heir of Isildur, Ranger of the North.",
        resides_in = [rivendell],
        companions = [gandalf]
    },
    gandalf:Character {
        name = "Gandalf",
        description = "Wizard and Istari.",
        resides_in = [rivendell],
        companions = [aragorn]
    },
    rivendell:Location {
        name = "Rivendell",
        description = "Last Alliance stronghold, home to Elrond."
    }
];
```

### TWXML Document (prose side)
```xml
<document>
  <meta name="author" content="J.R.R. Tolkien" />
  <section alias="A Shadow of the Past">
    <heading>Departure</heading>
    <paragraph>
      <hubref id="aragorn">Aragorn</hubref> stood at the edge of 
      <hubref id="rivendell">Rivendell</hubref>, his hand resting on the hilt of his sword. Beside him, 
      <hubref id="gandalf"><italic>Gandalf</italic></hubref> gazed into
      the distance.
    </paragraph>
  </section>
</document>
```

In this setup:
- Clicking "Aragorn" in the WYSIWYG editor jumps to the `aragorn:Character` Hub detail panel in graph view
- Clicking the Aragorn Hub shows all document locations where it's referenced (computed by LSP)
- The Outline View for Chapter 1 would show: Aragorn, Gandalf, Rivendell (and their interconnecting roles)

---

## Open Questions & Future Work

### Unresolved Design Decisions

| # | Question | Context |
|---|----------|---------|
| Q1 | **Document fragmentation details** | Documents can be split into directories of multiple TWXML fragments for corruption resilience. How does the stitch-together mechanism work at runtime? Should there be a manifest or index file that declares fragment order? Is the root `<document>` element always in one file with child fragments referenced, or is it purely conventional ordering (e.g., filename sorting)? |
| Q2 | **Collaboration architecture** | Future goal: Zed-inspired multiplayer editing and Discord-like server organization. Should the current architecture doc lay groundwork for this now (CRDTs, operational transforms, server-authoritative state), or keep it entirely aspirational? If groundwork is needed, what minimal abstractions should be in place today? |
| Q3 | **Tauri vs. GPUI** | Both are viable desktop targets. Tauri enables web tech (e.g., Svelte) for the frontend with a Rust backend; GPUI is fully native Rust rendering. Does either choice have implications for how editor views or graph visualization should be architected? Is this decision deferred until prototype phase? |
| Q4 | **HubGS schema imports — circular dependencies** | The IMPORTS mechanism allows splitting schemas across files. What happens with circular imports (File A imports from File B, which imports back from File A)? Should the system detect and reject cycles, or resolve them through a two-pass merge strategy? |

### Planned Features (Not Yet Specified)

| Feature | Description |
|---------|-------------|
| **AI Assistant Server** | LSP-like integration for AI interactions. Enabled via hover settings; uses a RAG pipeline to generate summaries/descriptions of referenced hubs. Every LSP action could be configurable and mappable to an AI command. Local small model or API key configuration. Sufficient for simple tasks like hub description generation. |
| **AI Provocations** | A second, more complex AI service that reads the current section and critiques highlighted passages — similar to collaborative document comments. Provides edit and/or commentary suggestions. Configurable for different tones, styles, or focuses (e.g., editing pass vs. creative brainstorming). |
| **Traits (Abstract Hubs)** | Hub types that can be inherited from but not directly instantiated. Enables shared behavior patterns across unrelated hub types without requiring full inheritance chains. |
| **EXTENDS Syntax** | Composite inheritance for Hub types. No diamond problem expected due to global field definitions and role conflict resolution at the schema level. Requires formal syntax and semantics specification. |
| **Expanded Data Types** | Additional supported types beyond current set: `UUID`, structured records (`struct`), and other useful primitives. Formal type system expansion needed. |
| **Expressive Strings / String Templates** | A template mechanism for constructing typed string values (e.g., date formatting, interpolated display names). Naming TBD — "expressive strings" is a working title. |
| **Decorators Reference Section** | A dedicated documentation section enumerating all decorators (`@computed()`, `@default()`, `@metadata`, and any future additions) with formal syntax, semantics, and examples. |
| **Schema Graph vs. Instance Graph** | Potential distinction between a "schema graph" (showing type relationships and inheritance) and an "instance graph" (showing concrete data). May partially reintroduce schema-level module concepts. Needs evaluation. |
| **Enums as inline field types?** | Should Enums remain as top-level definitions in DEFINITIONS, or could they be defined inline as field types? E.g., `status: Enum { ACTIVE, INACTIVE }`. Trade-off between convenience and cross-hub type sharing. |
