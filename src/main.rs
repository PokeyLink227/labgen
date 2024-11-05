use rand;
use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::BufWriter;
use std::ops::{Add, AddAssign};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
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
struct Vector2<T> {
    x: T,
    y: T,
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorRGB {
    red: u8,
    green: u8,
    blue: u8,
}

impl ColorRGB {
    fn as_bytes(color_array: &[ColorRGB]) -> Vec<u8> {
        let mut byte_array: Vec<u8> = vec![0; color_array.len() * 3];

        let mut pos = 0;
        for pixel in color_array {
            byte_array[pos + 0] = pixel.red;
            byte_array[pos + 1] = pixel.green;
            byte_array[pos + 2] = pixel.blue;
            pos += 3;
        }

        return byte_array;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionStatus {
    UnVisited,
    Visited,
    InMaze,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Debug, Clone, Copy)]
struct Tile {
    status: ConnectionStatus,
    connections: [bool; 4],
}

#[derive(Debug)]
struct Grid {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl Grid {
    fn get_tile(&self, pos: Vector2<u32>) -> Tile {
        self.tiles[(pos.x + pos.y * self.width) as usize]
    }

    fn set_tile(&mut self, pos: Vector2<u32>, new: Tile) {
        self.tiles[(pos.x + pos.y * self.width) as usize] = new;
    }

    fn contains(&self, pt: Point) -> bool {
        pt.x >= 0 && (pt.x as u32) < self.width && pt.y >= 0 && (pt.y as u32) < self.height
    }

    fn get_tile_pt(&self, pos: Point) -> Tile {
        assert!(self.contains(pos));
        self.tiles[(pos.x as u32 + pos.y as u32 * self.width) as usize]
    }

    fn get_tile_mut(&mut self, pos: Point) -> &mut Tile {
        assert!(self.contains(pos));
        &mut self.tiles[(pos.x as u32 + pos.y as u32 * self.width) as usize]
    }

