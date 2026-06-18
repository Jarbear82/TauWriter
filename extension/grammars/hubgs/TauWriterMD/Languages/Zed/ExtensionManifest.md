# Zed Extension Manifest
This file defines the TauWriter extension for the Zed editor.

```toml
# extension.toml

id = "tauwriter"
name = "TauWriter"
version = "0.1.0"
authors = ["Your Name"]
description = "IDE-style, graph-augmented word processor for novelists and worldbuilders."
repository = "https://github.com/your/tauwriter"

[languages.hubgs]
name = "HubGS"
grammar = "hubgs"
path_suffixes = ["hubgs"]

[languages.twxml]
name = "TWXML"
grammar = "twxml"
path_suffixes = ["twxml"]

[grammars.hubgs]
repository = "https://github.com/your/tree-sitter-hubgs"
rev = "main"

[grammars.twxml]
repository = "https://github.com/your/tree-sitter-twxml"
rev = "main"
```
