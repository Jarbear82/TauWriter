# TauWriter Project Status

## Goals
Build an industrial-grade LSP and Zed extension for the TauWriter ecosystem, enabling seamless cross-referencing between prose (TWXML) and structured knowledge (HubGS).

## Implementation Progress

| Feature | Status | Description |
|:---|:---:|:---|
| **Project Initialization** | ✅ | Rust workspace, Salsa DB, and Zed extension skeleton. |
| **Workspace Indexing** | ✅ | Background crawling and ingestion of all workspace files. |
| **Semantic Resolution** | ✅ | Cross-language symbol resolution (HubGS <-> TWXML). |
| **Diagnostics** | ✅ | Real-time error reporting, structural validation, and tag checking. |
| **Refactoring Support** | ✅ | Global renames and structural changes (References/Rename). |
| **AST Extraction** | ✅ | Tree-Sitter based AST extraction logic structure. |
| **Structural Awareness**| ✅ | Full index support for HubGS FIELDS, ENUMS, STRUCTS, and TWXML elements. |
| **Dynamic Evaluation**| 🔄 | In Progress - Engine building. Decorator parsing complete (`@computed`, `@default`). Evolving prototype into a robust computed graph data engine. |

## Production Roadmap

| Feature | Status | Description |
|:---|:---:|:---|
| **Parser Finalization** | ✅ | Generated C parsers and linked to Rust LSP via `build.rs`. |
| **Autocomplete** | ✅ | Context-aware suggestions for Hub IDs, fields, and roles. |
| **Hover Support** | ✅ | Formatted documentation display with Hub details. |
| **Type-checking** | ✅ | Validation of Hub roles, multiplicities, and field types. |
| **Semantic Tokens** | ✅ | Advanced context-aware syntax highlighting for all symbols. |
| **Import Scoping** | ✅ | Respects `IMPORTS` statements for scoped symbol resolution. |
| **Folding Ranges** | ✅ | Support for collapsing blocks, sections, and definitions. |
| **Workspace Symbol** | ✅ | Searching for Hubs and Types across the entire project. |
| **Testing Suite** | ✅ | Broad integration test coverage for LSP handlers, Salsa queries, and validation pipeline. Some handlers have minimal tests (e.g., didSave). |
| **CI/CD** | ✅ | Automated GitHub Actions for multi-platform binary distribution.
| **Zed Extension** | ⚠️  | Functional skeleton with grammars, language configs, and pre-built binaries in `extension/bin/`. Auto-downloading and one-click install require Zed marketplace verification. |
| **Editor QoL** | 🔄 | Tag auto-closing, snippet generation, and structural autocomplete for TWXML. |

## Current Focus

### 1. Structural Enforcement & Validation Pipeline (In Progress)
Strict schema enforcement for document and graph structures to ensure data integrity.
- [ ] **TWXML Skeleton Enforcement:** Validate that all TWXML documents strictly adhere to the root `<document>` schema containing exactly one `<metadata>` block (housing `<meta/>` tags) and one `<body>` block.
- [ ] **HubGS Dependency Validation:** Enforce section-level dependencies. If an `INSTANCES` block exists, validate that a `DEFINITIONS` block is present locally or fully satisfied via an `IMPORTS` statement.
- [ ] **Instance Resolution:** Ensure all declared instances successfully resolve to a defined Hub type.
- [x] Implement TWXML Nesting Rules (e.g., `<heading>` levels inside `<body>` or `<section>`)
- [x] Implement TWXML Referential Integrity (Unresolved references for `<hubref>`)
- [x] Implement HubGS Type & Multiplicity Enforcement

### 2. Dynamic Evaluation Engine (In Progress)
Evolving the prototype evaluator into a robust engine for computed graph data.
- [ ] Implement AST evaluator for `@computed` formulas (arithmetic, string concatenation).
- [ ] Implement cross-Hub field access via roles (e.g., `this.companions.length`).
- [ ] Enforce `@default` override rules during instance instantiation.

### 3. Editor Experience & LSP Capabilities (In Progress)
Enhancing the writing and data-entry flow natively within the editor.
- [ ] **TWXML Tag Auto-completion:** Context-aware suggestions for structural tags (`<section>`, `<heading>`, `<body>`).
- [ ] **TWXML Tag Auto-closing:** Automatically generate closing tags (e.g., typing `<metadata>` inserts `</metadata>`).
- [x] Context-aware autocomplete for Hub IDs, fields, and roles.

### 4. Formatter Module (`tauwriter-fmt`) (In Progress)
- [ ] Refactor formatting engine to natively support the new TWXML `<metadata>` and `<body>` skeleton.
- [ ] Standardize indentation and line-breaking rules for nested TWXML blocks.
- [x] Integrate LSP `textDocument/formatting` handler.

## JSON-RPC Testing Progress

| LSP Operation | Implementation Status | JSON-RPC Test |
|:---|:---:|:---:|
| `initialize` | ✅ | ✅ |
| `initialized` | ✅ | ✅ |
| `shutdown` | ✅ | ✅ |
| `textDocument/didOpen` | ✅ | ✅ |
| `textDocument/didChange` | ✅ | ✅ |
| `textDocument/didClose` | ✅ | ✅ |
| `textDocument/didSave` | ⚠️  Stub | ✅ |
| `textDocument/declaration` | ✅ | ✅ |
| `textDocument/definition` | ✅ | ✅ |
| `textDocument/typeDefinition` | ✅ | ✅ |
| `textDocument/implementation` | ✅ | ✅ |
| `textDocument/references` | ✅ | ✅ |
| `textDocument/hover` | ✅ | ✅ |
| `textDocument/completion` | ✅ | ✅ |
| `textDocument/rename` | ✅ | ✅ |
| `textDocument/formatting` | ✅ | ✅ |
| `textDocument/documentHighlight` | ✅ | ✅ |
| `textDocument/documentSymbol` | ✅ | ✅ |
| `textDocument/foldingRange` | ✅ | ✅ |
| `textDocument/semanticTokens/full` | ✅ | ✅ |
| `workspace/symbol` | ✅ | ✅ |
| `textDocument/publishDiagnostics` | ✅ | ✅ |

*Note: Many other optional LSP features from the specification are currently not implemented.*
