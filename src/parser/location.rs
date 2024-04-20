use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Location {
    pub file: Rc<str>,
    pub from: Position,
    pub to: Position
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub line: i32,
    pub column: i32
}

impl Location {
    pub fn extend(&self, pos: Position) -> Self {
        Location { file: self.file.clone(), from: self.from, to: pos }
    }
}