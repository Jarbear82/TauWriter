# Zed Language Configuration
Configuration for how Zed handles HubGS and TWXML files.

## HubGS Config
```toml
# languages/hubgs/config.toml

name = "HubGS"
type = "language"
file_types = ["hubgs"]
scope_name = "source.hubgs"

[line_comments]
start = "//"

[block_comments]
start = "/*"
end = "*/"

[indentation]
increase_pattern = "\\{\\s*$"
decrease_pattern = "^\\s*\\}"
```

## TWXML Config
```toml
# languages/twxml/config.toml

name = "TWXML"
type = "language"
file_types = ["twxml"]
scope_name = "text.twxml"

[line_comments]
# Note: XML doesn't have line comments, using block syntax
start = "<!--"
end = "-->"

[indentation]
increase_pattern = "<(?![\\/!])[^>]+(?:?<![\\/])>$"
decrease_pattern = "^\\s*<\\/"
```
