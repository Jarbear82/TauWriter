use tree_sitter::{Language, Parser, Node};

extern "C" {
    pub fn tree_sitter_hubgs() -> Language;
    pub fn tree_sitter_twxml() -> Language;
}

pub fn format_source(contents: &str, file_type: &str) -> String {
    if file_type == "twxml" {
        format_twxml(contents)
    } else if file_type == "hubgs" {
        format_hubgs(contents)
    } else {
        contents.to_string()
    }
}

fn format_twxml(contents: &str) -> String {
    let language = unsafe { tree_sitter_twxml() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(contents, None).unwrap();
    let root = tree.root_node();
    
    let mut result = String::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        result.push_str(&format_twxml_node(child, contents, 0, true));
    }
    result.trim().to_string() + "\n"
}

fn is_block_tag(name: &str) -> bool {
    const BLOCK_TAGS: &[&str] = &[
        "document", "meta", "section", "heading", "paragraph", "aside", 
        "blockquote", "codeblock", "ul", "ol", "li", "dl", "dt", "dd", 
        "details", "summary", "table", "tr", "th", "td", "footnote", "review"
    ];
    BLOCK_TAGS.contains(&name)
}

fn contains_block_tag(node: Node, contents: &str) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "element" || child.kind() == "self_closing_element" {
            if let Some(nm) = child.child_by_field_name("name") {
                let name = &contents[nm.byte_range()];
                if is_block_tag(name) {
                    return true;
                }
            }
            if child.kind() == "element" {
                if let Some(st) = child.child(0) {
                    if let Some(nm) = st.child_by_field_name("name") {
                        let name = &contents[nm.byte_range()];
                        if is_block_tag(name) {
                            return true;
                        }
                    }
                }
            }
        }
        if contains_block_tag(child, contents) {
            return true;
        }
    }
    false
}

fn format_twxml_node(node: Node, contents: &str, indent_level: usize, is_start_of_line: bool) -> String {
    let indent = "  ".repeat(indent_level);
    match node.kind() {
        "text" => {
            let txt = contents[node.byte_range()].trim();
            if txt.is_empty() {
                String::new()
            } else {
                if is_start_of_line {
                    format!("{}{}", indent, txt)
                } else {
                    txt.to_string()
                }
            }
        }
        "comment" => {
            let cmt = contents[node.byte_range()].trim();
            if is_start_of_line {
                format!("{}{}", indent, cmt)
            } else {
                cmt.to_string()
            }
        }
        "element" => {
            let start_tag = node.child(0).unwrap();
            let name_node = start_tag.child_by_field_name("name").unwrap();
            let tag_name = &contents[name_node.byte_range()];
            
            let mut attrs = Vec::new();
            let mut cursor = start_tag.walk();
            for child in start_tag.children(&mut cursor) {
                if child.kind() == "attribute" {
                    attrs.push(contents[child.byte_range()].to_string());
                }
            }
            let attrs_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(" {}", attrs.join(" "))
            };
            
            let is_block = is_block_tag(tag_name);
            let has_blocks = contains_block_tag(node, contents);
            
            if is_block && has_blocks {
                let mut inner = String::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "start_tag" && child.kind() != "end_tag" {
                        let formatted = format_twxml_node(child, contents, indent_level + 1, true);
                        if !formatted.is_empty() {
                            inner.push_str(&formatted);
                            inner.push('\n');
                        }
                    }
                }
                
                let start_part = if is_start_of_line {
                    format!("{}<{}{}>\n", indent, tag_name, attrs_str)
                } else {
                    format!("<{}{}>\n", tag_name, attrs_str)
                };
                let end_part = format!("{}</{}>", indent, tag_name);
                format!("{}{}{}", start_part, inner, end_part)
            } else {
                let mut inner = String::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "start_tag" && child.kind() != "end_tag" {
                        inner.push_str(&format_twxml_node(child, contents, 0, false));
                    }
                }
                
                if is_block && is_start_of_line {
                    format!("{}<{}{}>{}</{}>", indent, tag_name, attrs_str, inner, tag_name)
                } else {
                    format!("<{}{}>{}</{}>", tag_name, attrs_str, inner, tag_name)
                }
            }
        }
        "self_closing_element" => {
            let name_node = node.child_by_field_name("name").unwrap();
            let tag_name = &contents[name_node.byte_range()];
            let mut attrs = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "attribute" {
                    attrs.push(contents[child.byte_range()].to_string());
                }
            }
            let attrs_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(" {}", attrs.join(" "))
            };
            
            let is_block = is_block_tag(tag_name);
            if is_block && is_start_of_line {
                format!("{}<{}{} />", indent, tag_name, attrs_str)
            } else {
                format!("<{}{} />", tag_name, attrs_str)
            }
        }
        _ => {
            let mut inner = String::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                inner.push_str(&format_twxml_node(child, contents, indent_level, is_start_of_line));
            }
            inner
        }
    }
}

fn format_hubgs(contents: &str) -> String {
    let language = unsafe { tree_sitter_hubgs() };
    let mut parser = Parser::new();
    parser.set_language(language).ok();
    let tree = parser.parse(contents, None).unwrap();
    let root = tree.root_node();
    
    let mut sections = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        match child.kind() {
            "imports_section" => {
                sections.push(format_imports_section(child, contents));
            }
            "definitions_section" => {
                sections.push(format_definitions_section(child, contents));
            }
            "instances_section" => {
                sections.push(format_instances_section(child, contents));
            }
            _ => {}
        }
    }
    
    sections.join(",\n\n") + "\n"
}

