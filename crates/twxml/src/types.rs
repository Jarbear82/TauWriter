use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub hubref: Option<String>,
    pub link: Option<String>,
    pub footnote_ref: Option<String>,
    pub id: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub range: Option<std::ops::Range<usize>>,
}

impl Default for TextRun {
    fn default() -> Self {
        Self {
            text: String::new(),
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            code: false,
            superscript: false,
            subscript: false,
            hubref: None,
            link: None,
            footnote_ref: None,
            id: None,
            attributes: Vec::new(),
            range: None,
        }
    }
}

impl TextRun {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub runs: Vec<TextRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: usize,
        text: String,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Paragraph {
        runs: Vec<TextRun>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    BlockQuote {
        runs: Vec<TextRun>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Aside {
        runs: Vec<TextRun>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    CodeBlock {
        language: String,
        code: String,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    DescriptionList {
        items: Vec<(String, Vec<TextRun>)>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<Vec<TextRun>>>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    HorizontalRule {
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Image {
        src: String,
        alt: Option<String>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Audio {
        src: String,
        alt: Option<String>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Video {
        src: String,
        alt: Option<String>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Details {
        summary: String,
        blocks: Vec<Block>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Footnote {
        id: String,
        runs: Vec<TextRun>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Review {
        blocks: Vec<Block>,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
    },
    Include {
        src: String,
        id: Option<String>,
        attributes: Vec<(String, String)>,
        range: Option<std::ops::Range<usize>>,
        resolved_blocks: Option<Vec<Block>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutlineNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub start_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HubReferenceInfo {
    pub name: String,
    pub field: Option<String>,
    pub text: Option<String>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub is_reviewed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TwxmlTagInfo {
    pub name: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub parent_name: Option<String>,
}
