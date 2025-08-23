pub mod block;
mod costume;
mod finish;
mod operand;
mod uid;

pub use costume::Costume;
pub use operand::Operand;

use block::{Block, Fields, Input};
use std::{collections::HashMap, fmt::Write as _, io};
use uid::Uid;

pub struct Project {
    uid_generator: uid::Generator,
    targets: Vec<RealTarget>,
}

impl Default for Project {
    fn default() -> Self {
        let stage = RealTarget::new("Stage".to_owned(), true);
        Self {
            uid_generator: uid::Generator::default(),
            targets: vec![stage],
        }
    }
}

impl Project {
    pub fn stage(&mut self) -> Target<'_> {
        self.target(0)
    }

    pub fn add_sprite(&mut self, name: String) -> Target<'_> {
        self.targets.push(RealTarget::new(name, false));
        self.target(self.targets.len() - 1)
    }

    fn target(&mut self, index: usize) -> Target<'_> {
        Target {
            inner: &mut self.targets[index],
            uid_generator: &mut self.uid_generator,
            point: InsertionPoint {
                parent: None,
                place: Place::Next,
            },
        }
    }
}

struct RealTarget {
    name: String,
    is_stage: bool,
    costumes: Vec<Costume>,
    variables: HashMap<Uid, Variable>,
    lists: HashMap<Uid, List>,
    blocks: HashMap<Uid, block::Block>,
    comments: HashMap<Uid, Comment>,
}

impl RealTarget {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(
            writer,
            r#"{{"name":{:?},"isStage":{},"currentCostume":0,"costumes":["#,
            self.name, self.is_stage
        )?;
        for (i, costume) in self.costumes.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            costume.serialize(writer)?;
        }
        write!(writer, r#"],"sounds":[],"variables":{{"#)?;
        for (i, (id, variable)) in self.variables.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{id}:")?;
            variable.serialize(writer)?;
        }
        write!(writer, r#"}},"lists":{{"#)?;
        for (i, (id, list)) in self.lists.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{id}:")?;
            list.serialize(writer)?;
        }
        write!(writer, r#"}},"blocks":{{"#)?;
        for (i, (id, block)) in self.blocks.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{id}:")?;
            block.serialize(writer)?;
        }
        write!(writer, r#"}},"comments":{{"#)?;
        for (i, (id, comment)) in self.comments.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{id}:")?;
            comment.serialize(writer)?;
        }
        write!(writer, "}}}}")
    }
}

impl RealTarget {
    fn new(name: String, is_stage: bool) -> Self {
        Self {
            name,
            is_stage,
            costumes: Vec::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
            comments: HashMap::new(),
        }
    }
}

struct Comment {
    text: String,
}

impl Comment {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(
            writer,
            r#"{{"text":{:?},"blockId":null,"minimized":false,"x":null,"y":null,"width":0,"height":0}}"#,
            self.text
        )
    }
}

pub struct Target<'a> {
    inner: &'a mut RealTarget,
    uid_generator: &'a mut uid::Generator,
    point: InsertionPoint,
}

