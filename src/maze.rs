use rand;
use rand::{thread_rng, Rng};
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Copy, PartialEq)]
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl Vector2<u32> {
    fn add_offset(&self, other: Vector2<i32>) -> Vector2<u32> {
        Vector2 {
            x: ((self.x as i32) + other.x) as u32,
            y: ((self.y as i32) + other.y) as u32,
        }
    }
}

impl Vector2<f32> {
    fn dot(lhs: Vector2<f32>, rhs: Vector2<f32>) -> f32 {
        lhs.x * rhs.x + lhs.y * rhs.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ConnectionStatus {
    #[default]
    UnVisited,
    Visited,
    InMaze,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Direction {
    None = 0b0000,
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
            _ => Direction::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tile {
    pub status: ConnectionStatus,
    pub connections: u8,
}

#[derive(Debug)]
pub struct Grid {
    pub tiles: Vec<Tile>,
    pub width: u32,
    pub height: u32,
}

impl Grid {
    pub fn contains(&self, pt: Point) -> bool {
        pt.x >= 0 && (pt.x as u32) < self.width && pt.y >= 0 && (pt.y as u32) < self.height
    }

    pub fn get_tile(&self, pos: Point) -> Tile {
        assert!(self.contains(pos));
        self.tiles[(pos.x as u32 + pos.y as u32 * self.width) as usize]
    }

    pub fn get_tile_mut(&mut self, pos: Point) -> &mut Tile {
        assert!(self.contains(pos));
        &mut self.tiles[(pos.x as u32 + pos.y as u32 * self.width) as usize]
    }

    pub fn set_tile(&mut self, pos: Point, new: Tile) {
        assert!(self.contains(pos));
        self.tiles[(pos.x as u32 + pos.y as u32 * self.width) as usize] = new;
    }
}

fn opposite(src: u8) -> u8 {
    ((src << 2) | (src >> 2)) & 0b1111
}

fn pick_random(points: &[(usize, Point)]) -> Option<(usize, Point)> {
    if points.len() > 0 {
        Some(points[rand::random::<usize>() % points.len()])
    } else {
        None
    }
}

pub fn create_maze_backtrack(width: u32, height: u32) -> (Grid, Vec<(Point, Direction)>) {
    let blank: Tile = Tile {
        status: ConnectionStatus::UnVisited,
        connections: Direction::None as u8,
    };
    let num_tiles = width * height;

    let mut maze: Grid = Grid {
        tiles: vec![blank; num_tiles as usize],
        width: width,
        height: height,
    };
    let mut stack: Vec<Point> = Vec::new();
    let mut pos: Point = Point { x: 0, y: 0 };
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(num_tiles as usize);

    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    stack.push(pos);
    history.push((pos, Direction::None.into()));

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
        );

        match next {
            None => {
                pos = stack.pop().unwrap();
            }
            Some(next) => {
                let dir = 0b0001 << next.0;
                maze.get_tile_mut(pos).connections |= dir;

                pos = next.1;
                maze.get_tile_mut(pos).connections |= opposite(dir);
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                stack.push(pos);
                history.push((pos, opposite(dir).into()));
            }
        }
    }

    (maze, history)
}

pub fn create_maze_prim(width: u32, height: u32) -> (Grid, Vec<(Point, Direction)>) {
    let blank: Tile = Tile {
        status: ConnectionStatus::UnVisited,
        connections: Direction::None as u8,
    };
    let num_tiles = width * height;

    let mut maze: Grid = Grid {
        tiles: vec![blank; num_tiles as usize],
        width: width,
        height: height,
    };

    let mut open_tiles: Vec<Point> = Vec::new();
    let mut history: Vec<(Point, Direction)> = Vec::with_capacity(num_tiles as usize);
    let mut rng = thread_rng();
    let mut pos: Point = Point { x: 0, y: 0 };

    maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;
    open_tiles.push(pos);
    history.push((pos, Direction::None.into()));

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
        );

        match next {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(next) => {
                let dir = 0b0001 << next.0;
                maze.get_tile_mut(pos).connections |= dir;

                pos = next.1;
                maze.get_tile_mut(pos).connections |= opposite(dir);
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
                history.push((pos, opposite(dir).into()));
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

fn generate_noise(world_size: Vector2<u32>, grid_size: Vector2<u32>) -> Vec<f32> {
    // can over-estimate length and be fine
    let cell_size: Vector2<u32> = Vector2 {
        x: if world_size.x % (grid_size.x - 1) == 0 {
            world_size.x / (grid_size.x - 1)
        } else {
            world_size.x / (grid_size.x - 1) + 1
        },
        y: if world_size.y % (grid_size.y - 1) == 0 {
            world_size.y / (grid_size.y - 1)
        } else {
            world_size.y / (grid_size.y - 1) + 1
        },
    };

    let mut rng = thread_rng();
    let mut points: Vec<f32> = vec![0.0f32; (world_size.x * world_size.y) as usize];
    let mut grid: Vec<Vector2<f32>> = Vec::with_capacity((grid_size.x * grid_size.y) as usize);

    // fill grid with random direction vectors
    for _ in 0..(grid_size.x * grid_size.y) {
        grid.push(normalize(Vector2 {
            x: rng.gen_range(-1.0..=1.0),
            y: rng.gen_range(-1.0..=1.0),
        }));
    }

    // calculate perlin noise for each point in the world
    for y in 0..world_size.y {
        for x in 0..world_size.x {
            let grid_offset = Vector2 {
                x: x % cell_size.x,
                y: y % cell_size.y,
            };
            let grid_pos = Vector2 {
                x: x / cell_size.x,
                y: y / cell_size.y,
            };

            // offset vectors from each nearby grid point to current world point
            let offset_vectors: [Vector2<f32>; 4] = [
                Vector2 {
                    x: ((grid_offset.x) as f32),
                    y: ((grid_offset.y) as f32),
                },
                Vector2 {
                    x: -((cell_size.x - grid_offset.x) as f32),
                    y: ((grid_offset.y) as f32),
                },
                Vector2 {
                    x: ((grid_offset.x) as f32),
                    y: -((cell_size.y - grid_offset.y) as f32),
                },
                Vector2 {
                    x: -((cell_size.x - grid_offset.x) as f32),
                    y: -((cell_size.y - grid_offset.y) as f32),
                },
            ];

            // dot product of each offset vector and its respective direction vector
            let dots: [f32; 4] = [
                Vector2::dot(
                    grid[((grid_pos.x + 0) + (grid_pos.y + 0) * grid_size.x) as usize],
                    offset_vectors[0],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 1) + (grid_pos.y + 0) * grid_size.x) as usize],
                    offset_vectors[1],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 0) + (grid_pos.y + 1) * grid_size.x) as usize],
                    offset_vectors[2],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 1) + (grid_pos.y + 1) * grid_size.x) as usize],
                    offset_vectors[3],
                ),
            ];

            // calculate step for interpolation
            let step = Vector2 {
                x: (grid_offset.x as f32) / (cell_size.x as f32),
                y: (grid_offset.y as f32) / (cell_size.y as f32),
            };

            // interpolate over x and y direction
            let int_x1 = interpolate(dots[0], dots[1], step.x);
            let int_x2 = interpolate(dots[2], dots[3], step.x);
            let int_y = interpolate(int_x1, int_x2, step.y);

            // dot product will range from -cell_width to cell_width
            points[(x + y * world_size.x) as usize] = int_y / (cell_size.x as f32) * 1.5;
        }
    }

    for p in &mut points {
        *p = if *p <= 0.0 { -1.0 } else { 1.0 };
    }
    /*

    let path = Path::new(r"./noise.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, world_size.x as u32, world_size.y as u32);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![
        ColorRGB {
            red: 0,
            green: 0,
            blue: 0
        };
        (world_size.x * world_size.y) as usize
    ];

    for i in 0..(world_size.x * world_size.y) as usize {
        pixels[i] = get_color(points[i]);
    }

    writer
        .write_image_data(&ColorRGB::as_bytes(&pixels))
        .unwrap();
        */

    points
}

