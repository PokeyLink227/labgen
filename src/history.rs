use crate::maze::{MazeWrap};
use crate::grid::{
    Point, Tile, Grid, Direction, Rect, ConnectionStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeAction {
    Add(Point, Direction),
    Remove(Point, Direction),
    RemoveEdge(Point, Direction),
    //AddEdge(Point, Direction),
    AddTemp(Point, Direction),
    StartFrame,
    EndFrame,
    //AddUnwrapped(Point, Direction),
}

#[derive(Debug, Clone)]
pub struct MazeHistory {
    actions: Vec<MazeAction>,
    temp_cells: Vec<(Point, Direction)>,
    maze_width: u16,
    maze_height: u16,
    log_temps: bool,
}

impl MazeHistory {
    pub fn new(w: u16, h: u16, temps: bool) -> Self {
        Self {
            actions: Vec::new(),
            temp_cells: Vec::new(),
            maze_width: w,
            maze_height: h,
            log_temps: temps,
        }
    }

    pub fn with_size_hint(w: u16, h: u16, temps: bool, size: usize) -> Self {
        Self {
            actions: Vec::with_capacity(size),
            temp_cells: Vec::new(),
            maze_width: w,
            maze_height: h,
            log_temps: temps,
        }
    }

    pub fn enable_temp_cells(&mut self) {
        self.log_temps = true;
    }

    pub fn get_actions(&self) -> &[MazeAction] {
        &self.actions
    }

    pub fn carve(&mut self, new: Point, from_direction: Direction) {
        if !self.temp_cells.is_empty() {
            self.actions.push(MazeAction::StartFrame);

            let mut i = 0;
            while i < self.temp_cells.len() {
                if self.temp_cells[i].0 == new {
                    self.actions
                        .push(MazeAction::Remove(new, self.temp_cells[i].1));
                    self.temp_cells.swap_remove(i);
                } else if self.temp_cells[i].0.travel_wrapped(self.temp_cells[i].1, self.maze_width, self.maze_height) == new {
                    self.actions.push(MazeAction::RemoveEdge(self.temp_cells[i].0, self.temp_cells[i].1));
                    self.temp_cells[i].1 = Direction::NoDir;
                    i += 1;
                } else {
                    i += 1;
                }
            }

            self.actions.push(MazeAction::Add(new, from_direction));
            self.actions.push(MazeAction::EndFrame);
        } else {
            self.actions.push(MazeAction::Add(new, from_direction));
        }
    }

    pub fn add_cell(&mut self, new: Point) {
        self.carve(new, Direction::NoDir);
    }

    pub fn uncarve(&mut self, pt: Point, direction: Direction) {
        self.actions.push(MazeAction::Remove(pt, direction));
    }

    pub fn remove_cell(&mut self, new: Point) {
        self.actions.push(MazeAction::Remove(new, Direction::NoDir));
    }

    pub fn carve_temp(&mut self, new: Point, from_direction: Direction) {
        if !self.log_temps {
            return;
        }

        self.actions.push(MazeAction::AddTemp(new, from_direction));
        self.temp_cells.push((new, from_direction));
    }

    pub fn remove_temp_cells(&mut self) {
        self.actions.push(MazeAction::StartFrame);
        for edge in self.temp_cells.drain(..) {
            self.actions.push(MazeAction::Remove(edge.0, edge.1));
        }
        self.actions.push(MazeAction::EndFrame);
    }
}
