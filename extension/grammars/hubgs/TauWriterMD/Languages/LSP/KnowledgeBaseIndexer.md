# Knowledge Base Indexer
The Knowledge Base Indexer is the core "sync engine" of TauWriter. It runs as a background process within the Rust LSP to provide bidirectional linking between HubGS data and TWXML prose.

## Responsibilities
1. **Workspace Watcher**: Monitors the Knowledge Base directory for changes to `*.hubgs` and `*.twxml` files.
2. **Salsa Database**: Maintains an incremental, query-driven database of all Hub instances and their locations.
3. **Cross-Language Resolution**:
   - Resolves `<hubref id="...">` in TWXML to specific `INSTANCES` in HubGS.
   - Computes back-references (which documents reference which Hub) at query time.
4. **Validation**: Ensures that all `id`s in `<hubref>` tags exist in the global graph.

## Tech Stack
- **Rust**: Primary implementation language.
- **Salsa**: For incremental computation and reactive indexing.
- **Notify**: For file system watching.
- **Tree-Sitter**: For fast, partial re-parsing of changed files.

## Workflow Example: Hub Deletion
1. User deletes an instance `aragorn:Person` from `fantasy.hubgs`.
2. `Notify` triggers the Indexer.
3. `Salsa` identifies that the Hub graph has changed.
4. The LSP queries the indexer for all documents containing `<hubref id="aragorn">`.
5. The LSP sends a `workspace/applyEdit` to the client to strip the `<hubref>` tags from those documents.
