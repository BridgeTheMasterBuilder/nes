use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) struct Control {
    pub(super) nmi: bool,
    pub(super) master_slave: bool,
    pub(super) size: u8,
    pub(super) bg_pt_addr: u16,
    pub(super) spr_pt_addr: u16,
    pub(super) inc_vert: bool,
    pub(super) base_nt_addr: u16,
}

impl Control {
    pub fn new() -> Self {
        Self {
            nmi: true,
            master_slave: false,
            size: 8 * 8,
            bg_pt_addr: 0x0000,
            spr_pt_addr: 0x0000,
            inc_vert: false,
            base_nt_addr: 0x2000,
        }
    }
}

impl From<Control> for u8 {
    fn from(item: Control) -> Self {
        let nmi = item.nmi as u8;
        let master_slave = item.master_slave as u8;
        let size = (item.size == 8 * 8) as u8;
        let bg_pt_addr = (item.bg_pt_addr == 0x1000) as u8;
        let spr_pt_addr = (item.spr_pt_addr == 0x1000) as u8;
        let inc_vert = item.inc_vert as u8;
        let base_nt_addr = match item.base_nt_addr {
            0x2000 => 0,
            0x2400 => 1,
            0x2800 => 2,
            0x2C00 => 3,
            _ => unreachable!("Illegal base nametable address"),
        };

        nmi << 7
            | master_slave << 6
            | size << 5
            | bg_pt_addr << 4
            | spr_pt_addr << 3
            | inc_vert << 2
            | base_nt_addr
    }
}

impl From<u8> for Control {
    fn from(item: u8) -> Self {
        let nmi = item.bit(7);
        let master_slave = item.bit(6);
        let size = item.bit(5);
        let bg_pt_addr = item.bit(4);
        let spr_pt_addr = item.bit(3);
        let inc_vert = item.bit(2);
        let base_nt_addr = item.bits(0, 1);

        Self {
            nmi,
            master_slave,
            size: if size { 8 * 16 } else { 8 * 8 },
            bg_pt_addr: if bg_pt_addr { 0x1000 } else { 0x0000 },
            spr_pt_addr: if spr_pt_addr { 0x1000 } else { 0x0000 },
            inc_vert,
            base_nt_addr: match base_nt_addr {
                0 => 0x2000,
                1 => 0x2400,
                2 => 0x2800,
                3 => 0x2C00,
                _ => unreachable!(),
            },
        }
    }
}
