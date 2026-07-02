# TauWriter Project Status

## Goals

Build an industrial-grade LSP and Zed extension for the TauWriter ecosystem, enabling seamless cross-referencing between prose (TWXML) and structured knowledge (HubGS).

---

## Documentation

### Design Documents
| Resource | Status | Notes |
|:---|:---|:---|
| `TauWriterMD/TauWriterDesign.md` | ✅ Up to Date | Core design spec for the TauWriter ecosystem. |

### Things To Update
- [ ] `README.md` — example TWXML uses deprecated `<metadata>` wrapper; should show `<meta />` under root `<document>`.
- [ ] `README.md` — example HubGS should reflect `EXTENDS` syntax and newer type system features.
- [ ] Status sections for **Extension > Snippets** — to be defined once snippets are implemented.

---

## LSP: TauWriter Language Server

### Engine

| Feature | Status | Description |
|:---|:---:|:---|
| **Project Initialization** | ✅ | Rust workspace, Salsa DB, Zed extension skeleton. |
| **Workspace Indexing** | ✅ | Background crawling and ingestion of all workspace files (`walkdir`). |
| **Semantic Resolution** | ✅ | Cross-language symbol resolution (HubGS ↔ TWXML). |
| **Diagnostics** | ✅ | Real-time error reporting, structural validation, tag checking. |
| **Refactoring Support** | ✅ | Global renames and structural changes (References/Rename). |
| **Dynamic Evaluation** | ✅ | Decorator parsing (`@computed`, `@default`), AST-based expression evaluator (arithmetic, string concatenation, unary ops, parentheses), cross-Hub field access via roles, `@default` override enforcement. |
| **Concurrency & Performance** | ✅ | Lock-free concurrent read queries via database cloning; standard Mutex writes. |
| **Incremental Parsing** | ✅ | Cached tree-sitter parse trees; `InputEdit` coordinate translation; tree-sitter incremental parse on typing. |

### DB (Salsa)

| Module | Status | Notes |
|:---|:---:|:---|
| **`db/mod.rs`** — Workspace, SourceFile | ✅ | Core Salsa database traits and workspace state management. |
| **`db/types.rs`** — HubGS types (Hubs, Fields, Enums, Structs) | ✅ | Full index support for HubGS FIELDS, ENUMS, STRUCTS. |
| **`db/resolution.rs`** — Reference/Type resolution | ✅ | Cross-file symbol and type resolution logic. |
| **`db/validation.rs`** — Structural & semantic validation | ✅ | TWXML nesting rules, HubGS type/multiplicity enforcement, instance resolution. |
| **`db/evaluator.rs`** — `@computed` evaluation engine | ✅ | AST-based evaluation with collection operators (`len()`, `map()`, `join()`), arrow functions, and `@default` override. |

### Parser

| Module | Status | Notes |
|:---|:---:|:---|
| **`parser/mod.rs`** — Tree-sitter integration | ✅ | `build.rs` compiles C parsers; linked via FFI. |
| **`parser/twxml.rs`** — TWXML parsing | ✅ | Generated C parser + Rust glue for XML structure. |
| **`parser/hubgs.rs`** — HubGS parsing | ✅ | Generated C parser + Rust glue for DSL syntax. |
| **`parser/features.rs`** — AST feature extraction | ✅ | Extracts Hubs, Fields, Instances, Imports from parsed trees. |

### Formatter (`lsp/src/formatter/`)

| Feature | Status | Notes |
|:---|:---:|:---|
| **TWXML Formatting** | ✅ | 26 block tags known; 2-space indentation; block-level children indent at `+1`; self-closing tags respect indentation; inline content stays compact. |
| **HubGS Formatting** | ✅ | Full support via `formatter/hubgs.rs`, including chained method calls and arrow functions. |
| **`textDocument/formatting` handler** | ✅ | JSON-RPC tested and wired up. |
| **TWXML `<meta />` formatting** | ✅ | `<meta>` handled as `SelfClosingBlock` in `tag_behavior()`; listed in `VALID_TWXML_TAGS`. |

