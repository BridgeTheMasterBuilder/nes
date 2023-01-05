use std::error::Error;

use byte_slice_cast::AsByteSlice;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::Sdl;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub(super) struct Screen {
    pub fb: Texture,
    screen: WindowCanvas,
}

impl Screen {
    pub fn new(sdl_context: &Sdl) -> Result<Self, Box<dyn Error>> {
        let canvas = {
            let video_subsystem = sdl_context.video()?;

            sdl_context.mouse().show_cursor(false);

            let window = video_subsystem
                .window("NES Emulator", WIDTH, HEIGHT)
                .position_centered()
                .opengl()
                .build()?;

            window.into_canvas().present_vsync().build()?
        };

        let texture_creator = canvas.texture_creator();

        Ok(Self {
            fb: {
                texture_creator.create_texture_streaming(
                    Some(PixelFormatEnum::ABGR8888),
                    256,
                    240,
                )?
            },
            screen: canvas,
        })
    }

    pub fn render(&mut self, fb: &[u32]) -> Result<(), Box<dyn Error>> {
        self.fb.update(None, fb.as_byte_slice(), 256 * 4)?;

        self.screen.copy(&self.fb, None, None)?;

        self.screen.present();

        Ok(())
    }
}
