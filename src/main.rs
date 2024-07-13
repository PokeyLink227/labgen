use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use rand;
use rand::{thread_rng, Rng};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector2I {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector2U {
    x: u32,
    y: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector2F {
    x: f32,
    y: f32,
}

impl Vector2U {
    fn add_offset(&mut self, other: Vector2I) {
        *self = Vector2U {
            x: ((self.x as i32) + other.x) as u32,
            y: ((self.y as i32) + other.y) as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorRGB {
    red:   u8,
    green: u8,
    blue:  u8,
}

impl ColorRGB {
    fn as_bytes(color_array: &[ColorRGB]) -> Vec<u8> {
        let mut byte_array: Vec<u8> = vec![0; color_array.len() * 3];

        let mut pos = 0;
        for i in 0..color_array.len() {
            byte_array[pos + 0] = color_array[i].red;
            byte_array[pos + 1] = color_array[i].green;
            byte_array[pos + 2] = color_array[i].blue;
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
    East  = 1,
    South = 2,
    West  = 3,
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
    fn get_tile(&self, pos: Vector2U) -> Tile {
        self.tiles[(pos.x + pos.y * self.width) as usize]
    }

    fn set_tile(&mut self, pos: Vector2U, new: Tile) {
        self.tiles[(pos.x + pos.y * self.width) as usize] = new;
    }

}

fn get_unvisited_adj(maze: &Grid, pos: Vector2U) -> Option<usize> {
    let mut adj_tiles: Vec<usize> = Vec::with_capacity(4);

    if pos.y >= 1              && maze.get_tile(Vector2U {x: pos.x + 0, y: pos.y - 1}).status == ConnectionStatus::UnVisited { adj_tiles.push(0); }
    if pos.x < maze.width - 1  && maze.get_tile(Vector2U {x: pos.x + 1, y: pos.y + 0}).status == ConnectionStatus::UnVisited { adj_tiles.push(1); }
    if pos.y < maze.height - 1 && maze.get_tile(Vector2U {x: pos.x + 0, y: pos.y + 1}).status == ConnectionStatus::UnVisited { adj_tiles.push(2); }
    if pos.x >= 1              && maze.get_tile(Vector2U {x: pos.x - 1, y: pos.y + 0}).status == ConnectionStatus::UnVisited { adj_tiles.push(3); }

    if adj_tiles.len() == 0 {
        None
    } else {
        Some(adj_tiles[rand::random::<usize>() % adj_tiles.len()])
    }
}

fn create_maze_backtrack(width: u32, height: u32) -> Grid {
    let blank: Tile = Tile {status: ConnectionStatus::UnVisited, connections: [false, false, false, false]};
    let num_tiles = width * height;
    let directions = [
    Vector2I {x:  0, y: -1},
    Vector2I {x:  1, y:  0},
    Vector2I {x:  0, y:  1},
    Vector2I {x: -1, y:  0},
    ];

    let mut maze: Grid = Grid {
        tiles: vec![blank; (num_tiles) as usize],
        width: width,
        height: height,
    };
    let mut stack: Vec<Vector2U> = Vec::new();
    let mut num_visited = 0;
    let mut current_pos: Vector2U = Vector2U {x: 0, y: 0};
    let mut current_tile: Tile = maze.get_tile(current_pos);

    while num_visited < num_tiles {
        if current_tile.status == ConnectionStatus::UnVisited {
            current_tile.status = ConnectionStatus::InMaze;
            maze.set_tile(current_pos, current_tile);

            stack.push(current_pos);
            num_visited += 1;
        }

        match get_unvisited_adj(&maze, current_pos) {
            None       => {
                current_pos = stack.pop().unwrap();
                current_tile = maze.get_tile(current_pos);
            },
            Some(next) => {
                current_tile.connections[next] = true;
                maze.set_tile(current_pos, current_tile);

                current_pos.add_offset(directions[next]);
                current_tile = maze.get_tile(current_pos);
                current_tile.connections[(next + 2) % 4] = true; /* add connection in opposite direction on next tile */
                maze.set_tile(current_pos, current_tile);
            },
        }
    }

    maze
}


fn generate_image(maze: &Grid) {
    let cell_width = 2;
    let image_dimensions = Vector2U {x: maze.width * cell_width + 1, y: maze.height * cell_width + 1};


    let path = Path::new(r"./image.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image_dimensions.x, image_dimensions.y);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![ColorRGB {red: 0, green: 0, blue: 0}; (image_dimensions.x * image_dimensions.y) as usize];

    for y in 0..maze.height {
        for x in 0..maze.width {
            pixels[((x * cell_width + 1) + ((y * cell_width + 1) * image_dimensions.x)) as usize] = ColorRGB {red: 255, green: 255, blue: 255};
            if maze.get_tile(Vector2U {x: x, y: y}).connections[Direction::North as usize] {
                pixels[((x * cell_width + 1) + ((y * cell_width + 0) * image_dimensions.x)) as usize] = ColorRGB {red: 255, green: 255, blue: 255};
            }
            if maze.get_tile(Vector2U {x: x, y: y}).connections[Direction::West as usize] {
                pixels[((x * cell_width + 0) + ((y * cell_width + 1) * image_dimensions.x)) as usize] = ColorRGB {red: 255, green: 255, blue: 255};
            }
        }
    }

    writer.write_image_data(&ColorRGB::as_bytes(&pixels)).unwrap();
}

fn normalize(v: Vector2F) -> Vector2F {
    let len = (v.x * v.x + v.y * v.y).sqrt();
    Vector2F {x: v.x / len, y: v.y / len}
}

fn get_color(val: f32) -> ColorRGB {
    ColorRGB {
        red: ((255 as f32) * (val + 1.0) / 2.0) as u8,
        green: ((255 as f32) * (val + 1.0) / 2.0) as u8,
        blue: ((255 as f32) * (val + 1.0) / 2.0) as u8,
    }
}

fn generate_noise() {
    let width: usize = 100;
    let height: usize = 100;
    let cell_width: usize = 10;
    let grid_width: usize = 11;
    let grid_height: usize = 11;

    let mut rng = thread_rng();

    let mut points: Vec<f32> = vec![0.0f32; width * height];
    let mut grid: Vec<Vector2F> = Vec::with_capacity(grid_width * grid_height);

    for i in 0..(grid_width * grid_height) {
        grid.push(normalize(Vector2F {x: rng.gen_range(-1.0..=1.0), y: rng.gen_range(-1.0..=1.0)}));
    }

    for y in 0..height {
        for x in 0..width {
            let gx: uszie = x % cell_width;
            let gy: uszie = y % cell_width;

            let gvec =
                if gx < 5 && gy < 5 {grid[gx + gy * grid_width]}
                else if gx < 5 && gy >= 5 {grid[gx + (gy + 1) * grid_width]}
                else if gx >= 5 && gy < 5 {grid[(gx + 1) + gy * grid_width]}
                else {grid[(gx + 1) + (gy + 1) * grid_width]}
                ;

            points[x + y * width] =

        }
    }



    let path = Path::new(r"./noise.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![ColorRGB {red: 0, green: 0, blue: 0}; width * height];

    for i in 0..(width * height) {
        pixels[i] = get_color(points[i]);
    }

    writer.write_image_data(&ColorRGB::as_bytes(&pixels)).unwrap();
}

fn main() {

    generate_noise();
    return;




    let pix = create_maze_backtrack(3, 3);
    generate_image(&pix);

    println!("Successfully Generated Maze");
}