#### Current Focus: Formatter
- [x] Add HubGS formatting support for chained method calls and arrow functions (EXTENDS block is implemented).
- TWXML formatter fully updated for post-`<metadata>`-deprecation structure. All tests pass.

### JSON-RPC API

| LSP Operation | Implementation | Test Coverage |
|:---|:---:|:---:|
| `initialize` | ✅ | ✅ |
| `initialized` | ✅ | ✅ |
| `shutdown` | ✅ | ✅ |
| `exit` | ✅ (Native) | — |
| `$progress` | ✅ | ✅ |
| `workspace/didChangeConfiguration` | ✅ | — |
| `workspace/executeCommand` | ✅ | — |
| `workspace/didCreateFiles` | ✅ | — |
| `workspace/didRenameFiles` | ✅ | — |
| `workspace/didDeleteFiles` | ✅ | — |
| `textDocument/didOpen` | ✅ | ✅ |
| `textDocument/didChange` | ✅ | ✅ |
| `textDocument/didClose` | ✅ | ✅ |
| `textDocument/didSave` | ⚠️ Stub | ✅ |
| `textDocument/willSave` | ✅ (Stub) | — |
| `textDocument/willSaveWaitUntil` | ✅ (Stub) | — |
| `textDocument/declaration` | ✅ | ✅ |
| `textDocument/definition` | ✅ | ✅ |
| `textDocument/typeDefinition` | ✅ | ✅ |
| `textDocument/implementation` | ✅ | ✅ |
| `textDocument/references` | ✅ | ✅ |
| `textDocument/hover` | ✅ | ✅ |
| `textDocument/completion` | ✅ | ✅ |
| `textDocument/signatureHelp` | ✅ | ✅ |
| `textDocument/rename` | ✅ | ✅ |
| `textDocument/formatting` | ✅ | ✅ |
| `textDocument/rangeFormatting` | ✅ | ✅ |
| `textDocument/onTypeFormatting` | ✅ | ✅ |
| `textDocument/documentHighlight` | ✅ | ✅ |
| `textDocument/documentSymbol` | ✅ | ✅ |
| `textDocument/foldingRange` | ✅ | ✅ |
| `textDocument/selectionRange` | ✅ | ✅ |
| `textDocument/documentLink` / `resolve` | ✅ | ✅ |
| `textDocument/documentColor` / `presentation` | ✅ | ✅ |
| `textDocument/semanticTokens/full` | ✅ | ✅ |
| `prepareCallHierarchy` / `incomingCalls` / `outgoingCalls` | ✅ | ✅ |
| `workspace/symbol` | ✅ | ✅ |
| `textDocument/publishDiagnostics` | ✅ | ✅ |
| `textDocument/codeAction` | ✅ | ✅ |
| `textDocument/inlayHint` | ✅ | ✅ |
| `textDocument/codeLens` | ✅ | ✅ |
| `textDocument/moniker` | ✅ (Stub) | — |

**Summary:** 42 spec methods implemented. **100% of non-stub implementations have test coverage.**

#### Implemented — Handler Files
| Handler File | Operations Covered |
|:---|:---|
| `handlers/navigation.rs` | definition, typeDefinition, declaration, implementation |
| `handlers/symbols.rs` | workspaceSymbol, documentSymbol, references, prepareCallHierarchy, incomingCalls, outgoingCalls |
| `handlers/completion.rs` | completion (context-aware for Hub IDs, fields, roles) |
| `handlers/information.rs` | hover (Hub details with documentation) |
| `handlers/features.rs` | semanticTokens/full, codeAction, inlayHint, documentColor, colorPresentation, documentLink, rangeFormatting, signatureHelp, selectionRange |
| `handlers/code_lens.rs` | codeLens (inline "X references" hints above Hub instances) |
| `handlers/inlay_hints.rs` | inlayHints (`: TypeName` type annotations) — implemented, tested in `lsp_jsonrpc_tests.rs` |
| `handlers/documents.rs` | didOpen, didChange, didClose, didSave (stub), formatting, onTypeFormatting, foldingRange, didCreateFiles, didRenameFiles, didDeleteFiles |

