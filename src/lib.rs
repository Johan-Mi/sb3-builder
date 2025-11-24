pub mod block;
mod costume;

pub use costume::Costume;

use block::{Block, Fields, Input};
use std::{
    fmt::Write as _,
    io::{self, Write as _},
};

pub struct Project {
    targets: Vec<RealTarget>,
}

impl Default for Project {
    fn default() -> Self {
        let targets = Vec::from([RealTarget::new("Stage".to_owned(), true)]);
        Self { targets }
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
            next_parameter_id: 0,
            point: InsertionPoint {
                parent: None,
                place: Place::Next,
            },
        }
    }

    /// Writes the [`Project`] as a ZIP file to the given writer,
    /// typically a [`File`].
    ///
    /// # Errors
    ///
    /// This function will return an error if writing to the `writer` fails.
    ///
    /// [`File`]: std::fs::File
    pub fn finish(
        self,
        writer: impl io::Write + io::Seek,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut archive = rawzip::ZipArchiveWriter::new(writer);

        for costume in self.targets.iter().flat_map(|target| &target.costumes) {
            costume.add_to_archive(&mut archive)?;
        }

        let (mut entry, config) = archive
            .new_file("project.json")
            .compression_method(rawzip::CompressionMethod::Deflate)
            .start()?;
        let encoder =
            flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
        let mut file = config.wrap(encoder);

        write!(file, r#"{{"meta":{{"semver":"3.0.0"}},"targets":["#)?;
        for (i, target) in self.targets.iter().enumerate() {
            if i != 0 {
                write!(file, ",")?;
            }
            target.serialize(&mut file)?;
        }
        write!(file, "]}}")?;

        let (_, descriptor) = file.finish()?;
        let _: u64 = entry.finish(descriptor)?;

        _ = archive.finish()?;

        Ok(())
    }
}

struct RealTarget {
    name: String,
    is_stage: bool,
    costumes: Vec<Costume>,
    variables: Vec<Variable>,
    lists: Vec<List>,
    blocks: Vec<block::Block>,
    comments: Vec<Comment>,
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
        for (i, variable) in self.variables.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, r#""v{i}":"#)?;
            variable.serialize(writer)?;
        }
        write!(writer, r#"}},"lists":{{"#)?;
        for (i, list) in self.lists.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, r#""l{i}":"#)?;
            list.serialize(writer)?;
        }
        write!(writer, r#"}},"blocks":{{"#)?;
        for (i, block) in self.blocks.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{}:", block::Id(i))?;
            block.serialize(writer)?;
        }
        write!(writer, r#"}},"comments":{{"#)?;
        for (i, comment) in self.comments.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, r#""c{i}":"#)?;
            comment.serialize(writer)?;
        }
        write!(writer, "}}}}")
    }
}

