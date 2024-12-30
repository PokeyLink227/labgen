use crate::maze::{ConnectionStatus, Direction, Grid, Point};
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
pub struct ImageOptions {
    pub passage_width: u16,
    pub wall_width: u16,
    pub color_map: [u8; 6],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationOptions {
    pub frame_time: u16,
    pub pause_time: u16,
}

pub fn generate_gif_uncompressed(
    maze: &Grid,
    history: &[(Point, Direction)],
    opts: &ImageOptions,
    ani_opts: &AnimationOptions,
) {
    let cell_width: u16 = opts.passage_width + opts.wall_width;

    let (width, height) = (
        maze.width * cell_width + opts.wall_width,
        maze.height * cell_width + opts.wall_width,
    );

    let mut state: Vec<u8> = vec![0; width as usize * height as usize];
    let mut image = BufWriter::new(File::create("./animation.gif").unwrap());
    let mut encoder = Encoder::new(&mut image, width, height, &opts.color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    for (pt, dir) in history {
        let area_top: u16;
        let area_left: u16;
        let area_width: u16;
        let area_height: u16;

        match dir {
            Direction::NoDir => {
                area_width = opts.passage_width;
                area_height = opts.passage_width;
                area_top = pt.y as u16 * cell_width + opts.wall_width;
                area_left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::North => {
                area_width = opts.passage_width;
                area_height = cell_width;
                area_top = pt.y as u16 * cell_width + 0;
                area_left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::East => {
                area_width = cell_width;
                area_height = opts.passage_width;
                area_top = pt.y as u16 * cell_width + opts.wall_width;
                area_left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::South => {
                area_width = opts.passage_width;
                area_height = cell_width;
                area_top = pt.y as u16 * cell_width + opts.wall_width;
                area_left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::West => {
                area_width = cell_width;
                area_height = opts.passage_width;
                area_top = pt.y as u16 * cell_width + opts.wall_width;
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
        frame.delay = ani_opts.frame_time;
        frame.buffer = Cow::Borrowed(&state);
        encoder.write_frame(&frame).unwrap();
    }

    // final frame with a higher delay
    let mut frame = Frame::default();
    frame.width = width;
    frame.height = height;
    frame.delay = ani_opts.pause_time;
    frame.buffer = Cow::Borrowed(&state);
    encoder.write_frame(&frame).unwrap();
}

pub fn generate_gif(
    maze: &Grid,
    history: &[(Point, Direction)],
    opts: &ImageOptions,
    ani_opts: &AnimationOptions,
) {
    let cell_width: u16 = opts.passage_width + opts.wall_width;

    let (width, height) = (
        maze.width * cell_width + opts.wall_width,
        maze.height * cell_width + opts.wall_width,
    );

    let empty_maze: Vec<u8> = vec![0; width as usize * height as usize];
    let connected_cell: Vec<u8> = vec![1; (cell_width * cell_width) as usize];

    let mut image = BufWriter::new(File::create("./animation.gif").unwrap());
    let mut encoder = Encoder::new(&mut image, width, height, &opts.color_map).unwrap();
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
        frame.delay = ani_opts.frame_time;

        // set dimensions and position of frame
        match dir {
            Direction::NoDir => {
                frame.width = opts.passage_width;
                frame.height = opts.passage_width;
                frame.top = pt.y as u16 * cell_width + opts.wall_width;
                frame.left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::North => {
                frame.width = opts.passage_width;
                frame.height = cell_width;
                frame.top = pt.y as u16 * cell_width + 0;
                frame.left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::East => {
                frame.width = cell_width;
                frame.height = opts.passage_width;
                frame.top = pt.y as u16 * cell_width + opts.wall_width;
                frame.left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::South => {
                frame.width = opts.passage_width;
                frame.height = cell_width;
                frame.top = pt.y as u16 * cell_width + opts.wall_width;
                frame.left = pt.x as u16 * cell_width + opts.wall_width;
            }
            Direction::West => {
                frame.width = cell_width;
                frame.height = opts.passage_width;
                frame.top = pt.y as u16 * cell_width + opts.wall_width;
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
    frame.delay = ani_opts.pause_time;
    frame.buffer = Cow::Borrowed(&[0]);
    encoder.write_frame(&frame).unwrap();
}

pub fn generate_png(maze: &Grid, opts: &ImageOptions) {
    let cell_width: u16 = opts.passage_width + opts.wall_width;
    let (width, height) = (
        maze.width * cell_width + opts.wall_width,
        maze.height * cell_width + opts.wall_width,
    );

    let color_map = &[
        ColorRGB {
            red: 0,
            green: 0,
            blue: 0,
        },
        ColorRGB {
            red: 255,
            green: 255,
            blue: 255,
        },
    ];

    let path = Path::new(r"./image.png");
    let file = File::create(path).unwrap();
    let ref mut writer = BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![color_map[0]; width as usize * height as usize];

    for py in 0..maze.height {
        for px in 0..maze.width {
            let top: u16 = py as u16 * cell_width + opts.wall_width;
            let left: u16 = px as u16 * cell_width + opts.wall_width;
            let connections = maze
                .get_tile(Point {
                    x: px as i16,
                    y: py as i16,
                })
                .connections;

            for y in 0..opts.passage_width {
                for x in 0..opts.passage_width {
                    pixels[(x + left) as usize + ((y + top) as usize * width as usize)] =
                        color_map[1];
                }
            }
            if connections & Direction::East as u8 != 0 {
                for y in 0..opts.passage_width {
                    for x in opts.passage_width..cell_width {
                        pixels[(x + left) as usize + ((y + top) as usize * width as usize)] =
                            color_map[1];
                    }
                }
            }
            if connections & Direction::South as u8 != 0 {
                for y in opts.passage_width..cell_width {
                    for x in 0..opts.passage_width {
                        pixels[(x + left) as usize + ((y + top) as usize * width as usize)] =
                            color_map[1];
                    }
                }
            }
        }
    }

    writer
        .write_image_data(&ColorRGB::as_bytes(&pixels))
        .unwrap();
}
