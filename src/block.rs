use crate::uid::Uid;
use serde::{ser::SerializeStruct, Serialize};

pub(crate) struct Block {
    opcode: &'static str,
    pub(crate) parent: Option<Uid>,
    pub(crate) next: Option<Uid>,
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Block", 4)?;
        s.serialize_field("opcode", self.opcode)?;
        s.serialize_field("parent", &self.parent)?;
        s.serialize_field("next", &self.next)?;
        s.serialize_field("topLevel", &self.parent.is_none())?;
        s.end()
    }
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
