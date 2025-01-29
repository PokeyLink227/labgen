use crate::maze::{Direction, MazeWrap, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeAction {
    Add(Point, Direction),
    Remove(Point, Direction),
    AddTemp(Point, Direction),
    //AddUnwrapped(Point, Direction),
}

#[derive(Debug, Clone, Default)]
pub struct MazeHistory {
    actions: Vec<MazeAction>,
    temp_cells: Vec<Point>,
    wrap: Option<MazeWrap>,
}

impl MazeHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size_hint(size: usize) -> Self {
        Self {
            actions: Vec::with_capacity(size),
            ..Self::default()
        }
    }

    pub fn get_actions(&self) -> &[MazeAction] {
        &self.actions
    }

    pub fn carve(&mut self, new: Point, from_direction: Direction) {
        self.actions.push(MazeAction::Add(new, from_direction));
    }

    pub fn add_cell(&mut self, new: Point) {
        self.actions.push(MazeAction::Add(new, Direction::NoDir));
    }

    pub fn uncarve(&mut self, pt: Point, direction: Direction) {
        self.actions.push(MazeAction::Remove(pt, direction));
    }

    pub fn remove_cell(&mut self, new: Point) {
        self.actions.push(MazeAction::Remove(new, Direction::NoDir));
    }
}