impl Target<'_> {
    pub fn add_comment(&mut self, text: String) {
        let id = self.uid_generator.new_uid();
        self.inner.comments.insert(id, Comment { text });
    }

    pub fn add_costume(&mut self, costume: Costume) {
        self.inner.costumes.push(costume);
    }

    pub fn add_variable(&mut self, variable: Variable) -> VariableRef {
        let id = self.uid_generator.new_uid();
        let name = (&*variable.name).into();
        self.inner.variables.insert(id, variable);
        VariableRef { name, id }
    }

    pub fn add_list(&mut self, list: List) -> ListRef {
        let id = self.uid_generator.new_uid();
        let name = (&*list.name).into();
        self.inner.lists.insert(id, list);
        ListRef { name, id }
    }

    pub const fn insert_at(&mut self, point: InsertionPoint) -> InsertionPoint {
        std::mem::replace(&mut self.point, point)
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn add_custom_block(
        &mut self,
        name: String,
        parameters: Vec<Parameter>,
    ) -> (CustomBlock, InsertionPoint) {
        let param_ids =
            std::iter::repeat_with(|| &*self.uid_generator.new_uid().raw().to_string().leak())
                .take(parameters.len())
                .collect::<Vec<_>>();

        let mut argumentnames = "[".to_owned();
        for (i, param) in parameters.iter().enumerate() {
            if i != 0 {
                argumentnames.push(',');
            }
            write!(argumentnames, "{:?}", param.name).expect("writing to `String` cannot fail");
        }
        argumentnames.push(']');

        let mut argumentdefaults = "[".to_owned();
        for (i, param) in parameters.iter().enumerate() {
            if i != 0 {
                argumentdefaults.push(',');
            }
            argumentdefaults.push_str(match param.kind {
                ParameterKind::StringOrNumber => "\"\"",
                ParameterKind::Boolean => "false",
            });
        }
        argumentdefaults.push(']');

        let mut proccode = name;
        proccode.extend(parameters.iter().map(|param| match param.kind {
            ParameterKind::StringOrNumber => " %s",
            ParameterKind::Boolean => " %b",
        }));

        let mut argumentids = "[".to_owned();
        for (i, id) in param_ids.iter().enumerate() {
            if i != 0 {
                argumentids.push(',');
            }
            write!(argumentids, "{id:?}").expect("writing to `String` cannot fail");
        }
        argumentids.push(']');

        let mutation = Mutation {
            proccode,
            argumentids,
            argumentnames: None,
            argumentdefaults: None,
        };

        let inputs = param_ids
            .iter()
            .copied()
            .zip(
                parameters
                    .into_iter()
                    .map(|param| self.custom_block_parameter(param).0),
            )
            .collect();

        let definition = self.uid_generator.new_uid();
        let prototype = self.insert(Block {
            opcode: "procedures_prototype",
            parent: Some(definition),
            next: None,
            inputs: Some(inputs),
            fields: None,
            mutation: Some(Box::new(Mutation {
                argumentnames: Some(argumentnames),
                argumentdefaults: Some(argumentdefaults),
                ..mutation.clone()
            })),
        });

        self.inner.blocks.insert(
            definition,
            block::Stacking {
                opcode: "procedures_definition",
                inputs: Some([("custom_block", Input::Prototype(prototype))].into()),
                fields: None,
            }
            .into(),
        );

        (
            CustomBlock {
                mutation,
                param_ids,
            },
            InsertionPoint {
                parent: Some(definition),
                place: Place::Next,
            },
        )
    }

    pub fn use_custom_block(&mut self, block: &CustomBlock, arguments: Vec<Operand>) {
        let inputs = block
            .param_ids
            .iter()
            .copied()
            .zip(arguments.into_iter().map(|arg| arg.0))
            .collect();

        let block = Block {
            opcode: "procedures_call",
            parent: self.point.parent,
            next: None,
            inputs: Some(inputs),
            fields: None,
            mutation: Some(Box::new(block.mutation.clone())),
        };
        let id = self.insert(block);
        self.set_next(id);
        self.point = InsertionPoint {
            parent: Some(id),
            place: Place::Next,
        };
    }

    pub fn custom_block_parameter(&mut self, param: Parameter) -> Operand {
        let opcode = match param.kind {
            ParameterKind::StringOrNumber => "argument_reporter_string_number",
            ParameterKind::Boolean => "argument_reporter_boolean",
        };
        self.op(Block {
            opcode,
            parent: None,
            next: None,
            inputs: None,
            fields: Some(Fields::Value(param.name)),
            mutation: None,
        })
    }

    pub fn start_script(&mut self, hat: block::Hat) {
        let id = self.uid_generator.new_uid();
        self.inner.blocks.insert(id, hat.into());
        self.point = InsertionPoint {
            parent: Some(id),
            place: Place::Next,
        };
    }

    pub fn put(&mut self, block: block::Stacking) {
        let mut block = Block::from(block);
        block.parent = self.point.parent;
        let id = self.insert(block);
        self.set_next(id);
        self.point = InsertionPoint {
            parent: Some(id),
            place: Place::Next,
        };
    }

    pub fn forever(&mut self) {
        self.put(block::Stacking::new("control_forever"));
        self.point.place = Place::Substack1;
    }

    pub fn repeat(&mut self, times: Operand) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: "control_repeat",
            inputs: Some([("TIMES", times.0)].into()),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn for_(&mut self, variable: VariableRef, times: Operand) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: "control_for_each",
            inputs: Some([("VALUE", times.0)].into()),
            fields: Some(Fields::Variable(variable)),
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn if_(&mut self, condition: Operand) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: "control_if",
            inputs: Some([("CONDITION", condition.0)].into()),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn if_else(&mut self, condition: Operand) -> [InsertionPoint; 2] {
        self.put(block::Stacking {
            opcode: "control_if_else",
            inputs: Some([("CONDITION", condition.0)].into()),
            fields: None,
        });
        let after = self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        });
        let else_ = InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack2,
        };
        [after, else_]
    }

    pub fn while_(&mut self, condition: Operand) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: "control_while",
            inputs: Some([("CONDITION", condition.0)].into()),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn repeat_until(&mut self, condition: Operand) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: "control_repeat_until",
            inputs: Some([("CONDITION", condition.0)].into()),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    fn binary_operation(
        &mut self,
        opcode: &'static str,
        operands: [(&'static str, Input); 2],
    ) -> Operand {
        self.op(Block {
            opcode,
            parent: None,
            next: None,
            inputs: Some(operands.into()),
            fields: None,
            mutation: None,
        })
    }

    pub fn add(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_add", [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn sub(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_subtract", [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn mul(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_multiply", [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn div(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_divide", [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn modulo(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_mod", [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn lt(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_lt", [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)])
    }

    pub fn eq(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_equals",
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
    }

    pub fn gt(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_gt", [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)])
    }

    pub fn not(&mut self, operand: Operand) -> Operand {
        self.op(Block {
            opcode: "operator_not",
            parent: None,
            next: None,
            inputs: Some([("OPERAND", operand.0)].into()),
            fields: None,
            mutation: None,
        })
    }

    pub fn and(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_and", [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)])
    }

    pub fn or(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_or", [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)])
    }

    pub fn x_position(&mut self) -> Operand {
        self.op(Block::symbol("motion_xposition"))
    }

    pub fn y_position(&mut self) -> Operand {
        self.op(Block::symbol("motion_yposition"))
    }

    pub fn timer(&mut self) -> Operand {
        self.op(Block::symbol("sensing_timer"))
    }

    pub fn answer(&mut self) -> Operand {
        self.op(Block::symbol("sensing_answer"))
    }

    pub fn item_of_list(&mut self, list: ListRef, index: Operand) -> Operand {
        self.op(Block {
            opcode: "data_itemoflist",
            parent: None,
            next: None,
            inputs: Some([("INDEX", index.0)].into()),
            fields: Some(Fields::List(list)),
            mutation: None,
        })
    }

    pub fn item_num_of_list(&mut self, list: ListRef, item: Operand) -> Operand {
        self.op(Block {
            opcode: "data_itemnumoflist",
            parent: None,
            next: None,
            inputs: Some([("ITEM", item.0)].into()),
            fields: Some(Fields::List(list)),
            mutation: None,
        })
    }

    pub fn length(&mut self, string: Operand) -> Operand {
        self.op(Block {
            opcode: "operator_length",
            parent: None,
            next: None,
            inputs: Some([("STRING", string.0)].into()),
            fields: None,
            mutation: None,
        })
    }

    pub fn length_of_list(&mut self, list: ListRef) -> Operand {
        self.op(Block {
            opcode: "data_lengthoflist",
            parent: None,
            next: None,
            inputs: None,
            fields: Some(Fields::List(list)),
            mutation: None,
        })
    }

    pub fn letter_of(&mut self, string: Operand, index: Operand) -> Operand {
        self.op(Block {
            opcode: "operator_letter_of",
            parent: None,
            next: None,
            inputs: Some([("STRING", string.0), ("LETTER", index.0)].into()),
            fields: None,
            mutation: None,
        })
    }

    pub fn list_contains_item(&mut self, list: ListRef, item: Operand) -> Operand {
        self.op(Block {
            opcode: "data_listcontainsitem",
            parent: None,
            next: None,
            inputs: Some([("ITEM", item.0)].into()),
            fields: Some(Fields::List(list)),
            mutation: None,
        })
    }

    pub fn join(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation("operator_join", [("STRING1", lhs.0), ("STRING2", rhs.0)])
    }

    pub fn mathop(&mut self, operator: &'static str, num: Operand) -> Operand {
        self.op(Block {
            opcode: "operator_mathop",
            parent: None,
            next: None,
            inputs: Some([("NUM", num.0)].into()),
            fields: Some(Fields::Operator(operator)),
            mutation: None,
        })
    }

    pub fn mouse_x(&mut self) -> Operand {
        self.op(Block {
            opcode: "sensing_mousex",
            parent: None,
            next: None,
            inputs: None,
            fields: None,
            mutation: None,
        })
    }

    pub fn mouse_y(&mut self) -> Operand {
        self.op(Block {
            opcode: "sensing_mousey",
            parent: None,
            next: None,
            inputs: None,
            fields: None,
            mutation: None,
        })
    }

    pub fn key_is_pressed(&mut self, key: Operand) -> Operand {
        self.op(Block {
            opcode: "sensing_keypressed",
            parent: None,
            next: None,
            inputs: Some([("KEY_OPTION", key.0)].into()),
            fields: None,
            mutation: None,
        })
    }

    pub fn random(&mut self, from: Operand, to: Operand) -> Operand {
        self.op(Block {
            opcode: "operator_random",
            parent: None,
            next: None,
            inputs: Some([("FROM", from.0), ("TO", to.0)].into()),
            fields: None,
            mutation: None,
        })
    }

    pub fn clone_self(&mut self) {
        let menu = self.insert(Block {
            opcode: "control_create_clone_of_menu",
            parent: None,
            next: None,
            inputs: None,
            fields: Some(Fields::CloneSelf),
            mutation: None,
        });
        self.put(block::Stacking {
            opcode: "control_create_clone_of",
            inputs: Some([("CLONE_OPTION", Input::Prototype(menu))].into()),
            fields: None,
        });
    }

    fn op(&mut self, block: Block) -> Operand {
        Operand(Input::Substack(self.insert(block)))
    }

    fn insert(&mut self, block: Block) -> Uid {
        let id = self.uid_generator.new_uid();
        if let Some(inputs) = &block.inputs {
            for input in inputs.values() {
                if let Input::Substack(operand_block_id) | Input::Prototype(operand_block_id) =
                    input
                {
                    self.inner
                        .blocks
                        .get_mut(operand_block_id)
                        .expect("operand block does not exist")
                        .parent = Some(id);
                }
            }
        }
        self.inner.blocks.insert(id, block);
        id
    }

    fn set_next(&mut self, next: Uid) {
        let Some(parent) = self.point.parent else {
            return;
        };
        let parent = self
            .inner
            .blocks
            .get_mut(&parent)
            .expect("parent block does not exist");
        match self.point.place {
            Place::Next => parent.next = Some(next),
            Place::Substack1 => {
                parent
                    .inputs
                    .get_or_insert_with(HashMap::default)
                    .insert("SUBSTACK", Input::Substack(next));
            }
            Place::Substack2 => {
                parent
                    .inputs
                    .get_or_insert_with(HashMap::default)
                    .insert("SUBSTACK2", Input::Substack(next));
            }
        }
    }
}

pub struct InsertionPoint {
    parent: Option<Uid>,
    place: Place,
}

enum Place {
    Next,
    Substack1,
    Substack2,
}

pub enum Constant {
    String(String),
    Number(f64),
}

impl Constant {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::String(s) => write!(writer, "{s:?}"),
            Self::Number(n) => write!(writer, "{n}"),
        }
    }
}

pub struct Variable {
    pub name: String,
    pub value: Constant,
}

impl Variable {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(writer, "[{:?},", self.name)?;
        self.value.serialize(writer)?;
        write!(writer, "]")
    }
}

