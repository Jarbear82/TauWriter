# TauWriter

TauWriter is an industrial-grade, graph-augmented word processor designed for worldbuilders, novelists, and RPG Dungeon Masters. It bridges the gap between linear narrative prose and structured knowledge management.

## 🧠 Core Philosophy: Knowledge as a Codebase

TauWriter treats notes, lore, characters, locations, and all worldbuilding artifacts **as strongly-typed code**. Every piece of information is structured, version-controlled, and statically analyzable. Just as an IDE understands your source code—its types, references, and dependencies—the TauWriter LSP understands your knowledge base with the same rigor.

This means:
- **Structured data** replaces freeform notes: every entity has a defined type, validated fields, and explicit relationships.
- **Cross-references are first-class**: links between prose and structured data (and between entities themselves) are resolved at parse time, not search time.
- **Refactoring is safe**: rename a character, merge locations, or restructure a magic system with full traceability across all documents that reference it.
- **No external scripting runtime needed**: the entire analysis pipeline is pure Rust—fast, memory-safe, and deterministic.

## 🏗 Architecture

The entire backend and Language Server are implemented in **pure Rust**. There is no JavaScript engine, no Node.js runtime, and no external scripting dependency.

This design ensures:
- **High performance**: Incremental computation via Salsa keeps LSP responses fast even on large knowledge bases.
- **Cache locality**: The data model is designed for efficient CPU cache utilization, minimizing memory indirection during graph traversal.
- **Memory safety**: Rust's ownership system guarantees no dangling references or data races in the parsing and evaluation pipeline.

```
+---------------------+          +----------------------+          +---------------------------+
|   Document Layer    |          |    Graph Layer       |          |     Pure Rust LSP         |
|   (TWXML files)     | <------> |   (HubGS Script)     | <------> |   (Analysis Engine)       |
+---------------------+   refs   +----------------------+          +---------------------------+
| - Linear prose      |                                            | - Tag autocomplete        |
| - Chapters, scenes  |                                            | - Structural validation   |
| - Formatting        |                                            | - Formatting pipeline     |
+---------------------+                                            +---------------------------+
```

## 🚀 Overview

The core of TauWriter is a dual-layered approach to writing:
- **Prose Layer (TWXML):** A custom XML-based language for narrative text that supports semantic tagging of entities.
- **Knowledge Layer (HubGS):** A powerful DSL (Domain Specific Language) for defining and instantiating structured knowledge graphs (Characters, Locations, Lore, etc.).

These layers are linked via a custom **Language Server Protocol (LSP)** that provides real-time cross-referencing, autocomplete, and diagnostics within the editor.

## 🏗 Project Structure

```text
.
├── extension/          # Zed Editor Extension
│   ├── languages/      # Tree-sitter grammars for HubGS and TWXML
│   └── src/            # Rust bridge for Zed
├── lsp/                # Core LSP Implementation (Rust)
│   ├── src/            # Salsa DB, Parser, and LSP handlers
│   └── tests/          # Integration and validation tests
├── TauWriterMD/        # Design documentation and specifications
├── examples/           # Sample .twxml and .hubgs files
└── Status.md           # Implementation progress and roadmap
```

## 🛠 Key Technologies

- **Rust:** High-performance core implementation.
- **Salsa:** Incremental computation engine for extremely fast LSP updates.
- **Tree-sitter:** Incremental parsing for robust syntax highlighting and AST extraction.
- **Language Server Protocol (LSP):** Standardized communication between the editor and the analysis engine.
- **Zed Extension API:** Native integration with the Zed editor.

## 📖 Languages

### TWXML (TauWriter XML)
A semantic markup language for prose. It uses nesting depth to determine structure (headings) and `<hubref>` tags to link to the knowledge graph.

```xml
<document>
  <metadata>
  </metadata>
  <body>
    <section>
      <heading>The Journey Begins</heading>
      <paragraph>
        <hubref id="aragorn">Aragorn</hubref> looked toward <hubref id="mordor">Mordor</hubref>.
      </paragraph>
    </section>
  </body>
</document>
```

### HubGS (Hub Graph Script)
A DSL for defining typed nodes ("Hubs") and their relationships ("Roles").

```hubgs
HUBS [
    Character {
        name,
        resides_in -> (1) ALLOWS [Location]
    }
]

INSTANCES [
    aragorn:Character {
        name = "Aragorn",
        resides_in = [rivendell]
    }
]
```

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (latest stable)
- [Zed Editor](https://zed.dev/) (for using the extension)

### Building
Run the build script to compile the LSP and prepare the extension:
```bash
./build.sh
```

### Development
- The LSP source is in `lsp/`.
- The Zed extension source is in `extension/`.
- Integration tests can be run via `cargo test -p tauwriter-lsp`.

## 📈 Current Status

See [Status.md](Status.md) for a detailed breakdown of implemented LSP features and the production roadmap.

## 📜 Documentation

Detailed design documents, grammar specifications, and testing corpuses can be found in the [TauWriterMD/](TauWriterMD/) directory.

## ✨ LSP Features

The TauWriter Language Server provides IDE-grade editing support for both TWXML and HubGS:

| Feature | Description |
|---------|-------------|
| **Tag Auto-completion** | Context-aware suggestions for `<hubref>`, `<section>`, and all valid TWXML elements. Type-ahead filtering narrows results as you type, with prioritized rankings based on current nesting depth and scope. |
| **Structural Tag Auto-closing** | When you open a tag like `<section>` or `<paragraph>`, the LSP automatically inserts the matching closing tag at the correct nesting level. Nested structures are auto-balanced so orphaned tags never accumulate. |
| **Integrated Formatting Pipeline** | The `tauwriter-fmt` module hooks into the standard `textDocument/formatting` LSP request to normalize indentation, enforce consistent nesting depth, and standardize whitespace across both TWXML documents and HubGS schema files on-save or on-demand. |
| **Cross-Reference Validation** | Real-time diagnostics flag broken `<hubref>` links, unresolved instance references in HubGS, and type mismatches in role assignments before you save. |
| **Bidirectional Navigation** | Go-to-definition jumps from prose to Hub instances and back-links from Hubs to every document that mentions them. |
