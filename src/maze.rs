use crate::{
    grid::{ConnectionStatus, Direction, Grid, Point, Rect, Tile},
    history::MazeHistory,
    mazetext::{MazeFont, MazeText},
};
use rand::{Rng, seq::{IteratorRandom, SliceRandom, IndexedRandom}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vector2<T> {
    x: T,
    y: T,
}

impl Vector2<f32> {
    fn dot(lhs: Vector2<f32>, rhs: Vector2<f32>) -> f32 {
        lhs.x * rhs.x + lhs.y * rhs.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[repr(u8)]
pub enum MazeType {
    #[default]
    Backtrack,
    Prim,
    BinaryTree,
    Sidewinder,
    Noise,
    GrowingTree,
    Wilson,
    Kruskal,
    PrimSimple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[repr(u8)]
pub enum MazeWrap {
    #[default]
    Full,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge(Point, Direction);

/*
fn opposite(src: u8) -> u8 {
    ((src << 2) | (src >> 2)) & 0b1111
}
*/

fn pick_random(points: &[(usize, Point)], rng: &mut impl Rng) -> Option<(usize, Point)> {
    if !points.is_empty() {
        Some(points[rng.random_range(0..points.len())])
    } else {
        None
    }
}

pub fn generate_maze(
    width: u16,
    height: u16,
    mtype: MazeType,
    wrap: Option<MazeWrap>,
    rooms: &[Rect],
    exclusions: &[Rect],
    text: &[MazeText],
    uncarve_percent: u8,
    log_temps: bool,
    rng: &mut impl Rng,
) -> (Grid, MazeHistory) {
    let mut maze: Grid = Grid {
        tiles: vec![Tile::default(); width as usize * height as usize],
        width,
        height,
    };

    if !text.is_empty() {
        let font = match MazeFont::read_font("default_font.png") {
            Ok(f) => f,
            Err(e) => {
                println!("Error: {:?}", e);
                return (maze, MazeHistory::new(width, height, log_temps));
            }
        };

        for t in text {
            if let Err(e) = font.generate_text(*t, &mut maze) {
                println!("Error: {:?}", e);
                return (maze, MazeHistory::new(width, height, log_temps));
            }
        }
    }

    // remove all exclusions from the maze
    for r in exclusions {
        for y in r.y..(r.y + r.h) {
            for x in r.x..(r.x + r.w) {
                maze[(x, y)] = Tile {
                    status: ConnectionStatus::Removed,
                    connections: Direction::NoDir as u8,
                };
            }
        }
    }

    // add rooms to the maze
    let fully_connected: u8 = 0b11111111;
    for r in rooms {
        for y in 0..r.h {
            for x in 0..r.w {
                let mut connections = fully_connected;

                if y == 0 {
                    connections &= !(Direction::NorthWest as u8
                        | Direction::North as u8
                        | Direction::NorthEast as u8);
                }
                // might overflow
                if y == r.h - 1 {
                    connections &= !(Direction::SouthWest as u8
                        | Direction::South as u8
                        | Direction::SouthEast as u8);
                }
                if x == 0 {
                    connections &= !(Direction::NorthWest as u8
                        | Direction::West as u8
                        | Direction::SouthWest as u8);
                }
                // might overflow
                if x == r.w - 1 {
                    connections &= !(Direction::NorthEast as u8
                        | Direction::East as u8
                        | Direction::SouthEast as u8);
                }

                maze[(x + r.x, y + r.y)].status = ConnectionStatus::Room;
                maze[(x + r.x, y + r.y)].connections |= connections;
            }
        }
    }

    // seperate maze into regions
    let mut num_unvisited = 0;
    let mut region_map: Vec<u32> = (0..maze.tiles.len() as u32).collect();
    for y in 0..height as i16 {
        for x in 0..width as i16 {
            let pos = Point::new(x, y);
            let status = maze[pos].status;

            // we need to connect all unvisited tiles and all rooms to each other seperately
            if status == ConnectionStatus::UnVisited {
                num_unvisited += 1;
            } else if status != ConnectionStatus::Room {
                continue;
            }

            if (wrap == Some(MazeWrap::Full) || wrap == Some(MazeWrap::Horizontal) || x > 0)
                && maze[pos.travel_wrapped(Direction::West, maze.width, maze.height)].status
                    == status
            {
                merge_sets(
                    &mut region_map,
                    maze.get_index(pos),
                    maze.get_index(pos.travel_wrapped(Direction::West, maze.width, maze.height)),
                );
            }

            if (wrap == Some(MazeWrap::Full) || wrap == Some(MazeWrap::Vertical) || y > 0)
                && maze[pos.travel_wrapped(Direction::North, maze.width, maze.height)].status
                    == status
            {
                merge_sets(
                    &mut region_map,
                    maze.get_index(pos),
                    maze.get_index(pos.travel_wrapped(Direction::North, maze.width, maze.height)),
                );
            }
        }
    }

    // ensure all regions are equal to their parent for easy comparisons
    for i in 0..region_map.len() {
        set_lookup_flatten(&mut region_map, i);
        if false {
            print!("{:3} ", region_map[i]);
            if i as u16 % width == width - 1 {
                println!();
            }
        }
    }

    // early return to ensure maze algos always recieve a maze with at least
    // 1 unvisited cell
    if num_unvisited < 1 {
        return (maze, MazeHistory::new(width, height, log_temps));
    }

    // holds a list of index-region tuples of unvisited cells
    let mut sorted_region_map = region_map
        .iter()
        .enumerate()
        .filter(|&(i, _)| maze.tiles[i].status == ConnectionStatus::UnVisited)
        .map(|(i, &r)| (i, r))
        .collect::<Vec<(usize, u32)>>();

    // shuffle the region map for algos that need a shuffled reservoir
    if mtype == MazeType::Wilson {
        sorted_region_map.shuffle(rng);
    }
    sorted_region_map.sort_by(|(_, a), (_, b)| a.cmp(b));

    // holds a list of points in the maze in the same order as the region map
    let open_tiles: Vec<Point> = sorted_region_map
        .iter()
        .map(|&(i, _)| Point::new((i as u16 % width) as i16, (i as u16 / width) as i16))
        .collect();

    let mut region_slices: Vec<&[Point]> = Vec::new();
    let mut current_region = sorted_region_map[0].1;
    let mut start_index = 0;
    for (indx, &(_, r)) in sorted_region_map.iter().enumerate() {
        if r != current_region {
            region_slices.push(&open_tiles[start_index..indx]);
            start_index = indx;
            current_region = r;
        }
    }
    region_slices.push(&open_tiles[start_index..sorted_region_map.len()]);

    // generate maze
    let mut history = MazeHistory::with_size_hint(width, height, log_temps, maze.tiles.len());
    match mtype {
        MazeType::Backtrack => {
            for region in region_slices {
                create_maze_backtrack(
                    &mut maze,
                    *region.choose(rng).unwrap(),
                    wrap,
                    log_temps,
                    &mut history,
                    rng,
                );
            }
        }
        MazeType::Prim => {
            for region in region_slices {
                create_maze_prim_true(
                    &mut maze,
                    *region.choose(rng).unwrap(),
                    wrap,
                    &mut history,
                    rng,
                );
            }
        }
        MazeType::GrowingTree => {
            for region in region_slices {
                create_maze_growingtree(
                    &mut maze,
                    *region.choose(rng).unwrap(),
                    wrap,
                    GrowingTreeBias::Newest,
                    &mut history,
                    rng,
                );
            }
        }
        MazeType::Wilson => {
            for region in region_slices {
                if region.len() == 1 {
                    maze[region[0]] = Tile {
                        status: ConnectionStatus::InMaze,
                        connections: Direction::NoDir as u8,
                    };
                    history.add_cell(region[0]);
                } else {
                    create_maze_wilson(&mut maze, region, wrap, log_temps, &mut history, rng);
                }
            }
        }
        MazeType::BinaryTree => create_maze_binary(&mut maze, &mut history, rng),
        MazeType::Sidewinder => create_maze_sidewinder(&mut maze, wrap, &mut history, rng),
        MazeType::Noise => create_maze_noise(&mut maze, &mut history, rng),
        MazeType::Kruskal => {
            // kruskals only works on edges so it wont fill single tiles
            for region in region_slices {
                if region.len() == 1 {
                    maze[region[0]] = Tile {
                        status: ConnectionStatus::InMaze,
                        connections: Direction::NoDir as u8,
                    };
                    history.add_cell(region[0]);
                }
            }
            create_maze_kruskal(&mut maze, wrap, &mut history, rng);
        }
        MazeType::PrimSimple => {
            for region in region_slices {
                create_maze_prim_simple(
                    &mut maze,
                    *region.choose(rng).unwrap(),
                    wrap,
                    &mut history,
                    rng,
                );
            }
        }
    }

    // add in doors to connect rooms to the rest of the maze

    // list of edge-region tuples
    let mut edges: Vec<Edge> = Vec::new();
    for room in rooms {
        // generate the list of edges out of the room
        // filter by the ones that connect 2 different regions
        // group the edges by which region they connect to
        // for each group of edges pick one and add it to the maze

        for y in 0..room.h {
            for x in 0..room.w {
                let pos = Point::new(room.x + x, room.y + y);
                let adj = match wrap {
                    Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
                    None => pos.adjacent(),
                };
                adj.enumerate()
                    .filter(|&(_, x)| {
                        maze.contains(x) && maze[x].status == ConnectionStatus::InMaze
                    })
                    .for_each(|(i, _)| {
                        edges.push(Edge(pos, Direction::from_clock_cardinal(i as u8)));
                    });
            }
        }
    }

    edges.shuffle(rng);
    for e in edges {
        let node1 = e.0;
        let node2 = if wrap.is_some() {
            e.0.travel_wrapped(e.1, maze.width, maze.height)
        } else {
            e.0.travel(e.1)
        };

        // if edge connects 2 different regions
        if merge_sets(
            &mut region_map,
            maze.get_index(node1),
            maze.get_index(node2),
        ) {
            history.carve(e.0, e.1);
            maze[node1].connect(e.1);
            maze[node2].connect(e.1.opposite());
        }
    }

    // uncarve deadends
    if uncarve_percent > 0 {
        let mut deadends = Vec::new();

        for y in 0..maze.height as i16 {
            for x in 0..maze.width as i16 {
                let pos = Point::new(x, y);
                if maze[pos].status == ConnectionStatus::InMaze
                    && maze[pos].count_connections() <= 1
                {
                    deadends.push(pos);
                }
            }
        }

        let num_to_remove = num_unvisited * uncarve_percent as u32 / 100;
        let mut num_removed = 0;

        while num_removed < num_to_remove {
            // pick a random deadend
            let index = rng.random_range(0..deadends.len());
            let pos = deadends[index];
            maze[pos].status = ConnectionStatus::Removed;

            if maze[pos].count_connections() == 0 {
                history.remove_cell(pos);
                deadends.swap_remove(index);
            } else {
                let dir: Direction = maze[pos].connections.into();
                history.uncarve(pos, dir);

                let new_pos = pos.travel(dir);
                maze[new_pos].unconnect(dir.opposite());

                if maze[new_pos].count_connections() == 1 {
                    deadends[index] = new_pos;
                } else {
                    deadends.swap_remove(index);
                }
            }

            num_removed += 1;
        }
    }

    (maze, history)
}

fn create_maze_backtrack(
    maze: &mut Grid,
    start_pos: Point,
    wrap: Option<MazeWrap>,
    log_temps: bool,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    let mut stack: Vec<(Point, Direction)> = Vec::new();
    let mut pos = start_pos;

    if log_temps {
        maze[pos].status = ConnectionStatus::Visited;
        history.carve_temp(pos, Direction::NoDir);
        history.place_marker(pos);
    } else {
        maze[pos].status = ConnectionStatus::InMaze;
        history.add_cell(pos);
    }
    stack.push((pos, Direction::NoDir));

    while !stack.is_empty() {
        pos = stack.last().unwrap().0;

        let adj = match wrap {
            Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
            None => pos.adjacent(),
        };
        let next = adj
            .enumerate()
            .filter(|&(_, x)| maze.contains(x) && maze[x].carveable())
            .choose(rng);

        match next {
            None => {
                let (p, d) = stack.pop().unwrap();
                if log_temps {
                    // convert temp
                    maze[p].status = ConnectionStatus::InMaze;
                    //history.carve(p, d);
                    history.move_marker(d);
                }
            }
            Some(next) => {
                let dir = Direction::from_clock_cardinal(next.0 as u8);
                maze[pos].connect(dir);

                pos = next.1;
                maze[pos].connect(dir.opposite());

                if log_temps {
                    maze[pos].status = ConnectionStatus::Visited;
                    // need to add temp tracking to marker code
                    //history.carve_temp(pos, dir.opposite());
                    history.move_marker_temp(dir);
                } else {
                    maze[pos].status = ConnectionStatus::InMaze;
                    history.carve(pos, dir.opposite());
                }

                stack.push((pos, dir.opposite()));
            }
        }
    }

    // try to remove the 1 frame delay before disapearing
    // it is caused by "moving" the marker to the first cell twice
    if log_temps {
        history.remove_marker();
    }
}

fn create_maze_prim_simple(
    maze: &mut Grid,
    start_pos: Point,
    wrap: Option<MazeWrap>,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    let mut open_tiles: Vec<Point> = Vec::new();
    let mut pos = start_pos;

    maze[pos].status = ConnectionStatus::InMaze;
    open_tiles.push(pos);
    history.add_cell(pos);

    while !open_tiles.is_empty() {
        let current_tile_index: usize = rng.random_range(0..open_tiles.len());
        pos = open_tiles[current_tile_index];

        let adj = match wrap {
            Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
            None => pos.adjacent(),
        };
        let next = adj
            .enumerate()
            .filter(|&(_, x)| maze.contains(x) && maze[x].carveable())
            .choose(rng);

        match next {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(next) => {
                let dir = Direction::from_clock_cardinal(next.0 as u8);
                maze[pos].connect(dir);

                pos = next.1;
                maze[pos].connect(dir.opposite());
                maze[pos].status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
                history.carve(pos, dir.opposite());
            }
        }
    }
}

fn create_maze_binary(maze: &mut Grid, history: &mut MazeHistory, rng: &mut impl Rng) {
    use crate::maze::Direction::*;

    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            if !maze[(x, y)].carveable() {
                continue;
            }

            let north_open: bool = y > 0 && !maze[(x, y - 1)].uncarveable();
            let west_open: bool = x > 0 && !maze[(x - 1, y)].uncarveable();

            let dir: u8 = if west_open && north_open {
                rng.random_range(0..=1)
            } else if west_open {
                0
            } else if north_open {
                1
            } else {
                2
            };

            if dir == 0 {
                maze[(x, y)].connect(West);
                history.carve(Point::new(x, y), West);
                maze[(x - 1, y)].connect(East);
            } else if dir == 1 {
                maze[(x, y)].connect(North);
                history.carve(Point::new(x, y), North);
                maze[(x, y - 1)].connect(South);
            } else {
                history.carve(Point::new(x, y), NoDir);
            }

            maze[(x, y)].status = ConnectionStatus::InMaze;
        }
    }
}

fn create_maze_sidewinder(
    maze: &mut Grid,
    wrap: Option<MazeWrap>,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    use crate::maze::Direction as D;

    maze[(0, 0)].connect(D::East);
    maze[(0, 0)].status = ConnectionStatus::InMaze;
    history.carve(Point::new(0, 0), D::NoDir);

    for x in 1..(maze.width - 1) as i16 {
        maze[(x, 0)].connections |= D::East as u8 | D::West as u8;
        maze[(x, 0)].status = ConnectionStatus::InMaze;
        history.carve(Point::new(x, 0), D::West);
    }

    let end = ((maze.width - 1) as i16, 0);
    maze[end].connect(D::West);
    maze[end].status = ConnectionStatus::InMaze;
    history.carve(Point::new((maze.width - 1) as i16, 0), D::West);

    for y in 1..maze.height as i16 {
        let mut range_start = if wrap.is_some() {
            rng.random_range(0..maze.width)
        } else {
            0
        };
        let mut cells_added = 0;

        while cells_added < maze.width {
            // creates longer passages
            //let range_len = rng.random_range(1..=maze.width - cells_added);
            // emulates cell by cell choice to extend the passage
            let mut range_len = 1;
            while range_len < maze.width - cells_added && rng.random::<bool>() {
                range_len += 1;
            }

            let vert_pos = Point::new(
                ((rng.random_range(0..range_len) + range_start) % maze.width) as i16,
                y,
            );
            let mut pos = Point::new(range_start as i16, y);
            maze[pos].status = ConnectionStatus::InMaze;

            for _ in 1..range_len {
                maze[pos].connect(D::East);

                if wrap.is_some() {
                    pos = pos.travel_wrapped(D::East, maze.width, maze.height);
                } else {
                    pos = pos.travel(D::East);
                }

                maze[pos].status = ConnectionStatus::InMaze;
                maze[pos].connect(D::West);
            }

            maze[vert_pos].connect(D::North);
            maze[vert_pos.travel(D::North)].connect(D::South);
            range_start = (range_start + range_len) % maze.width;
            cells_added += range_len;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrowingTreeBias {
    Oldest,
    Newest,
    Random,
    Percent(u8),
}

impl Default for GrowingTreeBias {
    fn default() -> Self {
        GrowingTreeBias::Percent(10)
    }
}

fn create_maze_growingtree(
    maze: &mut Grid,
    start_pos: Point,
    wrap: Option<MazeWrap>,
    bias: GrowingTreeBias,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    let mut open: Vec<Point> = Vec::new();

    maze[start_pos].status = ConnectionStatus::InMaze;
    history.add_cell(start_pos);
    open.push(start_pos);

    while !open.is_empty() {
        let selected_index = match bias {
            GrowingTreeBias::Oldest => 0,              // lowest river factor
            GrowingTreeBias::Newest => open.len() - 1, // backtrack
            GrowingTreeBias::Random => rng.random_range(0..open.len()), // similar to prim
            GrowingTreeBias::Percent(p) => {
                rng.random_range((open.len() / 100 * (100 - p as usize))..open.len())
            }
        };
        let selected = open[selected_index];
        let adj = match wrap {
            Some(w) => selected.adjacent_wrapped(w, maze.width, maze.height),
            None => selected.adjacent(),
        };
        let next = adj
            .enumerate()
            .filter(|&(_, x)| maze.contains(x) && maze[x].carveable())
            .choose(rng);

        match next {
            None => {
                open.remove(selected_index);
            }
            Some(next) => {
                let dir = Direction::from_clock_cardinal(next.0 as u8);
                maze[selected].connect(dir);

                let selected = next.1;
                maze[selected].connect(dir.opposite());
                maze[selected].status = ConnectionStatus::InMaze;

                open.push(selected);
                history.carve(selected, dir.opposite());
            }
        }
    }
}

// TODO: improve the retracing to happen faster/be smoother
fn create_maze_wilson(
    maze: &mut Grid,
    reservoir: &[Point],
    wrap: Option<MazeWrap>,
    log_temps: bool,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    assert!(reservoir.len() > 1, "Cell reservoir too small");

    let mut reservoir_index = 0;
    let mut anchor: Point = reservoir[reservoir_index];

    maze[anchor].status = ConnectionStatus::InMaze;
    if log_temps {
        history.place_marker(anchor);
    } else {
        history.add_cell(anchor);
    }

    'outer: loop {
        // pick a cell not already in the maze
        while !maze[anchor].carveable() {
            reservoir_index += 1;
            if reservoir_index >= reservoir.len() {
                break 'outer;
            } else {
                anchor = reservoir[reservoir_index];
            }
        }
        let mut pos = anchor;

        if log_temps {
            history.replace_marker(pos);
        }

        // start a random loop erased walk from the chosen cell
        while maze[pos].status == ConnectionStatus::UnVisited {
            let adj = match wrap {
                Some(w) => pos.adjacent_wrapped(w, maze.width, maze.height),
                None => pos.adjacent(),
            };
            let next = adj
                .enumerate()
                .filter(|&(_, x)| maze.contains(x) && !maze[x].uncarveable())
                .choose(rng)
                .unwrap(); // safe to unwrap because a cell will always have at least 1 adjacent cell in the maze (as long as there is more than 1 cell in the region)

            let dir = Direction::from_clock_cardinal(next.0 as u8);
            maze[pos].set_connected(dir);
            if log_temps {
                history.move_marker_temp(dir);
            }
            pos = next.1;
        }

        // carve the final path into the maze
        pos = anchor;
        let mut dir = Direction::NoDir;
        if log_temps {
            history.replace_marker(pos);
        }
        while maze[pos].status != ConnectionStatus::InMaze {
            // this line will panic if tile has multiple connections
            let temp_dir = maze[pos].connections.into();
            maze[pos].status = ConnectionStatus::InMaze;
            maze[pos].connect(dir.opposite());
            dir = temp_dir;

            if log_temps {
                history.move_marker(dir);
            } else {
                history.carve(pos, dir);
            }
            if wrap.is_some() {
                pos = pos.travel_wrapped(dir, maze.width, maze.height);
            } else {
                pos = pos.travel(dir);
            }
        }
        maze[pos].connect(dir.opposite());
        if log_temps {
            //history.remove_marker();
            history.remove_temp_cells();
        }
    }
    history.remove_marker();
}

// merge_sets 60x faster than simple array and 600x faster with set_lookup_flatten
fn create_maze_kruskal(
    maze: &mut Grid,
    wrap: Option<MazeWrap>,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    let mut edges: Vec<(Point, Direction)> = Vec::with_capacity(maze.tiles.len() * 2);
    let mut region_map: Vec<u32> = (0..maze.tiles.len() as u32).collect();

    // generate edges
    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            let pos = Point::new(x, y);
            if maze[pos].uncarveable() {
                continue;
            }

            if (wrap == Some(MazeWrap::Full) || wrap == Some(MazeWrap::Horizontal) || x > 0)
                && maze[pos.travel_wrapped(Direction::West, maze.width, maze.height)].carveable()
            {
                edges.push((pos, Direction::West));
            }

            if (wrap == Some(MazeWrap::Full) || wrap == Some(MazeWrap::Vertical) || y > 0)
                && maze[pos.travel_wrapped(Direction::North, maze.width, maze.height)].carveable()
            {
                edges.push((pos, Direction::North));
            }
        }
    }
    edges.shuffle(rng);

    // generate maze
    for edge in edges {
        let node1 = edge.0;
        let node2 = if wrap.is_some() {
            edge.0.travel_wrapped(edge.1, maze.width, maze.height)
        } else {
            edge.0.travel(edge.1)
        };

        // if edge connects 2 different regions
        if merge_sets(
            &mut region_map,
            maze.get_index(node1),
            maze.get_index(node2),
        ) {
            if maze[node1].status != ConnectionStatus::InMaze {
                maze[node1].status = ConnectionStatus::InMaze;
            }
            history.carve(edge.0, edge.1);
            maze[node1].connect(edge.1);

            if maze[node2].status != ConnectionStatus::InMaze {
                maze[node2].status = ConnectionStatus::InMaze;
                history.add_cell(node2);
            }
            maze[node2].connect(edge.1.opposite());
        }
    }
}

// 10x faster than normal lookup
fn set_lookup_flatten(region_map: &mut [u32], node: usize) -> u32 {
    let mut node = node as u32;
    let mut root = node;
    // find root of set (normal lookup)
    while region_map[root as usize] != root {
        root = region_map[root as usize];
    }

    // update nodes in path to point directly to root
    while region_map[node as usize] != node {
        let parent = region_map[node as usize];
        region_map[node as usize] = root;
        node = parent;
    }

    node
}

// returns true if sets needed to be merged
fn merge_sets(region_map: &mut [u32], lhs: usize, rhs: usize) -> bool {
    // determine parent nodes of each set
    // can optimize by storing size of each set and picking the right one to merge or by flattening during lookup
    // either is a ~10x speedup over using neither, no additional perfomance from using both
    let lhs_parent = set_lookup_flatten(region_map, lhs);
    let rhs_parent = set_lookup_flatten(region_map, rhs);

    if lhs_parent == rhs_parent {
        return false;
    }

    region_map[lhs_parent as usize] = rhs_parent;

    true
}

fn create_maze_prim_true(
    maze: &mut Grid,
    start_pos: Point,
    wrap: Option<MazeWrap>,
    history: &mut MazeHistory,
    rng: &mut impl Rng,
) {
    let mut open: Vec<(Point, Direction)> = Vec::new();

    maze[start_pos].status = ConnectionStatus::InMaze;
    history.add_cell(start_pos);

    match wrap {
        Some(w) => start_pos.adjacent_wrapped(w, maze.width, maze.height),
        None => start_pos.adjacent(),
    }
    .enumerate()
    .filter(|&(_, p)| maze.contains(p) && maze[p].carveable())
    .for_each(|(i, p)| {
        open.push((p, Direction::from_clock_cardinal(i as u8).opposite()));
        history.carve_temp(p, Direction::from_clock_cardinal(i as u8).opposite());
    });

    while !open.is_empty() {
        let edge = open.swap_remove(rng.random_range(0..open.len()));

        if maze[edge.0].status != ConnectionStatus::UnVisited {
            continue;
        }

        maze[edge.0].status = ConnectionStatus::InMaze;
        maze[edge.0].connect(edge.1);

        history.carve(edge.0, edge.1);

        let target = if wrap.is_some() {
            edge.0.travel_wrapped(edge.1, maze.width, maze.height)
        } else {
            edge.0.travel(edge.1)
        };
        maze[target].connect(edge.1.opposite());

        match wrap {
            Some(w) => edge.0.adjacent_wrapped(w, maze.width, maze.height),
            None => edge.0.adjacent(),
        }
        .enumerate()
        .filter(|&(_, p)| maze.contains(p) && maze[p].carveable())
        .for_each(|(i, p)| {
            open.push((p, Direction::from_clock_cardinal(i as u8).opposite()));
            history.carve_temp(p, Direction::from_clock_cardinal(i as u8).opposite());
        });
    }
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

fn generate_noise(
    world_width: u16,
    world_height: u16,
    grid_width: u16,
    grid_height: u16,
    rng: &mut impl Rng,
) -> Vec<f32> {
    // can over-estimate length and be fine
    let cell_width = if world_width % (grid_width - 1) == 0 {
        world_width / (grid_width - 1)
    } else {
        world_width / (grid_width - 1) + 1
    };
    let cell_height = if world_height % (grid_height - 1) == 0 {
        world_height / (grid_height - 1)
    } else {
        world_height / (grid_height - 1) + 1
    };

    let mut points: Vec<f32> = vec![0.0f32; (world_width * world_height) as usize];
    let mut grid: Vec<Vector2<f32>> = Vec::with_capacity((grid_width * grid_height) as usize);

    // fill grid with random direction vectors
    for _ in 0..(grid_width * grid_height) {
        grid.push(normalize(Vector2 {
            x: rng.random_range(-1.0..=1.0),
            y: rng.random_range(-1.0..=1.0),
        }));
    }

    // calculate perlin noise for each point in the world
    for y in 0..world_height {
        for x in 0..world_width {
            let grid_offset = Vector2 {
                x: x % cell_width,
                y: y % cell_height,
            };
            let grid_pos = Vector2 {
                x: x / cell_width,
                y: y / cell_height,
            };

            // offset vectors from each nearby grid point to current world point
            let offset_vectors: [Vector2<f32>; 4] = [
                Vector2 {
                    x: ((grid_offset.x) as f32),
                    y: ((grid_offset.y) as f32),
                },
                Vector2 {
                    x: -((cell_width - grid_offset.x) as f32),
                    y: ((grid_offset.y) as f32),
                },
                Vector2 {
                    x: ((grid_offset.x) as f32),
                    y: -((cell_height - grid_offset.y) as f32),
                },
                Vector2 {
                    x: -((cell_width - grid_offset.x) as f32),
                    y: -((cell_height - grid_offset.y) as f32),
                },
            ];

            // dot product of each offset vector and its respective direction vector
            let dots: [f32; 4] = [
                Vector2::dot(
                    grid[((grid_pos.x + 0) + (grid_pos.y + 0) * grid_width) as usize],
                    offset_vectors[0],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 1) + (grid_pos.y + 0) * grid_width) as usize],
                    offset_vectors[1],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 0) + (grid_pos.y + 1) * grid_width) as usize],
                    offset_vectors[2],
                ),
                Vector2::dot(
                    grid[((grid_pos.x + 1) + (grid_pos.y + 1) * grid_width) as usize],
                    offset_vectors[3],
                ),
            ];

            // calculate step for interpolation
            let step = Vector2 {
                x: (grid_offset.x as f32) / (cell_width as f32),
                y: (grid_offset.y as f32) / (cell_height as f32),
            };

            // interpolate over x and y direction
            let int_x1 = interpolate(dots[0], dots[1], step.x);
            let int_x2 = interpolate(dots[2], dots[3], step.x);
            let int_y = interpolate(int_x1, int_x2, step.y);

            // dot product will range from -cell_width to cell_width
            points[(x + y * world_width) as usize] = int_y / (cell_width as f32) * 1.5;
        }
    }

    for p in &mut points {
        *p = if *p <= 0.0 { -1.0 } else { 1.0 };
    }
    /*

    let path = Path::new(r"./noise.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, world_width as u32, world_height as u32);
    encoder.set_color(png::ColorType::Rgb);

    let mut writer = encoder.write_header().unwrap();

    let mut pixels: Vec<ColorRGB> = vec![
        ColorRGB {
            red: 0,
            green: 0,
            blue: 0
        };
        (world_width * world_height) as usize
    ];

    for i in 0..(world_width * world_height) as usize {
        pixels[i] = get_color(points[i]);
    }

    writer
        .write_image_data(&ColorRGB::as_bytes(&pixels))
        .unwrap();
        */

    points
}

