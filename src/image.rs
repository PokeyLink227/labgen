use crate::maze::{ConnectionStatus, Direction, Grid, Point, Vector2};
use gif::{DisposalMethod, Encoder, Frame, Repeat};
use std::{borrow::Cow, fs::File, io::BufWriter, path::Path};

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

fn get_color(val: f32) -> ColorRGB {
    ColorRGB {
        red: ((255 as f32) * (val + 1.0) / 2.0) as u8,
        green: ((255 as f32) * (val + 1.0) / 2.0) as u8,
        blue: ((255 as f32) * (val + 1.0) / 2.0) as u8,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ImageOptions {
    passage_width: u16,
    wall_width: u16,
    frame_time: u16,
}

pub fn generate_gif_uncompressed(maze: &Grid, history: &[(Point, Direction)]) {
    let passage_width: u16 = 9;
    let wall_width: u16 = 3;
    let cell_width: u16 = passage_width + wall_width;
    let frame_time = 5;

    let color_map = &[0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF];
    let (width, height) = (
        maze.width as u16 * cell_width + wall_width,
        maze.height as u16 * cell_width + wall_width,
    );

    let mut state: Vec<u8> = vec![0; width as usize * height as usize];
    let mut image = File::create("./animation.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, width, height, color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    for (pt, dir) in history {
        let area_top: u16;
        let area_left: u16;
        let area_width: u16;
        let area_height: u16;

        match dir {
            Direction::None => {
                area_width = passage_width;
                area_height = passage_width;
                area_top = pt.y as u16 * cell_width + wall_width;
                area_left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::North => {
                area_width = passage_width;
                area_height = cell_width;
                area_top = pt.y as u16 * cell_width + 0;
                area_left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::East => {
                area_width = cell_width;
                area_height = passage_width;
                area_top = pt.y as u16 * cell_width + wall_width;
                area_left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::South => {
                area_width = passage_width;
                area_height = cell_width;
                area_top = pt.y as u16 * cell_width + wall_width;
                area_left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::West => {
                area_width = cell_width;
                area_height = passage_width;
                area_top = pt.y as u16 * cell_width + wall_width;
                area_left = pt.x as u16 * cell_width + 0;
            }
        }

        for y in area_top..(area_top + area_height) {
            for x in area_left..(area_left + area_width) {
                state[x as usize + (y as usize * width as usize)] = 1;
            }
        }

        // generate and save frame
        let mut frame = Frame::default();
        frame.width = width;
        frame.height = height;
        frame.delay = 10;
        frame.buffer = Cow::Borrowed(&state);
        encoder.write_frame(&frame).unwrap();
    }

    // final frame with a higher delay
    let mut frame = Frame::default();
    frame.width = width;
    frame.height = height;
    frame.delay = 100;
    frame.buffer = Cow::Borrowed(&state);
    encoder.write_frame(&frame).unwrap();
}

pub fn generate_gif(maze: &Grid, history: &[(Point, Direction)]) {
    let passage_width: u16 = 9;
    let wall_width: u16 = 3;
    let cell_width: u16 = passage_width + wall_width;
    let frame_time = 5;

    let color_map = &[0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF];
    let (width, height) = (
        maze.width as u16 * cell_width + wall_width,
        maze.height as u16 * cell_width + wall_width,
    );

    let empty_maze: Vec<u8> = vec![0; width as usize * height as usize];
    let connected_cell: Vec<u8> = vec![1; (cell_width * cell_width) as usize];

    let mut image = File::create("./animation.gif").unwrap();
    let mut encoder = Encoder::new(&mut image, width, height, color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    // initial frame to set background
    let mut frame = Frame::default();
    frame.width = width;
    frame.height = height;
    frame.delay = 0;
    frame.buffer = Cow::Borrowed(&empty_maze);
    encoder.write_frame(&frame).unwrap();

    for (pt, dir) in history {
        let mut frame = Frame::default();
        frame.delay = frame_time;

        // set dimensions and position of frame
        match dir {
            Direction::None => {
                frame.width = passage_width;
                frame.height = passage_width;
                frame.top = pt.y as u16 * cell_width + wall_width;
                frame.left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::North => {
                frame.width = passage_width;
                frame.height = cell_width;
                frame.top = pt.y as u16 * cell_width + 0;
                frame.left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::East => {
                frame.width = cell_width;
                frame.height = passage_width;
                frame.top = pt.y as u16 * cell_width + wall_width;
                frame.left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::South => {
                frame.width = passage_width;
                frame.height = cell_width;
                frame.top = pt.y as u16 * cell_width + wall_width;
                frame.left = pt.x as u16 * cell_width + wall_width;
            }
            Direction::West => {
                frame.width = cell_width;
                frame.height = passage_width;
                frame.top = pt.y as u16 * cell_width + wall_width;
                frame.left = pt.x as u16 * cell_width + 0;
            }
        }

        frame.buffer = Cow::Borrowed(&connected_cell);
        frame.dispose = DisposalMethod::Keep;
        encoder.write_frame(&frame).unwrap();
    }

    // final empty frame with a higher delay
    let mut frame = Frame::default();
    frame.width = 1;
    frame.height = 1;
    frame.dispose = DisposalMethod::Keep;
    frame.delay = 100;
    frame.buffer = Cow::Borrowed(&[0]);
    encoder.write_frame(&frame).unwrap();
}

pub fn generate_png(maze: &Grid) {
    let cell_width = 5;
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
            if maze
                .get_tile(Point {
                    x: x as i16,
                    y: y as i16,
                })
                .connections
                & Direction::North as u8
                != 0
            {
                pixels[((x * cell_width + 1) + ((y * cell_width + 0) * image_dimensions.x))
                    as usize] = ColorRGB {
                    red: 255,
                    green: 255,
                    blue: 255,
                };
            }
            if maze
                .get_tile(Point {
                    x: x as i16,
                    y: y as i16,
                })
                .connections
                & Direction::West as u8
                != 0
            {
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
