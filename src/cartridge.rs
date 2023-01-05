use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[allow(dead_code)]
struct INESHeader {
    magic: [u8; 4],
    prg_rom_size: u8,
    chr_rom_size: u8,
    flags6: u8,
    flags7: u8,
    flags8: u8,
    flags9: u8,
    flags10: u8,
    unused: [u8; 5],
}

fn parse_ines_header(header_bytes: &[u8]) -> Result<INESHeader, Box<dyn Error>> {
    let header = INESHeader {
        magic: header_bytes[0..4].try_into()?,
        prg_rom_size: header_bytes[4],
        chr_rom_size: header_bytes[5],
        flags6: header_bytes[6],
        flags7: header_bytes[7],
        flags8: header_bytes[8],
        flags9: header_bytes[9],
        flags10: header_bytes[10],
        unused: [0; 5],
    };

    if header.magic != [b'N', b'E', b'S', 0x1A] {
        return Err("Not an INES file".into());
    }

    Ok(header)
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum MapperType {
    #[default]
    Nrom,
    MMC1,
    Uxrom,
    Cnrom,
    MMC3,
    Axrom,
}

#[derive(Clone)]
pub struct Cartridge {
    pub chr_rom: Option<Vec<u8>>,
    pub mapper_type: MapperType,
    pub mirroring: Mirroring,
    pub prg_rom: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    OneScreenLowerBank,
    OneScreenUpperBank,
    FourScreen,
    SingleScreen,
}

impl Cartridge {
    pub fn new(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = fs::read(filename)?;

        let header = parse_ines_header(&file[..0x10])?;

        let prg_rom_size = header.prg_rom_size as usize * 0x400 * 16;
        let chr_rom_size = header.chr_rom_size as usize * 0x400 * 8;
        let prg_ram_size = header.flags8 as usize * 0x400 * 8;

        if prg_ram_size > 1 {
            todo!("PRG RAM size > 8KiB not supported.");
        }

        if header.flags6.bit(2) {
            todo!("Trainers are not supported.");
        }

        let ines_ver = header.flags7.bits(2, 3);

        if ines_ver == 2 {
            todo!("INES 2.0 not supported.");
        }

        let mapper_ln = header.flags6.bits(4, 7);
        let mapper_un = header.flags7.bits_abs(4, 7);
        let mapper_num = mapper_ln | mapper_un;

        let mirroring = if header.flags6.bit(3) {
            Mirroring::FourScreen
        } else if header.flags6.bit(0) {
            Mirroring::Vertical
        } else if mapper_num == 7 {
            Mirroring::SingleScreen
        } else {
            Mirroring::Horizontal
        };

        let start_of_chr_rom = 0x10 + prg_rom_size;

        let chr_rom = Vec::from(&file[start_of_chr_rom..start_of_chr_rom + chr_rom_size]);

        let prg_rom = Vec::from(&file[0x10..0x10 + prg_rom_size]);

        let mapper_type = match mapper_num {
            0 => MapperType::Nrom,
            1 => MapperType::MMC1,
            2 => MapperType::Uxrom,
            3 => MapperType::Cnrom,
            4 => MapperType::MMC3,
            7 => MapperType::Axrom,
            _ => {
                todo!("Mapper {mapper_num} not implemented yet")
            }
        };

        Ok(Self {
            chr_rom: Some(chr_rom),
            mapper_type,
            mirroring,
            prg_rom: Some(prg_rom),
        })
    }
}
