use crate::{ListRef, Mutation, Operand, RealTarget, VariableRef};
use std::{fmt, io};

pub(crate) struct Block<'strings> {
    pub(crate) opcode: &'static str,
    pub(crate) parent: Option<Id>,
    pub(crate) next: Option<Id>,
    pub(crate) inputs: Box<[(&'static str, Input<'strings>)]>,
    pub(crate) fields: Option<Fields<'strings>>,
    pub(crate) mutation: Option<Box<Mutation>>,
}

impl<'strings> Block<'strings> {
    pub fn serialize(&self, target: &RealTarget, writer: &mut dyn io::Write) -> io::Result<()> {
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
                input.serialize(target, writer)?;
            }
            write!(writer, "}}")?;
        }
        if let Some(fields) = &self.fields {
            write!(writer, r#","fields":"#)?;
            fields.serialize(target, writer)?;
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

    pub(crate) fn new(opcode: &'static str) -> Self {
        Self {
            opcode,
            parent: None,
            next: None,
            inputs: Box::new([]),
            fields: None,
            mutation: None,
        }
    }

    pub(crate) fn inputs(
        mut self,
        inputs: impl Into<Box<[(&'static str, Input<'strings>)]>>,
    ) -> Self {
        self.inputs = inputs.into();
        self
    }

    pub(crate) fn fields(mut self, fields: Fields<'strings>) -> Self {
        self.fields = Some(fields);
        self
    }
}

pub struct Hat<'strings> {
    opcode: &'static str,
    fields: Option<Fields<'strings>>,
}

impl<'strings> From<Hat<'strings>> for Block<'strings> {
    fn from(hat: Hat<'strings>) -> Self {
        Self {
            opcode: hat.opcode,
            parent: None,
            next: None,
            inputs: Box::new([]),
            fields: hat.fields,
            mutation: None,
        }
    }
}

pub struct Stacking<'strings> {
    pub(crate) opcode: &'static str,
    pub(crate) inputs: Box<[(&'static str, Input<'strings>)]>,
    pub(crate) fields: Option<Fields<'strings>>,
}

impl<'strings> From<Stacking<'strings>> for Block<'strings> {
    fn from(stacking: Stacking<'strings>) -> Self {
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

impl Stacking<'_> {
    pub(crate) fn new(opcode: &'static str) -> Self {
        Self {
            opcode,
            inputs: Box::new([]),
            fields: None,
        }
    }
}

#[must_use]
pub const fn when_flag_clicked() -> Hat<'static> {
    Hat {
        opcode: "event_whenflagclicked",
        fields: None,
    }
}

#[must_use]
pub const fn when_key_pressed(key: &str) -> Hat<'_> {
    Hat {
        opcode: "event_whenkeypressed",
        fields: Some(Fields::KeyOption(key)),
    }
}

#[must_use]
pub const fn when_cloned() -> Hat<'static> {
    Hat {
        opcode: "control_start_as_clone",
        fields: None,
    }
}

#[must_use]
pub const fn when_received(message: &str) -> Hat<'_> {
    Hat {
        opcode: "event_whenbroadcastreceived",
        fields: Some(Fields::BroadcastOption(message)),
    }
}

