pub mod block;
mod costume;

pub use costume::Costume;

use block::{Block, Fields, Input, Opcode};
use std::io::{self, Write as _};

pub struct Project<'strings> {
    targets: Vec<RealTarget<'strings>>,
}

impl Default for Project<'_> {
    fn default() -> Self {
        let targets = Vec::from([RealTarget::new("Stage", true)]);
        Self { targets }
    }
}

impl<'strings> Project<'strings> {
    pub fn stage(&mut self) -> Target<'strings, '_> {
        self.target(0)
    }

    pub fn add_sprite(&mut self, name: &'strings str) -> Target<'strings, '_> {
        self.targets.push(RealTarget::new(name, false));
        self.target(self.targets.len() - 1)
    }

    fn target(&mut self, index: usize) -> Target<'strings, '_> {
        Target {
            inner: &mut self.targets[index],
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

struct RealTarget<'strings> {
    name: &'strings str,
    is_stage: bool,
    costumes: Vec<Costume<'strings>>,
    variables: Vec<Variable<'strings>>,
    lists: Vec<List<'strings>>,
    blocks: Vec<block::Block<'strings>>,
    parameters: Vec<Parameter>,
    custom_blocks: Vec<CustomBlock>,
    comments: Vec<Comment>,
}

impl RealTarget<'_> {
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
            block.serialize(self, writer)?;
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

impl<'strings> RealTarget<'strings> {
    const fn new(name: &'strings str, is_stage: bool) -> Self {
        Self {
            name,
            is_stage,
            costumes: Vec::new(),
            variables: Vec::new(),
            lists: Vec::new(),
            blocks: Vec::new(),
            parameters: Vec::new(),
            custom_blocks: Vec::new(),
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

pub struct Target<'strings, 'project> {
    inner: &'project mut RealTarget<'strings>,
    point: InsertionPoint,
}

impl<'strings> Target<'strings, '_> {
    pub fn add_comment(&mut self, text: String) {
        self.inner.comments.push(Comment { text });
    }

    pub fn add_costume(&mut self, costume: Costume<'strings>) {
        self.inner.costumes.push(costume);
    }

    pub fn add_variable(&mut self, variable: Variable<'strings>) -> VariableRef {
        let id = self.inner.variables.len();
        self.inner.variables.push(variable);
        VariableRef(id)
    }

    pub fn add_list(&mut self, list: List<'strings>) -> ListRef {
        let id = self.inner.lists.len();
        self.inner.lists.push(list);
        ListRef(id)
    }

    pub const fn insert_at(&mut self, point: InsertionPoint) -> InsertionPoint {
        std::mem::replace(&mut self.point, point)
    }

    pub fn add_parameter(&mut self, parameter: Parameter) -> ParameterRef {
        let index = self.inner.parameters.len();
        self.inner.parameters.push(parameter);
        ParameterRef(index)
    }

    pub fn add_custom_block(&mut self, block: CustomBlock) -> (CustomBlockRef, InsertionPoint) {
        let inputs = block
            .parameters
            .iter()
            .map(|&it| (&*it.0.to_string().leak(), self.custom_block_parameter(it).0))
            .collect();

        let index = self.inner.custom_blocks.len();
        self.inner.custom_blocks.push(block);

        let definition = block::Id(self.inner.blocks.len() + 1);
        let prototype = self.insert(Block {
            opcode: Opcode::procedures_prototype,
            parent: Some(definition),
            next: None,
            inputs,
            fields: None,
            mutation: Mutation(CustomBlockRef(index)),
        });

        self.inner.blocks.push(
            block::Stacking {
                opcode: Opcode::procedures_definition,
                inputs: Box::new([("custom_block", Input::Prototype(prototype))]),
                fields: None,
            }
            .into(),
        );

        let point = InsertionPoint {
            parent: Some(definition),
            place: Place::Next,
        };
        (CustomBlockRef(index), point)
    }

    pub fn use_custom_block(&mut self, block: CustomBlockRef, arguments: Vec<Operand<'strings>>) {
        let inputs = self.inner.custom_blocks[block.0]
            .parameters
            .iter()
            .map(|it| &*it.0.to_string().leak())
            .zip(arguments.into_iter().map(|arg| arg.0))
            .collect();

        let id = self.insert(Block {
            opcode: Opcode::procedures_call,
            parent: self.point.parent,
            next: None,
            inputs,
            fields: None,
            mutation: Mutation(block),
        });
        self.set_next(id);
        self.point = InsertionPoint {
            parent: Some(id),
            place: Place::Next,
        };
    }

    pub fn custom_block_parameter(&mut self, parameter: ParameterRef) -> Operand<'strings> {
        let opcode = match self.inner.parameters[parameter.0].kind {
            ParameterKind::StringOrNumber => Opcode::argument_reporter_string_number,
            ParameterKind::Boolean => Opcode::argument_reporter_boolean,
        };
        self.op(Block::new(opcode).fields(Fields::Value(parameter)))
    }

    pub fn start_script(&mut self, hat: block::Hat<'strings>) {
        let id = block::Id(self.inner.blocks.len());
        self.inner.blocks.push(hat.into());
        self.point = InsertionPoint {
            parent: Some(id),
            place: Place::Next,
        };
    }

    pub fn put(&mut self, block: block::Stacking<'strings>) {
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
        self.put(block::Stacking {
            opcode: Opcode::control_forever,
            inputs: Box::new([("SUBSTACK", Input::EmptySubstack)]),
            fields: None,
        });
        self.point.place = Place::Substack1;
    }

    pub fn repeat(&mut self, times: Operand<'strings>) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: Opcode::control_repeat,
            inputs: Box::new([("SUBSTACK", Input::EmptySubstack), ("TIMES", times.0)]),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn for_(&mut self, variable: VariableRef, times: Operand<'strings>) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: Opcode::control_for_each,
            inputs: Box::new([("SUBSTACK", Input::EmptySubstack), ("VALUE", times.0)]),
            fields: Some(Fields::Variable(variable)),
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn if_(&mut self, condition: Operand<'strings>) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: Opcode::control_if,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn if_else(&mut self, condition: Operand<'strings>) -> [InsertionPoint; 2] {
        self.put(block::Stacking {
            opcode: Opcode::control_if_else,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("SUBSTACK2", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
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

    pub fn while_(&mut self, condition: Operand<'strings>) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: Opcode::control_while,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn repeat_until(&mut self, condition: Operand<'strings>) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: Opcode::control_repeat_until,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn add(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_add).inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn sub(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_subtract).inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn mul(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_multiply).inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn div(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_divide).inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn modulo(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_mod).inputs([("NUM1", lhs.0), ("NUM2", rhs.0)]))
    }

    pub fn lt(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_lt).inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn eq(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Block::new(Opcode::operator_equals).inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)])
        )
    }

    pub fn gt(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_gt).inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn not(&mut self, operand: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_not).inputs([("OPERAND", operand.0)]))
    }

    pub fn and(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_and).inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn or(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_or).inputs([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)]))
    }

    pub fn x_position(&mut self) -> Operand<'strings> {
        self.op(Block::new(Opcode::motion_xposition))
    }

    pub fn y_position(&mut self) -> Operand<'strings> {
        self.op(Block::new(Opcode::motion_yposition))
    }

    pub fn timer(&mut self) -> Operand<'strings> {
        self.op(Block::new(Opcode::sensing_timer))
    }

    pub fn answer(&mut self) -> Operand<'strings> {
        self.op(Block::new(Opcode::sensing_answer))
    }

    pub fn item_of_list(&mut self, list: ListRef, index: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::data_itemoflist)
            .inputs([("INDEX", index.0)])
            .fields(Fields::List(list)))
    }

    pub fn item_num_of_list(
        &mut self,
        list: ListRef,
        item: Operand<'strings>,
    ) -> Operand<'strings> {
        self.op(Block::new(Opcode::data_itemnumoflist)
            .inputs([("ITEM", item.0)])
            .fields(Fields::List(list)))
    }

    pub fn length(&mut self, string: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_length).inputs([("STRING", string.0)]))
    }

    pub fn length_of_list(&mut self, list: ListRef) -> Operand<'strings> {
        self.op(Block::new(Opcode::data_lengthoflist).fields(Fields::List(list)))
    }

    pub fn letter_of(
        &mut self,
        string: Operand<'strings>,
        index: Operand<'strings>,
    ) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_letter_of)
            .inputs([("STRING", string.0), ("LETTER", index.0)]))
    }

    pub fn contains(
        &mut self,
        haystack: Operand<'strings>,
        needle: Operand<'strings>,
    ) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_contains)
            .inputs([("STRING1", haystack.0), ("STRING2", needle.0)]))
    }

    pub fn list_contains_item(
        &mut self,
        list: ListRef,
        item: Operand<'strings>,
    ) -> Operand<'strings> {
        self.op(Block::new(Opcode::data_listcontainsitem)
            .inputs([("ITEM", item.0)])
            .fields(Fields::List(list)))
    }

    pub fn join(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_join).inputs([("STRING1", lhs.0), ("STRING2", rhs.0)]))
    }

    pub fn mathop(&mut self, operator: &'static str, num: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_mathop)
            .inputs([("NUM", num.0)])
            .fields(Fields::Operator(operator)))
    }

    pub fn mouse_x(&mut self) -> Operand<'strings> {
        self.op(Block::new(Opcode::sensing_mousex))
    }

    pub fn mouse_y(&mut self) -> Operand<'strings> {
        self.op(Block::new(Opcode::sensing_mousey))
    }

    pub fn key_is_pressed(&mut self, key: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::sensing_keypressed).inputs([("KEY_OPTION", key.0)]))
    }

    pub fn random(&mut self, from: Operand<'strings>, to: Operand<'strings>) -> Operand<'strings> {
        self.op(Block::new(Opcode::operator_random).inputs([("FROM", from.0), ("TO", to.0)]))
    }

    pub fn clone_self(&mut self) {
        let menu =
            self.insert(Block::new(Opcode::control_create_clone_of_menu).fields(Fields::CloneSelf));
        self.put(block::Stacking {
            opcode: Opcode::control_create_clone_of,
            inputs: Box::new([("CLONE_OPTION", Input::Prototype(menu))]),
            fields: None,
        });
    }

    fn op(&mut self, block: Block<'strings>) -> Operand<'strings> {
        Operand(Input::Substack(self.insert(block)))
    }

    fn insert(&mut self, block: Block<'strings>) -> block::Id {
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
            Place::Substack1 => parent.inputs[0].1 = Input::Substack(next),
            Place::Substack2 => parent.inputs[1].1 = Input::Substack(next),
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

pub enum Constant<'strings> {
    String(&'strings str),
    Number(f64),
}

impl Constant<'_> {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        match self {
            Self::String(s) => write!(writer, "{s:?}"),
            Self::Number(n) => write!(writer, "{n}"),
        }
    }
}

