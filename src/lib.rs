#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::cargo, clippy::unwrap_used)]

pub mod block;
mod costume;
mod finish;
mod operand;
mod uid;

pub use costume::Costume;
pub use operand::Operand;

use block::{Block, Fields, Input};
use serde::Serialize;
use std::collections::HashMap;
use uid::Uid;

pub struct Project {
    builder: Builder,
    targets: Vec<RealTarget>,
}

impl Default for Project {
    fn default() -> Self {
        let builder = Builder {
            uid_generator: uid::Generator::default(),
        };
        let stage = RealTarget::new("Stage".to_owned(), true);
        Self {
            builder,
            targets: vec![stage],
        }
    }
}

impl Project {
    pub fn stage(&mut self) -> Target {
        self.target(0)
    }

    pub fn add_sprite(&mut self, name: String) -> Target {
        self.targets.push(RealTarget::new(name, false));
        self.target(self.targets.len() - 1)
    }

    fn target(&mut self, index: usize) -> Target {
        Target {
            inner: &mut self.targets[index],
            builder: &mut self.builder,
            point: InsertionPoint {
                parent: None,
                place: Place::Next,
            },
        }
    }
}

struct Builder {
    uid_generator: uid::Generator,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RealTarget {
    name: String,
    is_stage: bool,
    current_costume: usize,
    costumes: Vec<Costume>,
    sounds: [(); 0],
    variables: HashMap<Uid, Variable>,
    lists: HashMap<Uid, List>,
    blocks: HashMap<Uid, block::Block>,
}

impl RealTarget {
    fn new(name: String, is_stage: bool) -> Self {
        Self {
            name,
            is_stage,
            current_costume: 0,
            costumes: Vec::new(),
            sounds: [],
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
        }
    }
}

pub struct Target<'a> {
    inner: &'a mut RealTarget,
    builder: &'a mut Builder,
    point: InsertionPoint,
}

impl Target<'_> {
    pub fn add_costume(&mut self, costume: Costume) {
        self.inner.costumes.push(costume);
    }

    pub fn add_variable(&mut self, variable: Variable) -> VariableRef {
        let id = self.builder.uid_generator.new_uid();
        let name = variable.name.clone();
        self.inner.variables.insert(id, variable);
        VariableRef { name, id }
    }

    pub fn add_list(&mut self, list: List) -> ListRef {
        let id = self.builder.uid_generator.new_uid();
        let name = list.name.clone();
        self.inner.lists.insert(id, list);
        ListRef { name, id }
    }

    pub fn insert_at(&mut self, point: InsertionPoint) -> InsertionPoint {
        std::mem::replace(&mut self.point, point)
    }

    pub fn add_custom_block(
        &mut self,
        name: String,
        parameters: Vec<Parameter>,
    ) -> (CustomBlockRef, InsertionPoint) {
        todo!()
    }

    pub fn use_custom_block(
        &mut self,
        block: CustomBlockRef,
        arguments: &[Operand],
    ) {
        todo!()
    }

    pub fn start_script(&mut self, hat: block::Hat) {
        let id = self.builder.uid_generator.new_uid();
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

    pub fn for_(
        &mut self,
        variable: VariableRef,
        times: Operand,
    ) -> InsertionPoint {
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
        })
    }

    pub fn add(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_add",
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn sub(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_subtract",
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn mul(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_multiply",
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn div(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_divide",
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn modulo(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_mod",
            [("NUM1", lhs.0), ("NUM2", rhs.0)],
        )
    }

    pub fn eq(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.binary_operation(
            "operator_equals",
            [("OPERAND1", lhs.0), ("OPERAND2", rhs.0)],
        )
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

    fn op(&mut self, block: Block) -> Operand {
        Operand(Input::Substack(self.insert(block)))
    }

    fn insert(&mut self, block: Block) -> Uid {
        let id = self.builder.uid_generator.new_uid();
        if let Some(inputs) = &block.inputs {
            for input in inputs.values() {
                if let Input::Substack(operand_block_id) = input {
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

pub struct Variable {
    pub name: String,
}

impl Serialize for Variable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&*self.name, 0.0).serialize(serializer)
    }
}

#[derive(Clone)]
pub struct VariableRef {
    name: String,
    id: Uid,
}

pub struct List {
    pub name: String,
}

impl Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&*self.name, [(); 0]).serialize(serializer)
    }
}

#[derive(Clone)]
pub struct ListRef {
    name: String,
    id: Uid,
}

pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,
}

pub enum ParameterKind {
    StringOrNumber,
    Boolean,
}

#[derive(Clone, Copy)]
pub struct CustomBlockRef {}
