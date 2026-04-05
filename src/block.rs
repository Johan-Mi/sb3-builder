use crate::{ListRef, Mutation, Operand, RealTarget, VariableRef};
use std::{fmt, io};

pub(crate) struct Block<'strings> {
    pub(crate) opcode: Opcode,
    pub(crate) parent: Option<Id>,
    pub(crate) next: Option<Id>,
    pub(crate) inputs: Box<[(&'static str, Input<'strings>)]>,
    pub(crate) mutation: Mutation,
}

impl<'strings> Block<'strings> {
    pub fn serialize(
        &self,
        fields: Option<Fields>,
        target: &RealTarget,
        writer: &mut dyn io::Write,
    ) -> io::Result<()> {
        write!(writer, r#"{{"opcode":"{:?}","parent":"#, self.opcode)?;
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
        if self
            .inputs
            .iter()
            .any(|(_, it)| !matches!(it, Input::EmptySubstack))
        {
            write!(writer, r#","inputs":{{"#)?;
            for (i, (name, input)) in self
                .inputs
                .iter()
                .filter(|(_, it)| !matches!(it, Input::EmptySubstack))
                .enumerate()
            {
                if i != 0 {
                    write!(writer, ",")?;
                }
                write!(writer, "{name:?}:")?;
                input.serialize(target, writer)?;
            }
            write!(writer, "}}")?;
        }
        if let Some(fields) = fields {
            write!(writer, r#","fields":"#)?;
            fields.serialize(target, writer)?;
        }
        if matches!(
            self.opcode,
            Opcode::procedures_prototype | Opcode::procedures_call
        ) {
            write!(writer, r#","mutation":"#)?;
            let is_prototype = matches!(self.opcode, Opcode::procedures_prototype);
            self.mutation.serialize(is_prototype, target, writer)?;
        }
        if matches!(self.opcode, Opcode::control_create_clone_of_menu) {
            write!(writer, r#","shadow":true"#)?;
        }
        write!(writer, "}}")
    }

    pub(crate) fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            parent: None,
            next: None,
            inputs: Box::new([]),
            mutation: Mutation::NONE,
        }
    }

    pub(crate) fn inputs(
        mut self,
        inputs: impl Into<Box<[(&'static str, Input<'strings>)]>>,
    ) -> Self {
        self.inputs = inputs.into();
        self
    }
}

#[derive(Clone, Copy)]
pub struct Hat<'strings> {
    pub(crate) opcode: Opcode,
    pub(crate) fields: Option<Fields<'strings>>,
}

pub struct Stacking<'strings> {
    pub(crate) opcode: Opcode,
    pub(crate) inputs: Box<[(&'static str, Input<'strings>)]>,
    pub(crate) fields: Option<Fields<'strings>>,
}

impl Stacking<'_> {
    pub(crate) fn new(opcode: Opcode) -> Self {
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
        opcode: Opcode::event_whenflagclicked,
        fields: None,
    }
}

#[must_use]
pub const fn when_key_pressed(key: &str) -> Hat<'_> {
    Hat {
        opcode: Opcode::event_whenkeypressed,
        fields: Some(Fields::KeyOption(key)),
    }
}

#[must_use]
pub const fn when_cloned() -> Hat<'static> {
    Hat {
        opcode: Opcode::control_start_as_clone,
        fields: None,
    }
}

#[must_use]
pub const fn when_received(message: &str) -> Hat<'_> {
    Hat {
        opcode: Opcode::event_whenbroadcastreceived,
        fields: Some(Fields::BroadcastOption(message)),
    }
}

