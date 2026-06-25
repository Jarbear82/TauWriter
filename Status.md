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
| **Dynamic Evaluation**| ✅ | Engine building complete. Decorator parsing (`@computed`, `@default`), AST-based expression evaluator (arithmetic, string concatenation, unary ops, parentheses), cross-Hub field access via roles (`this.role.length`, `this.role.field`), and `@default` override enforcement. |

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
| **CI/CD** | ✅ | Automated GitHub Actions for multi-platform binary distribution. |
| **Zed Extension** | ⚠️  | Functional skeleton with grammars, language configs, and pre-built binaries in `extension/bin/`. Auto-downloading and one-click install require Zed marketplace verification. |
| **Editor QoL** | ✅ | Tag auto-closing via `onTypeFormatting`. Snippet generation and structural autocomplete for TWXML still pending. |
| **Advanced LSP Ops** | ⚠️  | CodeLens implemented. Signature Help, Document Links, and Call Hierarchy mappings pending. |
| **Graph Expansion** | 🔜 | Traits (Abstract Hubs), `EXTENDS` syntax, Role Metadata, and complex data types (UUID, structs). |
| **AI Integration** | 🔜 | RAG pipeline for Hub summaries and 'AI Provocations' for collaborative critique. |

## Current Focus

### 1. Structural Enforcement & Validation Pipeline ✅
Strict schema enforcement for document and graph structures to ensure data integrity.
- [x] **TWXML Skeleton Enforcement:** Validate that all TWXML documents strictly adhere to the root <document> schema containing exactly one <metadata> block (housing <meta/> tags) and one <body> block.
- [x] **HubGS Dependency Validation:** Enforce section-level dependencies. If an `INSTANCES` block exists, validate that a `DEFINITIONS` block is present locally or fully satisfied via an `IMPORTS` statement.
- [x] **Instance Resolution:** Ensure all declared instances successfully resolve to a defined Hub type.
- [x] Implement TWXML Nesting Rules (e.g., `<heading>` levels inside `<body>` or `<section>`)
- [x] Implement TWXML Referential Integrity (Unresolved references for `<hubref>`)
- [x] Implement HubGS Type & Multiplicity Enforcement

### 2. Dynamic Evaluation Engine ✅
Robust engine for computed graph data.
- [x] Implement AST evaluator for `@computed` formulas (arithmetic, string concatenation).
- [x] Implement cross-Hub field access via roles (e.g., `this.companions.length`).
- [x] Enforce `@default` override rules during instance instantiation.

### 3. Formatter Module (`lsp/src/formatter/`) ✅
Tree-sitter based formatter for both TWXML and HubGS. Not a separate crate — lives inline in the LSP crate.
- [x] Native support for TWXML `<document>`, `<metadata>`, and `<body>` skeleton. Dedicated formatters (`format_document_block`, `format_metadata_block`, `format_body_block`) with proper recursion.
- [x] Standardized indentation (2-space) and line-breaking rules for nested TWXML blocks. Block-level elements indent children at `indent_level + 1`, self-closing tags respect indentation, inline content stays compact.
- [x] Full block tag coverage — 26 TWXML block tags known to the formatter (`section`, `heading`, `paragraph`, `table`, etc.).
- [x] LSP `textDocument/formatting` handler wired up and JSON-RPC tested.

### 4. Editor Experience & LSP Capabilities (In Progress)
Enhancing the writing and data-entry flow natively within the editor.
- [x] **TWXML Tag Auto-closing:** Automatically generate closing tags (e.g., typing `<metadata>` inserts `</metadata>`). Implemented via `textDocument/onTypeFormatting`, triggered on `>`. Self-closing, closing, and comment tags are excluded. JSON-RPC tested.
- [x] **TWXML Tag Auto-completion:** Context-aware suggestions for structural tags (`<section>`, `<heading>`, `<body>`). Triggered on `<`, filters out structurally invalid tags based on parent context.
- [x] **CodeLens Integration:** Display actionable inline hints (e.g., "X references") directly above Hub instances.
- [ ] **Signature Help:** Show parameter and field hints while authors are filling out HubGS definitions.
- [ ] **Advanced Formatting Hooks:** Implement `textDocument/rangeFormatting`.
- [x] Context-aware autocomplete for Hub IDs, fields, and roles.
- [x] Inlay hints for HubGS instance types (`: TypeName`). Implemented, no test yet.
- [x] Code actions for resolving `<review>` tags to `<hubref>`. Two quickfix actions implemented.

### 5. HubGS Language & Graph Capabilities (Planned)
- [ ] **Traits & EXTENDS Syntax:** Add support for "Abstract Hubs" and composite inheritance to share behavior patterns without deep inheritance chains.
- [ ] **Role Metadata:** Support weighted edges and edge-properties (e.g., temporal bounds on a role like `owns { start: Date, end: Date }`).
- [ ] **Expanded Data Types:** Introduce `UUID`, structured records (`struct`), and string templates ("expressive strings").
- [ ] **Circular Import Resolution:** Define a robust two-pass merge strategy for `.hubgs` files with cyclic dependencies.

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
| `textDocument/codeAction` | ✅ | ✅ |
| `textDocument/inlayHint` | ✅ | ✅ |
| `textDocument/onTypeFormatting` | ✅ | ✅ |
| `textDocument/codeLens` | ✅ | ✅ |

### Unimplemented LSP Methods (No Test)

| LSP Operation | Status |
|:---|:---:|
| `exit` | ❌ |
| `$progress` | ❌ |
| `workspace/didChangeConfiguration` | ❌ |
| `workspace/configuration` | ❌ |
| `workspace/executeCommand` | ❌ |
| File operation notifications | ❌ |
| `textDocument/willSave` | ❌ |
| `textDocument/willSaveNotify` | ❌ |
| `textDocument/signatureHelp` | ❌ |
| `codeAction/resolve` | ❌ |
| `codeLens/resolve` | ❌ |
| `textDocument/rangeFormatting` | ❌ |
| Call hierarchy (`prepareCallHierarchy`, `incomingCalls`, `outgoingCalls`) | ❌ |
| `textDocument/documentLink` | ❌ |
| `documentLink/resolve` | ❌ |
| `textDocument/documentColor` | ❌ |
| `textDocument/colorPresentation` | ❌ |
| `textDocument/selectionRange` | ❌ |
| `textDocument/inlineCompletion` | ❌ |
| `textDocument/moniker` | ❌ |

*23 of ~50 spec methods are implemented. Coverage is 100% of what's shipped.*
