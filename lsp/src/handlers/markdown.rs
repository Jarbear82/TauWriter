/// Incremental Markdown builder for LSP hover contents.
///
/// Used across multiple handler modules to construct
/// `MarkupContent` values without string concatenation overhead.

pub struct MarkdownContent {
    lines: Vec<String>,
}

impl MarkdownContent {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn heading(&mut self, level: u8, text: &str) {
        let prefix = "#".repeat(level as usize);
        self.lines.push(format!("{} {}", prefix, text));
    }

    pub fn text(&mut self, content: &str) {
        self.lines.push(content.to_string());
    }

    pub fn bold_list_item(&mut self, key: &str, value: &str) {
        self.lines.push(format!("- **{}:** {}", key, value));
    }

    pub fn bold(&mut self, text: &str) {
        self.lines.push(format!("**{}**", text));
    }

    pub fn text_item(&mut self, content: &str) {
        self.lines.push(format!("  - {}", content));
    }

    pub fn link_with_uri(&mut self, name: &str, uri: &str) {
        self.lines.push(format!("  - [{}]({})", name, uri));
    }

    pub fn separator(&mut self) {
        self.lines.push("---".to_string());
    }

    pub fn code_block(&mut self, content: &str, lang: &str) {
        self.lines.push(format!("```{}", lang));
        for line in content.lines() {
            self.lines.push(line.to_string());
        }
        self.lines.push("```".to_string());
    }

    pub fn to_markdown(&self) -> String {
        self.lines.join("\n")
    }
}

impl Default for MarkdownContent {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MarkdownContent {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}
