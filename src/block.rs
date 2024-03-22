use crate::{uid::Uid, ListRef, Mutation, Operand, VariableRef};
use serde::{
    ser::{SerializeMap, SerializeStruct},
    Serialize,
};
use std::collections::HashMap;

pub(crate) struct Block {
    pub(crate) opcode: &'static str,
    pub(crate) parent: Option<Uid>,
    pub(crate) next: Option<Uid>,
    pub(crate) inputs: Option<HashMap<&'static str, Input>>,
    pub(crate) fields: Option<Fields>,
    pub(crate) mutation: Option<Mutation>,
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Block", 7)?;
        s.serialize_field("opcode", self.opcode)?;
        s.serialize_field("parent", &self.parent)?;
        s.serialize_field("next", &self.next)?;
        s.serialize_field("topLevel", &self.parent.is_none())?;
        if let Some(inputs) = &self.inputs {
            s.serialize_field("inputs", inputs)?;
        }
        if let Some(fields) = &self.fields {
            s.serialize_field("fields", fields)?;
        }
        if let Some(mutation) = &self.mutation {
            s.serialize_field("mutation", mutation)?;
        }
        s.end()
    }
}

impl Block {
    pub(crate) const fn symbol(opcode: &'static str) -> Self {
        Self {
            opcode,
            parent: None,
            next: None,
            inputs: None,
            fields: None,
            mutation: None,
        }
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
            fields: None,
            mutation: None,
        }
    }
}

pub struct Stacking {
    pub(crate) opcode: &'static str,
    pub(crate) inputs: Option<HashMap<&'static str, Input>>,
    pub(crate) fields: Option<Fields>,
}

impl From<Stacking> for Block {
    fn from(stacking: Stacking) -> Self {
        Self {
            opcode: stacking.opcode,
            parent: None,
            next: None,
            inputs: stacking.inputs,
            fields: stacking.fields,
            mutation: None,
        }
    }
}

impl Stacking {
    pub(crate) const fn new(opcode: &'static str) -> Self {
        Self {
            opcode,
            inputs: None,
            fields: None,
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
pub fn append(list: ListRef, item: Operand) -> Stacking {
    Stacking {
        opcode: "data_addtolist",
        inputs: Some([("ITEM", item.0)].into()),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn ask(question: Operand) -> Stacking {
    Stacking {
        opcode: "sensing_askandwait",
        inputs: Some([("QUESTION", question.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn change_variable(variable: VariableRef, by: Operand) -> Stacking {
    Stacking {
        opcode: "data_changevariableby",
        inputs: Some([("VALUE", by.0)].into()),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn change_x(dx: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changexby",
        inputs: Some([("DX", dx.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn change_y(dy: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changeyby",
        inputs: Some([("DY", dy.0)].into()),
        fields: None,
    }
}

#[must_use]
pub const fn delete_all_of_list(list: ListRef) -> Stacking {
    Stacking {
        opcode: "data_deletealloflist",
        inputs: None,
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn delete_of_list(list: ListRef, index: Operand) -> Stacking {
    Stacking {
        opcode: "data_deleteoflist",
        inputs: Some([("INDEX", index.0)].into()),
        fields: Some(Fields::List(list)),
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
        fields: None,
    }
}

#[must_use]
pub const fn hide() -> Stacking {
    Stacking::new("looks_hide")
}

#[must_use]
pub fn insert_at_list(
    list: ListRef,
    item: Operand,
    index: Operand,
) -> Stacking {
    Stacking {
        opcode: "data_insertatlist",
        inputs: Some([("ITEM", item.0), ("INDEX", index.0)].into()),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn move_steps(steps: Operand) -> Stacking {
    Stacking {
        opcode: "motion_movesteps",
        inputs: Some([("STEPS", steps.0)].into()),
        fields: None,
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
pub fn replace(list: ListRef, index: Operand, item: Operand) -> Stacking {
    Stacking {
        opcode: "data_replaceitemoflist",
        inputs: Some([("INDEX", index.0), ("ITEM", item.0)].into()),
        fields: Some(Fields::List(list)),
    }
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
        fields: None,
    }
}

#[must_use]
pub fn say_for_seconds(seconds: Operand, message: Operand) -> Stacking {
    Stacking {
        opcode: "looks_say",
        inputs: Some([("SECS", seconds.0), ("MESSAGE", message.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn set_costume(costume: Operand) -> Stacking {
    Stacking {
        opcode: "looks_switchcostumeto",
        inputs: Some([("COSTUME", costume.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_color(color: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenColorTo",
        inputs: Some([("COLOR", color.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenSizeTo",
        inputs: Some([("SIZE", size.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn set_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "looks_setsizeto",
        inputs: Some([("SIZE", size.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn set_variable(variable: VariableRef, to: Operand) -> Stacking {
    Stacking {
        opcode: "data_setvariableto",
        inputs: Some([("VALUE", to.0)].into()),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn set_x(x: Operand) -> Stacking {
    Stacking {
        opcode: "motion_setx",
        inputs: Some([("X", x.0)].into()),
        fields: None,
    }
}

#[must_use]
pub fn set_y(y: Operand) -> Stacking {
    Stacking {
        opcode: "motion_sety",
        inputs: Some([("Y", y.0)].into()),
        fields: None,
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
pub fn stop_all() -> Stacking {
    Stacking {
        opcode: "control_stop",
        inputs: None,
        fields: Some(Fields::StopAll),
    }
}

#[must_use]
pub fn wait(seconds: Operand) -> Stacking {
    Stacking {
        opcode: "control_wait",
        inputs: Some([("DURATION", seconds.0)].into()),
        fields: None,
    }
}

pub(crate) enum Input {
    Substack(Uid),
    Number(f64),
    String(String),
    Variable(VariableRef),
    List(ListRef),
    Prototype(Uid),
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::Substack(uid) => (2, uid).serialize(serializer),
            Self::Number(n) => (1, (4, n)).serialize(serializer),
            Self::String(ref s) => (1, (10, s)).serialize(serializer),
            Self::Variable(VariableRef { ref name, id }) => {
                (2, (12, name, id)).serialize(serializer)
            }
            Self::List(ListRef { ref name, id }) => {
                (2, (13, name, id)).serialize(serializer)
            }
            Self::Prototype(uid) => (1, uid).serialize(serializer),
        }
    }
}

pub(crate) enum Fields {
    Variable(VariableRef),
    List(ListRef),
    Value(String),
    Operator(&'static str),
    StopAll,
}

impl Serialize for Fields {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Variable(VariableRef { id, name }) => {
                let mut m = serializer.serialize_map(Some(1))?;
                m.serialize_entry("VARIABLE", &(&**name, *id))?;
                m.end()
            }
            Self::List(ListRef { id, name }) => {
                let mut m = serializer.serialize_map(Some(1))?;
                m.serialize_entry("LIST", &(&**name, *id))?;
                m.end()
            }
            Self::Value(name) => {
                let mut m = serializer.serialize_map(Some(1))?;
                m.serialize_entry("VALUE", &(&**name, ()))?;
                m.end()
            }
            Self::Operator(operator) => {
                let mut m = serializer.serialize_map(Some(1))?;
                m.serialize_entry("OPERATOR", &(*operator, ()))?;
                m.end()
            }
            Self::StopAll => {
                let mut m = serializer.serialize_map(Some(1))?;
                m.serialize_entry("STOP_OPTION", &("all", ()))?;
                m.end()
            }
        }
    }
}
