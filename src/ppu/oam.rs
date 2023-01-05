use std::cell::Cell;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::util::bit::Bit;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Attributes {
    pub palette: u8,
    pub priority: bool,
    pub flip_horiz: bool,
    pub flip_vert: bool,
}

impl Attributes {
    pub fn new() -> Self {
        Self {
            palette: 0,
            priority: false,
            flip_horiz: false,
            flip_vert: false,
        }
    }
}

impl From<Attributes> for u8 {
    fn from(item: Attributes) -> Self {
        let palette = item.palette - 4;
        let priority = item.priority as u8;
        let flip_horiz = item.flip_horiz as u8;
        let flip_vert = item.flip_vert as u8;

        flip_vert << 7 | flip_horiz << 6 | priority << 5 | palette
    }
}

impl From<u8> for Attributes {
    fn from(item: u8) -> Self {
        let palette = item.bits(0, 1);
        let priority = item.bit(5);
        let flip_horiz = item.bit(6);
        let flip_vert = item.bit(7);

        Self {
            palette: palette + 4,
            priority,
            flip_horiz,
            flip_vert,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub y: u8,
    pub tile_idx: u8,
    pub attrib: Attributes,
    pub x: u8,
}

impl From<&[u8]> for Sprite {
    fn from(item: &[u8]) -> Self {
        let y = item[0];
        let tile_idx = item[1];
        let attrib = Attributes::from(item[2]);
        let x = item[3];

        Self {
            y,
            tile_idx,
            attrib,
            x,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Oam {
    #[serde(with = "BigArray")]
    pub(super) buffer: [u8; 256],
    pub(super) overflow: bool,
    pub(super) sprites: Vec<(usize, Sprite, usize)>,
    addr: Cell<u8>,
}

impl Oam {
    pub fn new() -> Self {
        Self {
            buffer: [0; 256],
            overflow: false,
            sprites: Vec::with_capacity(8),
            addr: Cell::new(0),
        }
    }

    pub fn addr(&self) -> u8 {
        self.addr.get()
    }

    pub fn calculate_sprites_on_scanline(&mut self, scanline: usize, size: u8) {
        let sprites = &mut self.sprites;
        sprites.clear();

        self.overflow = false;

        let scanline = scanline as i16;

        for spr_addr in (0..256).step_by(4) {
            let spr_y = self.buffer[spr_addr] as i16 + 1;
            let spr_yoff = scanline - spr_y;

            let spr_height = (size / 8) as i16;

            if spr_yoff >= 0 && spr_yoff < spr_height {
                if sprites.len() == 8 {
                    self.overflow = true;
                    break;
                }

                let sprite = Sprite::from(&self.buffer[spr_addr..spr_addr + 4]);

                sprites.push((spr_addr, sprite, spr_yoff as usize));
            }
        }
    }

    pub fn read(&self, increment: bool) -> u8 {
        let addr = self.addr.get();

        let data = self.buffer[addr as usize];

        if increment {
            self.addr.replace((addr).wrapping_add(1));
        }

        data
    }

    pub fn set_addr(&mut self, addr: u8) {
        self.addr.replace(addr);
    }

    pub fn sprites(&self) -> &[u8] {
        &self.buffer
    }

    pub fn write(&mut self, data: u8) {
        let addr = self.addr.get();

        self.buffer[addr as usize] = data;

        self.addr.replace((addr).wrapping_add(1));
    }
}
