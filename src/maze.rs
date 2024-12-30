use rand;
use rand::rngs::StdRng;
use rand::{Rng};
use std::ops::{Add, AddAssign};

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
    fn adjacent(self) -> [Point; 4] {
        [
            self + Point { x: 0, y: -1 },
            self + Point { x: 1, y: 0 },
            self + Point { x: 0, y: 1 },
            self + Point { x: -1, y: 0 },
        ]
    }

    fn new(x: i16, y: i16) -> Self {
        Self { x, y }
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    DirNone = 0b0000,
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
            _ => Direction::DirNone,
        }
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
}

#[derive(Debug)]
pub struct Grid {
    pub tiles: Vec<Tile>,
    pub width: u16,
    pub height: u16,
}

impl Grid {
    pub fn contains(&self, pt: Point) -> bool {
        pt.x >= 0 && (pt.x as u16) < self.width && pt.y >= 0 && (pt.y as u16) < self.height
    }

    pub fn get_tile(&self, pos: Point) -> Tile {
        assert!(self.contains(pos));
        self.tiles[(pos.x as u16 + pos.y as u16 * self.width) as usize]
    }

    pub fn get_tile_mut(&mut self, pos: Point) -> &mut Tile {
        assert!(self.contains(pos));
        &mut self.tiles[(pos.x as u16 + pos.y as u16 * self.width) as usize]
    }

    pub fn set_tile(&mut self, pos: Point, new: Tile) {
        assert!(self.contains(pos));
        self.tiles[(pos.x as u16 + pos.y as u16 * self.width) as usize] = new;
    }
}

fn opposite(src: u8) -> u8 {
    ((src << 2) | (src >> 2)) & 0b1111
}

fn pick_random(points: &[(usize, Point)], rng: &mut StdRng) -> Option<(usize, Point)> {
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
    rng: &mut StdRng,
) -> (Grid, Vec<(Point, Direction)>) {
    let num_tiles = width * height;

    let maze: Grid = Grid {
        tiles: vec![Tile::default(); num_tiles as usize],
        width: width,
        height: height,
    };

    match mtype {
        MazeType::Backtrack => create_maze_backtrack(maze, rng),
        MazeType::Prim => create_maze_prim(maze, rng),
        MazeType::BinaryTree => create_maze_binary(maze, rng),
        MazeType::Sidewinder => create_maze_sidewinder(maze, rng),
        MazeType::Noise => create_maze_noise(maze, rng),
    }
}

fn create_maze_backtrack(mut maze: Grid, rng: &mut StdRng) -> (Grid, Vec<(Point, Direction)>) {
    let num_tiles = maze.width * maze.height;

    let mut stack: Vec<Point> = Vec::new();
    let mut pos: Point = Point { x: 0, y: 0 };
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(num_tiles as usize);

    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    stack.push(pos);
    history.push((pos, Direction::DirNone.into()));

    while !stack.is_empty() {
        let next = pick_random(
            pos.adjacent()
                .into_iter()
                .enumerate()
                .filter(|(_, x)| {
                    maze.contains(*x) && maze.get_tile(*x).status == ConnectionStatus::UnVisited
                })
                .collect::<Vec<(usize, Point)>>()
                .as_ref(),
            rng,
        );

        match next {
            None => {
                pos = stack.pop().unwrap();
            }
            Some(next) => {
                let dir = 0b0001 << next.0;
                maze.get_tile_mut(pos).connect(dir.into());

                pos = next.1;
                maze.get_tile_mut(pos).connect(opposite(dir).into());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                stack.push(pos);
                history.push((pos, opposite(dir).into()));
            }
        }
    }

    (maze, history)
}

fn create_maze_prim(mut maze: Grid, rng: &mut StdRng) -> (Grid, Vec<(Point, Direction)>) {
    let num_tiles = maze.width * maze.height;

    let mut open_tiles: Vec<Point> = Vec::new();
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(num_tiles as usize);
    let mut pos: Point = Point { x: 0, y: 0 };

    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    open_tiles.push(pos);
    history.push((pos, Direction::DirNone.into()));

    while !open_tiles.is_empty() {
        let current_tile_index: usize = rng.gen_range(0..open_tiles.len());
        pos = open_tiles[current_tile_index];

        let next = pick_random(
            pos.adjacent()
                .into_iter()
                .enumerate()
                .filter(|(_, x)| {
                    maze.contains(*x) && maze.get_tile(*x).status == ConnectionStatus::UnVisited
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
                let dir = 0b0001 << next.0;
                maze.get_tile_mut(pos).connect(dir.into());

                pos = next.1;
                maze.get_tile_mut(pos).connect(opposite(dir).into());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
                history.push((pos, opposite(dir).into()));
            }
        }
    }

    (maze, history)
}

fn create_maze_binary(mut maze: Grid, rng: &mut StdRng) -> (Grid, Vec<(Point, Direction)>) {
    use crate::maze::Direction::*;

    let num_tiles = maze.width * maze.height;
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(num_tiles as usize);

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
                history.push((Point::new(x, y), DirNone));
            }
        }
    }

    (maze, history)
}

fn create_maze_sidewinder(mut maze: Grid, rng: &mut StdRng) -> (Grid, Vec<(Point, Direction)>) {
    use crate::maze::Direction::*;

    let num_tiles = maze.width * maze.height;
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(num_tiles as usize);

    maze.get_tile_mut(Point { x: 0, y: 0 }).connect(East);
    history.push((Point { x: 0, y: 0 }, DirNone));

    for x in 1..(maze.width - 1) as i16 {
        maze.get_tile_mut(Point { x: x, y: 0 }).connections |= East as u8 | West as u8;
        history.push((Point { x: x, y: 0 }, West));
    }

    maze.get_tile_mut(Point::new((maze.width - 1) as i16, 0))
        .connect(West);
    history.push((Point::new((maze.width - 1) as i16, 0), West));

    for y in 1..maze.height as i16 {
        let mut range_start = 0;
        for x in 0..maze.width as i16 {
            if rng.gen::<bool>() && (x as u16) < maze.width - 1 {
                maze.get_tile_mut(Point::new(x, y)).connect(East);
                maze.get_tile_mut(Point::new(x + 1, y)).connect(West);
                history.push((Point::new(x, y), East));
            } else {
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
    rng: &mut StdRng
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

fn flood_tile_prim(maze: &mut Grid, noise_map: &Vec<u8>, mut pos: Point, rng: &mut StdRng) {
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
                .into_iter()
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
                maze.get_tile_mut(pos).connect((0b0001 << next.0).into());

                pos = next.1;
                maze.get_tile_mut(pos)
                    .connect(opposite(0b0001 << next.0).into());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
            }
        }
    }
}

fn flood_tile_backtrack(maze: &mut Grid, noise_map: &Vec<u8>, mut pos: Point, rng: &mut StdRng) {
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
                .into_iter()
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
                maze.get_tile_mut(pos).connect((0b0001 << next.0).into());

                pos = next.1;
                maze.get_tile_mut(pos)
                    .connect(opposite(0b0001 << next.0).into());
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                tile_stack.push(pos);
            }
        }
    }
}

fn create_maze_noise(mut maze: Grid, rng: &mut StdRng) -> (Grid, Vec<(Point, Direction)>) {
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