pub struct Variable<'strings> {
    pub name: String,
    pub value: Constant<'strings>,
}

impl Variable<'_> {
    fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(writer, "[{:?},", self.name)?;
        self.value.serialize(writer)?;
        write!(writer, "]")
    }
}

#[derive(Clone, Copy)]
pub struct VariableRef(usize);

pub struct List<'strings> {
    pub name: String,
    pub items: Vec<Constant<'strings>>,
}

impl List<'_> {
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

#[derive(Clone, Copy)]
pub struct ListRef(usize);

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

#[derive(Clone, Copy)]
pub struct ParameterRef(usize);

pub struct CustomBlock {
    name: String,
    parameters: Vec<ParameterRef>,
}

#[derive(Clone, Copy)]
pub struct CustomBlockRef(usize);

#[derive(Clone, Copy)]
struct Mutation(CustomBlockRef);

impl Mutation {
    const NONE: Self = Self(CustomBlockRef(0));

    fn serialize(
        self,
        is_prototype: bool,
        target: &RealTarget,
        writer: &mut dyn io::Write,
    ) -> io::Result<()> {
        let block = &target.custom_blocks[self.0 .0];
        write!(
            writer,
            r#"{{"tagName":"mutation","children":[],"proccode":"{}"#,
            block.name.escape_debug()
        )?;
        for parameter in &block.parameters {
            writer.write_all(match target.parameters[parameter.0].kind {
                ParameterKind::StringOrNumber => b" %s",
                ParameterKind::Boolean => b" %b",
            })?;
        }
        write!(writer, r#"","argumentids":""#)?;
        for (index, parameter) in block.parameters.iter().enumerate() {
            if index != 0 {
                write!(writer, ",")?;
            }
            write!(writer, r#"\"{}\""#, parameter.0)?;
        }
        write!(writer, r#"","warp":true"#)?;
        if is_prototype {
            write!(writer, r#","argumentnames":"["#)?;
            for (index, parameter) in block.parameters.iter().enumerate() {
                if index != 0 {
                    write!(writer, ",")?;
                }
                write!(writer, r#"""#)?;
                for c in target.parameters[parameter.0]
                    .name
                    .escape_debug()
                    .flat_map(char::escape_debug)
                {
                    write!(writer, "{c}")?;
                }
                write!(writer, r#"""#)?;
            }
            write!(writer, r#"]","argumentdefaults":""#)?;
            for (index, parameter) in block.parameters.iter().enumerate() {
                if index != 0 {
                    write!(writer, ",")?;
                }
                writer.write_all(match target.parameters[parameter.0].kind {
                    ParameterKind::StringOrNumber => br#""""#,
                    ParameterKind::Boolean => b"false",
                })?;
            }
            write!(writer, r#"""#)?;
        }
        write!(writer, "}}")
    }
}

pub struct Operand<'strings>(Input<'strings>);

impl From<f64> for Operand<'_> {
    fn from(value: f64) -> Self {
        Self(Input::Number(value))
    }
}

impl<'strings> From<&'strings str> for Operand<'strings> {
    fn from(value: &'strings str) -> Self {
        Self(Input::String(value))
    }
}

impl From<VariableRef> for Operand<'_> {
    fn from(value: VariableRef) -> Self {
        Self(Input::Variable(value))
    }
}

impl From<ListRef> for Operand<'_> {
    fn from(value: ListRef) -> Self {
        Self(Input::List(value))
    }
}
