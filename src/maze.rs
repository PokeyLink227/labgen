use rand::{seq::SliceRandom, Rng};
use std::{
    array,
    ops::{Add, AddAssign},
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
            Direction::East => self + Point { x: 1, y: 0 },
            Direction::South => self + Point { x: 0, y: 1 },
            Direction::West => self + Point { x: -1, y: 0 },
        }
    }

    pub fn travel_wrapped(self, dir: Direction, width: u16, height: u16) -> Self {
        match dir {
            Direction::NoDir => self,
            Direction::North => {
                if self.y - 1 < 0 {
                    Point::new(self.x, height as i16 - 1)
                } else {
                    self + Point { x: 0, y: -1 }
                }
            }
            Direction::East => {
                if self.x + 1 >= width as i16 {
                    Point { x: 0, y: self.y }
                } else {
                    self + Point { x: 1, y: 0 }
                }
            }
            Direction::South => {
                if self.y + 1 >= height as i16 {
                    Point { x: self.x, y: 0 }
                } else {
                    self + Point { x: 0, y: 1 }
                }
            }
            Direction::West => {
                if self.x - 1 < 0 {
                    Point::new(width as i16 - 1, self.y)
                } else {
                    self + Point { x: -1, y: 0 }
                }
            }
        }
        /*
                match dir {
                    Direction::NoDir => self,
                    Direction::North => self + Point { x: 0, y: -1 },
                    Direction::East => self + Point { x: 1, y: 0 },
                    Direction::South => self + Point { x: 0, y: 1 },
                    Direction::West => self + Point { x: -1, y: 0 },
                }
        */
    }

    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub w: i16,
    pub h: i16,
}