#[must_use]
pub fn append(list: ListRef, item: Operand) -> Stacking {
    Stacking {
        opcode: "data_addtolist",
        inputs: Box::new([("ITEM", item.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn ask(question: Operand) -> Stacking {
    Stacking {
        opcode: "sensing_askandwait",
        inputs: Box::new([("QUESTION", question.0)]),
        fields: None,
    }
}

#[must_use]
pub fn broadcast_and_wait(message: Operand) -> Stacking {
    Stacking {
        opcode: "event_broadcastandwait",
        inputs: Box::new([("BROADCAST_INPUT", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn change_variable(variable: VariableRef, by: Operand) -> Stacking {
    Stacking {
        opcode: "data_changevariableby",
        inputs: Box::new([("VALUE", by.0)]),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn change_x(dx: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changexby",
        inputs: Box::new([("DX", dx.0)]),
        fields: None,
    }
}

#[must_use]
pub fn change_y(dy: Operand) -> Stacking {
    Stacking {
        opcode: "motion_changeyby",
        inputs: Box::new([("DY", dy.0)]),
        fields: None,
    }
}

#[must_use]
pub fn delete_all_of_list(list: ListRef) -> Stacking<'static> {
    Stacking {
        opcode: "data_deletealloflist",
        inputs: Box::new([]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn delete_of_list(list: ListRef, index: Operand) -> Stacking {
    Stacking {
        opcode: "data_deleteoflist",
        inputs: Box::new([("INDEX", index.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn erase_all() -> Stacking<'static> {
    Stacking::new("pen_clear")
}

#[must_use]
pub fn go_to_back_layer() -> Stacking<'static> {
    Stacking {
        opcode: "looks_gotofrontback",
        inputs: Box::new([("FRONT_BACK", Input::String("back"))]),
        fields: None,
    }
}

#[must_use]
pub fn go_to_front_layer() -> Stacking<'static> {
    Stacking {
        opcode: "looks_gotofrontback",
        inputs: Box::new([("FRONT_BACK", Input::String("front"))]),
        fields: None,
    }
}

#[must_use]
pub fn go_to_xy<'strings>(x: Operand<'strings>, y: Operand<'strings>) -> Stacking<'strings> {
    Stacking {
        opcode: "motion_gotoxy",
        inputs: Box::new([("X", x.0), ("Y", y.0)]),
        fields: None,
    }
}

#[must_use]
pub fn hide() -> Stacking<'static> {
    Stacking::new("looks_hide")
}

#[must_use]
pub fn insert_at_list<'strings>(
    list: ListRef,
    item: Operand<'strings>,
    index: Operand<'strings>,
) -> Stacking<'strings> {
    Stacking {
        opcode: "data_insertatlist",
        inputs: Box::new([("ITEM", item.0), ("INDEX", index.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn move_steps(steps: Operand) -> Stacking {
    Stacking {
        opcode: "motion_movesteps",
        inputs: Box::new([("STEPS", steps.0)]),
        fields: None,
    }
}

#[must_use]
pub fn pen_down() -> Stacking<'static> {
    Stacking::new("pen_penDown")
}

#[must_use]
pub fn pen_up() -> Stacking<'static> {
    Stacking::new("pen_penUp")
}

#[must_use]
pub fn replace<'strings>(
    list: ListRef,
    index: Operand<'strings>,
    item: Operand<'strings>,
) -> Stacking<'strings> {
    Stacking {
        opcode: "data_replaceitemoflist",
        inputs: Box::new([("INDEX", index.0), ("ITEM", item.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn reset_timer() -> Stacking<'static> {
    Stacking::new("sensing_resettimer")
}

#[must_use]
pub fn say(message: Operand) -> Stacking {
    Stacking {
        opcode: "looks_say",
        inputs: Box::new([("MESSAGE", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn say_for_seconds<'strings>(
    seconds: Operand<'strings>,
    message: Operand<'strings>,
) -> Stacking<'strings> {
    Stacking {
        opcode: "looks_say",
        inputs: Box::new([("SECS", seconds.0), ("MESSAGE", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_costume(costume: Operand) -> Stacking {
    Stacking {
        opcode: "looks_switchcostumeto",
        inputs: Box::new([("COSTUME", costume.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_color(color: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenColorTo",
        inputs: Box::new([("COLOR", color.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "pen_setPenSizeTo",
        inputs: Box::new([("SIZE", size.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_size(size: Operand) -> Stacking {
    Stacking {
        opcode: "looks_setsizeto",
        inputs: Box::new([("SIZE", size.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_variable(variable: VariableRef, to: Operand) -> Stacking {
    Stacking {
        opcode: "data_setvariableto",
        inputs: Box::new([("VALUE", to.0)]),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn set_x(x: Operand) -> Stacking {
    Stacking {
        opcode: "motion_setx",
        inputs: Box::new([("X", x.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_y(y: Operand) -> Stacking {
    Stacking {
        opcode: "motion_sety",
        inputs: Box::new([("Y", y.0)]),
        fields: None,
    }
}

#[must_use]
pub fn show() -> Stacking<'static> {
    Stacking::new("looks_show")
}

#[must_use]
pub fn stamp() -> Stacking<'static> {
    Stacking::new("pen_stamp")
}

#[must_use]
pub fn stop_all() -> Stacking<'static> {
    Stacking {
        opcode: "control_stop",
        inputs: Box::new([]),
        fields: Some(Fields::StopAll),
    }
}

#[must_use]
pub fn stop_this_script() -> Stacking<'static> {
    Stacking {
        opcode: "control_stop",
        inputs: Box::new([]),
        fields: Some(Fields::StopThisScript),
    }
}

#[must_use]
pub fn wait(seconds: Operand) -> Stacking {
    Stacking {
        opcode: "control_wait",
        inputs: Box::new([("DURATION", seconds.0)]),
        fields: None,
    }
}

pub(crate) enum Input<'strings> {
    Substack(Id),
    Number(f64),
    String(&'strings str),
    Variable(VariableRef),
    List(ListRef),
    Prototype(Id),
}

impl Input<'_> {
    fn serialize(&self, target: &RealTarget, writer: &mut dyn io::Write) -> io::Result<()> {
        match *self {
            Self::Substack(uid) => write!(writer, "[2,{uid}]"),
            Self::Number(n) if n == f64::INFINITY => write!(writer, r#"[1,[4,"Infinity"]]"#),
            Self::Number(n) if n == f64::NEG_INFINITY => write!(writer, r#"[1,[4,"-Infinity"]]"#),
            Self::Number(n) if n.is_nan() => write!(writer, r#"[1,[4,"NaN"]]"#),
            Self::Number(n) => write!(writer, r"[1,[4,{n}]]"),
            Self::String(s) => write!(writer, r"[1,[10,{s:?}]]"),
            Self::Variable(VariableRef(id)) => {
                let name = &target.variables[id].name;
                write!(writer, r#"[2,[12,{name:?},"v{id}"]]"#,)
            }
            Self::List(ListRef(id)) => {
                let name = &target.lists[id].name;
                write!(writer, r#"[2,[13,{name:?},"l{id}"]]"#,)
            }
            Self::Prototype(uid) => write!(writer, "[1,{uid}]"),
        }
    }
}

pub(crate) enum Fields<'strings> {
    Variable(VariableRef),
    List(ListRef),
    Value(String),
    Operator(&'static str),
    KeyOption(&'strings str),
    BroadcastOption(&'strings str),
    StopAll,
    StopThisScript,
    CloneSelf,
}

impl Fields<'_> {
    fn serialize(&self, target: &RealTarget, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::Variable(VariableRef(id)) => {
                let name = &target.variables[*id].name;
                write!(writer, r#"{{"VARIABLE":[{name:?},"{id}"]}}"#,)
            }
            Self::List(ListRef(id)) => {
                let name = &target.lists[*id].name;
                write!(writer, r#"{{"LIST":[{name:?},"{id}"]}}"#,)
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