fn format_imports_section(node: Node, contents: &str) -> String {
    let mut imports = Vec::new();
    let mut cursor = node.walk();
    for stmt in node.children(&mut cursor) {
        if stmt.kind() == "import_statement" {
            let mut types = Vec::new();
            let mut from_path = String::new();
            let mut stmt_cursor = stmt.walk();
            for child in stmt.children(&mut stmt_cursor) {
                match child.kind() {
                    "identifier" => {
                        types.push(contents[child.byte_range()].to_string());
                    }
                    "string" => {
                        from_path = contents[child.byte_range()].to_string();
                    }
                    _ => {}
                }
            }
            imports.push(format!("    [ {} ] FROM {}", types.join(", "), from_path));
        }
    }
    format!("IMPORTS [\n{}\n]", imports.join(",\n"))
}

fn format_definitions_section(node: Node, contents: &str) -> String {
    let mut blocks = Vec::new();
    let mut cursor = node.walk();
    for block in node.children(&mut cursor) {
        match block.kind() {
            "fields_block" => {
                let mut fields = Vec::new();
                let mut fb_cursor = block.walk();
                for field in block.children(&mut fb_cursor) {
                    if field.kind() == "field_definition" {
                        if let (Some(id_node), Some(type_node)) = (field.child(0), field.child(2)) {
                            let name = &contents[id_node.byte_range()];
                            let ty = &contents[type_node.byte_range()];
                            fields.push(format!("        {}: {}", name, ty));
                        }
                    }
                }
                blocks.push(format!("    FIELDS [\n{}\n    ]", fields.join(",\n")));
            }
            "enums_block" => {
                let mut enums = Vec::new();
                let mut eb_cursor = block.walk();
                for enum_def in block.children(&mut eb_cursor) {
                    if enum_def.kind() == "enum_definition" {
                        if let Some(name_node) = enum_def.child(0) {
                            let name = &contents[name_node.byte_range()];
                            let mut variants = Vec::new();
                            let mut v_cursor = enum_def.walk();
                            for child in enum_def.children(&mut v_cursor) {
                                if child.kind() == "identifier" && child.id() != name_node.id() {
                                    variants.push(contents[child.byte_range()].to_string());
                                }
                            }
                            enums.push(format!("        {} {{ {} }}", name, variants.join(", ")));
                        }
                    }
                }
                blocks.push(format!("    ENUMS [\n{}\n    ]", enums.join(",\n")));
            }
            "structs_block" => {
                let mut structs = Vec::new();
                let mut sb_cursor = block.walk();
                for struct_def in block.children(&mut sb_cursor) {
                    if struct_def.kind() == "struct_definition" {
                        if let Some(name_node) = struct_def.child(0) {
                            let name = &contents[name_node.byte_range()];
                            let mut fields = Vec::new();
                            let mut f_cursor = struct_def.walk();
                            for child in struct_def.children(&mut f_cursor) {
                                if child.kind() == "identifier" && child.id() != name_node.id() {
                                    fields.push(contents[child.byte_range()].to_string());
                                }
                            }
                            structs.push(format!("        {} {{\n            {}\n        }}", name, fields.join(",\n            ")));
                        }
                    }
                }
                blocks.push(format!("    STRUCTS [\n{}\n    ]", structs.join(",\n")));
            }
            "hubs_block" => {
                let mut hubs = Vec::new();
                let mut hb_cursor = block.walk();
                for hub_def in block.children(&mut hb_cursor) {
                    if hub_def.kind() == "hub_definition" {
                        if let Some(name_node) = hub_def.child(0) {
                            let name = &contents[name_node.byte_range()];
                            let mut items = Vec::new();
                            let mut item_cursor = hub_def.walk();
                            for child in hub_def.children(&mut item_cursor) {
                                match child.kind() {
                                    "hub_field" => {
                                        items.push(format!("            {}", &contents[child.byte_range()]));
                                    }
                                    "hub_role" => {
                                        items.push(format!("            {}", &contents[child.byte_range()]));
                                    }
                                    _ => {}
                                }
                            }
                            hubs.push(format!("        {} {{\n{}\n        }}", name, items.join(",\n")));
                        }
                    }
                }
                blocks.push(format!("    HUBS [\n{}\n    ]", hubs.join(",\n")));
            }
            _ => {}
        }
    }
    format!("DEFINITIONS [\n{}\n]", blocks.join(",\n"))
}

fn format_instances_section(node: Node, contents: &str) -> String {
    let mut instances = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "instance_block" {
            if let (Some(ref_node), Some(type_node)) = (child.child_by_field_name("ref"), child.child_by_field_name("type")) {
                let ref_name = &contents[ref_node.byte_range()];
                let type_name = &contents[type_node.byte_range()];
                
                let mut assignments = Vec::new();
                let mut max_ident_len = 0;
                let mut block_cursor = child.walk();
                for assign in child.children(&mut block_cursor) {
                    if assign.kind() == "instance_assignment" {
                        if let Some(id_node) = assign.child(0) {
                            let attr_name = contents[id_node.byte_range()].trim().to_string();
                            if let Some(expr_node) = assign.child(2) {
                                let val_str = contents[expr_node.byte_range()].trim().to_string();
                                max_ident_len = max_ident_len.max(attr_name.len());
                                assignments.push((attr_name, val_str));
                            }
                        }
                    }
                }
                
                let mut formatted_assigns = Vec::new();
                for (name, val) in assignments {
                    let padding = " ".repeat(max_ident_len - name.len());
                    formatted_assigns.push(format!("        {}{} = {}", name, padding, val));
                }
                
                let inner = if formatted_assigns.is_empty() {
                    String::new()
                } else {
                    format!("\n{}\n    ", formatted_assigns.join(",\n"))
                };
                
                instances.push(format!("    {}: {} {{{}}}", ref_name, type_name, inner));
            }
        }
    }
    format!("INSTANCES [\n{}\n]", instances.join(",\n"))
}


