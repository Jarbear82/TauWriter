# Core Architectural Fixes — Design Document

## Preface: Cross-Cutting Dependencies

Task A (Backend consolidation) and Task D (Tree-sitter upgrade) touch overlapping files (`lib.rs`, `parser/mod.rs`). **Order matters:** Complete Task A first, then Task C, then Task D. This avoids the complexity of merging two large diff sets simultaneously.

Task B (Pull Diagnostics) is independent — it can be done at any point, but should follow Task A because it depends on the cleaned-up Backend API.

**Recommended execution order: A → B → C → D**

---

## [Task A] Consolidate Backend + Fix Salsa Thread Safety

**Strategy**: Top-Down — This task removes a duplicate subsystem (server.rs) and restructures the core Backend struct. The change originates from the top-level type system (Backend, RootDatabase) and cascades downward to all handler call sites.

### 1. Encapsulation

**Structs Modified:**
- `Backend` (lib.rs:42–47): Change field `db: Arc<std::sync::Mutex<RootDatabase>>` → `db: SalsaDatabaseHandle`. A new type alias or opaque wrapper that holds the raw pointer/value but exposes only `.attach()` and `.clone()`.
- `CachedTree` (lib.rs:52–57): **Deleted** — redundant with Salsa's own caching. Remove alongside TREE_CACHE.

**Structs Deleted:**
- Entire file `lsp/src/server.rs` — dead code confirmed by grep (zero imports across all files).

**Protection:**
- `Backend::db` field becomes `pub(crate)`. Not public for consumers outside the crate.
- `RootDatabase`, `Db` trait, `Workspace`, `SourceFile`: remain `pub` as they are used by handler modules.
- All other items in lib.rs: keep existing visibility.

### 2. Type Relations

**Has-A composition changes:**
- `Backend` no longer directly owns a Mutex-wrapped RootDatabase. Instead holds `Arc<dyn SalsaDatabase>`.
- Remove `CachedTree` field from any struct — it was only stored in the global TREE_CACHE, which is also removed.

**Is-A trait changes:**
- Create `trait SalsaDatabase: salsa::Database + Send + 'static`: a thin abstract layer that provides `.attach()` and `.clone_db()`. RootDatabase implements this trait directly.
- This isolates the thread-safety mechanism from the rest of the codebase.

### 3. Component Architecture

**Pattern: Bridge** (from Pattern Matrix — Section 6, Interface Modification)
- Decouple the Salsa database abstraction from its mutex-enclosure independently.

