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
    pub fn extend(&self, loc: &Location) -> Self {
        Location {
            file: self.file.clone(),
            // TODO: Do we need to make this min of the two, and the other the max of the two?
            from: self.from,
            to: loc.to
        }
    }
}