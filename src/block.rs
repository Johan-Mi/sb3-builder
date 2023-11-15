use crate::uid::Uid;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Block {
    opcode: &'static str,
    parent: Option<Uid>,
    next: Option<Uid>,
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

#[must_use]
pub const fn when_flag_clicked() -> Hat {
    Hat {
        opcode: "event_whenflagclicked",
    }
}