#### Unimplemented LSP Methods (No Test)
| LSP Operation | Status |
|:---|:---:|
| `workspace/configuration` | ❌ |
| `codeAction/resolve` | ❌ |
| `codeLens/resolve` | ❌ |
| `textDocument/inlineCompletion` | ❌ |

#### Things To Add (Editor Experience)
- [x] **Signature Help** — parameter and field hints for HubGS definitions.
- [x] **Range Formatting** — `textDocument/rangeFormatting` handler.
- [x] **TWXML Tag Auto-closing** — update `onTypeFormatting` to stop inserting `</metadata>`, suggest `<meta />` at root.
- [x] **Call Hierarchy** — navigation for Hub → instances → cross-references chains.
- [x] **Document Links** — clickable `<hubref>` and IMPORTS paths.
- [x] **Broken Link Diagnostics** — validation check for unresolvable target files and anchors.
- [x] **Selection Range** — AST-node structure based selection expansion.

---

## Extension

### Languages

#### TWXML

##### Metadata (`languages/twxml/config.toml`)
| Property | Status | Value / Notes |
|:---|:---:|:---|
| `name` | ✅ | `"TWXML"` |
| `grammar` | ✅ | `"twxml"` |
| `path_suffixes` | ✅ | `["twxml"]` |
| `line_comments` | ✅ | `[]` (none) |
| `tab_size` | ⬜ Not set | Default (4 per Zed convention) |
| `hard_tabs` | ⬜ Not set | Default (false) |
| `first_line_patterns` | ⬜ Not set | No regex patterns for file type detection beyond suffix |
| `debuggers` | ❌ Not implemented | — |
| `injections` | ⚠️ Planned | Requires `queries/twxml/injections.scm` targeting `<codeblock>` elements (grammar already exposes `(tag_name)`, `attribute → (attribute_name)/(attribute_value)`, and `(text)` as injectable content).

##### Grammar
| Property | Status | Value |
|:---|:---:|:---|
| `repository` (extension.toml) | ✅ | `"https://github.com/Jarbear82/TauWriter"` |
| `rev` (extension.toml) | ✅ | `"main"` |
| `path` (extension.toml) | ✅ | `"extension/languages/twxml"` |
##### Grammar Structure
| Rule | Status | Notes |
|:---|:---:|:---|
| `source_file` → `document_block` | ✅ | Root node |
| `meta_tag` (self-closing) | ✅ | Replaces deprecated `<metadata>` wrapper; optional, zero or more |
| `body_block` (required) | ✅ | Exactly one required between `<document>` and `</document>` |
| C parser (`parser.c`) | ✅ | ABI 14, regenerated via `tree-sitter generate` |

##### Queries

###### Syntax Highlighting (`highlights.scm`)
| Capture | Status | Target |
|:---|:---:|:---|
| `<document>`, `</document>` | ✅ | `@keyword.control` (structural boundaries) |
| `<body>`, `</body>` | ✅ | `@keyword.control` |
| `<meta />` | ✅ | `@keyword.control` (new format, replaces deprecated <metadata>) |
| `<metadata>`, `</metadata>` | ⬜ Removed | Legacy wrapper — no longer in grammar or highlights
| `(tag_name)` | ✅ | `@tag` (generic elements) |
| `(attribute_name)` | ✅ | `@attribute` |
| `(attribute_value)` | ✅ | `@string` |
| `["<" ">" "</" "/>"]` | ✅ | `@punctuation.bracket` |
| `(comment)` | ✅ | `@comment` |
| `(tag_name) "section"` | ✅ | `@keyword.control` |
| `(tag_name) "hubref"` | ✅ | `@keyword` |
| `(tag_name) "bold"` | ✅ | `@markup.bold` |
| `(tag_name) "italic"` | ✅ | `@markup.italic` |
| `(tag_name) "review"` | ✅ | `@keyword.exception` |

