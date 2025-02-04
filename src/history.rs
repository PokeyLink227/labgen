use crate::grid::{ConnectionStatus, Direction, Grid, Point, Rect, Tile};
use crate::maze::MazeWrap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeAction {
    Add(Point, Direction),
    Remove(Point, Direction),
    RemoveEdge(Point, Direction),
    //AddEdge(Point, Direction),
    AddTemp(Point, Direction),
    AddMarker(Point),
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
    marker_pos: Point,
}

impl MazeHistory {
    pub fn new(w: u16, h: u16, temps: bool) -> Self {
        Self {
            actions: Vec::new(),
            temp_cells: Vec::new(),
            maze_width: w,
            maze_height: h,
            log_temps: temps,
            marker_pos: Point::new(0, 0),
        }
    }

    pub fn with_size_hint(w: u16, h: u16, temps: bool, size: usize) -> Self {
        Self {
            actions: Vec::with_capacity(size),
            temp_cells: Vec::new(),
            maze_width: w,
            maze_height: h,
            log_temps: temps,
            marker_pos: Point::new(0, 0),
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

            /*
            let mut i = 0;
            while i < self.temp_cells.len() {
                if self.temp_cells[i].0 == new {
                    self.actions
                        .push(MazeAction::Remove(new, self.temp_cells[i].1));
                    self.temp_cells.swap_remove(i);
                } else if self.temp_cells[i].0.travel_wrapped(
                    self.temp_cells[i].1,
                    self.maze_width,
                    self.maze_height,
                ) == new
                {
                    self.actions.push(MazeAction::RemoveEdge(
                        self.temp_cells[i].0,
                        self.temp_cells[i].1,
                    ));
                    self.temp_cells[i].1 = Direction::NoDir;
                    i += 1;
                } else {
                    i += 1;
                }
            }
            */

            self.remove_temps_at_pos(new);

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

    pub fn place_marker(&mut self, pos: Point) {
        self.marker_pos = pos;
        self.actions.push(MazeAction::AddMarker(pos));
    }

    pub fn replace_marker(&mut self, pos: Point) {
        self.actions.push(MazeAction::StartFrame);
        self.actions
            .push(MazeAction::Add(self.marker_pos, Direction::NoDir));
        self.marker_pos = pos;
        self.actions.push(MazeAction::AddMarker(pos));
        self.actions.push(MazeAction::EndFrame);
    }

    pub fn replace_marker_temp(&mut self, pos: Point) {
        self.actions.push(MazeAction::StartFrame);
        self.actions
            .push(MazeAction::AddTemp(self.marker_pos, Direction::NoDir));
        self.marker_pos = pos;
        self.actions.push(MazeAction::AddMarker(pos));
        self.actions.push(MazeAction::EndFrame);
    }

    fn remove_temps_at_pos(&mut self, pos: Point) {
        let mut i = 0;
        while i < self.temp_cells.len() {
            if self.temp_cells[i].0 == pos {
                self.actions
                    .push(MazeAction::Remove(pos, self.temp_cells[i].1));
                self.temp_cells.swap_remove(i);
            } else if self.temp_cells[i].0.travel_wrapped(
                self.temp_cells[i].1,
                self.maze_width,
                self.maze_height,
            ) == pos
            {
                self.actions.push(MazeAction::RemoveEdge(
                    self.temp_cells[i].0,
                    self.temp_cells[i].1,
                ));
                self.temp_cells[i].1 = Direction::NoDir;
                i += 1;
            } else {
                i += 1;
            }
        }
    }

    pub fn move_marker(&mut self, dir: Direction) {
        self.actions.push(MazeAction::StartFrame);
        self.remove_temps_at_pos(self.marker_pos);
        self.actions.push(MazeAction::Add(self.marker_pos, dir));
        self.marker_pos = self.marker_pos.travel(dir);
        self.actions.push(MazeAction::AddMarker(self.marker_pos));
        self.actions.push(MazeAction::EndFrame);
    }

    pub fn move_marker_temp(&mut self, dir: Direction) {
        self.actions.push(MazeAction::StartFrame);
        self.actions.push(MazeAction::AddTemp(self.marker_pos, dir));
        self.temp_cells.push((self.marker_pos, dir));
        self.marker_pos = self.marker_pos.travel(dir);
        self.actions.push(MazeAction::AddMarker(self.marker_pos));
        self.actions.push(MazeAction::EndFrame);
    }

    pub fn remove_marker(&mut self) {
        self.actions.push(MazeAction::StartFrame);

        self.remove_temps_at_pos(self.marker_pos);
        self.actions
            .push(MazeAction::Add(self.marker_pos, Direction::NoDir));
        self.actions.push(MazeAction::EndFrame);
    }
}
