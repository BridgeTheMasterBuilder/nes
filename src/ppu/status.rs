use serde::{Deserialize, Serialize};

use crate::util::bit::Bit;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) struct Status {
    pub(super) vblank: bool,
    pub(super) spr_0_hit: bool,
    pub(super) spr_overflow: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            vblank: false,
            spr_0_hit: false,
            spr_overflow: false,
        }
    }
}

impl From<Status> for u8 {
    fn from(item: Status) -> Self {
        let vblank = item.vblank as u8;
        let spr_0_hit = item.spr_0_hit as u8;
        let spr_overflow = item.spr_overflow as u8;

        vblank << 7 | spr_0_hit << 6 | spr_overflow << 5
    }
}

impl From<u8> for Status {
    fn from(item: u8) -> Self {
        let vblank = item.bit(7);
        let spr_0_hit = item.bit(6);
        let spr_overflow = item.bit(5);

        Self {
            vblank,
            spr_0_hit,
            spr_overflow,
        }
    }
}
