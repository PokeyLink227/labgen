use crate::{
    grid::Rect,
    image::{generate_gif, generate_gif_compressed, generate_png, AnimationOptions, ImageOptions},
    maze::{generate_maze, MazeType, MazeWrap},
};
use clap::Parser;
use rand::{rngs::StdRng, SeedableRng};
use std::{str::FromStr, time::Instant};

mod grid;
mod history;
mod image;
mod maze;
mod mazetext;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// width of the maze in cells
    #[arg(value_name = "width")]
    width: u16,

    /// height of the maze in cells
    #[arg(value_name = "height")]
    height: u16,

    /// generation method used for the maze
    #[arg(short = 'm', long = "method", default_value = "backtrack")]
    method: MazeType,

    /// file to save image to
    #[arg(
        short = 'o',
        long = "out",
        value_name = "file",
        default_value = "./maze"
    )]
    file_path: String,

    /// generate an animation rather than an image
    #[arg(short = 'a', long = "animate")]
    animate: bool,

    /// try to compress generated gif
    #[arg(short = 'c', long = "compress", default_value = "false")]
    compress: bool,

    /// number of new cells to draw per frame of animation
    #[arg(
        short = 'b',
        long = "batch",
        value_name = "batch size",
        default_value = "1"
    )]
    batch_size: u16,

    /// rng seed
    #[arg(short = 's', long = "seed")]
    seed: Option<u64>,

    /// pixel dimension of passages
    #[arg(long = "passagewidth", default_value = "4")]
    passage_width: u16,

    /// pixel dimension of walls
    #[arg(long = "wallwidth", default_value = "1")]
    wall_width: u16,

    /// length of time between frames (units of 10ms)
    #[arg(short = 'f', long = "frametime", default_value = "2")]
    frame_time: u16,

    /// length of time for final frame (units of 10ms)
    #[arg(short = 'p', long = "pausetime", default_value = "100")]
    pause_time: u16,

    /// directional wrapping across buondries
    #[arg(short = 'w', long = "wrap")]
    wrap: Option<MazeWrap>,

    /// remove percent% of the deadends from the maze
    #[arg(
        short = 'r',
        long = "remove-deadends",
        default_value = "0",
        value_name = "percent",
        value_parser = clap::value_parser!(u8).range(0..100),
    )]
    uncarve_percent: u8,

    /// include temporary cells in animated maze
    #[arg(long = "tempcells", default_value = "false")]
    log_temps: bool,

    /// suppress output
    #[arg(long = "nosave", default_value = "false")]
    nosave: bool,

    /// Comma seperated list of rects (x,y,w,h);(x,y,w,h)
    #[arg(long = "room")]
    rooms: Option<String>,

    /// Comma seperated list of rects (x,y,w,h);(x,y,w,h)
    #[arg(long = "exclude")]
    exclusions: Option<String>,
}

fn main() {
    let args = Args::parse();

    // parse args section
    let rooms: Vec<Rect> = if let Some(s) = args.rooms {
        let res = s.split(';').map(Rect::from_str).collect();
        match res {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        }
    } else {
        Vec::new()
    };

    let exclude: Vec<Rect> = if let Some(s) = args.exclusions {
        let res = s.split(';').map(Rect::from_str).collect();
        match res {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        }
    } else {
        Vec::new()
    };

    let seed: u64 = args.seed.unwrap_or(rand::random::<u64>());
    let mut rng: StdRng = StdRng::seed_from_u64(seed);
    println!("seed: {}", seed);

    let mut now = Instant::now();
    let (nodes, hist) = generate_maze(
        args.width,
        args.height,
        args.method,
        args.wrap,
        &rooms,
        &exclude,
        args.uncarve_percent,
        args.log_temps && args.animate,
        &mut rng,
    );
    let maze_time = now.elapsed();

    now = Instant::now();
    let opts = ImageOptions {
        file_path: args.file_path,
        passage_width: args.passage_width,
        wall_width: args.wall_width,
        color_map: [
            0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x80, 0x80, 0x80, 0xFF, 0x80, 0x80,
        ],
    };
    let ani_opts = AnimationOptions {
        frame_time: args.frame_time,
        pause_time: args.pause_time,
        batch_size: args.batch_size,
    };

    if !args.nosave {
        if args.animate {
            if args.compress {
                generate_gif_compressed(&nodes, hist.get_actions(), &rooms, &opts, &ani_opts);
            } else {
                generate_gif(&nodes, hist.get_actions(), &rooms, &opts, &ani_opts);
            }
        } else {
            generate_png(&nodes, &opts);
        }
    }
    let image_time = now.elapsed();

    //println!("dbg: {:?}", nodes.tiles);
    println!(
        "Elapsed time: maze {}.{:09.9}s, gif {}.{:09.9}s",
        maze_time.as_secs(),
        maze_time.as_nanos(),
        image_time.as_secs(),
        image_time.as_nanos()
    );
}