**Justification:**
The bridge pattern cleanly separates two concerns: (1) what operations the LSP needs from the database (the Db trait's surface), and (2) how thread-safety is provided (currently a Mutex, later could be an Arc + attach without changing any consumer code). The next simplest alternative — direct RootDatabase with ad-hoc `.attach()` scattered across handlers — would violate redundancy elimination by repeating the lock/attach boilerplate at every call site.

**Trade-offs:**
- **Added abstraction layer**: One new trait and one adapter type. Offset by eliminating ~12 locations of `db.lock().unwrap()` boilerplate.
- **Runtime cost**: Trait dispatch is virtual (vtable) vs monomorphic inline. Negligible for LSP I/O-bound workloads where parsing dominates CPU time.

### 4. Artifact

```
lsp/src/
├── lib.rs                      ← PRIMARY: Backend, RootDatabase, SalsaHandle, utilities
├── main.rs                     ← Unchanged (instantiates Backend)
├── server.rs                   ← 🗑️ DELETED (dead code, zero imports)
├── db/
│   ├── mod.rs                  ← Db trait re-exports
│   ├── types.rs                ← RootDatabase, Workspace, SourceFile, tracked structs
│   ├── resolution.rs           ← parse_hubgs, resolve_reference, all_hub_instances etc.
│   ├── validation/             ← hubgs.rs, twxml.rs, mod.rs (unchanged)
│   └── evaluator/              ← (unchanged)
├── parser/
│   ├── mod.rs                  ← get_language(), language getters (preps for ts 0.26)
│   ├── cache.rs                ← 🗑️ DELETED per Task D — but not in this task
│   ├── hubgs.rs                ← parse_hubgs_ast, etc.
│   ├── twxml.rs                ← parse_twxml_ast, etc.
│   └── features/               ← (unchanged)
├── handlers/
│   ├── mod.rs                  ← re-exports (unchanged)
│   ├── navigation.rs           ← callers of server.db.lock() → server.attach_db()
│   ├── completion.rs           ← same pattern
│   ├── information.rs          ← same pattern
│   ├── documents.rs            ← heavy consumer: did_open, did_change, did_save etc.
│   └── ...                     ← all handler files touch .lock() → must be updated
├── formatter/                  ← (unchanged)
└── main.rs                     ← unchanged
```

### 5. Detailed Implementation Notes

#### Step A-0: Create the SalsaThreadHandle type in lib.rs

Add after line 47 (after Backend def):

```rust
/// Thread-safe handle for the salsa database.
/// Provides `.attach()` to enter a salsa query context and `.clone_db()` to duplicate.
pub struct SalsaThreadHandle(Arc<Mutex<RootDatabase>>);

impl SalsaThreadHandle {
    /// Enter a salsa-attached context for the duration of the closure.
    /// All salsa tracked calls must occur within this scope.
    pub fn with_db<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&RootDatabase) -> R,
    {
        let guard = self.0.lock().unwrap();
        // RootDatabase::default() already impls salsa::Database via #[salsa::db]
        // attach requires &dyn salsa::Database — we can pass the wrapped reference
        f(&*guard)  // RootDatabase coerces to RootDatabase, which implements salsa::Database
    }

    pub fn clone_db(&self) -> SalsaThreadHandle {
        SalsaThreadHandle(self.0.clone())
    }
}

impl Clone for SalsaThreadHandle {
    fn clone(&self) -> Self {
        self.clone_db()
    }
}
```

#### Step A-1: Update Backend struct (lib.rs:42–47)

**Before:**
```rust
pub struct Backend {
    pub client: Client,
    pub db: Arc<std::sync::Mutex<RootDatabase>>,
    pub workspace_input: db::Workspace,
    pub open_files: Arc<DashMap<Url, Rope>>,
}
```

**After:**
```rust
pub struct Backend {
    pub(crate) client: Client,          // Restrict to crate-only (LSP trait needs it but tower-lsp requires Clone + Send)
    pub(crate) db: SalsaThreadHandle,
    pub workspace_input: db::Workspace,
    pub open_files: Arc<DashMap<Url, Rope>>,
}
```

Note: `tower_lsp::LanguageServer` may require `client` to be pub. If so, keep it as-is. The key change is the `db` field type.

#### Step A-2: Update main.rs (instantiation)

**Before (main.rs:10–17):**
```rust
let db = RootDatabase::default();
let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());
let (service, socket) = LspService::new(|client| Backend {
    client,
    db,                          // ← Arc<Mutex<RootDatabase>> inferred
    workspace_input,
    open_files: Arc::new(DashMap::new()),
});
```

**After:**
```rust
let db = RootDatabase::default();
let workspace_input = tauwriter_lsp::db::Workspace::new(&mut db, Vec::new());
let salsa_handle = SalsaThreadHandle(Arc::new(Mutex::new(db)));
let (service, socket) = LspService::new(|client| Backend {
    client,
    db: salsa_handle.clone(),
    workspace_input,
    open_files: Arc::new(DashMap::new()),
});
```

#### Step A-3: Update read_db() (lib.rs:183–186)

**Before:**
```rust
pub fn read_db(&self) -> (RootDatabase, db::Workspace) {
    let db = self.db.lock().unwrap();
    (db.clone(), self.workspace_input)
}
```

**After:**
```rust
/// Return a new Salsa handle and the workspace input.
/// Use this when handlers need to call salsa tracked queries.
pub fn db_handle(&self) -> SalsaThreadHandle {
    self.db.clone()
}

/// Read the current database snapshot for inspection.
/// WARNING: The returned RootDatabase is NOT attached to a task context.
/// Do not call salsa tracked queries on it — use `db_handle()` instead.
pub fn peek_db(&self) -> &RootDatabase {
    self.db.0.lock().unwrap()
}
```

Rename to avoid confusion. The old name `read_db` implied returning an attached DB which was the bug.

#### Step A-4: Update index_directory() (lib.rs:188–263)

This is in a `tokio::spawn` closure — the mutex lock held across await points is dangerous. With SalsaThreadHandle:

**Before (lines 220–236):**
```rust
{
    let mut db = db_mutex.lock().unwrap();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str());
            if matches!(ext, Some("hubgs") | Some("twxml")) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let path_str = path.to_string_lossy().to_string();
                    let source = db::SourceFile::new(&mut *db, path_str, content);
                    files.push(source);
                }
            }
        }
    }
    ws.set_files(&mut *db).to(files);
}
```

**After:**
```rust
{
    // Short-lived lock for Salsa input mutation only — never across await
    let source_files = db_handle.with_db(|db| {
        WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|s| s.to_str());
                    if matches!(ext, Some("hubgs") | Some("twxml")) {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let path_str = path.to_string_lossy().to_string();
                            let source = db::SourceFile::new(db, path_str, content);
                            return Some((path_str, source));
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>()
    });

    // Update workspace input outside the db scope but within salsa attach
    db_handle.with_db(|db| {
        let mut files = ws.files(db).clone();
        files.push(source_files.into_iter().map(|(_, s)| s).collect::<Vec<_>>());
        ws.set_files(db).to(files);
    });
}
```

#### Step A-5: Update publish_diagnostics() (lib.rs:265–292)

**Before:**
```rust
let errors = {
    let mut db = self.db.lock().unwrap();
    let source_file = db::SourceFile::new(&mut *db, path, content.clone());
    db::validate_file(&*db, self.workspace_input, source_file)
};
```

**After:**
```rust
let errors = self.db.with_db(|db| {
    let source_file = db::SourceFile::new(db, path, content);
    db::validate_file(db, self.workspace_input, source_file)
});
```

#### Step A-6: Update all handler call sites — `.lock()` → `.with_db()` pattern

Every handler that calls `server.db.lock().unwrap()` must be updated. There are 12 locations across two files:

| File | Function | Line Range | Pattern Change |
|------|----------|------------|----------------|
| `lib.rs` | read_db() | 183–186 | Replace with `db_handle()` (see A-3 above) |
| `lib.rs` | index_directory() | 220–236, 239 | Replace all `lock()` → `with_db(|db| ...)` (see A-4) |
| `lib.rs` | publish_diagnostics() | 273–277 | Replace lock block with `with_db(|db| ...)` (see A-5) |
| `handlers/documents.rs` | did_open() | ~25–35 | `server.db.lock().unwrap()` → `server.db.with_db(...)` |
| `handlers/documents.rs` | did_change() | ~192–202 | Same pattern |
| `handlers/documents.rs` | did_change_watched_files() | ~515–582 | Multiple lock sites — batch all salsa ops in one with_db scope |
| `handlers/documents.rs` | did_create_files() | ~588–617 | Same pattern |
| `handlers/documents.rs` | did_rename_files() | ~624–676 | Same pattern |
| `handlers/documents.rs` | did_delete_files() | ~682–712 | Same pattern |

**Generic conversion template for any lock site:**

```rust
// BEFORE:
let mut db = server.db.lock().unwrap();
let ws = server.workspace_input;
let mut files = ws.files(&*db).clone();
files.push(new_source);
ws.set_files(&mut *db).to(files);

// AFTER:
server.db.with_db(|db| {
    let ws = server.workspace_input;
    let mut files = ws.files(db).clone();
    files.push(new_source);
    ws.set_files(db).to(files);
});
```

Key rule: **one with_db scope per critical section**. Do not nest with_db calls — Salsa panics on nested attach. Collect all mutations and reads needed, then perform them in one closure.

#### Step A-7: Delete server.rs

```bash
rm lsp/src/server.rs
```

Verify no references remain:
```bash
grep -r "server\.rs\|mod server\|use.*server" lsp/src/  # should return nothing
```

#### Step A-8: Update main.rs Backend import

main.rs line 2 imports `Backend` from lib. No changes needed — the struct type is still exported as `Backend`, only its fields changed internally.

---

## [Task B] Pull Diagnostics

**Strategy**: Bottom-Up — Start with the new handler module (the primitive), then wire it into the ServerCapabilities builder and LanguageServer trait impl. The handler logic composes existing Salsa validation queries, so it builds from proven primitives upward.

### 1. Encapsulation

**New Files:**
- `lsp/src/handlers/features/diagnostics.rs` — single public function: `diagnostic_pull_handler`

**New Structs:**
- None needed. Use existing LSP types (`DiagnosticRelatedInformation`, `RelatedFullDocumentDiagnosticReport`) from `lsp-types`.

**Modified Items:**
- `lib.rs`: `initialize` handler — add `diagnostic_provider` to ServerCapabilities builder (~line 310 region)
- `handlers/features/mod.rs`: export `diagnostics` module
- `handlers/mod.rs`: re-export the new handler function
- `lib.rs`: `impl LanguageServer for Backend` — add `async fn diagnostic()` method

### 2. Type Relations

**Has-A:** No new struct composition. The handler is a pure function: `(Backend, DiagnosticPullParams) → Result<Option<DiagnosticReport>>`.

**Is-A:** No trait changes needed. Uses existing `lsp_types::DiagnosticServerCapabilities` and `lsp_types::DiagnosticPullOptions`.

### 3. Component Architecture

**Pattern: Direct Invocation** (from Pattern Matrix — Section 6, Command Passing)
- The LSP client makes a request → Tower routes to our handler fn → direct call returns response.

**Justification:** Pull diagnostics is a straightforward request/response pattern with no intermediate coordination needed. No mediator, observer, or command queue is required.

**Trade-offs:**
- Each pull diagnostic request recomputes validation. For large workspaces this could be expensive. Mitigated by Salsa's incremental caching (only changed files revalidate).
- Does not support push diagnostics (the alternative would use `textDocument/publishDiagnostics` which already exists in the codebase via `publish_diagnostics()`).

### 4. Artifact

```
lsp/src/
├── lib.rs                          ← add diagnostic_provider to ServerCapabilities, add method stub
├── handlers/
│   ├── mod.rs                      ← re-export diagnostics
│   └── features/
│       ├── mod.rs                  ← add pub mod diagnostics
│       └── diagnostics.rs          ← 🆕 NEW: pull diagnostics handler
```

### 5. Detailed Implementation Notes

#### Step B-0: Create the handler file

Create `lsp/src/handlers/features/diagnostics.rs`:

```rust
use lsp_types::{
    Diagnostic, DiagnosticPullOptions, DiagnosticRelatedInformation, DiagnosticServerCapabilities,
    DiagnosticReport, Location, RelatedFullDocumentDiagnosticReport, WorkspaceDiagnosticReport,
    WorkspaceDiagnosticReportKind,
};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::DiagnosticRequest;
use std::sync::OnceLock;

use crate::{Backend, db::ValidationError};

/// Handler for textDocument/diagnostic (LSP 3.17 pull diagnostics).
pub async fn diagnostic_pull_handler(
    server: &Backend,
    params: lsp_types::DiagnosticParams,
) -> Result<Option<WorkspaceDiagnosticReport>> {
    let uri = params.text_document.uri;

    // 1. Get the document content — either from open files or by reading from disk
    let content = match server.open_files.get(&uri) {
        Some(rope) => rope.to_string(),
        None => {
            // File not open — read from file system
            if let Ok(path_buf) = uri.to_file_path() {
                std::fs::read_to_string(&path_buf).map_err(|_| {
                    tower_lsp::jsonrpc::Error::new(tower_lms::jsonrpc::code::INTERNAL_ERROR.into(), "Failed to read file from disk".to_string())
                })?
            } else {
                // Non-file URI — return nothing (only support file:// URIs)
                return Ok(None);
            }
        }
    };

    // 2. Run validation through Salsa
    let path = uri.to_file_path().map(|p| p.to_string_lossy().to_string());
    let errors = server.db.with_db(|db| {
        let source_file = db::SourceFile::new(db, path.clone().unwrap_or_default(), content.clone());
        db::validate_file(db, server.workspace_input, source_file)
    });

    // 3. Convert ValidationError → LSP Diagnostic
    let diagnostics: Vec<Diagnostic> = errors
        .into_iter()
        .map(|err| Diagnostic {
            range: err.range.into(),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            source: Some("tauwriter".to_string()),
            message: err.message,
            related_information: None,
            ..Default::default()
        })
        .collect();

    // 4. Return full document diagnostic report
    Ok(Some(WorkspaceDiagnosticReport {
        items: vec![lsp_types::WorkspaceFileDiagnostic {
            uri,
            version: None,
            items: lsp_types::File DiagnosticReport {
                kind: WorkspaceDiagnosticReportKind::Full,
                items: diagnostics,
                next_change: None,
            },
        }],
    }))
}
```

Wait — there's a type mismatch. Let me correct the types using `lsp-types 0.94` API which maps to LSP 3.17:

```rust
use lsp_types::{
    Diagnostic, DiagnosticServerCapabilities, DiagnosticPullOptions, WorkspaceDiagnosticReport,
};
use tower_lsp::jsonrpc::Result;

use crate::Backend;

/// Handler for textDocument/diagnostic (LSP 3.17 pull diagnostics).
pub async fn diagnostic_pull_handler(
    server: &Backend,
    params: lsp_types::DiagnosticParams,
) -> Result<Option<WorkspaceDiagnosticReport>> {
    let uri = params.text_document.uri;

    let content = if let Some(rope) = server.open_files.get(&uri) {
        rope.to_string()
    } else {
        // File not open — read from disk
        match uri.to_file_path() {
            Ok(path_buf) => std::fs::read_to_string(&path_buf).unwrap_or_default(),
            Err(_) => return Ok(None), // Non-file URI, nothing to do
        }
    };

    let path_str = uri.to_file_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let errors = server.db.with_db(|db| {
        let source_file = crate::db::SourceFile::new(db, path_str.clone(), content);
        crate::db::validate_file(db, server.workspace_input, source_file)
    });

    let diagnostics: Vec<Diagnostic> = errors
        .into_iter()
        .map(|err| Diagnostic {
            range: err.range.into(),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            source: Some("tauwriter".to_string()),
            message: err.message,
            related_information: None,
            ..Default::default()
        })
        .collect();

    Ok(Some(WorkspaceDiagnosticReport {
        items: vec![lsp_types::WorkspaceFileDiagnostic {
            uri,
            version: None,
            items: lsp_types::FileDiagnosticReport {
                kind: lsp_types::WorkspaceDiagnosticReportKind::Full,
                items: diagnostics,
                next_change: None,
            },
        }],
    }))
}
```

**Error handling decisions:**
- **File not open**: Attempt to read from file system. If that fails (non-file URI), return `Ok(None)` — the LSP spec allows this for unsupported URIs.
- **Validation fails**: Return empty diagnostics list (`Vec::new()`), which signals "no errors" to the client rather than an error response. This is correct per the LSP 3.17 spec.

#### Step B-1: Wire into features/mod.rs

Add to `lsp/src/handlers/features/mod.rs`:

```rust
pub mod diagnostics;
// ... existing modules ...
pub use diagnostics::*;
```

#### Step B-2: Wire into handlers/mod.rs (optional — already re-exported via features)

No change needed — the pub use cascade flows through.

#### Step B-3: Add diagnostic_provider to ServerCapabilities in initialize()

In `lib.rs` ~line 310, inside `ServerCapabilities { ... }`, add BEFORE `..Default::default()`:

```rust
// Add after line 395 (selection_range_provider), before workspace:
diagnostic_provider: Some(DiagnosticServerCapabilities::PullOptions(
    DiagnosticPullOptions {
        work_done_progress_options: lsp_types::WorkDoneProgressOptions {
            work_done_progress: None,
        },
        interval: None, // Pull — no polling interval needed; client requests on-demand
    }
)),
```

#### Step B-4: Add the method to LanguageServer impl

Add to `impl LanguageServer for Backend` in lib.rs (~line 658 area), after `moniker`:

```rust
async fn diagnostic(&self, params: lsp_types::DiagnosticParams) -> Result<Option<WorkspaceDiagnosticReport>> {
    handlers::diagnostics::diagnostic_pull_handler(self, params).await
}
```

Note: The method name must match the trait definition in tower-lsp 0.20. Verify that `tower_lsp::LanguageServer` has a `diagnostic` method signature for LSP 3.17 support. If it doesn't (some versions of tower-lsp lag behind the spec), you may need to add the method via a manual request handler registration. In most cases with tower-lsp 0.20, the trait includes this method since LSP 3.17 was finalized.

---

## [Task C] Indexed Lookup Query

**Strategy**: Bottom-Up — The index is a new primitive query that existing queries can compose. Build it first (the proven primitive), then modify downstream consumers (`resolve_reference`, optionally `find_all_references`) to use it.

### 1. Encapsulation

**New Structs:**
- None. Use `HashMap<String, Vec<HubInstance<'db>>>` — the value is a Vec because multiple instances can share the same name across files (duplicate names).

**Modified Tracked Functions:**
- `build_instance_index()` — **new**: computes once per workspace revision, returns HashMap indexed by instance name
- `resolve_reference()` — **modified**: uses index instead of linear scan via `all_hub_instances()`
- Optionally `find_all_references()` — see analysis below

**Unchanged:**
- `all_hub_instances()`, `all_hub_types()`, `all_global_fields()`, `all_enums()`, `all_structs()` — these remain as general-purpose queries for other consumers. The index is an *additional* optimization, not a replacement.

### 2. Type Relations

**Has-A:** No new composition fields. The index is computed on-demand and memoized by Salsa.

**Is-A:** No trait changes. The index query returns a plain `HashMap` — no traits needed for this data type.

### 3. Component Architecture

**Pattern: Prototype (caching) + Direct Invocation** (from Pattern Matrix, Section 6)
- `build_instance_index()` is memoized by Salsa (prototype pattern: expensive computation cached and reused).
- `resolve_reference()` directly invokes the index HashMap for O(1) lookup.

**Justification:**
Salsa's built-in memoization makes this a natural fit — no need for custom caching infrastructure. The `HashMap` value (Vec of instances per name) handles duplicate names correctly: `resolve_reference` picks the first match, and `find_all_references` would iterate all matches.

**Trade-offs:**
- **Memory**: Index stores a copy of the instance name HashMap. For 50+ files with hundreds of instances, this is negligible (<1 MB).
- **Invalidation**: Index automatically invalidates when any workspace file changes (Salsa tracks `Workspace` input — when `files` setter fires, the index recomputes). This is correct but means re-indexing happens on every file save. Acceptable because indexing is O(n) and Salsa makes downstream queries incremental.

### 4. Artifact

```
lsp/src/db/
├── resolution.rs                          ← modified: add build_instance_index, update resolve_reference
│                                           │ (new fn at top, modify existing fn below)
├── types.rs                               ← unchanged
├── validation/                            ← unchanged
└── evaluator/                             ← unchanged

lsp/src/handlers/
├── navigation.rs                          ← uses resolve_reference() — transparent upgrade, no change needed
├── completion.rs                          ← may use all_hub_instances() in some paths — still works
├── information.rs                         ← unchanged
└── ...                                    ← unchanged (other handlers unaffected)
```

### 5. Detailed Implementation Notes

#### Step C-0: Add build_instance_index() to resolution.rs

Add **after** the `all_hub_instances()` function (~line 56):

```rust
/// Build a HashMap index of all hub instances, keyed by instance name.
/// Memoized by Salsa — recomputes when workspace.files changes.
/// Value is Vec because duplicate names across files are possible.
#[salsa::tracked]
pub fn build_instance_index(
    db: &dyn Db,
    workspace: Workspace,
) -> HashMap<String, Vec<HubInstance<'>>> {
    let mut index = HashMap::new();
    for file in workspace.files(db) {
        if file.path(db).ends_with(".hubgs") {
            let result = parse_hubgs(db, file);
            for instance in result.instances(db).iter() {
                let name = instance.name(db).to_string();
                index.entry(name).or_default().push(instance.clone());
            }
        }
    }
    index
}
```

**Key design points:**
- Value is `Vec<HubInstance>` not a single `HubInstance` because duplicate names are possible across files.
- Uses `HashMap::entry()` API for ergonomic insertion.
- Salsa tracks the return type `HashMap<String, Vec<HubInstance>>`. Ensure all items in the HashMap implement Hash+EQ (they do — HubInstance is stored via salsa tracked pointer).

#### Step C-1: Modify resolve_reference() to use the index

**Before (resolution.rs:102–111):**
```rust
#[salsa::tracked]
pub fn resolve_reference(
    db: &dyn Db,
    workspace: Workspace,
    name: String,
) -> Option<HubInstance<'_>> {
    all_hub_instances(db, workspace)
        .into_iter()
        .find(|i| i.name(db) == name)
}
```

**After:**
```rust
#[salsa::tracked]
pub fn resolve_reference(
    db: &dyn Db,
    workspace: Workspace,
    name: String,
) -> Option<HubInstance<'_>> {
    let index = build_instance_index(db, workspace);
    index.get(&name).and_then(|instances| instances.first().cloned())
}
```

**Why first():** The spec doesn't define a tie-breaking rule for duplicate instance names. Pick the first one consistently (insertion order from WalkDir). If this becomes problematic in practice, add an additional disambiguator by file path later.

#### Step C-2: Consider find_all_references() optimization

The current `find_all_references()` scans ALL files for references to a given name (for TWXML hubref lookups and HubGS identifier references within assignments). This is different from `resolve_reference` which resolves a name → definition.

**For find_all_references():** The index does NOT help here because:
1. It indexes *definitions* (HubInstance by name), not *references* (where that instance is referenced in other files)
2. `find_all_references` searches across TWXML files for `hubref` tags and HubGS assignment values — this requires parsing content, not looking up names

**Verdict: Do NOT optimize find_all_references() with an index.** The complexity of a reverse-index (name → list of locations where it's referenced) is not justified. If profiling shows this is a bottleneck, a separate `build_reference_index()` could be added later. Keep the implementation unchanged for now.

#### Step C-3: Update all_hub_instances() callers

Some handlers call `all_hub_instances()` directly (not through `resolve_reference`). These include:
- `goto_implementation` in navigation.rs — finds all instances of a given type

These callers continue to work because `all_hub_instances()` is unchanged. No modifications needed to callers.

#### Step C-4: Add HashMap import to resolution.rs

Add to the top of `lsp/src/db/resolution.rs`:
```rust
use std::collections::HashMap;
```

---

## [Task D] Tree-sitter 0.26 + TREE_CACHE Removal

**Strategy**: Layered — Changes cascade from the dependency layer (Cargo.toml) → type definitions (lib.rs, parser/mod.rs) → call sites (all files using set_language and parse_with_cache). Each layer must compile before the next is touched.

### 1. Encapsulation

**Files Deleted:**
- `lsp/src/parser/cache.rs` — entire file removed
- `CachedTree` struct — deleted from lib.rs
- `TREE_CACHE` static + `get_tree_cache()` fn — deleted from lib.rs

**Structs Modified:**
- None directly. The removal of CachedTree eliminates a redundant type. No replacement needed because Salsa tracks parsed ASTs as `HubgsParseResult<'db>` / `Vec<HubReference<'db>>`.

**Files Created:**
- None.

### 2. Type Relations

**Has-A composition changes:**
- Remove all references to `CachedTree` and `TREE_CACHE` from `lib.rs`
- No new Has-A relationships introduced

**Is-A trait changes:**
- `tree_sitter::Language` type semantics change in API surface — no custom traits affected

### 3. Component Architecture

**Pattern: Façade + Adapter** (from Pattern Matrix, Section 6)
- Tree-sitter upgrade is an Adapter: the grammar getters still provide `Language` objects; only the way they're consumed changes.
- The removal of TREE_CACHE is a Façade simplification: Salsa IS the cache layer now — no intermediate facade needed.

**Justification:**
The tree-sitter API change (`set_language(Language)` → `set_language(&Language)`) is a straightforward type-level adapter. There's only one consumer pattern (create Parser, set language, parse). No complex bridging or decoration needed.

**Trade-offs:**
- **Grammar ABI breakage**: tree-sitter 0.26 may have different grammar versions. The `cc` build dependency must regenerate the parser FFI bindings. This requires re-running `tree-sitter generate` on the grammars if they were generated in-tree, or updating to grammar crates compatible with ts 0.26.
- **Loss of incremental reparsing**: TREE_CACHE's incremental parse feature (`parser.parse(text, Some(&cached_tree))`) is removed. Salsa will re-parse on every file change. Acceptable because Salsa invalidation is already precise — only changed files re-execute downstream queries.

### 4. Artifact

```
lsp/
├── Cargo.toml                              ← modify: tree-sitter "0.20" → "0.26"
└── src/
    ├── lib.rs                              ← delete TREE_CACHE, CachedTree, get_tree_cache; update parser imports
    ├── server.rs                           ← 🗑️ DELETED (Task A)
    ├── main.rs                             ← unchanged
    ├── parser/
    │   ├── mod.rs                          ← update set_language calls: Language → &Language
    │   ├── cache.rs                        ← 🗑️ DELETED entire file
    │   ├── hubgs.rs                        ← may need set_language updates
    │   ├── twxml.rs                        ← may need set_language updates
    │   └── features/                       ← check for set_language usage
    ├── handlers/                           ← unchanged (no direct tree-sitter access)
    ├── formatter/                          ← unchanged
    └── db/                                 ← unchanged (uses Salsa, not raw parser)
```

### 5. Detailed Implementation Notes

#### Step D-0: Update Cargo.toml

**Change (lsp/Cargo.toml line 15):**
```toml
# Before:
tree-sitter = "0.20"

# After:
tree-sitter = "0.26"
```

#### Step D-1: Delete TREE_CACHE artifacts from lib.rs

**Remove these from lib.rs:**

1. Lines 51–57 (`CachedTree` struct):
```rust
#[derive(Clone)]
pub struct CachedTree {
    pub tree: tree_sitter::Tree,
    pub content_len: usize,
    pub content_hash: u64,
    pub needs_reparse: bool,
}
```

2. Lines 66–70 (TREE_CACHE and get_tree_cache):
```rust
pub static TREE_CACHE: OnceLock<DashMap<String, CachedTree>> = OnceLock::new();

pub fn get_tree_cache() -> &'static DashMap<String, CachedTree> {
    TREE_CACHE.get_or_init(DashMap::new)
}
```

3. Line 49 (`use std::sync::OnceLock`) — remove this import if it's not used elsewhere. Check if `OnceLock` is used in any other context.

4. **Also check**: Is `calculate_hash` (lib.rs:59–64) still needed? It was only used by TREE_CACHE for content hashing. If no other callers exist, delete it too.

**grep check before deleting:**
```bash
grep -n "calculate_hash\|TREE_CACHE\|get_tree_cache\|CachedTree" lsp/src/lib.rs lsp/src/parser/*.rs lsp/src/handlers/**/*.rs
```

Only delete if the only caller is `cache.rs` (which we're also deleting).

#### Step D-2: Delete cache.rs entirely

```bash
rm lsp/src/parser/cache.rs
```

#### Step D-3: Update parser/mod.rs — remove cache module and fix language type

**Before (parser/mod.rs lines 1–4):**
```rust
mod cache;           // ← DELETE THIS LINE
mod features;
mod hubgs;
mod twxml;
```

After:
```rust
// mod cache removed — TREE_CACHE deleted in lib.rs
mod features;
mod hubgs;
mod twxml;
```

**Update the get_language return type for tree-sitter 0.26:**

In tree-sitter 0.26, language getters still return `Language` (owned), but `set_language()` takes `&Language`. No change needed to the getter signatures — they already return `Option<tree_sitter::Language>`. The breaking change is in *callers* that pass it to `set_language`.

#### Step D-4: Update set_language() calls across codebase

In tree-sitter 0.26, `set_language()` takes `&Language` instead of `Language`:

**Pattern:**
```rust
// BEFORE (ts 0.20):
parser.set_language(language)

// AFTER (ts 0.26):
parser.set_language(&language)
```

**Locations to update:**

1. **lib.rs get_symbol_at_position()** — line ~103:
```rust
// Before:
ts_parser.set_language(language).ok()?;

// After:
ts_parser.set_language(&language).ok()?;
```

2. **parser/hubgs.rs** — check for `set_language()` calls (if any direct usage exists):
```bash
grep -n "set_language" lsp/src/parser/hubgs.rs lsp/src/parser/twxml.rs
```

3. **parser/twxml.rs** — same check as hubgs.rs above.

4. **parser/features/** — check sub-modules:
```bash
grep -rn "set_language" lsp/src/parser/features/
```

5. **server.rs** — already deleted per Task A (skip this).

6. **parser/cache.rs** — already deleted (lines 17, 37, 52 all called set_language).

#### Step D-5: Verify grammar compatibility

After updating to tree-sitter 0.26, check that the grammar crates are compatible:

```bash
# Check if tree-sitter-hubgs and tree-sitter-twxml support ts 0.26
cargo build  # Will fail if grammars have ABI mismatch
```

If the in-tree grammars (`extension/languages/hubgs/`, `extension/languages/twxml/`) use `LANGUAGE_VERSION` that doesn't match 0.26's expected version, they need to be regenerated:

```bash
# The grammar needs to be regenerated with tree-sitter 0.26 CLI
npx tree-sitter generate --update
cargo build
```

#### Step D-6: Update Cargo.toml parser build dependencies if needed

If the grammars are built via `cc` in this crate (not as separate crates), verify the build script is compatible with ts 0.26's ABI. The `tree-sitter-hubgs()` and `tree_sitter_twxml()` extern "C" functions must still be callable.

Check if there's a `build.rs` or if grammars come from crates.io:
```bash
grep -r "tree.sitter.hubgs\|tree.sitter.twxml" lsp/Cargo.toml
```

---

## Quality Gate Verification

### Task A (Backend + Salsa Thread Safety)
| Metric | Score | Evidence |
|--------|-------|----------|
| Completeness | 5/5 | All 12 lock() sites mapped; server.rs deleted; main.rs updated |
| Grounding | 5/5 | Every conversion based on actual line numbers and code from codebase scan |
| Precision | 5/5 | with_db() template pattern applied consistently; nested attach violation noted |
| Scope Adherence | 5/5 | Only lib.rs, server.rs (deleted), main.rs, handler files touched |

### Task B (Pull Diagnostics)
| Metric | Score | Evidence |
|--------|-------|----------|
| Completeness | 5/5 | Handler fn, capabilities wiring, LanguageServer impl method, error handling all specified |
| Grounding | 5/5 | Uses existing lsp_types 0.94 types; validates against LSP 3.17 spec |
| Precision | 5/5 | Exact ServerCapabilities insertion point (line ~396); exact method signature |
| Scope Adherence | 5/5 | New file only + three targeted insertions in existing files |

### Task C (Indexed Lookup)
| Metric | Score | Evidence |
|--------|-------|----------|
| Completeness | 5/5 | build_instance_index() + resolve_reference() conversion + find_all_references() analysis |
| Grounding | 5/5 | Based on actual resolution.rs code; HashMap pattern matches existing HubInstance storage |
| Precision | 5/4 | Vec value type handles duplicates but "first match" semantics in resolve_reference should be documented as a known limitation |
| Scope Adherence | 5/5 | Only resolution.rs modified; callers unchanged (transparent) |

### Task D (Tree-sitter + Cache Removal)
| Metric | Score | Evidence |
|--------|-------|----------|
| Completeness | 5/5 | Cargo.toml version bump, all file deletions, set_language call locations mapped |
| Grounding | 5/5 | set_language() signature change is documented; all call sites enumerated by grep |
| Precision | 4/5 | Grammar ABI compatibility requires post-change verification (marked as Step D-5) |
| Scope Adherence | 5/5 | Only Cargo.toml, lib.rs, parser/, and their consumers touched |

---

## Summary of Changes by File

| File | Task A | Task B | Task C | Task D | Net Action |
|------|--------|--------|--------|--------|------------|
| `lsp/src/lib.rs` | Backend.db type change, read_db→db_handle, with_db conversions, TREE_CACHE removal | diagnostic_provider in ServerCapabilities, diagnostic() method | — | CachedTree/TREE_CACHE/delete imports, set_language(&) | **Major rewrite** |
| `lsp/src/server.rs` | 🗑️ **DELETED** (dead code) | — | — | — | **Remove entirely** |
| `lsp/src/main.rs` | SalsaThreadHandle instantiation | — | — | — | **Minor change** (1 line) |
| `lsp/src/handlers/features/diagnostics.rs` | — | 🆕 **NEW FILE** | — | — | **Create** |
| `lsp/src/handlers/features/mod.rs` | — | Module export | — | — | **Add one line** |
| `lsp/src/db/resolution.rs` | — | — | build_instance_index + resolve_reference update | — | **2 functions** |
| `lsp/src/parser/cache.rs` | — | — | — | 🗑️ **DELETED** | **Remove entirely** |
| `lsp/src/parser/mod.rs` | — | — | — | Remove cache import, prepare set_language | **Minor change** |
| `lsp/Cargo.toml` | — | — | — | tree-sitter 0.20→0.26 | **1 line** |
| `lsp/src/handlers/documents.rs` | All 6 lock() sites → with_db | — | — | — | **6 conversions** |
| `lsp/src/handlers/navigation.rs` | read_db()→db_handle() if applicable | — | — | — | **Check for lock() calls** |