impl Rect {
    pub fn new(x: i16, y: i16, w: i16, h: i16) -> Self {
        assert_ne!(w, 0);
        assert_ne!(h, 0);

        Rect { x, y, w, h }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vector2<T> {
    x: T,
    y: T,
}

impl Vector2<f32> {
    fn dot(lhs: Vector2<f32>, rhs: Vector2<f32>) -> f32 {
        lhs.x * rhs.x + lhs.y * rhs.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    UnVisited,
    Visited,
    InMaze,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[repr(u8)]
pub enum MazeType {
    #[default]
    Backtrack,
    Prim,
    BinaryTree,
    Sidewinder,
    Noise,
    GrowingTree,
    Wilson,
    Kruskal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[repr(u8)]
pub enum MazeWrap {
    #[default]
    Full,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    NoDir = 0b0000,
    North = 0b0001,
    East = 0b0010,
    South = 0b0100,
    West = 0b1000,
}

impl From<u8> for Direction {
    fn from(src: u8) -> Direction {
        match src {
            0b0001 => Direction::North,
            0b0010 => Direction::East,
            0b0100 => Direction::South,
            0b1000 => Direction::West,
            _ => Direction::NoDir,
        }
    }
}

impl Direction {
    pub fn opposite(self) -> Self {
        ((((self as u8) << 2) | ((self as u8) >> 2)) & 0b1111).into()
        /*
                match self {
                    Direction::North => Direction::South,
                    Direction::East => Direction::West,
                    Direction::South => Direction::North,
                    Direction::West => Direction::East,
                    Direction::NoDir => Direction::NoDir,
                }
        */
    }

    // constructs a direction by starting at north and rotation clockwise
    // until a desired direction is reached
    pub fn from_clock(rot: u8) -> Self {
        (0b00000001 << rot).into()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    pub status: ConnectionStatus,
    pub connections: u8,
}

impl Tile {
    pub fn connect(&mut self, dir: Direction) {
        self.connections |= dir as u8;
    }

    pub fn connected(&self, dir: Direction) -> bool {
        self.connections & dir as u8 != 0
    }

    pub fn set_connected(&mut self, dir: Direction) {
        self.connections = dir as u8;
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
        assert!(self.contains(pos));
        self.tiles[pos.x as usize + pos.y as usize * self.width as usize]
    }

    pub fn get_tile_mut(&mut self, pos: Point) -> &mut Tile {
        assert!(self.contains(pos));
        &mut self.tiles[pos.x as usize + pos.y as usize * self.width as usize]
    }

    pub fn set_tile(&mut self, pos: Point, new: Tile) {
        assert!(self.contains(pos));
        self.tiles[pos.x as usize + pos.y as usize * self.width as usize] = new;
    }
}

/*
fn opposite(src: u8) -> u8 {
    ((src << 2) | (src >> 2)) & 0b1111
}
*/

fn pick_random(points: &[(usize, Point)], rng: &mut impl Rng) -> Option<(usize, Point)> {
    if points.len() > 0 {
        Some(points[rng.gen_range(0..points.len())])
    } else {
        None
    }
}

pub fn generate_maze(
    width: u16,
    height: u16,
    mtype: MazeType,
    wrap: Option<MazeWrap>,
    rooms: &[Rect],
    exclusions: &[Rect],
    rng: &mut impl Rng,
) -> (Grid, Vec<(Point, Direction)>) {
    let mut maze: Grid = Grid {
        tiles: vec![Tile::default(); width as usize * height as usize],
        width,
        height,
    };

    // remove all exclusions from the maze
    for r in exclusions {
        for y in r.y..(r.y + r.h) {
            for x in r.x..(r.x + r.w) {
                maze.set_tile(
                    Point::new(x, y),
                    Tile {
                        status: ConnectionStatus::Removed,
                        connections: Direction::NoDir as u8,
                    },
                );
            }
        }
    }

    match mtype {
        MazeType::Backtrack => create_maze_backtrack(maze, wrap, rng),
        MazeType::Prim => create_maze_prim(maze, wrap, rng),
        MazeType::BinaryTree => create_maze_binary(maze, rng),
        MazeType::Sidewinder => create_maze_sidewinder(maze, rng),
        MazeType::Noise => create_maze_noise(maze, rng),
        MazeType::GrowingTree => create_maze_growingtree(maze, wrap, GrowingTreeBias::Newest, rng),
        MazeType::Wilson => create_maze_wilson(maze, wrap, rng),
        MazeType::Kruskal => create_maze_kruskal(maze, rng),
    }
}

fn create_maze_backtrack(
    mut maze: Grid,
    wrap: Option<MazeWrap>,
    rng: &mut impl Rng,
) -> (Grid, Vec<(Point, Direction)>) {
    let mut stack: Vec<Point> = Vec::new();
    let mut pos: Point = Point::new(
        rng.gen_range(0..maze.width) as i16,
        rng.gen_range(0..maze.height) as i16,
    );
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len());

    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    stack.push(pos);
    history.push((pos, Direction::NoDir.into()));

    while !stack.is_empty() {
        let adj = match wrap {
            Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
            None => pos.adjacent(),
        };
        let next = adj
            .enumerate()
            .filter(|(_, x)| {
                maze.contains(*x) && maze.get_tile(*x).status == ConnectionStatus::UnVisited
            })
            .collect::<Vec<(usize, Point)>>()
            .choose(rng)
            .copied();

        match next {
            None => {
                pos = stack.pop().unwrap();
            }
            Some(next) => {
                let dir = Direction::from_clock(next.0 as u8);
                maze.get_tile_mut(pos).connect(dir);

                pos = next.1;
                maze.get_tile_mut(pos).connect(dir.opposite());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                stack.push(pos);
                history.push((pos, dir.opposite()));
            }
        }
    }

    (maze, history)
}

fn create_maze_prim(
    mut maze: Grid,
    wrap: Option<MazeWrap>,
    rng: &mut impl Rng,
) -> (Grid, Vec<(Point, Direction)>) {
    let mut open_tiles: Vec<Point> = Vec::new();
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len());
    let mut pos: Point = Point::new(
        rng.gen_range(0..maze.width) as i16,
        rng.gen_range(0..maze.height) as i16,
    );

    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    open_tiles.push(pos);
    history.push((pos, Direction::NoDir.into()));

    while !open_tiles.is_empty() {
        let current_tile_index: usize = rng.gen_range(0..open_tiles.len());
        pos = open_tiles[current_tile_index];

        let adj = match wrap {
            Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
            None => pos.adjacent(),
        };
        let next = adj
            .enumerate()
            .filter(|(_, x)| {
                maze.contains(*x) && maze.get_tile(*x).status == ConnectionStatus::UnVisited
            })
            .collect::<Vec<(usize, Point)>>()
            .choose(rng)
            .copied();

        match next {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(next) => {
                let dir = Direction::from_clock(next.0 as u8);
                maze.get_tile_mut(pos).connect(dir);

                pos = next.1;
                maze.get_tile_mut(pos).connect(dir.opposite());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
                history.push((pos, dir.opposite()));
            }
        }
    }

    (maze, history)
}

fn create_maze_binary(mut maze: Grid, rng: &mut impl Rng) -> (Grid, Vec<(Point, Direction)>) {
    use crate::maze::Direction::*;

    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len());

    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            let dir: u8 = if x > 0 && y > 0 {
                rng.gen_range(0..=1)
            } else if x > 0 {
                0
            } else if y > 0 {
                1
            } else {
                2
            };

            if dir == 0 {
                maze.get_tile_mut(Point::new(x, y)).connect(West);
                history.push((Point::new(x, y), West));
                maze.get_tile_mut(Point::new(x - 1, y)).connect(East);
            } else if dir == 1 {
                maze.get_tile_mut(Point::new(x, y)).connect(North);
                history.push((Point::new(x, y), North));
                maze.get_tile_mut(Point::new(x, y - 1)).connect(South);
            } else {
                history.push((Point::new(x, y), NoDir));
            }

            maze.get_tile_mut(Point::new(x, y)).status = ConnectionStatus::InMaze;
        }
    }

    (maze, history)
}

fn create_maze_sidewinder(mut maze: Grid, rng: &mut impl Rng) -> (Grid, Vec<(Point, Direction)>) {
    use crate::maze::Direction::*;

    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len() * 3 / 2);

