use crate::maze::MazeWrap;
use regex::Regex;
use std::{
    array,
    cell::LazyCell,
    ops::{Add, AddAssign},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point {
    pub fn adjacent(self) -> array::IntoIter<Point, 4> {
        [
            self + Point { x: 0, y: -1 },
            self + Point { x: 1, y: 0 },
            self + Point { x: 0, y: 1 },
            self + Point { x: -1, y: 0 },
        ]
        .into_iter()
    }

    pub fn adjacent_wrapped(
        self,
        dir: MazeWrap,
        width: u16,
        height: u16,
    ) -> array::IntoIter<Point, 4> {
        [
            if self.y - 1 < 0 && (dir == MazeWrap::Full || dir == MazeWrap::Vertical) {
                Point::new(self.x, height as i16 - 1)
            } else {
                self + Point { x: 0, y: -1 }
            },
            if self.x + 1 >= width as i16 && (dir == MazeWrap::Full || dir == MazeWrap::Horizontal)
            {
                Point { x: 0, y: self.y }
            } else {
                self + Point { x: 1, y: 0 }
            },
            if self.y + 1 >= height as i16 && (dir == MazeWrap::Full || dir == MazeWrap::Vertical) {
                Point { x: self.x, y: 0 }
            } else {
                self + Point { x: 0, y: 1 }
            },
            if self.x - 1 < 0 && (dir == MazeWrap::Full || dir == MazeWrap::Horizontal) {
                Point::new(width as i16 - 1, self.y)
            } else {
                self + Point { x: -1, y: 0 }
            },
        ]
        .into_iter()
    }

    pub fn travel(self, dir: Direction) -> Self {
        match dir {
            Direction::NoDir => self,
            Direction::North => self + Point { x: 0, y: -1 },
            Direction::NorthEast => self + Point { x: 1, y: -1 },
            Direction::East => self + Point { x: 1, y: 0 },
            Direction::SouthEast => self + Point { x: 1, y: 1 },
            Direction::South => self + Point { x: 0, y: 1 },
            Direction::SouthWest => self + Point { x: -1, y: 1 },
            Direction::West => self + Point { x: -1, y: 0 },
            Direction::NorthWest => self + Point { x: -1, y: -1 },
        }
    }

    pub fn travel_wrapped(self, dir: Direction, width: u16, height: u16) -> Self {
        let mut new_pos = self.travel(dir);
        if new_pos.x < 0 {
            new_pos.x = width as i16 - 1;
        }
        if new_pos.y < 0 {
            new_pos.y = height as i16 - 1;
        }
        if new_pos.x >= width as i16 {
            new_pos.x = 0;
        }
        if new_pos.y >= height as i16 {
            new_pos.y = 0;
        }

        new_pos
    }

    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ParseRectError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
}

impl FromStr for Rect {
    type Err = ParseRectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new(r"\(\s*(-?\d+)\s*,\s*(-?\d+)\s*,\s*(-?\d+)\s*,\s*(-?\d+)\)").unwrap()
        });

        let caps = re.captures(s).ok_or(ParseRectError)?;

        return Ok(Rect {
            x: caps[1].parse().or(Err(ParseRectError))?,
            y: caps[2].parse().or(Err(ParseRectError))?,
            w: caps[3].parse().or(Err(ParseRectError))?,
            h: caps[4].parse().or(Err(ParseRectError))?,
        });
    }
}

impl Rect {
    pub fn new(x: i16, y: i16, w: i16, h: i16) -> Self {
        assert_ne!(w, 0);
        assert_ne!(h, 0);

        Rect { x, y, w, h }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    UnVisited,
    Visited,
    InMaze,
    Removed,
    Room,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    NoDir = 0b0000,
    North = 0b00000001,
    NorthEast = 0b00000010,
    East = 0b00000100,
    SouthEast = 0b00001000,
    South = 0b00010000,
    SouthWest = 0b00100000,
    West = 0b01000000,
    NorthWest = 0b010000000,
}

impl From<u8> for Direction {
    fn from(src: u8) -> Direction {
        match src {
            0b00000001 => Direction::North,
            0b00000010 => Direction::NorthEast,
            0b00000100 => Direction::East,
            0b00001000 => Direction::SouthEast,
            0b00010000 => Direction::South,
            0b00100000 => Direction::SouthWest,
            0b01000000 => Direction::West,
            0b10000000 => Direction::NorthWest,
            _ => Direction::NoDir,
        }
    }
}

impl Direction {
    pub fn opposite(self) -> Self {
        // only needed to mask when there were 4 bits being used
        // ((((self as u8) << 4) | ((self as u8) >> 4)) & 0b11111111).into()
        (((self as u8) << 4) | ((self as u8) >> 4)).into()
    }

    // constructs a direction by starting at north and rotation clockwise
    // until a desired direction is reached
    pub fn from_clock_cardinal(rot: u8) -> Self {
        (0b00000001 << (rot * 2)).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge(Point, Direction);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    pub status: ConnectionStatus,
    pub connections: u8,
}

impl Tile {
    pub fn connect(&mut self, dir: Direction) {
        self.connections |= dir as u8;
    }

    pub fn unconnect(&mut self, dir: Direction) {
        self.connections &= !(dir as u8);
    }

    pub fn connected(self, dir: Direction) -> bool {
        self.connections & dir as u8 != 0
    }

    pub fn set_connected(&mut self, dir: Direction) {
        self.connections = dir as u8;
    }

    pub fn carveable(self) -> bool {
        self.status == ConnectionStatus::UnVisited
    }

    pub fn uncarveable(self) -> bool {
        self.status == ConnectionStatus::Removed || self.status == ConnectionStatus::Room
    }

    pub fn count_connections(self) -> u8 {
        let mut num_connections = 0;

        for shift in 0..8 {
            num_connections += (self.connections >> shift) & 1;
        }

        num_connections
    }
}

#[derive(Debug)]
pub struct Grid {
    pub tiles: Vec<Tile>,
    pub width: u16,
    pub height: u16,
}

impl Grid {
    pub fn get_index(&self, pos: Point) -> usize {
        pos.x as usize + pos.y as usize * self.width as usize
    }

    pub fn contains(&self, pt: Point) -> bool {
        pt.x >= 0 && (pt.x as u16) < self.width && pt.y >= 0 && (pt.y as u16) < self.height
    }

    pub fn get_tile(&self, pos: Point) -> Tile {
        assert!(self.contains(pos), "{:?} out of bounds", pos);
        self.tiles[pos.x as usize + pos.y as usize * self.width as usize]
    }

    pub fn get_tile_mut(&mut self, pos: Point) -> &mut Tile {
        assert!(self.contains(pos), "{:?} out of bounds", pos);
        &mut self.tiles[pos.x as usize + pos.y as usize * self.width as usize]
    }

    pub fn set_tile(&mut self, pos: Point, new: Tile) {
        assert!(self.contains(pos), "{:?} out of bounds", pos);
        self.tiles[pos.x as usize + pos.y as usize * self.width as usize] = new;
    }
}
