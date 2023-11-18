use crate::{uid::Uid, Operand};
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
    pub(crate) inputs: Option<HashMap<&'static str, Input>>,
}

impl From<Stacking> for Block {
    fn from(stacking: Stacking) -> Self {
        Self {
            opcode: stacking.opcode,
            parent: None,
            next: None,
            inputs: stacking.inputs,
        }
    }
}

impl Stacking {
    const fn new(opcode: &'static str) -> Self {
        Self {
            opcode,
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
pub fn ask(question: Operand) -> Stacking {
    Stacking {
        opcode: "sensing_askandwait",
        inputs: Some([("QUESTION", question.0)].into()),
    }
}

#[must_use]
pub fn change_x(dx: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changexby",
        inputs: Some([("DX", dx.0)].into()),
    }
}

#[must_use]
pub fn change_y(dy: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changeyby",
        inputs: Some([("DY", dy.0)].into()),
    }
}

#[must_use]
pub const fn erase_all() -> Stacking {
    Stacking::new("pen_clear")
}

#[must_use]
pub fn go_to_xy(x: Operand, y: Operand) -> Stacking {
    Stacking {
        opcode: "motion_gotoxy",
        inputs: Some([("X", x.0), ("Y", y.0)].into()),
    }
}

#[must_use]
pub const fn hide() -> Stacking {
    Stacking::new("looks_hide")
}

#[must_use]
pub fn move_steps(steps: Operand) -> Stacking {
    Stacking {
        opcode: "motion_movesteps",
        inputs: Some([("STEPS", steps.0)].into()),
    }
}

#[must_use]
pub const fn pen_down() -> Stacking {
    Stacking::new("pen_penDown")
}

#[must_use]
pub const fn pen_up() -> Stacking {
    Stacking::new("pen_penUp")
}

#[must_use]
pub const fn reset_timer() -> Stacking {
    Stacking::new("sensing_resettimer")
}

#[must_use]
pub fn say(message: Operand) -> Stacking {
    Stacking {
        opcode: "looks_say",
        inputs: Some([("MESSAGE", message.0)].into()),
    }
}

#[must_use]
pub fn say_for_seconds(seconds: Operand, message: Operand) -> Stacking {
    Stacking {
        opcode: "looks_say",
        inputs: Some([("SECS", seconds.0), ("MESSAGE", message.0)].into()),
    }
}

#[must_use]
pub fn set_costume(costume: Operand) -> Stacking {
    Stacking {
        opcode: "looks_switchcostumeto",
        inputs: Some([("COSTUME", costume.0)].into()),
    }
}

#[must_use]
pub fn set_pen_color(color: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenColorTo",
        inputs: Some([("COLOR", color.0)].into()),
    }
}

#[must_use]
pub fn set_pen_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenSizeTo",
        inputs: Some([("SIZE", size.0)].into()),
    }
}

#[must_use]
pub fn set_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "looks_setsizeto",
        inputs: Some([("SIZE", size.0)].into()),
    }
}

#[must_use]
pub fn set_x(x: Operand) -> Stacking {
    Stacking {
        opcode: "motion_setx",
        inputs: Some([("X", x.0)].into()),
    }
}

#[must_use]
pub fn set_y(y: Operand) -> Stacking {
    Stacking {
        opcode: "motion_sety",
        inputs: Some([("Y", y.0)].into()),
    }
}

#[must_use]
pub const fn show() -> Stacking {
    Stacking::new("looks_show")
}

#[must_use]
pub const fn stamp() -> Stacking {
    Stacking::new("pen_stamp")
}

#[must_use]
pub fn wait(seconds: Operand) -> Stacking {
    Stacking {
        opcode: "control_wait",
        inputs: Some([("DURATION", seconds.0)].into()),
    }
}

pub(crate) enum Input {
    Substack(Uid),
    Number(f64),
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::Substack(uid) => (2, uid).serialize(serializer),
            Self::Number(n) => (1, (4, n)).serialize(serializer),
        }
    }
}
