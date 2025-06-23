use crate::{
    grid::{ConnectionStatus, Direction, Grid, Point, Rect},
    history::MazeAction,
};
use gif::{DisposalMethod, Encoder, Frame, Repeat};
use std::{
    borrow::Cow,
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, clap::ValueEnum)]
pub enum ImageFormat {
    Gif,
    CompressedGif,
    #[default]
    Png,
    Svg,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageOptions {
    pub file_path: String,
    pub passage_width: u16,
    pub wall_width: u16,
    pub color_map: [u8; 12],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationOptions {
    pub frame_time: u16,
    pub pause_time: u16,
    pub batch_size: u16,
}

fn get_bounds(
    pt: Point,
    dir: Direction,
    cell_width: u16,
    passage_width: u16,
    wall_width: u16,
) -> (u16, u16, u16, u16) {
    match dir {
        Direction::NoDir => (
            pt.y as u16 * cell_width + wall_width,
            pt.x as u16 * cell_width + wall_width,
            passage_width,
            passage_width,
        ),
        Direction::North => (
            pt.y as u16 * cell_width + 0,
            pt.x as u16 * cell_width + wall_width,
            passage_width,
            cell_width,
        ),
        Direction::East => (
            pt.y as u16 * cell_width + wall_width,
            pt.x as u16 * cell_width + wall_width,
            cell_width,
            passage_width,
        ),
        Direction::South => (
            pt.y as u16 * cell_width + wall_width,
            pt.x as u16 * cell_width + wall_width,
            passage_width,
            cell_width,
        ),
        Direction::West => (
            pt.y as u16 * cell_width + wall_width,
            pt.x as u16 * cell_width + 0,
            cell_width,
            passage_width,
        ),
        _ => todo!("Diagonal travel is not supported yet"),
    }
}

fn get_edge_bounds(
    pt: Point,
    dir: Direction,
    cell_width: u16,
    passage_width: u16,
    wall_width: u16,
) -> (u16, u16, u16, u16) {
    match dir {
        Direction::NoDir => panic!("Cant remove edge in NoDir"),
        Direction::North => (
            pt.y as u16 * cell_width + 0,
            pt.x as u16 * cell_width + wall_width,
            passage_width,
            wall_width,
        ),
        Direction::East => (
            pt.y as u16 * cell_width + wall_width,
            pt.x as u16 * cell_width + cell_width,
            wall_width,
            passage_width,
        ),
        Direction::South => (
            pt.y as u16 * cell_width + cell_width,
            pt.x as u16 * cell_width + wall_width,
            passage_width,
            wall_width,
        ),
        Direction::West => (
            pt.y as u16 * cell_width + wall_width,
            pt.x as u16 * cell_width + 0,
            wall_width,
            passage_width,
        ),
        _ => todo!("Diagonal travel is not supported yet"),
    }
}

pub fn generate_gif(
    maze: &Grid,
    history: &[MazeAction],
    rooms: &[Rect],
    opts: &ImageOptions,
    ani_opts: &AnimationOptions,
) -> Result<(), std::io::Error> {
    let cell_width: u16 = opts.passage_width + opts.wall_width;

    let (width, height) = (
        maze.width * cell_width + opts.wall_width,
        maze.height * cell_width + opts.wall_width,
    );

    let mut state: Vec<u8> = vec![0; width as usize * height as usize];
    let mut image = BufWriter::new(File::create(format!("{}.gif", &opts.file_path).as_str())?);
    let mut encoder = Encoder::new(&mut image, width, height, &opts.color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    // draw all rooms in one pass
    for r in rooms {
        let area_width = r.w as u16 * cell_width - opts.wall_width;
        let area_height = r.h as u16 * cell_width - opts.wall_width;
        let area_top = r.y as u16 * cell_width + opts.wall_width;
        let area_left = r.x as u16 * cell_width + opts.wall_width;

        for y in area_top..(area_top + area_height) {
            for x in area_left..(area_left + area_width) {
                state[x as usize + (y as usize * width as usize)] = 1;
            }
        }
    }

    let mut frame_num = 0;
    let mut write_frame = true;
    let mut skip_draw;

    for action in history {
        let (pt, dir, cell_filling);
        skip_draw = false;
        match *action {
            MazeAction::Add(p, d) => {
                (pt, dir, cell_filling) = (p, d, 1);
            }
            MazeAction::Remove(p, d) => {
                (pt, dir, cell_filling) = (p, d, 0);
            }
            MazeAction::RemoveEdge(p, d) => {
                if d == Direction::NoDir {
                    continue;
                }

                (pt, dir, cell_filling) = (p, d, 0);

                let (area_top, area_left, area_width, area_height) =
                    get_edge_bounds(pt, dir, cell_width, opts.passage_width, opts.wall_width);

                for y in area_top..(area_top + area_height) {
                    for x in area_left..(area_left + area_width) {
                        state[x as usize + (y as usize * width as usize)] = cell_filling;
                    }
                }

                if !maze.contains(pt.travel(dir)) {
                    let (area_top, area_left, area_width, area_height) = get_edge_bounds(
                        pt.travel_wrapped(dir, maze.width, maze.height),
                        dir.opposite(),
                        cell_width,
                        opts.passage_width,
                        opts.wall_width,
                    );

                    for y in area_top..(area_top + area_height) {
                        for x in area_left..(area_left + area_width) {
                            state[x as usize + (y as usize * width as usize)] = cell_filling;
                        }
                    }
                }

                skip_draw = true;
            }
            MazeAction::AddTemp(p, d) => {
                (pt, dir, cell_filling) = (p, d, 2);
            }
            MazeAction::AddMarker(p) => {
                (pt, dir, cell_filling) = (p, Direction::NoDir, 3);
            }
            MazeAction::StartFrame => {
                write_frame = false;
                continue;
            }
            MazeAction::EndFrame => {
                (pt, dir, cell_filling) = (Point::new(0, 0), Direction::NoDir, 0);
                skip_draw = true;
                write_frame = true;
            }
        }

        if !skip_draw {
            let (area_top, area_left, area_width, area_height) =
                get_bounds(pt, dir, cell_width, opts.passage_width, opts.wall_width);

            for y in area_top..(area_top + area_height) {
                for x in area_left..(area_left + area_width) {
                    state[x as usize + (y as usize * width as usize)] = cell_filling;
                }
            }

            if !maze.contains(pt.travel(dir)) {
                let (area_top, area_left, area_width, area_height) = get_bounds(
                    pt.travel_wrapped(dir, maze.width, maze.height),
                    dir.opposite(),
                    cell_width,
                    opts.passage_width,
                    opts.wall_width,
                );

                for y in area_top..(area_top + area_height) {
                    for x in area_left..(area_left + area_width) {
                        state[x as usize + (y as usize * width as usize)] = cell_filling;
                    }
                }
            }
        }

        if write_frame {
            frame_num += 1;
        }

        // generate and save frame
        if write_frame && frame_num % ani_opts.batch_size == 0 {
            let frame = Frame {
                width,
                height,
                delay: ani_opts.frame_time,
                buffer: Cow::Borrowed(&state),
                ..Frame::default()
            };
            encoder.write_frame(&frame).unwrap();
        }
    }

    // final frame with a higher delay
    let frame = Frame {
        width,
        height,
        delay: ani_opts.pause_time,
        buffer: Cow::Borrowed(&state),
        ..Frame::default()
    };
    encoder.write_frame(&frame).unwrap();

    Ok(())
}

pub fn generate_gif_compressed(
    maze: &Grid,
    history: &[MazeAction],
    rooms: &[Rect],
    opts: &ImageOptions,
    ani_opts: &AnimationOptions,
) -> Result<(), std::io::Error> {
    let cell_width: u16 = opts.passage_width + opts.wall_width;

    let (width, height) = (
        maze.width * cell_width + opts.wall_width,
        maze.height * cell_width + opts.wall_width,
    );

    let empty_maze: Vec<u8> = vec![0; width as usize * height as usize];
    let full_maze: Vec<u8> = vec![1; width as usize * height as usize];
    let connected_cell: Vec<u8> = vec![1; (cell_width * cell_width) as usize];
    let blank_cell: Vec<u8> = vec![0; (cell_width * cell_width) as usize];

    let mut image = BufWriter::new(File::create(format!("{}.gif", &opts.file_path).as_str())?);
    let mut encoder = Encoder::new(&mut image, width, height, &opts.color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    // initial frame to set background
    let frame = Frame {
        width,
        height,
        delay: 0,
        buffer: Cow::Borrowed(&empty_maze),
        ..Frame::default()
    };
    encoder.write_frame(&frame).unwrap();

    // add rooms to maze
    for r in rooms {
        let frame = Frame {
            delay: ani_opts.frame_time,
            width: r.w as u16 * cell_width - opts.wall_width,
            height: r.h as u16 * cell_width - opts.wall_width,
            top: r.y as u16 * cell_width + opts.wall_width,
            left: r.x as u16 * cell_width + opts.wall_width,
            buffer: Cow::Borrowed(&full_maze),
            dispose: DisposalMethod::Keep,
            ..Frame::default()
        };

        encoder.write_frame(&frame).unwrap();
    }

    for action in history {
        let (pt, dir, cell_filling) = match *action {
            MazeAction::Add(pt, dir) => (pt, dir, &connected_cell),
            MazeAction::Remove(pt, dir) => (pt, dir, &blank_cell),
            _ => todo!(),
        };
        let mut frame = Frame {
            delay: ani_opts.frame_time,
            buffer: Cow::Borrowed(cell_filling),
            dispose: DisposalMethod::Keep,
            ..Frame::default()
        };
        // set dimensions and position of frame
        (frame.top, frame.left, frame.width, frame.height) =
            get_bounds(pt, dir, cell_width, opts.passage_width, opts.wall_width);
        encoder.write_frame(&frame).unwrap();

        if !maze.contains(pt.travel(dir)) {
            (frame.top, frame.left, frame.width, frame.height) = get_bounds(
                pt.travel_wrapped(dir, maze.width, maze.height),
                dir.opposite(),
                cell_width,
                opts.passage_width,
                opts.wall_width,
            );
            frame.buffer = Cow::Borrowed(cell_filling);
            frame.dispose = DisposalMethod::Keep;
            encoder.write_frame(&frame).unwrap();
        }
    }

    // final empty frame with a higher delay
    let frame = Frame {
        width: 1,
        height: 1,
        dispose: DisposalMethod::Keep,
        delay: ani_opts.pause_time,
        buffer: Cow::Borrowed(&[0]),
        ..Frame::default()
    };
    encoder.write_frame(&frame).unwrap();

    Ok(())
}

pub fn generate_png(maze: &Grid, opts: &ImageOptions) -> Result<(), std::io::Error> {
    let cell_width: u16 = opts.passage_width + opts.wall_width;
    let (width, height) = (
        maze.width * cell_width + opts.wall_width,
        maze.height * cell_width + opts.wall_width,
    );

    let file = File::create(format!("{}.png", &opts.file_path).as_str())?;
    let writer = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_palette(&opts.color_map);
    encoder
        .add_text_chunk("Author".to_owned(), "PokeyLink227".to_owned())
        .unwrap();
    encoder
        .add_text_chunk("Software".to_owned(), "Labgen".to_owned())
        .unwrap();

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<u8> = vec![0; width as usize * height as usize];

    for py in 0..maze.height {
        for px in 0..maze.width {
            let tile = maze[(px as i16, py as i16)];
            if !(tile.status == ConnectionStatus::InMaze || tile.status == ConnectionStatus::Room) {
                continue;
            }

            let top: u16 = py * cell_width + opts.wall_width;
            let left: u16 = px * cell_width + opts.wall_width;

            for y in 0..opts.passage_width {
                for x in 0..opts.passage_width {
                    pixels[(x + left) as usize + ((y + top) as usize * width as usize)] = 1;
                }
            }
            if tile.connected(Direction::East) {
                for y in 0..opts.passage_width {
                    for x in opts.passage_width..cell_width {
                        pixels[(x + left) as usize + ((y + top) as usize * width as usize)] = 1;
                    }
                }
            }
            if tile.connected(Direction::South) {
                for y in opts.passage_width..cell_width {
                    for x in 0..opts.passage_width {
                        pixels[(x + left) as usize + ((y + top) as usize * width as usize)] = 1;
                    }
                }
            }
            if tile.connected(Direction::SouthEast) {
                for y in opts.passage_width..cell_width {
                    for x in opts.passage_width..cell_width {
                        pixels[(x + left) as usize + ((y + top) as usize * width as usize)] = 1;
                    }
                }
            }

            // only needed for wrapping mazes
            // only chekc on edges to reduce overdraw
            if px == 0 && tile.connected(Direction::West) {
                for y in 0..opts.passage_width {
                    for x in 0..=opts.wall_width {
                        pixels[(left - x) as usize + ((y + top) as usize * width as usize)] = 1;
                    }
                }
            }

            if py == 0 && tile.connected(Direction::North) {
                for y in 0..=opts.wall_width {
                    for x in 0..opts.passage_width {
                        pixels[(x + left) as usize + ((top - y) as usize * width as usize)] = 1;
                    }
                }
            }
        }
    }

    writer.write_image_data(&pixels)?;

    Ok(())
}

type TileMap = [char; 16];

// TODO:
// - add conversion from string for custom characters
// - maybe convert pixel array to u8 and make the u8 to char to string conversion all at once
//   - this is not faster sadly
static TILE_MAPS: [TileMap; 7] = [
    [
        ' ', '╵', '╶', '└', '╷', '│', '┌', '├', '╴', '┘', '─', '┴', '┐', '┤', '┬', '┼',
    ],
    [
        ' ', '╵', '╶', '╰', '╷', '│', '╭', '├', '╴', '╯', '─', '┴', '╮', '┤', '┬', '┼',
    ],
    [
        ' ', '╵', '╶', '🮡', '╷', '│', '🮣', '├', '╴', '🮠', '─', '┴', '🮢', '┤', '┬', '┼',
    ],
    [
        ' ', '+', '+', '+', '+', '|', '+', '+', '+', '+', '-', '+', '+', '+', '+', '+',
    ],
    [
        ' ', '+', '+', '\\', '+', '|', '/', '+', '+', '/', '-', '+', '\\', '+', '+', '+',
    ],
    [
        ' ', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█', '█',
    ],
    [
        ' ', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#', '#',
    ],
];
static INTERSECTION_MAP: &TileMap = &TILE_MAPS[0];

fn set_intersection(pixels: &mut [char], width: usize, height: usize, px: usize, py: usize) {
    let mut walls = 0x00;

    if py > 0 && pixels[px + (py - 1) * width] != INTERSECTION_MAP[0] {
        walls |= 0x01;
    }
    if px < width - 2 && pixels[(px + 1) + py * width] != INTERSECTION_MAP[0] {
        walls |= 0x02;
    }
    if py < height - 1 && pixels[px + (py + 1) * width] != INTERSECTION_MAP[0] {
        walls |= 0x04;
    }
    if px > 0 && pixels[(px - 1) + py * width] != INTERSECTION_MAP[0] {
        walls |= 0x08;
    }

    pixels[px + py * width] = INTERSECTION_MAP[walls];
}

pub fn generate_text(maze: &Grid, opts: &ImageOptions) -> Result<(), std::io::Error> {
    let horiz = INTERSECTION_MAP[10];
    let vert = INTERSECTION_MAP[5];

    let opts = &ImageOptions {
        wall_width: 1,
        ..opts.clone()
    };

    // TODO: move passage width/height to image option
    // or make into ratio to make a default value makre more sense
    let passage_width = 3;
    let passage_height = 1;
    let cell_width: u16 = passage_width as u16 + opts.wall_width;
    let cell_height: u16 = passage_height as u16 + opts.wall_width;

    // width + 1 to account for '\n'
    let (width, height) = (
        (maze.width * cell_width + opts.wall_width + 1) as usize,
        (maze.height * cell_height + opts.wall_width) as usize,
    );

    let file = File::create(format!("{}.txt", &opts.file_path).as_str())?;
    let writer = &mut BufWriter::new(file);

    let mut pixels: Vec<char> = vec![INTERSECTION_MAP[0]; width * height];

    for x in 0..width {
        if !maze[(x as i16 / cell_width as i16, 0)].connected(Direction::North) {
            pixels[x] = horiz;
        }
    }
    pixels[width - 1] = '\n';

    for py in 0..maze.height {
        let top: usize = (py * cell_height + opts.wall_width) as usize;

        for y in 0..cell_height as usize {
            if !maze[(0, py as i16)].connected(Direction::West) {
                pixels[(top + y) * width] = vert;
            }
            pixels[(width - 1) + ((top + y) * width)] = '\n';
        }

        for px in 0..maze.width {
            let tile = maze[(px as i16, py as i16)];
            let left: usize = (px * cell_width + opts.wall_width) as usize;

            // check upper left corner for intersection type
            set_intersection(&mut pixels, width, height, left - 1, top - 1);

            if !(tile.status == ConnectionStatus::InMaze || tile.status == ConnectionStatus::Room) {
                continue;
            }

            if !tile.connected(Direction::East) {
                for y in 0..cell_height as usize {
                    pixels[(left + passage_width) + ((top + y) * width)] = vert;
                }
            }
            if !tile.connected(Direction::South) {
                for x in 0..cell_width as usize {
                    pixels[(left + x) + ((top + passage_height) * width)] = horiz;
                }
            }
        }

        set_intersection(&mut pixels, width, height, width - 2, top - 1);
    }

    (0..width - 1)
        .step_by(2)
        .for_each(|i| set_intersection(&mut pixels, width, height, i, height - 1));

    writer.write(pixels.into_iter().collect::<String>().as_bytes())?;

    Ok(())
}

pub fn generate_svg(maze: &Grid, opts: &ImageOptions) -> Result<(), std::io::Error> {
    let file = File::create(format!("{}.svg", &opts.file_path).as_str())?;
    let mut buf = BufWriter::new(file);

    buf.write_all(
        format!(
            "<svg viewBox=\"-1 -1 {} {}\" xmlns=\"http://www.w3.org/2000/svg\" stroke=\"black\" stroke-width=\"0.25\" stroke-linecap=\"square\" shape-rendering=\"crispEdges\">",
            maze.width + 2,
            maze.height + 2,
        ).as_bytes()
    )?;

    for y in 0..maze.height {
        for x in 0..maze.width {
            let tile = maze[(x as i16, y as i16)];

            if tile.status == ConnectionStatus::Removed {
                buf.write_all(
                    format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                        x, y, 1, 1
                    )
                    .as_bytes(),
                )?;
            } else {
                if !tile.connected(Direction::North) {
                    buf.write_all(
                        format!(
                            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                            x,
                            y,
                            x + 1,
                            y
                        )
                        .as_bytes(),
                    )?;
                }
                if !tile.connected(Direction::West) {
                    buf.write_all(
                        format!(
                            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
                            x,
                            y,
                            x,
                            y + 1
                        )
                        .as_bytes(),
                    )?;
                }
            }
        }
    }

    buf.write_all(
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
            maze.width, 0, maze.width, maze.height,
        )
        .as_bytes(),
    )?;

    buf.write_all(
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"/>",
            0, maze.height, maze.width, maze.height,
        )
        .as_bytes(),
    )?;

    buf.write_all(b"</svg>")?;

    Ok(())
}