    maze.get_tile_mut(Point { x: 0, y: 0 }).connect(East);
    maze.get_tile_mut(Point::new(0, 0)).status = ConnectionStatus::InMaze;
    history.push((Point { x: 0, y: 0 }, NoDir));

    for x in 1..(maze.width - 1) as i16 {
        maze.get_tile_mut(Point { x: x, y: 0 }).connections |= East as u8 | West as u8;
        maze.get_tile_mut(Point::new(x, 0)).status = ConnectionStatus::InMaze;
        history.push((Point { x: x, y: 0 }, West));
    }

    maze.get_tile_mut(Point::new((maze.width - 1) as i16, 0))
        .connect(West);
    maze.get_tile_mut(Point::new((maze.width - 1) as i16, 0))
        .status = ConnectionStatus::InMaze;
    history.push((Point::new((maze.width - 1) as i16, 0), West));

    for y in 1..maze.height as i16 {
        let mut range_start = 0;
        for x in 0..maze.width as i16 {
            if rng.gen::<bool>() && (x as u16) < maze.width - 1 {
                maze.get_tile_mut(Point::new(x, y)).connect(East);
                maze.get_tile_mut(Point::new(x, y)).status = ConnectionStatus::InMaze;
                maze.get_tile_mut(Point::new(x + 1, y)).connect(West);
                history.push((Point::new(x, y), East));
            } else {
                maze.get_tile_mut(Point::new(x, y)).status = ConnectionStatus::InMaze;

                if maze.get_tile(Point::new(x, y)).connected(West) {
                    history.push((Point::new(x, y), West));
                }

                let chosen = rng.gen_range(range_start..=x);
                maze.get_tile_mut(Point::new(chosen, y)).connect(North);
                maze.get_tile_mut(Point::new(chosen, y - 1)).connect(South);
                history.push((Point::new(chosen, y), North));

                range_start = x + 1;
            }
        }
    }

