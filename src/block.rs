use crate::{ListRef, Mutation, Operand, VariableRef};
use std::{fmt, io};

pub(crate) struct Block {
    pub(crate) opcode: &'static str,
    pub(crate) parent: Option<Id>,
    pub(crate) next: Option<Id>,
    pub(crate) inputs: Vec<(&'static str, Input)>,
    pub(crate) fields: Option<Fields>,
    pub(crate) mutation: Option<Box<Mutation>>,
}

impl Block {
    pub fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(writer, r#"{{"opcode":{:?},"parent":"#, self.opcode)?;
        if let Some(parent) = self.parent {
            write!(writer, "{parent}")
        } else {
            write!(writer, "null")
        }?;
        write!(writer, r#","next":"#)?;
        if let Some(next) = self.next {
            write!(writer, "{next}")
        } else {
            write!(writer, "null")
        }?;
        write!(writer, r#","topLevel":{}"#, self.parent.is_none())?;
        if !self.inputs.is_empty() {
            write!(writer, r#","inputs":{{"#)?;
            for (i, (name, input)) in self.inputs.iter().enumerate() {
                if i != 0 {
                    write!(writer, ",")?;
                }
                write!(writer, "{name:?}:")?;
                input.serialize(writer)?;
            }
            write!(writer, "}}")?;
        }
        if let Some(fields) = &self.fields {
            write!(writer, r#","fields":"#)?;
            fields.serialize(writer)?;
        }
        if let Some(mutation) = &self.mutation {
            write!(writer, r#","mutation":"#)?;
            mutation.serialize(writer)?;
        }
        if self.opcode == "control_create_clone_of_menu" {
            write!(writer, r#","shadow":true"#)?;
        }
        write!(writer, "}}")
    }

    pub(crate) const fn new(opcode: &'static str) -> Self {
        Self {
            opcode,
            parent: None,
            next: None,
            inputs: Vec::new(),
            fields: None,
            mutation: None,
        }
    }

    pub(crate) fn inputs(mut self, inputs: impl Into<Vec<(&'static str, Input)>>) -> Self {
        self.inputs = inputs.into();
        self
    }

    pub(crate) fn fields(mut self, fields: Fields) -> Self {
        self.fields = Some(fields);
        self
    }
}

pub struct Hat {
    opcode: &'static str,
    fields: Option<Fields>,
}

impl From<Hat> for Block {
    fn from(hat: Hat) -> Self {
        Self {
            opcode: hat.opcode,
            parent: None,
            next: None,
            inputs: Vec::new(),
            fields: hat.fields,
            mutation: None,
        }
    }
}

pub struct Stacking {
    pub(crate) opcode: &'static str,
    pub(crate) inputs: Vec<(&'static str, Input)>,
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
            inputs: Vec::new(),
            fields: None,
        }
    }
}

#[must_use]
pub const fn when_flag_clicked() -> Hat {
    Hat {
        opcode: "event_whenflagclicked",
        fields: None,
    }
}

#[must_use]
pub const fn when_key_pressed(key: String) -> Hat {
    Hat {
        opcode: "event_whenkeypressed",
        fields: Some(Fields::KeyOption(key)),
    }
}

#[must_use]
pub const fn when_cloned() -> Hat {
    Hat {
        opcode: "control_start_as_clone",
        fields: None,
    }
}

#[must_use]
pub const fn when_received(message: String) -> Hat {
    Hat {
        opcode: "event_whenbroadcastreceived",
        fields: Some(Fields::BroadcastOption(message)),
    }
}

#[must_use]
pub fn append(list: ListRef, item: Operand) -> Stacking {
    Stacking {
        opcode: "data_addtolist",
        inputs: Vec::from([("ITEM", item.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn ask(question: Operand) -> Stacking {
    Stacking {
        opcode: "sensing_askandwait",
        inputs: Vec::from([("QUESTION", question.0)]),
        fields: None,
    }
}

#[must_use]
pub fn broadcast_and_wait(message: Operand) -> Stacking {
    Stacking {
        opcode: "event_broadcastandwait",
        inputs: Vec::from([("BROADCAST_INPUT", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn change_variable(variable: VariableRef, by: Operand) -> Stacking {
    Stacking {
        opcode: "data_changevariableby",
        inputs: Vec::from([("VALUE", by.0)]),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn change_x(dx: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changexby",
        inputs: Vec::from([("DX", dx.0)]),
        fields: None,
    }
}

#[must_use]
pub fn change_y(dy: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changeyby",
        inputs: Vec::from([("DY", dy.0)]),
        fields: None,
    }
}

#[must_use]
pub const fn delete_all_of_list(list: ListRef) -> Stacking {
    Stacking {
        opcode: "data_deletealloflist",
        inputs: Vec::new(),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn delete_of_list(list: ListRef, index: Operand) -> Stacking {
    Stacking {
        opcode: "data_deleteoflist",
        inputs: Vec::from([("INDEX", index.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub const fn erase_all() -> Stacking {
    Stacking::new("pen_clear")
}

#[must_use]
pub fn go_to_back_layer() -> Stacking {
    Stacking {
        opcode: "looks_gotofrontback",
        inputs: Vec::from([("FRONT_BACK", Input::String("back".to_owned()))]),
        fields: None,
    }
}

#[must_use]
pub fn go_to_front_layer() -> Stacking {
    Stacking {
        opcode: "looks_gotofrontback",
        inputs: Vec::from([("FRONT_BACK", Input::String("front".to_owned()))]),
        fields: None,
    }
}

#[must_use]
pub fn go_to_xy(x: Operand, y: Operand) -> Stacking {
    Stacking {
        opcode: "motion_gotoxy",
        inputs: Vec::from([("X", x.0), ("Y", y.0)]),
        fields: None,
    }
}

#[must_use]
pub const fn hide() -> Stacking {
    Stacking::new("looks_hide")
}

#[must_use]
pub fn insert_at_list(list: ListRef, item: Operand, index: Operand) -> Stacking {
    Stacking {
        opcode: "data_insertatlist",
        inputs: Vec::from([("ITEM", item.0), ("INDEX", index.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn move_steps(steps: Operand) -> Stacking {
    Stacking {
        opcode: "motion_movesteps",
        inputs: Vec::from([("STEPS", steps.0)]),
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
        inputs: Vec::from([("INDEX", index.0), ("ITEM", item.0)]),
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
        inputs: Vec::from([("MESSAGE", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn say_for_seconds(seconds: Operand, message: Operand) -> Stacking {
    Stacking {
        opcode: "looks_say",
        inputs: Vec::from([("SECS", seconds.0), ("MESSAGE", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_costume(costume: Operand) -> Stacking {
    Stacking {
        opcode: "looks_switchcostumeto",
        inputs: Vec::from([("COSTUME", costume.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_color(color: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenColorTo",
        inputs: Vec::from([("COLOR", color.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenSizeTo",
        inputs: Vec::from([("SIZE", size.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "looks_setsizeto",
        inputs: Vec::from([("SIZE", size.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_variable(variable: VariableRef, to: Operand) -> Stacking {
    Stacking {
        opcode: "data_setvariableto",
        inputs: Vec::from([("VALUE", to.0)]),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn set_x(x: Operand) -> Stacking {
    Stacking {
        opcode: "motion_setx",
        inputs: Vec::from([("X", x.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_y(y: Operand) -> Stacking {
    Stacking {
        opcode: "motion_sety",
        inputs: Vec::from([("Y", y.0)]),
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
pub const fn stop_all() -> Stacking {
    Stacking {
        opcode: "control_stop",
        inputs: Vec::new(),
        fields: Some(Fields::StopAll),
    }
}

#[must_use]
pub const fn stop_this_script() -> Stacking {
    Stacking {
        opcode: "control_stop",
        inputs: Vec::new(),
        fields: Some(Fields::StopThisScript),
    }
}

#[must_use]
pub fn wait(seconds: Operand) -> Stacking {
    Stacking {
        opcode: "control_wait",
        inputs: Vec::from([("DURATION", seconds.0)]),
        fields: None,
    }
}

pub(crate) enum Input {
    Substack(Id),
    Number(f64),
    String(String),
    Variable(VariableRef),
    List(ListRef),
    Prototype(Id),
}

impl Input {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match *self {
            Self::Substack(uid) => write!(writer, "[2,{uid}]"),
            Self::Number(n) if n == f64::INFINITY => write!(writer, r#"[1,[4,"Infinity"]]"#),
            Self::Number(n) if n == f64::NEG_INFINITY => write!(writer, r#"[1,[4,"-Infinity"]]"#),
            Self::Number(n) if n.is_nan() => write!(writer, r#"[1,[4,"NaN"]]"#),
            Self::Number(n) => write!(writer, r"[1,[4,{n}]]"),
            Self::String(ref s) => write!(writer, r"[1,[10,{s:?}]]"),
            Self::Variable(VariableRef { ref name, id }) => {
                write!(writer, r#"[2,[12,{name:?},"v{id}"]]"#)
            }
            Self::List(ListRef { ref name, id }) => write!(writer, r#"[2,[13,{name:?},"l{id}"]]"#),
            Self::Prototype(uid) => write!(writer, "[1,{uid}]"),
        }
    }
}

pub(crate) enum Fields {
    Variable(VariableRef),
    List(ListRef),
    Value(String),
    Operator(&'static str),
    KeyOption(String),
    BroadcastOption(String),
    StopAll,
    StopThisScript,
    CloneSelf,
}

impl Fields {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::Variable(VariableRef { id, name }) => {
                write!(writer, r#"{{"VARIABLE":[{name:?},"{id}"]}}"#)
            }
            Self::List(ListRef { id, name }) => {
                write!(writer, r#"{{"LIST":[{name:?},"{id}"]}}"#)
            }
            Self::Value(name) => write!(writer, r#"{{"VALUE":[{name:?},null]}}"#),
            Self::Operator(operator) => write!(writer, r#"{{"OPERATOR":[{operator:?},null]}}"#),
            Self::KeyOption(key) => write!(writer, r#"{{"KEY_OPTION":[{key:?},null]}}"#),
            Self::BroadcastOption(broadcast) => {
                write!(writer, r#"{{"BROADCAST_OPTION":[{broadcast:?},null]}}"#)
            }
            Self::StopAll => write!(writer, r#"{{"STOP_OPTION":["all",null]}}"#),
            Self::StopThisScript => write!(writer, r#"{{"STOP_OPTION":["this script",null]}}"#),
            Self::CloneSelf => write!(writer, r#"{{"CLONE_OPTION":["_myself_",null]}}"#),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Id(pub usize);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""b{}""#, self.0)
    }
}
