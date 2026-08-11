use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

pub fn utf16_idx_to_byte_idx(s: &str, utf16_idx: usize) -> usize {
    let mut utf16_current = 0;
    let mut byte_current = 0;
    for c in s.chars() {
        if utf16_current >= utf16_idx {
            break;
        }
        utf16_current += c.len_utf16();
        byte_current += c.len_utf8();
    }
    if utf16_current < utf16_idx {
        s.len()
    } else {
        byte_current
    }
}

pub fn byte_idx_to_utf16_idx(s: &str, byte_idx: usize) -> usize {
    let mut utf16_current = 0;
    let mut byte_current = 0;
    for c in s.chars() {
        if byte_current >= byte_idx {
            break;
        }
        utf16_current += c.len_utf16();
        byte_current += c.len_utf8();
    }
    utf16_current
}

pub trait RopeExt {
    fn position_to_byte_offset(&self, pos: Position) -> usize;
    fn byte_offset_to_position(&self, offset: usize) -> Position;
}

impl RopeExt for Rope {
    fn position_to_byte_offset(&self, pos: Position) -> usize {
        let line_idx = pos.line as usize;
        if line_idx >= self.len_lines() {
            return self.len_bytes();
        }
        let line_char_offset = self.line_to_char(line_idx);
        let line = self.line(line_idx);
        let line_str = line.to_string();
        let col_byte = utf16_idx_to_byte_idx(&line_str, pos.character as usize);
        self.char_to_byte(line_char_offset) + col_byte
    }

    fn byte_offset_to_position(&self, offset: usize) -> Position {
        let char_idx = self.byte_to_char(offset);
        let line_idx = self.char_to_line(char_idx);
        let line_char_offset = self.line_to_char(line_idx);
        let line = self.line(line_idx);
        let line_str = line.to_string();
        let _char_offset_in_line = char_idx - line_char_offset;
        let byte_offset_in_line = self.char_to_byte(char_idx) - self.char_to_byte(line_char_offset);
        let utf16_idx = byte_idx_to_utf16_idx(&line_str, byte_offset_in_line);
        Position::new(line_idx as u32, utf16_idx as u32)
    }
}
