pub mod block;
mod costume;

pub use costume::Costume;

use block::{Block, Fields, Input, Opcode};
use std::io::{self, Write as _};
use std::ops::Range;

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
            place: Place::Nowhere,
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
    blocks: Vec<block::Block>,
    inputs: Vec<(&'static str, Input<'strings>)>,
    fields: Vec<Fields<'strings>>,
    mutations: Vec<Mutation>,
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
        let mut all_inputs = &*self.inputs;
        let mut fields = self.fields.iter().copied();
        let mut mutations = self.mutations.iter().copied();
        for (i, block) in self.blocks.iter().enumerate() {
            if i != 0 {
                write!(writer, ",")?;
            }
            write!(writer, "{}:", block::Id(i))?;
            let mutation = matches!(
                block.opcode,
                Opcode::procedures_call | Opcode::procedures_prototype
            )
            .then(|| mutations.next().unwrap_or_else(|| unreachable!()));
            let input_count = block.opcode.input_count().unwrap_or_else(|| {
                self.custom_blocks[mutation.unwrap_or_else(|| unreachable!()).0 .0]
                    .parameters
                    .len()
            });
            let inputs = all_inputs
                .split_off(..input_count)
                .unwrap_or_else(|| unreachable!());
            let fields = block
                .opcode
                .has_fields()
                .then(|| fields.next().unwrap_or_else(|| unreachable!()));
            block.serialize(inputs, fields, mutation, self, writer)?;
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
            inputs: Vec::new(),
            fields: Vec::new(),
            mutations: Vec::new(),
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
    place: Place,
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

    #[expect(
        clippy::needless_pass_by_value,
        reason = "`InsertionPoint` is intentionally impossible to copy"
    )]
    pub const fn insert_at(&mut self, point: InsertionPoint) -> InsertionPoint {
        self.insert_at_(point.0)
    }

    const fn insert_at_(&mut self, place: Place) -> InsertionPoint {
        InsertionPoint(std::mem::replace(&mut self.place, place))
    }

    pub fn add_custom_block(
        &mut self,
        name: String,
        parameters: impl Iterator<Item = Parameter>,
    ) -> (CustomBlockRef, InsertionPoint) {
        let start = self.inner.parameters.len();
        self.inner.parameters.extend(parameters);

        let index = self.inner.custom_blocks.len();
        self.inner.custom_blocks.push(CustomBlock {
            name,
            parameters: start..self.inner.parameters.len(),
        });

        let definition = block::Id(self.inner.blocks.len() + 1);
        self.inner.mutations.push(Mutation(CustomBlockRef(index)));
        let prototype = self.insert(Block {
            opcode: Opcode::procedures_prototype,
            parent: definition.into(),
            next: None.into(),
        });

        for parameter in start..self.inner.parameters.len() {
            let id = &*parameter.to_string().leak();
            let input = self.custom_block_parameter_raw(parameter).0;
            self.add_inputs(prototype, [(id, input)]);
        }

        let id = block::Id(self.inner.blocks.len());
        self.add_inputs(id, [("custom_block", Input::Prototype(prototype))]);
        self.inner.blocks.push(Block {
            opcode: Opcode::procedures_definition,
            parent: None.into(),
            next: None.into(),
        });

        let point = InsertionPoint(Place::After(definition));
        (CustomBlockRef(index), point)
    }

    /// # Panics
    ///
    /// Panics if no script has been started.
    pub fn use_custom_block(&mut self, block: CustomBlockRef, arguments: Vec<Operand<'strings>>) {
        self.inner.mutations.push(Mutation(block));
        let (Place::After(parent) | Place::Inside { block: parent, .. }) = self.place else {
            panic!("cannot put block when no script has been started");
        };
        let id = self.insert(Block {
            opcode: Opcode::procedures_call,
            parent: parent.into(),
            next: None.into(),
        });
        self.add_inputs(
            id,
            self.inner.custom_blocks[block.0]
                .parameters
                .clone()
                .map(|it| &*it.to_string().leak())
                .zip(arguments.into_iter().map(|arg| arg.0)),
        );
        self.set_next(id);
        self.place = Place::After(id);
    }

    /// # Panics
    ///
    /// Panics if `index` is out of bounds for `block`.
    pub fn custom_block_parameter(
        &mut self,
        block: CustomBlockRef,
        index: usize,
    ) -> Operand<'strings> {
        let range = &self.inner.custom_blocks[block.0].parameters;
        assert!(index < range.len());
        self.custom_block_parameter_raw(range.start + index)
    }

    pub fn custom_block_parameter_raw(&mut self, absolute_index: usize) -> Operand<'strings> {
        let opcode = match self.inner.parameters[absolute_index].kind {
            ParameterKind::StringOrNumber => Opcode::argument_reporter_string_number,
            ParameterKind::Boolean => Opcode::argument_reporter_boolean,
        };
        self.inner.fields.push(Fields::Value(absolute_index));
        self.op(opcode, [])
    }

    pub fn start_script(&mut self, hat: block::Hat<'strings>) {
        self.inner.fields.extend(hat.fields);
        let id = block::Id(self.inner.blocks.len());
        self.inner.blocks.push(Block {
            opcode: hat.opcode,
            parent: None.into(),
            next: None.into(),
        });
        self.place = Place::After(id);
    }

    pub fn put(&mut self, block: block::Stacking<'strings>) {
        let _: block::Id = self.put_(block);
    }

    fn put_(&mut self, block: block::Stacking<'strings>) -> block::Id {
        self.inner.fields.extend(block.fields);
        let (Place::After(parent) | Place::Inside { block: parent, .. }) = self.place else {
            panic!("cannot put block when no script has been started");
        };
        let id = self.insert(Block {
            opcode: block.opcode,
            parent: parent.into(),
            next: None.into(),
        });
        self.add_inputs(id, block.inputs);
        self.set_next(id);
        self.place = Place::After(id);
        id
    }

    pub fn forever(&mut self) {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_forever,
            inputs: Box::new([("SUBSTACK", Input::EmptySubstack)]),
            fields: None,
        });
        self.place = Place::Inside { block, input };
    }

    pub fn repeat(&mut self, times: Operand<'strings>) -> InsertionPoint {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_repeat,
            inputs: Box::new([("SUBSTACK", Input::EmptySubstack), ("TIMES", times.0)]),
            fields: None,
        });
        self.insert_at_(Place::Inside { block, input })
    }

    pub fn for_(&mut self, variable: VariableRef, times: Operand<'strings>) -> InsertionPoint {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_for_each,
            inputs: Box::new([("SUBSTACK", Input::EmptySubstack), ("VALUE", times.0)]),
            fields: Some(Fields::Variable(variable)),
        });
        self.insert_at_(Place::Inside { block, input })
    }

    pub fn if_(&mut self, condition: Operand<'strings>) -> InsertionPoint {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_if,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        self.insert_at_(Place::Inside { block, input })
    }

    pub fn if_else(&mut self, condition: Operand<'strings>) -> [InsertionPoint; 2] {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_if_else,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("SUBSTACK2", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        let after = self.insert_at_(Place::Inside { block, input });
        let else_ = InsertionPoint(Place::Inside {
            block,
            input: input + 1,
        });
        [after, else_]
    }

    pub fn while_(&mut self, condition: Operand<'strings>) -> InsertionPoint {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_while,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        self.insert_at_(Place::Inside { block, input })
    }

    pub fn repeat_until(&mut self, condition: Operand<'strings>) -> InsertionPoint {
        let input = self.inner.inputs.len();
        let block = self.put_(block::Stacking {
            opcode: Opcode::control_repeat_until,
            inputs: Box::new([
                ("SUBSTACK", Input::EmptySubstack),
                ("CONDITION", condition.0),
            ]),
            fields: None,
        });
        self.insert_at_(Place::Inside { block, input })
    }

    pub fn add(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::operator_add, [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn sub(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_subtract,
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn mul(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_multiply,
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn div(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::operator_divide, [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn modulo(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::operator_mod, [("NUM1", lhs.0), ("NUM2", rhs.0)])
    }

    pub fn lt(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_lt,
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
    }

    pub fn eq(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_equals,
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
    }

    pub fn gt(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_gt,
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
    }

    pub fn not(&mut self, operand: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::operator_not, [("OPERAND", operand.0)])
    }

    pub fn and(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_and,
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
    }

    pub fn or(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_or,
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
    }

    pub fn x_position(&mut self) -> Operand<'strings> {
        self.op(Opcode::motion_xposition, [])
    }

    pub fn y_position(&mut self) -> Operand<'strings> {
        self.op(Opcode::motion_yposition, [])
    }

    pub fn timer(&mut self) -> Operand<'strings> {
        self.op(Opcode::sensing_timer, [])
    }

    pub fn answer(&mut self) -> Operand<'strings> {
        self.op(Opcode::sensing_answer, [])
    }

    pub fn item_of_list(&mut self, list: ListRef, index: Operand<'strings>) -> Operand<'strings> {
        self.inner.fields.push(Fields::List(list));
        self.op(Opcode::data_itemoflist, [("INDEX", index.0)])
    }

    pub fn item_num_of_list(
        &mut self,
        list: ListRef,
        item: Operand<'strings>,
    ) -> Operand<'strings> {
        self.inner.fields.push(Fields::List(list));
        self.op(Opcode::data_itemnumoflist, [("ITEM", item.0)])
    }

    pub fn length(&mut self, string: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::operator_length, [("STRING", string.0)])
    }

    pub fn length_of_list(&mut self, list: ListRef) -> Operand<'strings> {
        self.inner.fields.push(Fields::List(list));
        self.op(Opcode::data_lengthoflist, [])
    }

    pub fn letter_of(
        &mut self,
        string: Operand<'strings>,
        index: Operand<'strings>,
    ) -> Operand<'strings> {
        self.op(
            Opcode::operator_letter_of,
            [("STRING", string.0), ("LETTER", index.0)],
        )
    }

    pub fn contains(
        &mut self,
        haystack: Operand<'strings>,
        needle: Operand<'strings>,
    ) -> Operand<'strings> {
        self.op(
            Opcode::operator_contains,
            [("STRING1", haystack.0), ("STRING2", needle.0)],
        )
    }

    pub fn list_contains_item(
        &mut self,
        list: ListRef,
        item: Operand<'strings>,
    ) -> Operand<'strings> {
        self.inner.fields.push(Fields::List(list));
        self.op(Opcode::data_listcontainsitem, [("ITEM", item.0)])
    }

    pub fn join(&mut self, lhs: Operand<'strings>, rhs: Operand<'strings>) -> Operand<'strings> {
        self.op(
            Opcode::operator_join,
            [("STRING1", lhs.0), ("STRING2", rhs.0)],
        )
    }

    pub fn mathop(&mut self, operator: &'static str, num: Operand<'strings>) -> Operand<'strings> {
        self.inner.fields.push(Fields::Operator(operator));
        self.op(Opcode::operator_mathop, [("NUM", num.0)])
    }

    pub fn mouse_x(&mut self) -> Operand<'strings> {
        self.op(Opcode::sensing_mousex, [])
    }

    pub fn mouse_y(&mut self) -> Operand<'strings> {
        self.op(Opcode::sensing_mousey, [])
    }

    pub fn key_is_pressed(&mut self, key: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::sensing_keypressed, [("KEY_OPTION", key.0)])
    }

    pub fn random(&mut self, from: Operand<'strings>, to: Operand<'strings>) -> Operand<'strings> {
        self.op(Opcode::operator_random, [("FROM", from.0), ("TO", to.0)])
    }

    pub fn clone_self(&mut self) {
        self.inner.fields.push(Fields::CloneSelf);
        let menu = self.insert(Block::new(Opcode::control_create_clone_of_menu));
        self.put(block::Stacking {
            opcode: Opcode::control_create_clone_of,
            inputs: Box::new([("CLONE_OPTION", Input::Prototype(menu))]),
            fields: None,
        });
    }

    fn op(
        &mut self,
        opcode: Opcode,
        inputs: impl IntoIterator<Item = (&'static str, Input<'strings>)>,
    ) -> Operand<'strings> {
        let id = self.insert(Block {
            opcode,
            parent: None.into(),
            next: None.into(),
        });
        self.add_inputs(id, inputs);
        Operand(Input::Substack(id))
    }

    fn insert(&mut self, block: Block) -> block::Id {
        let id = block::Id(self.inner.blocks.len());
        self.inner.blocks.push(block);
        id
    }

    fn set_next(&mut self, next: block::Id) {
        match self.place {
            Place::Nowhere => {}
            Place::After(block) => self.inner.blocks[block.0].parent = next.into(),
            Place::Inside { input, .. } => self.inner.inputs[input].1 = Input::Substack(next),
        }
    }

    fn add_inputs(
        &mut self,
        parent: block::Id,
        inputs: impl IntoIterator<Item = (&'static str, Input<'strings>)>,
    ) {
        self.inner.inputs.extend(inputs.into_iter().inspect(|it| {
            if let Input::Substack(it) = it.1 {
                self.inner.blocks[it.0].parent = parent.into();
            }
        }));
    }
}

