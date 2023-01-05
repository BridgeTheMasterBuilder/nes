use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl From<Rgba> for [u8; 4] {
    fn from(rgba: Rgba) -> Self {
        [rgba.3, rgba.2, rgba.1, rgba.0]
    }
}

impl From<Rgba> for u32 {
    fn from(rgba: Rgba) -> Self {
        let a = rgba.0 as u32;
        let b = rgba.1 as u32;
        let g = rgba.2 as u32;
        let r = rgba.3 as u32;

        r << 24 | g << 16 | b << 8 | a
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Palette(#[serde(with = "BigArray")] [Rgba; 64]);

impl Index<usize> for Palette {
    type Output = Rgba;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Palette {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PaletteTable {
    #[serde(with = "BigArray")]
    palettes: [Palette; 8],
    selected: usize,
}

impl PaletteTable {
    pub fn new() -> Self {
        let buf = include_bytes!("../../ntscpalette.pal");
        let mut palette_arr: [Palette; 8] = [Palette([Rgba(0, 0, 0, 0); 64]); 8];

        buf.chunks_exact(64 * 3)
            .enumerate()
            .for_each(|(palette_number, palette)| {
                palette
                    .chunks_exact(3)
                    .enumerate()
                    .for_each(|(palette_offset, chunk)| {
                        let r = chunk[0];
                        let g = chunk[1];
                        let b = chunk[2];
                        palette_arr[palette_number][palette_offset] = Rgba(r, g, b, 255);
                    })
            });

        Self {
            palettes: palette_arr,
            selected: 0,
        }
    }

    pub fn select_palette(&mut self, idx: usize) {
        self.selected = idx;
    }
}

impl Index<u8> for PaletteTable {
    type Output = Rgba;

    fn index(&self, index: u8) -> &Self::Output {
        &self.palettes[self.selected][index as usize]
    }
}