    (maze, history)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowingTreeBias {
    Oldest,
    Newest,
    Random,
    Percent(u8),
}

impl Default for GrowingTreeBias {
    fn default() -> Self {
        GrowingTreeBias::Percent(10)
    }
}

fn create_maze_growingtree(
    mut maze: Grid,
    wrap: Option<MazeWrap>,
    bias: GrowingTreeBias,
    rng: &mut impl Rng,
) -> (Grid, Vec<(Point, Direction)>) {
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len());
    let mut open: Vec<Point> = Vec::new();

    let pos = Point::new(
        rng.gen_range(0..maze.width) as i16,
        rng.gen_range(0..maze.height) as i16,
    );
    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    history.push((pos, Direction::NoDir));
    open.push(pos);

    while !open.is_empty() {
        let selected_index = match bias {
            GrowingTreeBias::Oldest => 0,              // lowest river factor
            GrowingTreeBias::Newest => open.len() - 1, // backtrack
            GrowingTreeBias::Random => rng.gen_range(0..open.len()), // similar to prim
            GrowingTreeBias::Percent(p) => {
                rng.gen_range((open.len() / 100 * (100 - p as usize))..open.len())
            }
        };
        let selected = open[selected_index];
        let adj = match wrap {
            Some(w) => selected.adjacent_wrapped(w, maze.width, maze.height),
            None => selected.adjacent(),
        };
        let next = adj
            .enumerate()
            .filter(|(_, x)| {
                maze.contains(*x) && maze.get_tile(*x).status == ConnectionStatus::UnVisited
            })
            .collect::<Vec<(usize, Point)>>()
            .choose(rng)
            .copied();

        match next {
            None => {
                open.remove(selected_index);
            }
            Some(next) => {
                let dir = Direction::from_clock(next.0 as u8);
                maze.get_tile_mut(selected).connect(dir);

                let selected = next.1;
                maze.get_tile_mut(selected).connect(dir.opposite());
                maze.get_tile_mut(selected).status = ConnectionStatus::InMaze;

                open.push(selected);
                history.push((selected, dir.opposite()));
            }
        }
    }

    (maze, history)
}

fn create_maze_wilson(
    mut maze: Grid,
    wrap: Option<MazeWrap>,
    rng: &mut impl Rng,
) -> (Grid, Vec<(Point, Direction)>) {
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len());
    let mut reservoir: Vec<Point> = Vec::with_capacity(maze.tiles.len());

    // generate reservoir
    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            reservoir.push(Point::new(x, y));
        }
    }
    for i in 0..reservoir.len() {
        let index = rng.gen_range(i..reservoir.len());
        let temp = reservoir[i];
        reservoir[i] = reservoir[index];
        reservoir[index] = temp;
    }

    let mut anchor = reservoir.pop().unwrap();
    while maze.get_tile(anchor).status != ConnectionStatus::UnVisited {
        anchor = match reservoir.pop() {
            Some(v) => v,
            None => return (maze, history),
        }
    }
    maze.get_tile_mut(anchor).status = ConnectionStatus::InMaze;
    history.push((anchor, Direction::NoDir));

    'outer: while !reservoir.is_empty() {
        // pick a cell not already in the maze
        while maze.get_tile(anchor).status != ConnectionStatus::UnVisited {
            anchor = match reservoir.pop() {
                Some(v) => v,
                None => break 'outer,
            }
        }
        let mut pos = anchor;

        // start a random loop erased walk from the chosen cell
        while maze.get_tile(pos).status == ConnectionStatus::UnVisited {
            let adj = match wrap {
                Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
                None => pos.adjacent(),
            };
            let next = adj
                .enumerate()
                .filter(|&(_, x)| maze.contains(x) && maze.get_tile(x).status != ConnectionStatus::Removed)
                .collect::<Vec<(usize, Point)>>()
                .choose(rng)
                .copied()
                .unwrap(); // safe to unwrap because a cell will always have at least 1 adjacent cell in the maze

            let dir = Direction::from_clock(next.0 as u8);
            maze.get_tile_mut(pos).set_connected(dir);
            pos = next.1;
        }

        // carve the final path into the maze
        pos = anchor;
        let mut dir = Direction::NoDir;
        while maze.get_tile(pos).status != ConnectionStatus::InMaze {
            // this line will panic if tile has multiple connections
            let temp_dir = maze.get_tile(pos).connections.into();
            maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
            maze.get_tile_mut(pos).connect(dir.opposite());
            dir = temp_dir;

            history.push((pos, dir));
            if wrap.is_some() {
                pos = pos.travel_wrapped(dir, maze.width, maze.height);
            } else {
                pos = pos.travel(dir);
            }
        }
        maze.get_tile_mut(pos).connect(dir.opposite());
    }

    (maze, history)
}

