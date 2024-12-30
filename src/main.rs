use crate::{
    image::{
        generate_gif, generate_gif_uncompressed, generate_png, AnimationOptions, ImageOptions,
    },
    maze::{generate_maze, MazeType},
};
use rand::{
    SeedableRng,
    rngs::StdRng,
};
use clap::Parser;
use std::time::Instant;

mod image;
mod maze;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    width: u16,

    #[arg()]
    height: u16,

    #[arg(short = 'm', long = "method", default_value = "0")]
    method: MazeType,

    #[arg(short = 'o', long = "out", value_name = "output file", id = "out")]
    destination_file: Option<String>,

    #[arg(short = 'a', long = "animate")]
    animate: bool,

    #[arg(short = 'u', long = "uncompressed", default_value = "false")]
    uncompressed: bool,

    #[arg(short = 's', long = "seed",)]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();

    let seed: u64 = args.seed.unwrap_or(rand::random::<u64>());
    let mut rng: StdRng = StdRng::seed_from_u64(seed);
    println!("seed: {}", seed);

    let mut now = Instant::now();
    let (nodes, hist) = generate_maze(args.width, args.height, args.method, &mut rng);
    let maze_time = now.elapsed();

    now = Instant::now();
    let opts = ImageOptions {
        passage_width: 3,
        wall_width: 1,
        color_map: [0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF],
    };
    let ani_opts = AnimationOptions {
        frame_time: 2,
        pause_time: 100,
    };

    if args.animate {
        if args.uncompressed {
            generate_gif_uncompressed(&nodes, &hist, &opts, &ani_opts);
        } else {
            generate_gif(&nodes, &hist, &opts, &ani_opts);
        }
    } else {
        generate_png(&nodes, &opts);
    }
    let image_time = now.elapsed();

    println!(
        "Elapsed time: maze {}.{:09.9}s, gif {}.{:09.9}s",
        maze_time.as_secs(),
        maze_time.as_nanos(),
        image_time.as_secs(),
        image_time.as_nanos()
    );
}