**Fallback captures:** No custom fallback captures defined. Uses Zed default language fallback.

###### Bracket Matching
| Capture | Status | Notes |
|:---|:---:|:---|
| `["<" ">" "</" "/>"]` → `@punctuation.bracket` | ✅ Implemented | Dedicated `brackets.scm` file exists. |

###### Code Outline (`outlines.scm`)
| Capture | Status | Notes |
|:---|:---:|:---|
| — | ✅ Implemented | Supported via `outlines.scm`. |

###### Auto-indentation (`indents.scm`)
| Capture | Status | Notes |
|:---|:---:|:---|
| — | ✅ Implemented | Supported via `indents.scm`. |

###### Code Injections
| Status | Notes |
|:---|:---|
| ✅ Implemented | Targets `<codeblock>` elements with a `language` attribute via `injections.scm`. |

**Notes:**
- Zed extracts the string from `@injection.language` and loads the matching sub-parser.
- This enables embedding Rust, Python, JSON, etc. inside TWXML prose with dynamic highlighting.

###### Syntax Overrides (`overrides.scm`)
| Property | Status | Notes |
|:---|:---:|:---|
| `overriden_scope` | ❌ Not implemented | No overrides file exists. |

**Note:** Metadata properties like `tab_size`, `hard_tabs`, `first_line_patterns` are defined in `config.toml` above, not here. Config-driven overrides (scope mapping, etc.) should also be listed under config.toml metadata when relevant.

###### Text Objects (`textobjects.scm`)
| Capture | Status | Notes |
|:---|:---:|:---|
| — | ✅ Implemented | Supported via `textobjects.scm` for vim-mode navigation. |

###### Text Redactions (`redactions.scm`)
| Status | Notes |
|:---|:---|
| ⬜ Not needed | — |

###### Runnable Code Detection (`runnables.scm`)
| Status | Notes |
|:---|:---|
| ⬜ Not needed | TWXML is a prose format, not executable code. |

##### Language Server & Semantic Tokens
| Item | Status | Notes |
|:---|:---:|:---|
| Language server (`tauwriter-lsp`) in `extension.toml` | ✅ | Registered under `[language_servers.tauwriter-lsp]` |
| Syntax highlighting via semantic tokens | ✅ (combined) | Semantic tokens registered for both languages in LSP; TWXML uses `LEGEND_TYPE` = [CLASS, PROPERTY, VARIABLE, ENUM]. Highlighted via "combined" approach. |

#### HubGS

##### Metadata (`languages/hubgs/config.toml`)
| Property | Status | Value / Notes |
|:---|:---:|:---|
| `name` | ✅ | `"HubGS"` |
| `grammar` | ✅ | `"hubgs"` |
| `path_suffixes` | ✅ | `["hubgs"]` |
| `line_comments` | ✅ | `["// "]` (C++-style) |
| `tab_size` | ⬜ Not set | Default (4 per Zed convention) |
| `hard_tabs` | ⬜ Not set | Default (false) |
| `first_line_patterns` | ⬜ Not set | No regex patterns for file type detection beyond suffix |
| `debuggers` | ❌ Not implemented | — |

##### Grammar
| Property | Status | Value |
|:---|:---:|:---|
| `repository` (extension.toml) | ✅ | `"https://github.com/Jarbear82/TauWriter"` |
| `rev` (extension.toml) | ✅ | `"main"` |
| `path` (extension.toml) | ✅ | `"extension/languages/hubgs"` |

##### Queries