// merge_sets 60x faster than simple array and 600x faster with set_lookup_flatten
fn create_maze_kruskal(mut maze: Grid, rng: &mut impl Rng) -> (Grid, Vec<(Point, Direction)>) {
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len());
    let mut edges: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len() * 2);
    let mut region_map: Vec<u32> = (0..maze.tiles.len() as u32).collect();

    // generate edges
    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            if x > 0 {
                edges.push((Point::new(x, y), Direction::West));
            }
            if y > 0 {
                edges.push((Point::new(x, y), Direction::North));
            }
        }
    }
    // shuffle edges
    for i in 0..edges.len() {
        let index = rng.gen_range(i..edges.len());
        let temp = edges[i];
        edges[i] = edges[index];
        edges[index] = temp;
    }

    // generate maze
    for edge in edges {
        // if edge connects 2 different regions
        if merge_sets(
            &mut region_map,
            maze.get_index(edge.0),
            maze.get_index(edge.0.travel(edge.1)),
        ) {
            if maze.get_tile(edge.0).status != ConnectionStatus::InMaze {
                maze.get_tile_mut(edge.0).status = ConnectionStatus::InMaze;
            }
            history.push(edge);
            maze.get_tile_mut(edge.0).connect(edge.1);

            if maze.get_tile(edge.0.travel(edge.1)).status != ConnectionStatus::InMaze {
                maze.get_tile_mut(edge.0.travel(edge.1)).status = ConnectionStatus::InMaze;
                history.push((edge.0.travel(edge.1), Direction::NoDir));
            }
            maze.get_tile_mut(edge.0.travel(edge.1))
                .connect(edge.1.opposite());
        }
    }

    (maze, history)
}

// 10x faster than normal lookup
fn set_lookup_flatten(region_map: &mut [u32], node: usize) -> u32 {
    let mut node = node as u32;
    let mut root = node;
    // find root of set (normal lookup)
    while region_map[root as usize] != root {
        root = region_map[root as usize];
    }

    // update nodes in path to point directly to root
    while region_map[node as usize] != node {
        let parent = region_map[node as usize];
        region_map[node as usize] = root;
        node = parent;
    }

    node
}

// returns true if sets needed to be merged
fn merge_sets(region_map: &mut [u32], lhs: usize, rhs: usize) -> bool {
    // determine parent nodes of each set
    // can optimize by storing size of each set and picking the right one to merge or by flattening during lookup
    // either is a ~10x speedup over using neither, no additional perfomance from using both
    let lhs_parent = set_lookup_flatten(region_map, lhs);
    let rhs_parent = set_lookup_flatten(region_map, rhs);

    if lhs_parent == rhs_parent {
        return false;
    }

    region_map[lhs_parent as usize] = rhs_parent;

    true
}