fn flood_tile_prim(maze: &mut Grid, noise_map: &Vec<u8>, mut pos: Point) {
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
    let mut rng = thread_rng();

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
        );

        match next {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(next) => {
                maze.get_tile_mut(pos).connections |= 0b0001 << next.0;

                pos = next.1;
                maze.get_tile_mut(pos).connections |= opposite(0b0001 << next.0);
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
            }
        }
    }
}

fn flood_tile_backtrack(maze: &mut Grid, noise_map: &Vec<u8>, mut pos: Point) {
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
        );

        match next {
            None => {
                // we can upwrap here because we ensure the stack is non-empty in the loop clause
                pos = tile_stack.pop().unwrap();
            }
            Some(next) => {
                maze.get_tile_mut(pos).connections |= 0b0001 << next.0;

                pos = next.1;
                maze.get_tile_mut(pos).connections |= opposite(0b0001 << next.0);
                maze.get_tile_mut(pos).status = ConnectionStatus::InMaze;

                tile_stack.push(pos);
            }
        }
    }
}

pub fn gen_maze(size: Vector2<u32>) -> Grid {
    let mut maze = Grid {
        tiles: vec![
            Tile {
                status: ConnectionStatus::UnVisited,
                connections: 0,
            };
            (size.x * size.y) as usize
        ],
        width: size.x,
        height: size.y,
    };
    let noise_map: Vec<u8> = generate_noise(size, Vector2 { x: 7, y: 7 })
        .iter()
        .map(|x| if *x < 0.0 { 0 } else { 1 })
        .collect();

    for y in 0..size.y as i16 {
        for x in 0..size.x as i16 {
            flood_tile_prim(&mut maze, &noise_map, Point { x, y });
            flood_tile_backtrack(&mut maze, &noise_map, Point { x, y });
        }
    }

    /*
        need to add random stopping and then also implement connecting of maze regions
    */

    maze
}
