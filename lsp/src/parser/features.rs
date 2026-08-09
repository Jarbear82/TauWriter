use crate::db::{Db, LspRange, SemanticToken, SourceFile};
use streaming_iterator::StreamingIterator;
use tree_sitter::Parser;

mod semantic_tokens {
    pub const TYPE: u32 = 0; // CLASS
    pub const FIELD: u32 = 1; // PROPERTY
    pub const INSTANCE: u32 = 2; // VARIABLE
    pub const ENUM: u32 = 3; // ENUM

    pub const DECLARATION: u32 = 1 << 1; // bit 1
    pub const DEFINITION: u32 = 1 << 2; // bit 2
}

pub fn compute_semantic_tokens(db: &dyn Db, file: SourceFile) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    if path.ends_with(".hubgs") {
        append_hubgs_semantic_tokens(&contents, &mut tokens);
    } else if path.ends_with(".twxml") {
        append_twxml_semantic_tokens(db, file, &mut tokens);
    }

    tokens.sort_by(|a, b| {
        if a.line != b.line {
            a.line.cmp(&b.line)
        } else {
            a.character.cmp(&b.character)
        }
    });
    tokens
}

fn append_hubgs_semantic_tokens(contents: &str, tokens: &mut Vec<SemanticToken>) {
    let language = match super::get_hubgs_language() {
        Some(lang) => lang,
        None => return, // Graceful degradation if grammar unavailable
    };
    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        return;
    }
    let tree = match parser.parse(contents, None) {
        Some(t) => t,
        None => return,
    };

    // user-review: Semantic token query for hubgs grammar. Maps AST nodes to LSP token types.
    let query_str = r#"
        (hub_definition (identifier) @type_def)
        (instance_block (identifier) @inst_name (identifier) @inst_type)
        (hub_field (identifier) @field_name)
        (hub_role (identifier) @role_name)
        (instance_assignment (identifier) @assign_name)
        (enum_definition (identifier) @enum_name)
    "#;
    let query = tree_sitter::Query::new(&language, query_str).expect("query syntax error");
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches_result = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    while let Some(m) = matches_result.next() {
        for capture in m.captures {
            let name = &query.capture_names()[capture.index as usize];
            let node = capture.node;
            let range = node.range();
            let (token_type, modifiers) = match *name {
                "type_def" => (
                    semantic_tokens::TYPE,
                    semantic_tokens::DECLARATION | semantic_tokens::DEFINITION,
                ),
                "inst_name" => (semantic_tokens::INSTANCE, semantic_tokens::DEFINITION),
                "inst_type" => (semantic_tokens::TYPE, 0),
                "field_name" | "role_name" => (
                    semantic_tokens::FIELD,
                    semantic_tokens::DECLARATION | semantic_tokens::DEFINITION,
                ),
                "assign_name" => (semantic_tokens::FIELD, 0),
                "enum_name" => (
                    semantic_tokens::ENUM,
                    semantic_tokens::DECLARATION | semantic_tokens::DEFINITION,
                ),
                _ => continue,
            };
            tokens.push(SemanticToken {
                line: range.start_point.row as u32,
                character: range.start_point.column as u32,
                length: (range.end_byte - range.start_byte) as u32,
                token_type,
                token_modifiers: modifiers,
            });
        }
    }
}

fn append_twxml_semantic_tokens(db: &dyn Db, file: SourceFile, tokens: &mut Vec<SemanticToken>) {
    let refs = crate::db::parse_twxml(db, file);
    for r in refs {
        let range = r.range(db);
        let length = if range.end.character > range.start.character + 2 {
            range.end.character - range.start.character - 2
        } else {
            0
        };
        tokens.push(SemanticToken {
            line: range.start.line,
            character: range.start.character + 1,
            length,
            token_type: semantic_tokens::INSTANCE, // Use named constant instead of magic number
            token_modifiers: 0,
        });
    }
}

pub fn compute_folding_ranges(db: &dyn Db, file: SourceFile) -> Vec<LspRange> {
    let mut ranges = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    let language = match path.as_bytes() {
        p if p.ends_with(b".hubgs") => super::get_hubgs_language(),
        p if p.ends_with(b".twxml") => super::get_twxml_language(),
        _ => return ranges,
    };

    let Some(language) = language else {
        eprintln!("TauWriter warning: tree-sitter grammar unavailable for {path}");
        return ranges;
    };

    let mut parser = Parser::new();
    if parser.set_language(&language).is_err() {
        return ranges;
    }
    let tree = match parser.parse(&contents, None) {
        Some(t) => t,
        None => return ranges,
    };

    // Determine foldable node kinds from file extension to avoid matching irrelevant AST shapes
    let is_hubgs = path.ends_with(".hubgs");
    let foldable_kinds: &[&str] = if is_hubgs {
        &[
            "imports_section",
            "definitions_section",
            "fields_block",
            "enums_block",
            "hubs_block",
            "hub_definition",
            "instances_section",
            "instance_block",
        ]
    } else {
        &["element"]
    };

    let mut stack = vec![tree.root_node()];

    while let Some(node) = stack.pop() {
        let range = node.range();
        if range.start_point.row != range.end_point.row && foldable_kinds.contains(&node.kind()) {
            ranges.push(super::ts_range_to_lsp(range));
        }

        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            stack.push(child);
        }
    }

    ranges
}
