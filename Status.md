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
| **Dynamic Evaluation**| 🔄 | Prototype evaluation for `@computed` and `@default` decorator expressions. |

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
| Testing Suite | ✅ | Comprehensive integration tests for LSP handlers and Salsa queries. |
| **CI/CD** | ✅ | Automated GitHub Actions for multi-platform binary distribution. |
| **Zed Extension** | ✅ | Functional Rust bridge with "one-click install" and auto-downloading. |

## Current Focus
- [ ] Implement full expression evaluation engine for `@computed` fields (math, string ops, cross-hub references).
- [ ] Add diagnostic validation for Enum assignments and `Array<Type>` structural verification.
- [ ] Explore document fragmentation/stitching for multi-file TWXML documents.
- [ ] Lay groundwork for Traits (`EXTENDS` syntax) as defined in the vision document.
- [ ] Implement advanced UI Views logic (e.g., Graph visualization data exports for the client).

## JSON-RPC Testing Progress

| LSP Operation | Implementation Status | JSON-RPC Test |
|:---|:---:|:---:|
| `initialize` | ✅ | ✅ |
| `initialized` | ✅ | ✅ |
| `shutdown` | ✅ | ✅ |
| `textDocument/didOpen` | ✅ | ✅ |
| `textDocument/didChange` | ✅ | ✅ |
| `textDocument/didClose` | ✅ | ✅ |
| `textDocument/didSave` | ✅ | ✅ |
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