impl RealTarget {
    const fn new(name: String, is_stage: bool) -> Self {
        Self {
            name,
            is_stage,
            costumes: Vec::new(),
            variables: Vec::new(),
            lists: Vec::new(),
            blocks: Vec::new(),
            comments: Vec::new(),
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
    next_parameter_id: usize,
    point: InsertionPoint,
}

impl Target<'_> {
    pub fn add_comment(&mut self, text: String) {
        self.inner.comments.push(Comment { text });
    }

    pub fn add_costume(&mut self, costume: Costume) {
        self.inner.costumes.push(costume);
    }

    pub fn add_variable(&mut self, variable: Variable) -> VariableRef {
        let id = self.inner.variables.len();
        let name = (&*variable.name).into();
        self.inner.variables.push(variable);
        VariableRef { name, id }
    }

    pub fn add_list(&mut self, list: List) -> ListRef {
        let id = self.inner.variables.len();
        let name = (&*list.name).into();
        self.inner.lists.push(list);
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
        let param_ids = (self.next_parameter_id..self.next_parameter_id + parameters.len())
            .map(|id| &*id.to_string().leak())
            .collect::<Vec<_>>();
        self.next_parameter_id += parameters.len();

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

        let definition = block::Id(self.inner.blocks.len() + 1);
        let prototype = self.insert(Block {
            opcode: "procedures_prototype",
            parent: Some(definition),
            next: None,
            inputs,
            fields: None,
            mutation: Some(Box::new(Mutation {
                argumentnames: Some(argumentnames),
                argumentdefaults: Some(argumentdefaults),
                ..mutation.clone()
            })),
        });

        self.inner.blocks.push(
            block::Stacking {
                opcode: "procedures_definition",
                inputs: Vec::from([("custom_block", Input::Prototype(prototype))]),
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
        let inputs = std::iter::zip(
            block.param_ids.iter().copied(),
            arguments.into_iter().map(|arg| arg.0),
        )
        .collect::<Vec<_>>();

        let id = self.insert(Block {
            opcode: "procedures_call",
            parent: self.point.parent,
            next: None,
            inputs,
            fields: None,
            mutation: Some(Box::new(block.mutation.clone())),
        });
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
        self.op(Block::new(opcode).fields(Fields::Value(param.name)))
    }

    pub fn start_script(&mut self, hat: block::Hat) {
        let id = block::Id(self.inner.blocks.len());
        self.inner.blocks.push(hat.into());
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
            inputs: Vec::from([("TIMES", times.0)]),
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
            inputs: Vec::from([("VALUE", times.0)]),
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
            inputs: Vec::from([("CONDITION", condition.0)]),
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
            inputs: Vec::from([("CONDITION", condition.0)]),
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
            inputs: Vec::from([("CONDITION", condition.0)]),
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
            inputs: Vec::from([("CONDITION", condition.0)]),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn add(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_add").inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn sub(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_subtract").inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn mul(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_multiply").inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn div(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_divide").inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn modulo(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_mod").inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn lt(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_lt").inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn eq(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_equals").inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn gt(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_gt").inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn not(&mut self, operand: Operand) -> Operand {
        self.op(Block::new("operator_not").inputs([("OPERAND", operand.0)]))
    }

    pub fn and(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_and").inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn or(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_or").inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn x_position(&mut self) -> Operand {
        self.op(Block::new("motion_xposition"))
    }

    pub fn y_position(&mut self) -> Operand {
        self.op(Block::new("motion_yposition"))
    }

    pub fn timer(&mut self) -> Operand {
        self.op(Block::new("sensing_timer"))
    }

    pub fn answer(&mut self) -> Operand {
        self.op(Block::new("sensing_answer"))
    }

    pub fn item_of_list(&mut self, list: ListRef, index: Operand) -> Operand {
        self.op(Block::new("data_itemoflist")
            .inputs([("INDEX", index.0)])
            .fields(Fields::List(list)))
    }

    pub fn item_num_of_list(&mut self, list: ListRef, item: Operand) -> Operand {
        self.op(Block::new("data_itemnumoflist")
            .inputs([("ITEM", item.0)])
            .fields(Fields::List(list)))
    }

    pub fn length(&mut self, string: Operand) -> Operand {
        self.op(Block::new("operator_length").inputs([("STRING", string.0)]))
    }

    pub fn length_of_list(&mut self, list: ListRef) -> Operand {
        self.op(Block::new("data_lengthoflist").fields(Fields::List(list)))
    }

    pub fn letter_of(&mut self, string: Operand, index: Operand) -> Operand {
        self.op(Block::new("operator_letter_of").inputs([("STRING", string.0), ("LETTER", index.0)]))
    }

    pub fn contains(&mut self, haystack: Operand, needle: Operand) -> Operand {
        self.op(Block::new("operator_contains")
            .inputs([("STRING1", haystack.0), ("STRING2", needle.0)]))
    }

    pub fn list_contains_item(&mut self, list: ListRef, item: Operand) -> Operand {
        self.op(Block::new("data_listcontainsitem")
            .inputs([("ITEM", item.0)])
            .fields(Fields::List(list)))
    }

    pub fn join(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block::new("operator_join").inputs([("STRING1", lhs.0), ("STRING2", rhs.0)]))
    }

    pub fn mathop(&mut self, operator: &'static str, num: Operand) -> Operand {
        self.op(Block::new("operator_mathop")
            .inputs([("NUM", num.0)])
            .fields(Fields::Operator(operator)))
    }

    pub fn mouse_x(&mut self) -> Operand {
        self.op(Block::new("sensing_mousex"))
    }

    pub fn mouse_y(&mut self) -> Operand {
        self.op(Block::new("sensing_mousey"))
    }

    pub fn key_is_pressed(&mut self, key: Operand) -> Operand {
        self.op(Block::new("sensing_keypressed").inputs([("KEY_OPTION", key.0)]))
    }

    pub fn random(&mut self, from: Operand, to: Operand) -> Operand {
        self.op(Block::new("operator_random").inputs([("FROM", from.0), ("TO", to.0)]))
    }

    pub fn clone_self(&mut self) {
        let menu =
            self.insert(Block::new("control_create_clone_of_menu").fields(Fields::CloneSelf));
        self.put(block::Stacking {
            opcode: "control_create_clone_of",
            inputs: Vec::from([("CLONE_OPTION", Input::Prototype(menu))]),
            fields: None,
        });
    }

    fn op(&mut self, block: Block) -> Operand {
        Operand(Input::Substack(self.insert(block)))
    }

    fn insert(&mut self, block: Block) -> block::Id {
        let id = block::Id(self.inner.blocks.len());
        for (_, input) in &block.inputs {
            if let Input::Substack(operand_block_id) | Input::Prototype(operand_block_id) = input {
                self.inner.blocks[operand_block_id.0].parent = Some(id);
            }
        }
        self.inner.blocks.push(block);
        id
    }

    fn set_next(&mut self, next: block::Id) {
        let Some(parent) = self.point.parent else {
            return;
        };
        let parent = &mut self.inner.blocks[parent.0];
        match self.point.place {
            Place::Next => parent.next = Some(next),
            Place::Substack1 => parent.inputs.push(("SUBSTACK", Input::Substack(next))),
            Place::Substack2 => parent.inputs.push(("SUBSTACK2", Input::Substack(next))),
        }
    }
}

pub struct InsertionPoint {
    parent: Option<block::Id>,
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
    id: usize,
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
    id: usize,
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

pub struct Operand(Input);

impl From<f64> for Operand {
    fn from(value: f64) -> Self {
        Self(Input::Number(value))
    }
}

impl From<String> for Operand {
    fn from(value: String) -> Self {
        Self(Input::String(value))
    }
}

impl From<VariableRef> for Operand {
    fn from(value: VariableRef) -> Self {
        Self(Input::Variable(value))
    }
}

impl From<ListRef> for Operand {
    fn from(value: ListRef) -> Self {
        Self(Input::List(value))
    }
}