    fn set_tile_pt(&mut self, pos: Point, new: Tile) {
        assert!(self.contains(pos));
        self.tiles[(pos.x as u32 + pos.y as u32 * self.width) as usize] = new;
    }
}

fn pick_random(points: &[(usize, Point)]) -> Option<(usize, Point)> {
    if points.len() > 0 {
        Some(points[rand::random::<usize>() % points.len()])
    } else {
        None
    }
}

fn create_maze_backtrack(maze_size: Vector2<u32>) -> Grid {
    let blank: Tile = Tile {
        status: ConnectionStatus::UnVisited,
        connections: [false, false, false, false],
    };
    let num_tiles = maze_size.x * maze_size.y;

    let mut maze: Grid = Grid {
        tiles: vec![blank; num_tiles as usize],
        width: maze_size.x,
        height: maze_size.y,
    };
    let mut stack: Vec<Point> = Vec::new();
    let mut num_visited = 0;
    let mut current_pos: Point = Point { x: 0, y: 0 };

    maze.get_tile_mut(current_pos).status = ConnectionStatus::InMaze;
    stack.push(current_pos);
    num_visited += 1;

    while num_visited < num_tiles {
        let next = pick_random(
            current_pos
                .adjacent()
                .into_iter()
                .enumerate()
                .filter(|(_, x)| {
                    maze.contains(*x) && maze.get_tile_pt(*x).status == ConnectionStatus::UnVisited
                })
                .collect::<Vec<(usize, Point)>>()
                .as_ref(),
        );

        match next {
            None => {
                current_pos = stack.pop().unwrap();
            }
            Some(next) => {
                maze.get_tile_mut(current_pos).connections[next.0] = true;

                current_pos = next.1;
                maze.get_tile_mut(current_pos).connections[(next.0 + 2) % 4] = true;
                maze.get_tile_mut(current_pos).status = ConnectionStatus::InMaze;

                stack.push(current_pos);
                num_visited += 1;
            }
        }
    }

    maze
}

fn get_valid_adj(maze: &Grid, noise_map: &Vec<u8>, pos: Vector2<u32>, valid: u8) -> Option<usize> {
    let mut adj_tiles: Vec<usize> = Vec::with_capacity(4);

    if pos.y >= 1
        && noise_map[((pos.x + 0) + (pos.y - 1) * maze.width) as usize] == valid
        && maze
            .get_tile(Vector2 {
                x: pos.x + 0,
                y: pos.y - 1,
            })
            .status
            == ConnectionStatus::UnVisited
    {
        adj_tiles.push(0);
    }
    if pos.x < maze.width - 1
        && noise_map[((pos.x + 1) + (pos.y + 0) * maze.width) as usize] == valid
        && maze
            .get_tile(Vector2 {
                x: pos.x + 1,
                y: pos.y + 0,
            })
            .status
            == ConnectionStatus::UnVisited
    {
        adj_tiles.push(1);
    }
    if pos.y < maze.height - 1
        && noise_map[((pos.x + 0) + (pos.y + 1) * maze.width) as usize] == valid
        && maze
            .get_tile(Vector2 {
                x: pos.x + 0,
                y: pos.y + 1,
            })
            .status
            == ConnectionStatus::UnVisited
    {
        adj_tiles.push(2);
    }
    if pos.x >= 1
        && noise_map[((pos.x - 1) + (pos.y + 0) * maze.width) as usize] == valid
        && maze
            .get_tile(Vector2 {
                x: pos.x - 1,
                y: pos.y + 0,
            })
            .status
            == ConnectionStatus::UnVisited
    {
        adj_tiles.push(3);
    }

    if adj_tiles.len() == 0 {
        None
    } else {
        Some(adj_tiles[rand::random::<usize>() % adj_tiles.len()])
    }
}

fn flood_tile_prim(maze: &mut Grid, noise_map: &Vec<u8>, pos: Vector2<u32>) {
    if pos.x >= maze.width || pos.y >= maze.height {
        return;
    }
    if noise_map[(pos.x + pos.y * maze.width) as usize] != 0 {
        return;
    }
    if maze.tiles[(pos.x + pos.y * maze.width) as usize].status != ConnectionStatus::UnVisited {
        return;
    }

    let directions = [
        Vector2 { x: 0, y: -1 },
        Vector2 { x: 1, y: 0 },
        Vector2 { x: 0, y: 1 },
        Vector2 { x: -1, y: 0 },
    ];

    let mut open_tiles: Vec<Vector2<u32>> = Vec::new();
    let mut rng = thread_rng();

    open_tiles.push(pos);
    maze.tiles[(pos.x + pos.y * maze.width) as usize].status = ConnectionStatus::InMaze;

    while open_tiles.len() > 0 {
        let current_tile_index: usize = rng.gen_range(0..open_tiles.len());
        let current_pos: Vector2<u32> = open_tiles[current_tile_index];

        // calculate the number of adjacent and valid empty tiles
        match get_valid_adj(maze, noise_map, open_tiles[current_tile_index], 0) {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(dir) => {
                let new_pos = current_pos.add_offset(directions[dir]);
                maze.tiles[(new_pos.x + new_pos.y * maze.width) as usize].status =
                    ConnectionStatus::InMaze;
                open_tiles.push(new_pos);

                maze.tiles[(current_pos.x + current_pos.y * maze.width) as usize].connections
                    [dir] = true;
                maze.tiles[(new_pos.x + new_pos.y * maze.width) as usize].connections
                    [(dir + 2) % 4] = true;
            }
        }
    }
}

fn flood_tile_backtrack(maze: &mut Grid, noise_map: &Vec<u8>, pos: Vector2<u32>) {
    if pos.x >= maze.width || pos.y >= maze.height {
        return;
    }
    if noise_map[(pos.x + pos.y * maze.width) as usize] != 1 {
        return;
    }
    if maze.tiles[(pos.x + pos.y * maze.width) as usize].status != ConnectionStatus::UnVisited {
        return;
    }

    let directions = [
        Vector2 { x: 0, y: -1 },
        Vector2 { x: 1, y: 0 },
        Vector2 { x: 0, y: 1 },
        Vector2 { x: -1, y: 0 },
    ];

    let mut tile_stack: Vec<Vector2<u32>> = Vec::new();

    tile_stack.push(pos);
    maze.tiles[(pos.x + pos.y * maze.width) as usize].status = ConnectionStatus::InMaze;

    while !tile_stack.is_empty() {
        let current_pos: Vector2<u32> = *tile_stack.last().unwrap();

        match get_valid_adj(maze, noise_map, current_pos, 1) {
            None => {
                tile_stack.pop();
            }
            Some(dir) => {
                let new_pos = current_pos.add_offset(directions[dir]);
                maze.tiles[(new_pos.x + new_pos.y * maze.width) as usize].status =
                    ConnectionStatus::InMaze;
                tile_stack.push(new_pos);

                maze.tiles[(current_pos.x + current_pos.y * maze.width) as usize].connections
                    [dir] = true;
                maze.tiles[(new_pos.x + new_pos.y * maze.width) as usize].connections
                    [(dir + 2) % 4] = true;
            }
        }
    }
}

fn gen_maze(size: Vector2<u32>) -> Grid {
    let mut maze = Grid {
        tiles: vec![
            Tile {
                status: ConnectionStatus::UnVisited,
                connections: [false, false, false, false]
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

    for y in 0..size.y {
        for x in 0..size.x {
            flood_tile_prim(&mut maze, &noise_map, Vector2 { x: x, y: y });
            flood_tile_backtrack(&mut maze, &noise_map, Vector2 { x: x, y: y });
        }
    }

    /*
        need to add random stopping and then also implement connecting of maze regions
    */

    maze
}

fn generate_image(maze: &Grid) {
    let cell_width = 2;
    let image_dimensions = Vector2 {
        x: maze.width * cell_width + 1,
        y: maze.height * cell_width + 1,
    };

    let path = Path::new(r"./image.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image_dimensions.x, image_dimensions.y);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![
        ColorRGB {
            red: 0,
            green: 0,
            blue: 0
        };
        (image_dimensions.x * image_dimensions.y) as usize
    ];

    for y in 0..maze.height {
        for x in 0..maze.width {
            pixels[((x * cell_width + 1) + ((y * cell_width + 1) * image_dimensions.x)) as usize] =
                ColorRGB {
                    red: 255,
                    green: 255,
                    blue: 255,
                };
            if maze.get_tile(Vector2 { x: x, y: y }).connections[Direction::North as usize] {
                pixels[((x * cell_width + 1) + ((y * cell_width + 0) * image_dimensions.x))
                    as usize] = ColorRGB {
                    red: 255,
                    green: 255,
                    blue: 255,
                };
            }
            if maze.get_tile(Vector2 { x: x, y: y }).connections[Direction::West as usize] {
                pixels[((x * cell_width + 0) + ((y * cell_width + 1) * image_dimensions.x))
                    as usize] = ColorRGB {
                    red: 255,
                    green: 255,
                    blue: 255,
                };
            }
        }
    }

    writer
        .write_image_data(&ColorRGB::as_bytes(&pixels))
        .unwrap();
}

fn normalize(v: Vector2<f32>) -> Vector2<f32> {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    Vector2 {
        x: v.x / len,
        y: v.y / len,
    }
}

fn get_color(val: f32) -> ColorRGB {
    ColorRGB {
        red: ((255 as f32) * (val + 1.0) / 2.0) as u8,
        green: ((255 as f32) * (val + 1.0) / 2.0) as u8,
        blue: ((255 as f32) * (val + 1.0) / 2.0) as u8,
    }
}

fn interpolate(a: f32, b: f32, s: f32) -> f32 {
    // a + (b - a) * s
    // a + (b - a) * s * s * (3.0 - s * 2.0)
    a + (b - a) * ((s * (s * 6.0 - 15.0) + 10.0) * s * s * s)
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

    points
}

fn main() {
    let maze_size = Vector2 { x: 3, y: 3 };
    //generate_noise(maze_size, Vector2 {x: 7, y: 7});
    let nodes = create_maze_backtrack(maze_size);
    generate_image(&nodes);

    println!("Successfully Generated Maze");
}
