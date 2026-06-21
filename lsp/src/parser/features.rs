use crate::db::{Db, LspRange, SemanticToken, SourceFile};
use tree_sitter::Parser;

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
    let language = unsafe { super::tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let query_str = r#"
        (hub_definition (identifier) @type_def)
        (instance_block (identifier) @inst_name (identifier) @inst_type)
        (hub_field (identifier) @field_name)
        (hub_role (identifier) @role_name)
        (instance_assignment (identifier) @assign_name)
        (enum_definition (identifier) @enum_name)
    "#;
    let query = tree_sitter::Query::new(language, query_str).unwrap();
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, tree.root_node(), contents.as_bytes());

    for m in matches {
        for capture in m.captures {
            let name = &query.capture_names()[capture.index as usize];
            let node = capture.node;
            let range = node.range();
            let (token_type, modifiers) = match name.as_str() {
                "type_def" => (0, 3),
                "inst_name" => (2, 2),
                "inst_type" => (0, 0),
                "field_name" | "role_name" => (1, 3),
                "assign_name" => (1, 0),
                "enum_name" => (3, 3),
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
            token_type: 2,
            token_modifiers: 0,
        });
    }
}

pub fn compute_folding_ranges(db: &dyn Db, file: SourceFile) -> Vec<LspRange> {
    let mut ranges = Vec::new();
    let contents = file.contents(db);
    let path = file.path(db);

    let language = if path.ends_with(".hubgs") {
        unsafe { super::tree_sitter_hubgs() }
    } else if path.ends_with(".twxml") {
        unsafe { super::tree_sitter_twxml() }
    } else {
        return ranges;
    };

    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(&contents, None).unwrap();

    let mut stack = vec![tree.root_node()];

    while let Some(node) = stack.pop() {
        let range = node.range();
        if range.start_point.row != range.end_point.row {
            let is_foldable = matches!(
                node.kind(),
                "imports_section"
                    | "definitions_section"
                    | "fields_block"
                    | "enums_block"
                    | "hubs_block"
                    | "hub_definition"
                    | "instances_section"
                    | "instance_block"
                    | "element"
            );

            if is_foldable {
                ranges.push(super::ts_range_to_lsp(range));
            }
        }

        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            stack.push(child);
        }
    }

    ranges
}
