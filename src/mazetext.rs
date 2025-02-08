use crate::grid::{ConnectionStatus, Direction, Grid, Point, Rect, Tile};
use regex::Regex;
use std::{cell::LazyCell, fs::File, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeTextError {
    UnsupportedSymbol,
    BadFontFileDimensions,
    FontFileMissing,
    CouldntParseText,
    MazeTooSmall,
}

#[derive(Debug, Clone, Copy)]
pub struct MazeText<'a>(pub Point, pub &'a str);

impl<'a> MazeText<'a> {
    pub fn from_str(s: &'a str) -> Result<MazeText<'a>, MazeTextError> {
        let re: LazyCell<Regex> =
            LazyCell::new(|| Regex::new(r"\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(.+)\)").unwrap());

        let caps = re.captures(s).ok_or(MazeTextError::CouldntParseText)?;

        return Ok(MazeText(
            Point::new(
                caps[1].parse().or(Err(MazeTextError::CouldntParseText))?,
                caps[2].parse().or(Err(MazeTextError::CouldntParseText))?,
            ),
            caps.get(3).unwrap().as_str(),
        ));
    }
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
    pub fn read_font(file_path: &str) -> Result<Self, MazeTextError> {
        let symbol_width = 8;
        let symbol_height = 9;

        let decoder =
            png::Decoder::new(File::open(file_path).or(Err(MazeTextError::FontFileMissing))?);
        let mut reader = decoder
            .read_info()
            .or(Err(MazeTextError::FontFileMissing))?;
        let image_info = reader.info();
        let image_width = image_info.width;
        let image_height = image_info.height;

        if image_height != symbol_height * 3 || image_width != symbol_width * 32 {
            return Err(MazeTextError::BadFontFileDimensions);
        }

        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader
            .next_frame(&mut buf)
            .or(Err(MazeTextError::FontFileMissing))?;
        let bytes = &buf[..info.buffer_size()];

        let mut font = MazeFont {
            symbols: [FontSymbol::default(); 96],
        };

        // read in the symbols from the image
        for (i, &b) in bytes.into_iter().enumerate() {
            font.symbols[(i & 0b0_00011111) + (i >> 5) / 9 * 32].pixels[(i >> 5) % 9] = b;
        }

        // generate the width of each symbol
        font.symbols[0].width = 1;
        for i in 1..font.symbols.len() {
            let mut max_len = 0;
            for row in 0..9 {
                max_len = std::cmp::max(max_len, 8 - font.symbols[i].pixels[row].trailing_zeros());
            }
            font.symbols[i].width = max_len as u8;
        }

        Ok(font)
    }

    pub fn get_symbol(&self, c: char) -> Result<FontSymbol, MazeTextError> {
        if (c as u32) & 0b10000000 != 0 || (c as u32) < 32 {
            Err(MazeTextError::UnsupportedSymbol)
        } else {
            Ok(self.symbols[c as usize - 32])
        }
    }

    pub fn generate_text(&self, text: MazeText, maze: &mut Grid) -> Result<(), MazeTextError> {
        let mut pos = text.0;
        let tile = Tile {
            status: ConnectionStatus::Removed,
            connections: Direction::NoDir as u8,
        };

        for c in text.1.chars() {
            let sym = self.get_symbol(c)?;

            if pos.x as u16 + sym.width as u16 >= maze.width {
                return Err(MazeTextError::MazeTooSmall);
            }

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