fn flood_tile_prim(maze: &mut Grid, noise_map: &[u8], mut pos: Point, rng: &mut impl Rng) {
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

    open_tiles.push(pos);
    maze.tiles[(pos.x + pos.y * maze.width as i16) as usize].status = ConnectionStatus::InMaze;
    while !open_tiles.is_empty() {
        let current_tile_index: usize = rng.random_range(0..open_tiles.len());
        pos = open_tiles[current_tile_index];

        let next = pick_random(
            pos.adjacent()
                .enumerate()
                .filter(|&(_, x)| {
                    maze.contains(x)
                        && maze[x].status == ConnectionStatus::UnVisited
                        && noise_map[(x.x + x.y * maze.width as i16) as usize] == 1
                })
                .collect::<Vec<(usize, Point)>>()
                .as_ref(),
            rng,
        );

        match next {
            None => {
                open_tiles.swap_remove(current_tile_index);
            }
            Some(next) => {
                let dir = Direction::from_clock_cardinal(next.0 as u8);
                maze[pos].connect(dir);

                pos = next.1;
                maze[pos].connect(dir.opposite());
                maze[pos].status = ConnectionStatus::InMaze;

                open_tiles.push(pos);
            }
        }
    }
}

fn flood_tile_backtrack(maze: &mut Grid, noise_map: &[u8], mut pos: Point, rng: &mut impl Rng) {
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
    maze[pos].status = ConnectionStatus::InMaze;

    while !tile_stack.is_empty() {
        let next = pick_random(
            pos.adjacent()
                .enumerate()
                .filter(|&(_, x)| {
                    maze.contains(x)
                        && maze[x].status == ConnectionStatus::UnVisited
                        && noise_map[(x.x + x.y * maze.width as i16) as usize] == 1
                })
                .collect::<Vec<(usize, Point)>>()
                .as_ref(),
            rng,
        );

        match next {
            None => {
                // we can upwrap here because we ensure the stack is non-empty in the loop clause
                pos = tile_stack.pop().unwrap();
            }
            Some(next) => {
                let dir = Direction::from_clock_cardinal(next.0 as u8);
                maze[pos].connect(dir);

                pos = next.1;
                maze[pos].connect(dir.opposite());
                maze[pos].status = ConnectionStatus::InMaze;

                tile_stack.push(pos);
            }
        }
    }
}

