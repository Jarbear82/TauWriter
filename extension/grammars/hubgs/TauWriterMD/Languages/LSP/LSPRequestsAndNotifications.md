The Language Server Protocol (LSP) defines a standardized set of JSON-RPC messages exchanged between a development tool (the client, like VS Code or Neovim) and a language smartness provider (the server).

Because the specification is extensive, the commands, actions, and operations (referred to as **Requests** and **Notifications** in the spec) are best organized by the lifecycle and feature categories they support.

Here is a comprehensive list of the core LSP operations (up through the 3.17 specification):

### **1. General & Lifecycle Messages**

These handle the connection and basic capabilities between the client and server.

- **`initialize` (Request):** Sent once as the first request from the client to establish capabilities and workspace details.
    
- **`initialized` (Notification):** Sent from the client after receiving the `initialize` response.
    
- **`client/registerCapability` (Request):** Server asks the client to dynamically register a feature.
    
- **`client/unregisterCapability` (Request):** Server asks the client to unregister a feature.
    
- **`shutdown` (Request):** Asks the server to safely shut down.
    
- **`exit` (Notification):** Tells the server to exit its process.
    
- **`$/cancelRequest` (Notification):** Cancels an ongoing request.
    

### **2. Window Messages**

These allow the server to interact with the user interface of the editor.

- **`window/showMessage` (Notification):** Asks the client to display a standard message (info, warning, error).
    
- **`window/showMessageRequest` (Request):** Displays a message and waits for user input/action.
    
- **`window/showDocument` (Request):** Asks the client to open or reveal a specific document.
    
- **`window/logMessage` (Notification):** Sends a message to the client's output channel/log.
    
- **`window/workDoneProgress/create` (Request):** Server asks to create a progress bar/tracker.
    
- **`progress` (Notification):** Used to report progress on an ongoing operation.
    
- **`telemetry/event` (Notification):** Sends telemetry data from the server to the client.
    

### **3. Workspace Features**

These manage the broader project context rather than a single file.

- **`workspace/workspaceFolders` (Request):** Fetches the current open workspace folders.
    
- **`workspace/didChangeWorkspaceFolders` (Notification):** Tells the server that folders were added or removed.
    
- **`workspace/didChangeConfiguration` (Notification):** Signals that user settings/configurations have changed.
    
- **`workspace/configuration` (Request):** Server asks the client for configuration settings.
    
- **`workspace/didChangeWatchedFiles` (Notification):** Notifies the server of file system events (create, change, delete).
    
- **`workspace/symbol` (Request):** Searches for symbols across the entire workspace.
    
- **`workspace/executeCommand` (Request):** Executes a custom, server-specific command.
    
- **`workspace/applyEdit` (Request):** Server asks the client to apply a workspace-wide edit (e.g., across multiple files).
    
- **`workspace/semanticTokens/refresh` (Request):** Asks the client to refresh semantic highlighting.
    
- **`workspace/inlayHint/refresh` (Request):** Asks the client to refresh inlay hints.
    
- **`workspace/inlineValue/refresh` (Request):** Asks the client to refresh inline values.
    
- **`workspace/diagnostic/refresh` (Request):** Asks the client to refresh pull-based diagnostics.
    

### **4. Text Document Synchronization**

These keep the server's internal representation of the code in sync with the user's editor.

- **`textDocument/didOpen` (Notification):** A file was opened; server takes ownership of its content.
    
- **`textDocument/didChange` (Notification):** The user typed or edited the document.
    
- **`textDocument/willSave` (Notification):** The document is about to be saved.
    
- **`textDocument/willSaveWaitUntil` (Request):** The document is about to be saved, and the server can modify it before the save completes (e.g., format on save).
    
- **`textDocument/didSave` (Notification):** The document was saved to disk.
    
- **`textDocument/didClose` (Notification):** The document was closed.
    

### **5. Language Features**

These are the core "smart" features triggered by user actions like typing, hovering, or clicking.

**Navigation & Information:**

- **`textDocument/declaration` (Request):** Go to declaration.
    
- **`textDocument/definition` (Request):** Go to definition.
    
- **`textDocument/typeDefinition` (Request):** Go to the definition of the symbol's type.
    
- **`textDocument/implementation` (Request):** Find implementations of an interface or trait.
    
- **`textDocument/references` (Request):** Find all references to a symbol.
    
- **`textDocument/hover` (Request):** Show documentation/type info on hover.
    
- **`textDocument/signatureHelp` (Request):** Show parameter hints while typing a function call.
    

**Editing & Refactoring:**

- **`textDocument/completion` (Request):** Request autocomplete suggestions.
    
- **`completionItem/resolve` (Request):** Fetch additional details (like docs) for a selected completion item.
    
- **`textDocument/rename` (Request):** Rename a symbol across the project.
    
- **`textDocument/prepareRename` (Request):** Test if a symbol can be renamed and get its boundaries.
    
- **`textDocument/codeAction` (Request):** Request quick fixes, refactorings, or source actions.
    
- **`codeAction/resolve` (Request):** Fetch the actual edits for a code action (lazy evaluation).
    

**Formatting:**

- **`textDocument/formatting` (Request):** Format the whole document.
    
- **`textDocument/rangeFormatting` (Request):** Format a selected portion of the document.
    
- **`textDocument/onTypeFormatting` (Request):** Auto-format as the user types specific characters (e.g., `;` or `}`).
    

**UI & Highlighting:**

- **`textDocument/documentHighlight` (Request):** Highlight all occurrences of a symbol in the current file.
    
- **`textDocument/documentSymbol` (Request):** Provide a tree/list of symbols in the document (used for outline views).
    
- **`textDocument/codeLens` (Request):** Request actionable inline hints (e.g., "Run Test" or "3 references").
    
- **`codeLens/resolve` (Request):** Resolve details for a specific CodeLens.
    
- **`textDocument/documentLink` (Request):** Find clickable URLs or file paths in the document.
    
- **`documentLink/resolve` (Request):** Resolve the target of a document link.
    
- **`textDocument/color` (Request):** Find color definitions (hex/rgb) for color pickers.
    
- **`textDocument/colorPresentation` (Request):** Get format options for a modified color.
    
- **`textDocument/foldingRange` (Request):** Get code folding regions (e.g., collapsing a function body).
    
- **`textDocument/selectionRange` (Request):** Expand/shrink selection logically based on AST (Abstract Syntax Tree).
    
- **`textDocument/semanticTokens/full` (Request):** Request full syntax highlighting based on semantic meaning.
    
- **`textDocument/semanticTokens/full/delta` (Request):** Request only changes in semantic highlighting.
    
- **`textDocument/semanticTokens/range` (Request):** Semantic tokens for a visible range.
    
- **`textDocument/inlayHint` (Request):** Request inline hints (e.g., implicit parameter names or inferred types).
    
- **`inlayHint/resolve` (Request):** Fetch additional details for an inlay hint.
    
- **`textDocument/inlineValue` (Request):** Show execution values inline (often used during debugging).
    
- **`textDocument/moniker` (Request):** Get a unique identifier for a symbol (used in LSIF for code intelligence).
    
- **`textDocument/linkedEditingRange` (Request):** Support linked editing (e.g., changing an opening HTML tag auto-changes the closing tag).
    

### **6. Diagnostics**

- **`textDocument/publishDiagnostics` (Notification):** Server pushes errors, warnings, and hints to the client (Push model).
    
- **`textDocument/diagnostic` (Request):** Client pulls diagnostics from the server (Pull model - introduced in LSP 3.17).