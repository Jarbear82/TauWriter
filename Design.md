  ## Updated Design Proposal
  *Following constraints from DESIGNING.md*
  Strategy: Layered
  Strategy Selection Criteria: The application operates as a linear transformation pipeline split across three logical boundaries:
  1. Source/Parsing Layer: Merges in-app Tree-sitter parsing (using  outlines.scm  for document trees) and JSON-RPC LSP diagnostics/references, falling back to local parsing if the LSP is offline.
  2. Buffer/State Layer: Maintains open document buffers and mode select configurations (WYSIWYG, Raw, Markdown) sharing the same underlying text buffer.
  3. Presentation Layer: Coordinates the split side-by-side GPUI workspace layout (Left Document Tabbed View, Right Graph Tabbed View) and canvas renders.
  ### 1. Encapsulation (Level 3)
  • Structs:
      •  DocumentTabState : Manages a single open document's active mode selection ( Raw ,  Wysiwyg ,  Markdown ), scroll offsets, and local Tree-sitter AST nodes.
      •  WorkspaceState : Tracks all open document tabs, the active selected document index, file explorer node state, and LSP connection status.
      •  GraphVisualization : Encapsulates 2D canvas coordinates, zoom metrics, drag gestures, and active node highlights.
      •  DocumentGraph : Stores document outline nodes parsed locally from Tree-sitter queries using the  outlines.scm  grammar definition.
  • Protection:
      •  DocumentTabState::new(path) : Fallibly opens and initializes buffers.
      •  WorkspaceState::default() : Safely constructs an empty workspace.
      •  GraphVisualization::new() : Sets up the default zoom bounds and spring parameters.


  ### 2. Type Relations (Level 4)
  • Has-A:
      •  Workspace  Has-A  Vec<DocumentTabState>  (multiple open document tabs).
      •  MainView  Has-A  Workspace  +  GraphPaneView  (mediating event propagation between them).
      •  DocumentTabState  Has-A  InputState  (re-used for the Raw Editor and sharing buffer state across modes).
  • Is-A:
      •  gpui::Render  implemented on view components.
      • Custom  DocumentRenderer  trait to unify block rendering structures (WYSIWYG layout vs Markdown generator outputs).


  ### 3. Component Architecture (Level 5)

  • Pattern: Central Mediator ( MainView ) coordination.
  • Justification: Panes must communicate cross-pane updates (e.g. clicking a WYSIWYG  <hubref>  updates right-pane tab focus and canvas highlight; clicking a graph node opens a new tab and sets editor cursor position). A central
  mediator coordinates these without creating cyclic references between  DocumentPane  and  GraphPane .
  • Trade-offs: Centralizes routing logic inside  MainView , but guarantees independent, testable components.

  ### 4. Artifact (ASCII Module Hierarchy Tree)

    host/src/
    ├── main.rs (App entrypoint & window coordinator)
    ├── lsp_client.rs (JSON-RPC communications client)
    ├── graph_sim.rs (Force-directed layout physics)
    ├── parser/
    │   ├── mod.rs (TWXML parsing & recursive <include /> stitching)
    │   └── tree_sitter_query.rs (In-app tree-sitter query via outlines.scm)
    ├── ui/
    │   ├── mod.rs (Workspace layout & pane split coordination)
    │   ├── sidebar.rs (File tree tree-view via virtualized lists)
    │   ├── titlebar.rs (CSD title bar & settings panel)
    │   ├── document_view/
    │   │   ├── mod.rs (WYSIWYG/Raw/Markdown mode rendering)
    │   │   ├── renderers.rs (TWXML block elements render engine)
    │   │   └── tab_bar.rs (Document tab control bar)
    │   └── graph_pane/
    │       ├── mod.rs (Graph view tabs pane)
    │       ├── doc_graph.rs (XML Outline / Tree visualization)
    │       └── relation_graph.rs (Definitions & instances canvas)
    └── utils/
        └── mod.rs (Workspace path resolver)
    ──────
  ## Refactored Step-by-Step Implementation Plan

  ### Step 1: UI Split Layout & Document/Graph Panes

  • Objective: Re-architect the workspace into two permanent side-by-side resizable split panes with independent tab controls.
  • Implementation:
      1. Refactor  Workspace  state in mod.rs to maintain:
          •  open_documents: Vec<DocumentTabState> 
          •  active_document_idx: Option<usize> 
          •  active_graph_tab: GraphTab  ( DocumentGraph ,  DefinitionsSchema ,  InstancesRelation )
      2. Implement  DocumentTabState  containing the path, raw text buffer, active display mode ( Raw ,  Wysiwyg ,  Markdown ), and Tree-sitter query AST caches.
      3. Render the left half containing  DocumentView  and the right half containing  GraphPaneView  using  gpui_component::resizable::h_resizable("workspace-split") .
      4. Build the Document Tab Bar ( DocumentTabBar ) to render one tab per open document, with close buttons and click listeners to switch  active_document_idx .
      5. Add a mode selection dropdown element next to the active document tab to toggle its mode.


  ### Step 2: Mode Renderers & Include Stitching ( <include /> )

  • Objective: Render document content differently depending on the active mode and compile included files recursively.
  • Implementation:
      1. Raw Mode: Render the raw XML text editor with syntax highlighting using  gpui_component::input::Input . Render the  <include src="..." />  tags literally.
      2. WYSIWYG Mode: Implement recursive stitching in the TWXML parser. When encountering  <include src="filename.twxml" /> , parse that file and insert its body children into the parent block hierarchy. Render the resulting
      flattened document as styled preview blocks.
      3. Markdown Mode: Implement a read-only Markdown converter that formats the TWXML AST as styled Markdown text. Render included files using the  ![[document-name]]  syntax.
      4. Ensure all three modes reference the same underlying text buffer (editing in Raw mode is immediately reflected when switching to WYSIWYG or Markdown).


  ### Step 3: In-Application Tree-Sitter Parser & Outlines

  • Objective: Leverage Tree-sitter queries using the extension's  outlines.scm  to build outlines, falling back to local queries if the LSP is offline.
  • Implementation:
      1. Load  extension/languages/twxml/queries/outlines.scm  into the application.
      2. Parse the active TWXML document using the tree-sitter-twxml grammar, execute the outline queries, and map captured heading/section nodes to a hierarchical outline tree.
      3. Render this tree outline in the Twxml Document Graph tab on the right Graph Pane as an interactive visualization.
      4. Clicking outline nodes jumps the cursor and scrolls the active editor tab.


  ### Step 4: Bidirectional Linking & Navigation

  • Objective: Connect prose references to graph nodes and vice versa.
  • Implementation:
      1. Document → Graph Navigation:
          • Bind a click listener to  <hubref>  elements in the WYSIWYG block renderer.
          • When clicked, notify the main view to:
              1. Switch the right pane to the  InstancesRelation  tab.
              2. Focus/select the target Hub node on the canvas.
              3. Center and zoom the canvas around the node.

      2. Graph → Document Navigation:
          • Bind click listeners to nodes in the  InstancesRelation  canvas.
          • When a node is clicked, check if the referencing document is open in  open_documents . If not, load it as a new document tab and make it active.
          • Move the cursor and scroll the editor window to the nearest  <hubref>  instance matching the node's Hub ID.



  ### Step 5: LSP Integration & Fallback Path

  • Objective: Query LSP to enhance structure and validation, gracefully falling back to local parsers if the LSP fails.
  • Implementation:
      1. Check LSP server availability.
      2. If online: query type definitions, validation multiplicity errors, and auto-completes via standard LSP requests.
      3. If offline: execute validation checks and AST highlights using the local tree-sitter parser, displaying warnings about missing LSP diagnostics.

  ──────
  ### Summary of Documentation Updates Made:

  We have updated TauWriterDesign.md to:

  1. Define the side-by-side pane split view layout (Left pane: Document tabs with Raw/WYSIWYG/Markdown dropdown mode selector; Right pane: Twxml Document Graph, HubGs Definitions Schema, and HubGs Instances Relation tabs).
  2. Explicitly define how includes behave in WYSIWYG (flattened), Raw (tag-only), and Markdown ( ![[document-name]]  format).
  3. Set up the logic for bidirectional linking (Click  <hubref>  in WYSIWYG -> highlight/zoom right Instance node; Click right Instance node -> open document tab, scroll and move cursor to instance).
  4. Update unresolved design questions to define document fragmentation rules.
