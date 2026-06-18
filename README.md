# TauWriter

TauWriter is an industrial-grade, graph-augmented word processor designed for worldbuilders, novelists, and RPG Dungeon Masters. It bridges the gap between linear narrative prose and structured knowledge management.

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
  <section>
    <heading>The Journey Begins</heading>
    <paragraph>
      <hubref id="aragorn">Aragorn</hubref> looked toward <hubref id="mordor">Mordor</hubref>.
    </paragraph>
  </section>
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
