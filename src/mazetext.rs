use crate::grid::{ConnectionStatus, Direction, Grid, Point, Rect, Tile};
use std::fs::File;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeFontError {
    UnsupportedSymbol,
    BadDimensions,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FontSymbol {
    // pixels stored as bit planes
    pixels: [u8; 9],
    width: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MazeFont {
    symbols: [FontSymbol; 96],
}

impl MazeFont {
    /// read in a font from a png
    pub fn read_font(file_path: &str) -> Result<Self, MazeFontError> {
        let symbol_width = 8;
        let symbol_height = 9;

        let decoder = png::Decoder::new(File::open(file_path).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let image_info = reader.info();
        let image_width = image_info.width;
        let image_height = image_info.height;

        if image_height != symbol_height * 3 || image_width != symbol_width * 32 {
            return Err(MazeFontError::BadDimensions);
        }

        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..info.buffer_size()];

        let mut font = MazeFont {
            symbols: [FontSymbol::default(); 96],
        };

        // read in the symbols from the image
        for (i, &b) in bytes.into_iter().enumerate() {
            font.symbols[(i & 0b0_00011111) + (i >> 5) / 9 * 32].pixels[(i >> 5) % 9] = b;
        }

        // generate the width of each symbol
        for i in 0..font.symbols.len() {
            let mut max_len = 0;
            for row in 0..9 {
                max_len = std::cmp::max(max_len, 8 - font.symbols[i].pixels[row].trailing_zeros());
            }
            font.symbols[i].width = max_len as u8;
        }

        Ok(font)
    }

    pub fn get_symbol(&self, c: char) -> Result<FontSymbol, MazeFontError> {
        if (c as u32) & 0b10000000 != 0 || (c as u32) < 32 {
            Err(MazeFontError::UnsupportedSymbol)
        } else {
            Ok(self.symbols[c as usize - 32])
        }
    }

    pub fn generate_text(
        &self,
        s: &str,
        mut pos: Point,
        maze: &mut Grid,
    ) -> Result<(), MazeFontError> {
        let symbol_width = 6;
        let tile = Tile {
            status: ConnectionStatus::Removed,
            connections: Direction::NoDir as u8,
        };

        for c in s.chars() {
            let sym = self.get_symbol(c)?;

            for y in 0..9 {
                for x in 0..8 {
                    if (sym.pixels[y as usize] >> (7 - x)) & 1 == 1 {
                        maze.set_tile(pos + Point { x, y }, tile);
                    }
                }
            }

            pos.x += sym.width as i16 + 1;
        }

        Ok(())
    }
}