###### Syntax Highlighting (`highlights.scm`)
| Capture | Status | Target |
|:---|:---:|:---|
| `IMPORTS`, `FROM`, `DEFINITIONS` | ✅ | `@keyword` |
| `FIELDS`, `ENUMS`, `STRUCTS`, `HUBS` | ✅ | `@keyword` |
| `ALLOWS`, `INSTANCES` | ✅ | `@keyword` |
| `[]`, `{}`, `()`, `:` | ✅ | `@punctuation.bracket` / `@punctuation.delimiter` |
| `=`, `->`, `<-`, `<->`, `&&`, `\|\|`, `==`, `!=`, `+`, `*`, `/`, `!`, `..` | ✅ | `@operator` |
| `(string)`, `(template_string)` | ✅ | `@string` |
| `(number)` | ✅ | `@number` |
| `(boolean)` | ✅ | `@constant.builtin.boolean` |
| `(comment)` | ✅ | `@comment @spell` |
| `(identifier)` | ✅ | `@variable` |
| `instance_block ref: (identifier)` | ✅ | `@variable.member` |
| `instance_block type: (identifier)` | ✅ | `@type` |
| `field_definition (identifier)` | ✅ | `@variable.field` |
| `enum_definition (identifier)` | ✅ | `@type` |
| `struct_definition (identifier)` | ✅ | `@type` |
| `hub_definition (identifier)` | ✅ | `@type` |
| `generic_type (identifier)` | ✅ | `@type` |
| `(decorator) "@computed" \| "@default"` | ✅ | `@function.builtin` |
| `"@metadata"` | ✅ | `@keyword.directive` |

**Fallback captures:** No custom fallback captures defined. Uses Zed default language fallback.

###### Bracket Matching
| Capture | Status | Notes |
|:---|:---:|:---|
| `[]`, `{}`, `()` → `@punctuation.bracket` | ✅ Implemented | Dedicated `brackets.scm` file exists.

###### Code Outline (`outlines.scm`)
| Capture | Status | Notes |
|:---|:---:|:---|
| — | ✅ Implemented | Supported via `outlines.scm`.

###### Auto-indentation (`indents.scm`)
| Capture | Status | Notes |
|:---|:---:|:---|
| — | ✅ Implemented | Supported via `indents.scm`.

###### Code Injections
| Status | Notes |
|:---|:---|
| ⬜ Not needed | — |

###### Syntax Overrides (`overrides.scm`)
| Property | Status | Notes |
|:---|:---:|:---|
| `overriden_scope` | ❌ Not implemented | No overrides file exists. |

**Note:** Metadata properties like `tab_size`, `hard_tabs`, `first_line_patterns` are defined in `config.toml` above, not here. Config-driven overrides should also be listed under config.toml metadata when relevant.

###### Text Objects (`textobjects.scm`)
| Capture | Status | Notes |
|:---|:---:|:---|
| — | ✅ Implemented | Supported via `textobjects.scm` for vim-mode navigation.

###### Text Redactions (`redactions.scm`)
| Status | Notes |
|:---|:---|
| ⬜ Not needed | — |

###### Runnable Code Detection (`runnables.scm`)
| Status | Notes |
|:---|:---|
| ⬜ Not needed | HubGS is a schema DSL, not executable code. |

##### Language Server & Semantic Tokens
| Item | Status | Notes |
|:---|:---:|:---|
| Language server (`tauwriter-lsp`) in `extension.toml` | ✅ | Registered under `[language_servers.tauwriter-lsp]` with languages `["HubGS", "TWXML"]` |
| Syntax highlighting via semantic tokens | ✅ (combined) | Semantic tokens registered for both languages; HubGS gets CLASS, PROPERTY, VARIABLE, ENUM token types + DECLARATION/DEFINITION modifiers. |

---

### Snippets

| Item | Status | Notes |
|:---|:---:|:---|
| Snippet paths in `extension.toml` | ❌ Not configured | No `[snippets]` section defined. |
| Snippet files | ❌ Not implemented | No snippets directory exists yet. |

**TODO:** Add `[snippets]` sections for TWXML and HubGS in `extension.toml`, then create snippet JSON files.

---

### Editor QoL Features (In Progress)

| Feature | Status | Notes |
|:---|:---:|:---|
| **TWXML Tag Auto-completion** | ✅ | Context-aware suggestions for structural tags (`<section>`, `<heading>`, `<body>`). Filters invalid tags by parent context. |
| **CodeLens Integration** | ✅ | Inline hints ("X references") above Hub instances. |
| **Inlay Hints** | ✅ | Type annotations (`: TypeName`) for HubGS instances — implemented, no dedicated test yet. |
| **Code Actions** | ✅ | Quickfix actions to resolve `<review>` tags to `<hubref>`. Two quickfix actions implemented. |
| **Tag Auto-closing** | ✅ | Updated `onTypeFormatting` to prevent auto-closing of `<metadata>`, suggesting `<meta />` at document root instead. |

