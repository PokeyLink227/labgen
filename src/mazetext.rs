use crate::grid::{ConnectionStatus, Direction, Grid, Point, Rect, Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FontSymbol {
    pixels: [[u8; 5]; 9],
    width: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MazeFont {
    symbols: [FontSymbol; 3],
}

impl MazeFont {
    /// read in a font from a png
    pub fn read_font(file_path: &str) -> Self {
        MazeFont {
            symbols: [
                FontSymbol {
                    pixels: [
                        [0, 0, 1, 0, 0],
                        [0, 1, 0, 1, 0],
                        [0, 1, 0, 1, 0],
                        [1, 0, 0, 0, 1],
                        [1, 1, 1, 1, 1],
                        [1, 0, 0, 0, 1],
                        [1, 0, 0, 0, 1],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                    ],
                    width: 5,
                },
                FontSymbol {
                    pixels: [
                        [1, 1, 1, 1, 0],
                        [1, 0, 0, 0, 1],
                        [1, 0, 0, 0, 1],
                        [1, 1, 1, 1, 0],
                        [1, 0, 0, 0, 1],
                        [1, 0, 0, 0, 1],
                        [1, 1, 1, 1, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                    ],
                    width: 5,
                },
                FontSymbol {
                    pixels: [
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                        [0, 0, 0, 0, 0],
                    ],
                    width: 1,
                },
            ],
        }
    }

    pub fn get_symbol(&self, c: char) -> FontSymbol {
        if c == 'A' {
            self.symbols[0]
        } else if c == ' ' {
            self.symbols[2]
        } else {
            self.symbols[1]
        }
    }

    pub fn generate_text(&self, s: &str, mut pos: Point, maze: &mut Grid) {
        let symbol_width = 6;
        let tile = Tile {
            status: ConnectionStatus::Removed,
            connections: Direction::NoDir as u8,
        };

        for c in s.chars() {
            let sym = self.get_symbol(c);

            for y in 0..9 {
                for x in 0..5 {
                    if sym.pixels[y as usize][x as usize] == 1 {
                        maze.set_tile(pos + Point { x, y }, tile);
                    }
                }
            }

            pos.x += sym.width as i16 + 1;
        }
    }
}