fn interpolate(a: f32, b: f32, s: f32) -> f32 {
    // a + (b - a) * s
    // a + (b - a) * s * s * (3.0 - s * 2.0)
    a + (b - a) * ((s * (s * 6.0 - 15.0) + 10.0) * s * s * s)
}

fn normalize(v: Vector2<f32>) -> Vector2<f32> {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    Vector2 {
        x: v.x / len,
        y: v.y / len,
    }
}

fn generate_noise(
    world_width: u16,
    world_height: u16,
    grid_width: u16,
    grid_height: u16,
    rng: &mut impl Rng,
) -> Vec<f32> {
    // can over-estimate length and be fine
    let cell_width = if world_width % (grid_width - 1) == 0 {
        world_width / (grid_width - 1)
    } else {
        world_width / (grid_width - 1) + 1
    };
    let cell_height = if world_height % (grid_height - 1) == 0 {
        world_height / (grid_height - 1)
    } else {
        world_height / (grid_height - 1) + 1
    };

    let mut points: Vec<f32> = vec![0.0f32; (world_width * world_height) as usize];
    let mut grid: Vec<Vector2<f32>> = Vec::with_capacity((grid_width * grid_height) as usize);

    // fill grid with random direction vectors
    for _ in 0..(grid_width * grid_height) {
        grid.push(normalize(Vector2 {
            x: rng.gen_range(-1.0..=1.0),
            y: rng.gen_range(-1.0..=1.0),
        }));
    }

    // calculate perlin noise for each point in the world
    for y in 0..world_height {
        for x in 0..world_width {
            let grid_offset = Vector2 {
                x: x % cell_width,
                y: y % cell_height,
            };
            let grid_pos = Vector2 {
                x: x / cell_width,
                y: y / cell_height,
            };

            // offset vectors from each nearby grid point to current world point
            let offset_vectors: [Vector2<f32>; 4] = [
                Vector2 {
                    x: ((grid_offset.x) as f32),
                    y: ((grid_offset.y) as f32),
                },
                Vector2 {
                    x: -((cell_width - grid_offset.x) as f32),
                    y: ((grid_offset.y) as f32),
                },
                Vector2 {
                    x: ((grid_offset.x) as f32),
                    y: -((cell_height - grid_offset.y) as f32),
                },
                Vector2 {
                    x: -((cell_width - grid_offset.x) as f32),
                    y: -((cell_height - grid_offset.y) as f32),
                },
            ];

            // dot product of each offset vector and its respective direction vector
            let dots: [f32; 4] = [
                Vector2::dot(
                    grid[((grid_pos.x + 0) + (grid_pos.y + 0) * grid_width) as usize],
                    offset_vectors[0],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 1) + (grid_pos.y + 0) * grid_width) as usize],
                    offset_vectors[1],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 0) + (grid_pos.y + 1) * grid_width) as usize],
                    offset_vectors[2],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 1) + (grid_pos.y + 1) * grid_width) as usize],
                    offset_vectors[3],
                ),
            ];

            // calculate step for interpolation
            let step = Vector2 {
                x: (grid_offset.x as f32) / (cell_width as f32),
                y: (grid_offset.y as f32) / (cell_height as f32),
            };

            // interpolate over x and y direction
            let int_x1 = interpolate(dots[0], dots[1], step.x);
            let int_x2 = interpolate(dots[2], dots[3], step.x);
            let int_y = interpolate(int_x1, int_x2, step.y);

            // dot product will range from -cell_width to cell_width
            points[(x + y * world_width) as usize] = int_y / (cell_width as f32) * 1.5;
        }
    }

    for p in &mut points {
        *p = if *p <= 0.0 { -1.0 } else { 1.0 };
    }
    /*

    let path = Path::new(r"./noise.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, world_width as u32, world_height as u32);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![
        ColorRGB {
            red: 0,
            green: 0,
            blue: 0
        };
        (world_width * world_height) as usize
    ];

    for i in 0..(world_width * world_height) as usize {
        pixels[i] = get_color(points[i]);
    }

    writer
        .write_image_data(&ColorRGB::as_bytes(&pixels))
        .unwrap();
        */

    points
}