#[derive(Clone)]
pub struct VariableRef {
    name: Box<str>,
    id: Uid,
}

pub struct List {
    pub name: String,
    pub items: Vec<Constant>,
}

impl List {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(writer, "[{:?}, [", self.name)?;
        for (i, item) in self.items.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            item.serialize(writer)?;
        }
        write!(writer, "]]")
    }
}

#[derive(Clone)]
pub struct ListRef {
    name: Box<str>,
    id: Uid,
}

#[derive(Clone)]
pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,
}

#[derive(Clone, Copy)]
pub enum ParameterKind {
    StringOrNumber,
    Boolean,
}

pub struct CustomBlock {
    mutation: Mutation,
    param_ids: Vec<&'static str>,
}

#[derive(Clone)]
struct Mutation {
    proccode: String,
    argumentids: String,
    argumentnames: Option<String>,
    argumentdefaults: Option<String>,
}

impl Mutation {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(
            writer,
            r#"{{"tagName":"mutation","children":[],"proccode":{:?},"argumentids":{:?},"warp":true"#,
            self.proccode, self.argumentids
        )?;
        if let Some(argumentnames) = &self.argumentnames {
            write!(writer, r#","argumentnames":{argumentnames:?}"#)?;
        }
        if let Some(argumentdefaults) = &self.argumentdefaults {
            write!(writer, r#","argumentdefaults":{argumentdefaults:?}"#)?;
        }
        write!(writer, "}}")
    }
}
