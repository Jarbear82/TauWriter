use crate::parser::Block;

pub(crate) fn find_block_type_by_id(blocks: &[Block], target_id: &str) -> Option<&'static str> {
    for block in blocks {
        match block {
            Block::Heading { id, .. } if id.as_deref() == Some(target_id) => {
                return Some("Heading")
            }
            Block::Paragraph { id, .. } if id.as_deref() == Some(target_id) => {
                return Some("Paragraph")
            }
            Block::BlockQuote { id, .. } if id.as_deref() == Some(target_id) => {
                return Some("BlockQuote")
            }
            Block::Aside { id, .. } if id.as_deref() == Some(target_id) => return Some("Aside"),
            Block::CodeBlock { id, .. } if id.as_deref() == Some(target_id) => {
                return Some("CodeBlock")
            }
            Block::List { id, .. } if id.as_deref() == Some(target_id) => return Some("List"),
            Block::DescriptionList { id, .. } if id.as_deref() == Some(target_id) => {
                return Some("DescriptionList")
            }
            Block::Table { id, .. } if id.as_deref() == Some(target_id) => return Some("Table"),
            Block::HorizontalRule { id, .. } if id.as_deref() == Some(target_id) => {
                return Some("HorizontalRule")
            }
            Block::Image { id, .. } if id.as_deref() == Some(target_id) => return Some("Image"),
            Block::Audio { id, .. } if id.as_deref() == Some(target_id) => return Some("Audio"),
            Block::Video { id, .. } if id.as_deref() == Some(target_id) => return Some("Video"),
            Block::Details {
                id, blocks: inner, ..
            } => {
                if id.as_deref() == Some(target_id) {
                    return Some("Details");
                }
                if let Some(t) = find_block_type_by_id(inner, target_id) {
                    return Some(t);
                }
            }
            Block::Footnote { id, .. } if id == target_id => return Some("Footnote"),
            Block::Review {
                id, blocks: inner, ..
            } => {
                if id.as_deref() == Some(target_id) {
                    return Some("Review");
                }
                if let Some(t) = find_block_type_by_id(inner, target_id) {
                    return Some(t);
                }
            }
            Block::Include {
                id, resolved_blocks: Some(inner), ..
            } => {
                if id.as_deref() == Some(target_id) {
                    return Some("Include");
                }
                if let Some(t) = find_block_type_by_id(inner, target_id) {
                    return Some(t);
                }
            }
            Block::Include { id, .. } if id.as_deref() == Some(target_id) => return Some("Include"),
            _ => {}
        }
    }
    None
}

pub(crate) fn find_block_range_by_id(
    blocks: &[Block],
    target_id: &str,
) -> Option<std::ops::Range<usize>> {
    for block in blocks {
        match block {
            Block::Heading { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Paragraph { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::BlockQuote { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Aside { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::CodeBlock { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::List { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::DescriptionList { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Table { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::HorizontalRule { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Image { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Audio { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Video { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
            Block::Details {
                id,
                blocks: inner,
                range,
                ..
            } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
                if let Some(r) = find_block_range_by_id(inner, target_id) {
                    return Some(r);
                }
            }
            Block::Footnote { id, range, .. } => {
                if id == target_id {
                    return range.clone();
                }
            }
            Block::Review {
                id,
                blocks: inner,
                range,
                ..
            } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
                if let Some(r) = find_block_range_by_id(inner, target_id) {
                    return Some(r);
                }
            }
            Block::Include {
                id,
                resolved_blocks: Some(inner),
                range,
                ..
            } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
                if let Some(r) = find_block_range_by_id(inner, target_id) {
                    return Some(r);
                }
            }
            Block::Include { id, range, .. } => {
                if id.as_deref() == Some(target_id) {
                    return range.clone();
                }
            }
        }
    }
    None
}

pub(crate) fn offset_to_position(
    text: &str,
    offset: usize,
) -> Option<gpui_component::input::Position> {
    let mut row = 0;
    let mut col = 0;
    for (i, c) in text.char_indices() {
        if i >= offset {
            return Some(gpui_component::input::Position::new(row, col));
        }
        if c == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Some(gpui_component::input::Position::new(row, col))
}