fn flood_tile_prim(maze: &mut Grid, noise_map: &Vec<u8>, mut pos: Point, rng: &mut impl Rng) {
    if pos.x >= maze.width as i16 || pos.y >= maze.height as i16 {
        return;
    }
    if noise_map[(pos.x + pos.y * maze.width as i16) as usize] != 0 {
        return;
    }
    if maze.tiles[(pos.x + pos.y * maze.width as i16) as usize].status
        != ConnectionStatus::UnVisited
    {
        return;
    }

    let mut open_tiles: Vec<Point> = Vec::new();

    open_tiles.push(pos);
    maze.tiles[(pos.x + pos.y * maze.width as i16) as usize].status = ConnectionStatus::InMaze;
    while !open_tiles.is_empty() {
        let current_tile_index: usize = rng.gen_range(0..open_tiles.len());
        pos = open_tiles[current_tile_index];

        let next = pick_random(
            pos.adjacent()
                .enumerate()
                .filter(|(_, x)| {
                    maze.contains(*x)
                        && maze.get_tile(*x).status == ConnectionStatus::UnVisited
                        && noise_map[(x.x + x.y * maze.width as i16) as usize] == 1
                })
                .collect::<Vec<(usize, Point)>>()
                .as_ref(),
            rng,
        );

        match next {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(next) => {
                let dir = Direction::from_clock(next.0 as u8);
                maze.get_tile_mut(pos).connect(dir);

                pos = next.1;
                maze.get_tile_mut(pos).connect(dir.opposite());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
            }
        }
    }
}

fn flood_tile_backtrack(maze: &mut Grid, noise_map: &Vec<u8>, mut pos: Point, rng: &mut impl Rng) {
    if pos.x >= maze.width as i16 || pos.y >= maze.height as i16 {
        return;
    }
    if noise_map[(pos.x + pos.y * maze.width as i16) as usize] != 1 {
        return;
    }
    if maze.tiles[(pos.x + pos.y * maze.width as i16) as usize].status
        != ConnectionStatus::UnVisited
    {
        return;
    }

    let mut tile_stack: Vec<Point> = Vec::new();

    tile_stack.push(pos);
    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

    while !tile_stack.is_empty() {
        let next = pick_random(
            pos.adjacent()
                .enumerate()
                .filter(|(_, x)| {
                    maze.contains(*x)
                        && maze.get_tile(*x).status == ConnectionStatus::UnVisited
                        && noise_map[(x.x + x.y * maze.width as i16) as usize] == 1
                })
                .collect::<Vec<(usize, Point)>>()
                .as_ref(),
            rng,
        );

        match next {
            None => {
                // we can upwrap here because we ensure the stack is non-empty in the loop clause
                pos = tile_stack.pop().unwrap();
            }
            Some(next) => {
                let dir = Direction::from_clock(next.0 as u8);
                maze.get_tile_mut(pos).connect(dir);

                pos = next.1;
                maze.get_tile_mut(pos).connect(dir.opposite());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                tile_stack.push(pos);
            }
        }
    }
}

fn create_maze_noise(mut maze: Grid, rng: &mut impl Rng) -> (Grid, Vec<(Point, Direction)>) {
    let noise_map: Vec<u8> = generate_noise(maze.width, maze.height, 7, 7, rng)
        .iter()
        .map(|x| if *x < 0.0 { 0 } else { 1 })
        .collect();

    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            flood_tile_prim(&mut maze, &noise_map, Point { x, y }, rng);
            flood_tile_backtrack(&mut maze, &noise_map, Point { x, y }, rng);
        }
    }

    /*
        need to add random stopping and then also implement connecting of maze regions
    */

    (maze, Vec::new())
}
