use crate::uid::Uid;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Block {
    opcode: &'static str,
    pub(crate) parent: Option<Uid>,
    pub(crate) next: Option<Uid>,
    top_level: bool,
}

pub struct Hat {
    opcode: &'static str,
}

impl From<Hat> for Block {
    fn from(hat: Hat) -> Self {
        Self {
            opcode: hat.opcode,
            parent: None,
            next: None,
            top_level: true,
        }
    }
}

pub struct Stacking {
    opcode: &'static str,
}

impl From<Stacking> for Block {
    fn from(stacking: Stacking) -> Self {
        Self {
            opcode: stacking.opcode,
            parent: None,
            next: None,
            top_level: false,
        }
    }
}

#[must_use]
pub const fn when_flag_clicked() -> Hat {
    Hat {
        opcode: "event_whenflagclicked",
    }
}

#[must_use]
pub const fn reset_timer() -> Stacking {
    Stacking {
        opcode: "sensing_resettimer",
    }
}
