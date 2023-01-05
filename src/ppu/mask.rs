use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) struct Mask {
    pub(super) emphasize_blue: bool,
    pub(super) emphasize_green: bool,
    pub(super) emphasize_red: bool,
    pub(super) show_sprites: bool,
    pub(super) show_background: bool,
    pub(super) show_sprites_leftmost_8: bool,
    pub(super) show_background_leftmost_8: bool,
    pub(super) greyscale: bool,
}

impl Mask {
    pub fn new() -> Self {
        Self {
            emphasize_blue: false,
            emphasize_green: false,
            emphasize_red: false,
            show_sprites: true,
            show_background: true,
            show_sprites_leftmost_8: true,
            show_background_leftmost_8: true,
            greyscale: false,
        }
    }
}

impl From<Mask> for u8 {
    fn from(item: Mask) -> Self {
        let emphasize_blue = item.emphasize_blue as u8;
        let emphasize_green = item.emphasize_green as u8;
        let emphasize_red = item.emphasize_red as u8;
        let show_sprites = item.show_sprites as u8;
        let show_background = item.show_background as u8;
        let show_sprites_leftmost_8 = item.show_sprites_leftmost_8 as u8;
        let show_background_leftmost_8 = item.show_background_leftmost_8 as u8;
        let greyscale = item.greyscale as u8;

        emphasize_blue << 7
            | emphasize_green << 6
            | emphasize_red << 5
            | show_sprites << 4
            | show_background << 3
            | show_sprites_leftmost_8 << 2
            | show_background_leftmost_8 << 1
            | greyscale
    }
}

impl From<u8> for Mask {
    fn from(item: u8) -> Self {
        let emphasize_blue = item.bit(7);
        let emphasize_green = item.bit(6);
        let emphasize_red = item.bit(5);
        let show_sprites = item.bit(4);
        let show_background = item.bit(3);
        let show_sprites_leftmost_8 = item.bit(2);
        let show_background_leftmost_8 = item.bit(1);
        let greyscale = item.bit(0);

        Self {
            emphasize_blue,
            emphasize_green,
            emphasize_red,
            show_sprites,
            show_background,
            show_sprites_leftmost_8,
            show_background_leftmost_8,
            greyscale,
        }
    }
}
