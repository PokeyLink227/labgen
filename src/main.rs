use crate::{
    image::{
        generate_gif, generate_gif_uncompressed, generate_png, AnimationOptions, ImageOptions,
    },
    maze::{create_maze_backtrack, create_maze_binary, create_maze_sidewinder, create_maze_prim, gen_maze},
};
use clap::Parser;
use std::time::Instant;

mod image;
mod maze;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    width: u32,

    #[arg()]
    height: u32,

    #[arg(short = 'm', long = "method", default_value = "0")]
    method: u8,

    #[arg(short = 'o', long = "out", value_name = "output file", id = "out")]
    destination_file: Option<String>,

    #[arg(short = 'a', long = "animate")]
    animate: bool,

    #[arg(short = 'u', long = "uncompressed", default_value = "false")]
    uncompressed: bool,
}

fn main() {
    let args = Args::parse();

    let mut now = Instant::now();
    let (nodes, hist) = match args.method {
        0 => create_maze_backtrack(args.width, args.height),
        1 => create_maze_prim(args.width, args.height),
        2 => create_maze_binary(args.width, args.height),
        3 => create_maze_sidewinder(args.width, args.height),
        4 => gen_maze(args.width, args.height),
        _ => create_maze_backtrack(args.width, args.height),
    };
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