#[must_use]
pub fn append(list: ListRef, item: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::data_addtolist,
        inputs: Box::new([("ITEM", item.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn ask(question: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::sensing_askandwait,
        inputs: Box::new([("QUESTION", question.0)]),
        fields: None,
    }
}

#[must_use]
pub fn broadcast_and_wait(message: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::event_broadcastandwait,
        inputs: Box::new([("BROADCAST_INPUT", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn change_variable(variable: VariableRef, by: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::data_changevariableby,
        inputs: Box::new([("VALUE", by.0)]),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn change_x(dx: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::motion_changexby,
        inputs: Box::new([("DX", dx.0)]),
        fields: None,
    }
}

#[must_use]
pub fn change_y(dy: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::motion_changeyby,
        inputs: Box::new([("DY", dy.0)]),
        fields: None,
    }
}

#[must_use]
pub fn delete_all_of_list(list: ListRef) -> Stacking<'static> {
    Stacking {
        opcode: Opcode::data_deletealloflist,
        inputs: Box::new([]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn delete_of_list(list: ListRef, index: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::data_deleteoflist,
        inputs: Box::new([("INDEX", index.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn erase_all() -> Stacking<'static> {
    Stacking::new(Opcode::pen_clear)
}

#[must_use]
pub fn go_to_back_layer() -> Stacking<'static> {
    Stacking {
        opcode: Opcode::looks_gotofrontback,
        inputs: Box::new([("FRONT_BACK", Input::String("back"))]),
        fields: None,
    }
}

#[must_use]
pub fn go_to_front_layer() -> Stacking<'static> {
    Stacking {
        opcode: Opcode::looks_gotofrontback,
        inputs: Box::new([("FRONT_BACK", Input::String("front"))]),
        fields: None,
    }
}

#[must_use]
pub fn go_to_xy<'strings>(x: Operand<'strings>, y: Operand<'strings>) -> Stacking<'strings> {
    Stacking {
        opcode: Opcode::motion_gotoxy,
        inputs: Box::new([("X", x.0), ("Y", y.0)]),
        fields: None,
    }
}

#[must_use]
pub fn hide() -> Stacking<'static> {
    Stacking::new(Opcode::looks_hide)
}

#[must_use]
pub fn insert_at_list<'strings>(
    list: ListRef,
    item: Operand<'strings>,
    index: Operand<'strings>,
) -> Stacking<'strings> {
    Stacking {
        opcode: Opcode::data_insertatlist,
        inputs: Box::new([("ITEM", item.0), ("INDEX", index.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn move_steps(steps: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::motion_movesteps,
        inputs: Box::new([("STEPS", steps.0)]),
        fields: None,
    }
}

#[must_use]
pub fn pen_down() -> Stacking<'static> {
    Stacking::new(Opcode::pen_penDown)
}

#[must_use]
pub fn pen_up() -> Stacking<'static> {
    Stacking::new(Opcode::pen_penUp)
}

#[must_use]
pub fn replace<'strings>(
    list: ListRef,
    index: Operand<'strings>,
    item: Operand<'strings>,
) -> Stacking<'strings> {
    Stacking {
        opcode: Opcode::data_replaceitemoflist,
        inputs: Box::new([("INDEX", index.0), ("ITEM", item.0)]),
        fields: Some(Fields::List(list)),
    }
}

#[must_use]
pub fn reset_timer() -> Stacking<'static> {
    Stacking::new(Opcode::sensing_resettimer)
}

#[must_use]
pub fn say(message: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::looks_say,
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
        opcode: Opcode::looks_say,
        inputs: Box::new([("SECS", seconds.0), ("MESSAGE", message.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_costume(costume: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::looks_switchcostumeto,
        inputs: Box::new([("COSTUME", costume.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_color(color: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::pen_setPenColorTo,
        inputs: Box::new([("COLOR", color.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_pen_size(size: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::pen_setPenSizeTo,
        inputs: Box::new([("SIZE", size.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_size(size: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::looks_setsizeto,
        inputs: Box::new([("SIZE", size.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_variable(variable: VariableRef, to: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::data_setvariableto,
        inputs: Box::new([("VALUE", to.0)]),
        fields: Some(Fields::Variable(variable)),
    }
}

#[must_use]
pub fn set_x(x: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::motion_setx,
        inputs: Box::new([("X", x.0)]),
        fields: None,
    }
}

#[must_use]
pub fn set_y(y: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::motion_sety,
        inputs: Box::new([("Y", y.0)]),
        fields: None,
    }
}

#[must_use]
pub fn show() -> Stacking<'static> {
    Stacking::new(Opcode::looks_show)
}

#[must_use]
pub fn stamp() -> Stacking<'static> {
    Stacking::new(Opcode::pen_stamp)
}

#[must_use]
pub fn stop_all() -> Stacking<'static> {
    Stacking {
        opcode: Opcode::control_stop,
        inputs: Box::new([]),
        fields: Some(Fields::StopAll),
    }
}

#[must_use]
pub fn stop_this_script() -> Stacking<'static> {
    Stacking {
        opcode: Opcode::control_stop,
        inputs: Box::new([]),
        fields: Some(Fields::StopThisScript),
    }
}

#[must_use]
pub fn wait(seconds: Operand) -> Stacking {
    Stacking {
        opcode: Opcode::control_wait,
        inputs: Box::new([("DURATION", seconds.0)]),
        fields: None,
    }
}

pub(crate) enum Input<'strings> {
    Substack(Id),
    EmptySubstack,
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
            Self::EmptySubstack => unreachable!(),
            Self::Number(n) if n == f64::INFINITY => write!(writer, r#"[1,[4,"Infinity"]]"#),
            Self::Number(n) if n == f64::NEG_INFINITY => write!(writer, r#"[1,[4,"-Infinity"]]"#),
            Self::Number(n) if n.is_nan() => write!(writer, r#"[1,[4,"NaN"]]"#),
            Self::Number(n) => write!(writer, r"[1,[4,{n}]]"),
            Self::String(s) => write!(writer, r"[1,[10,{s:?}]]"),
            Self::Variable(VariableRef(id)) => {
                let name = &target.variables[id].name;
                write!(writer, r#"[2,[12,{name:?},"v{id}"]]"#)
            }
            Self::List(ListRef(id)) => {
                let name = &target.lists[id].name;
                write!(writer, r#"[2,[13,{name:?},"l{id}"]]"#)
            }
            Self::Prototype(uid) => write!(writer, "[1,{uid}]"),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Fields<'strings> {
    Variable(VariableRef),
    List(ListRef),
    Value(usize),
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
                write!(writer, r#"{{"VARIABLE":[{name:?},"{id}"]}}"#)
            }
            Self::List(ListRef(id)) => {
                let name = &target.lists[*id].name;
                write!(writer, r#"{{"LIST":[{name:?},"{id}"]}}"#)
            }
            Self::Value(parameter) => {
                let name = &target.parameters[*parameter].name;
                write!(writer, r#"{{"VALUE":[{name:?},null]}}"#)
            }
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

#[expect(
    non_camel_case_types,
    reason = "exact names are used by derived `Debug` impl"
)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Opcode {
    argument_reporter_boolean,
    argument_reporter_string_number,
    control_create_clone_of,
    control_create_clone_of_menu,
    control_for_each,
    control_forever,
    control_if,
    control_if_else,
    control_repeat,
    control_repeat_until,
    control_start_as_clone,
    control_stop,
    control_wait,
    control_while,
    data_addtolist,
    data_changevariableby,
    data_deletealloflist,
    data_deleteoflist,
    data_insertatlist,
    data_itemnumoflist,
    data_itemoflist,
    data_lengthoflist,
    data_listcontainsitem,
    data_replaceitemoflist,
    data_setvariableto,
    event_broadcastandwait,
    event_whenbroadcastreceived,
    event_whenflagclicked,
    event_whenkeypressed,
    looks_gotofrontback,
    looks_hide,
    looks_say,
    looks_setsizeto,
    looks_show,
    looks_switchcostumeto,
    motion_changexby,
    motion_changeyby,
    motion_gotoxy,
    motion_movesteps,
    motion_setx,
    motion_sety,
    motion_xposition,
    motion_yposition,
    operator_add,
    operator_and,
    operator_contains,
    operator_divide,
    operator_equals,
    operator_gt,
    operator_join,
    operator_length,
    operator_letter_of,
    operator_lt,
    operator_mathop,
    operator_mod,
    operator_multiply,
    operator_not,
    operator_or,
    operator_random,
    operator_subtract,
    pen_clear,
    pen_penDown,
    pen_penUp,
    pen_setPenColorTo,
    pen_setPenSizeTo,
    pen_stamp,
    procedures_call,
    procedures_definition,
    procedures_prototype,
    sensing_answer,
    sensing_askandwait,
    sensing_keypressed,
    sensing_mousex,
    sensing_mousey,
    sensing_resettimer,
    sensing_timer,
}

impl Opcode {
    pub(crate) const fn has_fields(self) -> bool {
        #[expect(
            clippy::match_same_arms,
            reason = "easier to keep opcodes in order when the arms aren't merged"
        )]
        match self {
            Self::argument_reporter_boolean => true,
            Self::argument_reporter_string_number => true,
            Self::control_create_clone_of => false,
            Self::control_create_clone_of_menu => true,
            Self::control_for_each => false,
            Self::control_forever => false,
            Self::control_if => false,
            Self::control_if_else => false,
            Self::control_repeat => false,
            Self::control_repeat_until => false,
            Self::control_start_as_clone => false,
            Self::control_stop => true,
            Self::control_wait => false,
            Self::control_while => false,
            Self::data_addtolist => true,
            Self::data_changevariableby => true,
            Self::data_deletealloflist => true,
            Self::data_deleteoflist => true,
            Self::data_insertatlist => true,
            Self::data_itemnumoflist => true,
            Self::data_itemoflist => true,
            Self::data_lengthoflist => true,
            Self::data_listcontainsitem => true,
            Self::data_replaceitemoflist => true,
            Self::data_setvariableto => true,
            Self::event_broadcastandwait => false,
            Self::event_whenbroadcastreceived => true,
            Self::event_whenflagclicked => false,
            Self::event_whenkeypressed => true,
            Self::looks_gotofrontback => false,
            Self::looks_hide => false,
            Self::looks_say => false,
            Self::looks_setsizeto => false,
            Self::looks_show => false,
            Self::looks_switchcostumeto => false,
            Self::motion_changexby => false,
            Self::motion_changeyby => false,
            Self::motion_gotoxy => false,
            Self::motion_movesteps => false,
            Self::motion_setx => false,
            Self::motion_sety => false,
            Self::motion_xposition => false,
            Self::motion_yposition => false,
            Self::operator_add => false,
            Self::operator_and => false,
            Self::operator_contains => false,
            Self::operator_divide => false,
            Self::operator_equals => false,
            Self::operator_gt => false,
            Self::operator_join => false,
            Self::operator_length => false,
            Self::operator_letter_of => false,
            Self::operator_lt => false,
            Self::operator_mathop => true,
            Self::operator_mod => false,
            Self::operator_multiply => false,
            Self::operator_not => false,
            Self::operator_or => false,
            Self::operator_random => false,
            Self::operator_subtract => false,
            Self::pen_clear => false,
            Self::pen_penDown => false,
            Self::pen_penUp => false,
            Self::pen_setPenColorTo => false,
            Self::pen_setPenSizeTo => false,
            Self::pen_stamp => false,
            Self::procedures_call => false,
            Self::procedures_definition => false,
            Self::procedures_prototype => false,
            Self::sensing_answer => false,
            Self::sensing_askandwait => false,
            Self::sensing_keypressed => false,
            Self::sensing_mousex => false,
            Self::sensing_mousey => false,
            Self::sensing_resettimer => false,
            Self::sensing_timer => false,
        }
    }
}