---

### Inheritance & Extensibility (HubGS) — Planned / In Progress

| Feature | Status | Notes |
|:---|:---:|:---|
| **EXTENDS AST Parsing** | ⚠️ Pending | Update AST extraction to support composite inheritance definitions. |
| **Set-Union Compilation** | ❌ Not started | Child hubs must inherit all FIELDS and roles from EXTENDS parents. |
| **Polymorphism Rules** | ❌ Not started | Child type instances valid for parent `ALLOWS` roles. |
| **Decorator Precedence** | ❌ Not started | Child can override parent `@default()`, but not `@computed()`. |

---

### Graph Capabilities — Planned

| Feature | Status | Notes |
|:---|:---:|:---|
| **Role Metadata** | ❌ Not started | Weighted edges, edge-properties (e.g., temporal bounds on roles). |
| **Expanded Data Types** | ❌ Not started | `UUID`, structured records (`struct`), string templates. |
| **Circular Import Resolution** | ❌ Not started | Two-pass merge strategy for cyclic `.hubgs` dependencies. |

---

## CI/CD

| Item | Status | Notes |
|:---|:---:|:---|
| **GitHub Actions workflow** | ✅ | `.github/workflows/dev-bundle.yml` — builds LSP on `main` push for Linux x64, macOS ARM64, macOS x64. |
| **Pre-built binaries** | ✅ | Three platforms in `extension/bin/`: `tauwriter-lsp-linux-x64`, `tauwriter-lsp-macos-arm64`, `tauwriter-lsp-macos-x64`. |
| **Zed Marketplace Verification** | ⚠️ Pending | Auto-downloading and one-click install require marketplace submission. |

---

## Current Focus: Structural Enforcement & Validation Pipeline ⚠️

Strict schema enforcement for document and graph structures to ensure data integrity.

- [x] **TWXML Skeleton Enforcement:** Update validation to enforce that all TWXML documents strictly adhere to the root `<document>` schema containing zero or more `<meta/>` tags directly, followed by exactly one `<body>` block (removing the deprecated `<metadata>` wrapper).
- [x] **HubGS Dependency Validation:** Enforce section-level dependencies. If an `INSTANCES` block exists, validate that a `DEFINITIONS` block is present locally or fully satisfied via an `IMPORTS` statement.
- [x] **Instance Resolution:** Ensure all declared instances successfully resolve to a defined Hub type.
- [x] Implement TWXML Nesting Rules (e.g., `<heading>` levels inside `<body>` or `<section>`)
- [x] Implement TWXML Referential Integrity (Unresolved references for `<hubref>`)
- [x] Implement HubGS Type & Multiplicity Enforcement

## Dynamic Evaluation Engine ⚠️

Robust engine for computed graph data.

- [x] Implement AST evaluator for `@computed` formulas (arithmetic, string concatenation).
- [x] Implement cross-Hub field access via roles (e.g., `this.companions.length`).
- [x] Extend AST evaluator to execute collection operators (`.len()`, `.map(expr)`, `.join(delimiter)`) and arrow functions.
- [x] Enforce `@default` override rules during instance instantiation.

## Things To Remove

- Deprecated `<metadata>` wrapper support — migration path from old TWXML schema.
- Any lingering references to legacy formatting (`format_metadata_block`) in code and docs.

## Things To Clarify

- The distinction between the two "Languages" sections (TWXML vs HubGS) — should both languages share a common grammar config property set?
- Whether `tab_size` / `hard_tabs` defaults are sufficient or need explicit declaration per-language in `config.toml`.

---

*42 of ~50 LSP spec methods are implemented. Coverage is 100% of non-stub implementations.*
