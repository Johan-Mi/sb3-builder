use crate::uid::Uid;
use serde::{ser::SerializeStruct, Serialize};
use std::collections::HashMap;

pub(crate) struct Block {
    opcode: &'static str,
    pub(crate) parent: Option<Uid>,
    pub(crate) next: Option<Uid>,
    pub(crate) inputs: Option<HashMap<&'static str, Input>>,
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Block", 5)?;
        s.serialize_field("opcode", self.opcode)?;
        s.serialize_field("parent", &self.parent)?;
        s.serialize_field("next", &self.next)?;
        s.serialize_field("topLevel", &self.parent.is_none())?;
        if let Some(inputs) = &self.inputs {
            s.serialize_field("inputs", inputs)?;
        }
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
            inputs: None,
        }
    }
}

pub struct Stacking {
    pub(crate) opcode: &'static str,
}

impl From<Stacking> for Block {
    fn from(stacking: Stacking) -> Self {
        Self {
            opcode: stacking.opcode,
            parent: None,
            next: None,
            inputs: None,
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

pub(crate) enum Input {
    Substack(Uid),
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Substack(uid) => (2, uid).serialize(serializer),
        }
    }
}
