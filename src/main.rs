use std::{
    time::Instant,
};
use clap::Parser;
use crate::{
    maze::{
        Vector2,
        create_maze_prim,
        create_maze_backtrack,
        gen_maze,
    },
    image::{
        generate_gif,
        generate_png,
    },
};

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
}

fn main() {
    let args = Args::parse();

    let mut now = Instant::now();
    let (nodes, hist) = match args.method {
        0 => create_maze_backtrack(args.width, args.height),
        1 => create_maze_prim(args.width, args.height),
        _ => create_maze_backtrack(args.width, args.height),
    };
    let maze_time = now.elapsed();

    now = Instant::now();
    if args.animate {
        generate_gif(&nodes, &hist);
    } else {
        generate_png(&nodes);
    }
    let image_time = now.elapsed();

    // need to add proper 0 padding
    println!("Elapsed time: maze {}.{:09.9}s, gif {}.{:09.9}s", maze_time.as_secs(), maze_time.as_nanos(), image_time.as_secs(), image_time.as_nanos());
}