pub struct InsertionPoint(Place);

#[derive(Clone, Copy)]
enum Place {
    Nowhere,
    After(block::Id),
    Inside { block: block::Id, input: usize },
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

struct CustomBlock {
    name: String,
    parameters: Range<usize>,
}

#[derive(Clone, Copy)]
pub struct CustomBlockRef(usize);

#[derive(Clone, Copy)]
struct Mutation(CustomBlockRef);

impl Mutation {
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
        for parameter in &target.parameters[block.parameters.clone()] {
            writer.write_all(match parameter.kind {
                ParameterKind::StringOrNumber => b" %s",
                ParameterKind::Boolean => b" %b",
            })?;
        }
        write!(writer, r#"","argumentids":""#)?;
        for (index, parameter) in block.parameters.clone().enumerate() {
            if index != 0 {
                write!(writer, ",")?;
            }
            write!(writer, r#"\"{parameter}\""#)?;
        }
        write!(writer, r#"","warp":true"#)?;
        if is_prototype {
            write!(writer, r#","argumentnames":"["#)?;
            for (index, parameter) in target.parameters[block.parameters.clone()]
                .iter()
                .enumerate()
            {
                if index != 0 {
                    write!(writer, ",")?;
                }
                write!(writer, r#"""#)?;
                for c in parameter.name.escape_debug().flat_map(char::escape_debug) {
                    write!(writer, "{c}")?;
                }
                write!(writer, r#"""#)?;
            }
            write!(writer, r#"]","argumentdefaults":""#)?;
            for (index, parameter) in target.parameters[block.parameters.clone()]
                .iter()
                .enumerate()
            {
                if index != 0 {
                    write!(writer, ",")?;
                }
                writer.write_all(match parameter.kind {
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
