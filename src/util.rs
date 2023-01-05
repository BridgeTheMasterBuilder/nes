pub mod bit;
pub mod shift_reg;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(filename: &str) -> Config {
        Config {
            filename: String::from(filename),
        }
    }
}

pub fn crosses_page(src: u16, offset: i32) -> bool {
    let normalized_src = (src % 0x100) as i32;
    let target = normalized_src + offset;

    !(0..=0xFF).contains(&target)
}

pub fn load_ram(filename: &str) -> [u8; 0x2000] {
    let mut name = Path::new(filename).to_path_buf();

    name.set_extension("sav");

    if let Ok(mut save_file) = File::open(name) {
        let mut buf: [u8; 0x2000] = [0; 0x2000];

        save_file.read(&mut buf).map_or([0; 0x2000], |_| buf)
    } else {
        [0; 0x2000]
    }
}

pub fn save_ram(filename: &str, data: &[u8]) {
    let mut name = Path::new(filename).to_path_buf();

    name.set_extension("sav");

    if let Ok(mut save_file) = File::create(name) {
        save_file
            .write_all(data)
            .unwrap_or_else(|_| eprintln!("WARNING: Failed to save RAM to disk"));
    } else {
        eprintln!("Unable to open save file.");
    }
}