fn create_maze_noise(maze: &mut Grid, _history: &mut MazeHistory, rng: &mut impl Rng) {
    let noise_map: Vec<u8> = generate_noise(maze.width, maze.height, 7, 7, rng)
        .iter()
        .map(|x| if *x < 0.0 { 0 } else { 1 })
        .collect();

    for y in 0..maze.height as i16 {
        for x in 0..maze.width as i16 {
            flood_tile_prim(maze, &noise_map, Point { x, y }, rng);
            flood_tile_backtrack(maze, &noise_map, Point { x, y }, rng);
        }
    }

    /*
        need to add random stopping and then also implement connecting of maze regions
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite() {
        let result_north = Direction::North.opposite();
        assert_eq!(result_north, Direction::South);
        let result_northeast = Direction::NorthEast.opposite();
        assert_eq!(result_northeast, Direction::SouthWest);

        let result_east = Direction::East.opposite();
        assert_eq!(result_east, Direction::West);
        let result_eastsouth = Direction::SouthEast.opposite();
        assert_eq!(result_eastsouth, Direction::NorthWest);

        let result_south = Direction::South.opposite();
        assert_eq!(result_south, Direction::North);
        let result_southwest = Direction::SouthWest.opposite();
        assert_eq!(result_southwest, Direction::NorthEast);

        let result_west = Direction::West.opposite();
        assert_eq!(result_west, Direction::East);
        let result_westnorth = Direction::NorthWest.opposite();
        assert_eq!(result_westnorth, Direction::SouthEast);
    }

    #[test]
    fn clock() {
        let result_north = Direction::from_clock_cardinal(0);
        assert_eq!(result_north, Direction::North);

        let result_east = Direction::from_clock_cardinal(1);
        assert_eq!(result_east, Direction::East);

        let result_south = Direction::from_clock_cardinal(2);
        assert_eq!(result_south, Direction::South);

        let result_west = Direction::from_clock_cardinal(3);
        assert_eq!(result_west, Direction::West);
    }

    #[test]
    fn tile() {
        let mut t = Tile::default();

        assert_eq!(t.count_connections(), 0);

        t.connect(Direction::North);
        assert_eq!(t.count_connections(), 1);

        t.connect(Direction::South);
        assert_eq!(t.count_connections(), 2);

        t.connect(Direction::East);
        assert_eq!(t.count_connections(), 3);

        t.unconnect(Direction::South);
        assert_eq!(t.count_connections(), 2);
    }
}
