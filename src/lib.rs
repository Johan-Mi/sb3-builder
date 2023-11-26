#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::unwrap_used)]

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
        self.inner.variables.insert(id, variable);
        VariableRef(id)
    }

    pub fn get_variable(&mut self, variable: VariableRef) -> Operand {
        let id = variable.0;
        let name = self.inner.variables[&id].name.clone();
        Operand(Input::Variable { name, id })
    }

    pub fn add_list(&mut self, list: List) -> ListRef {
        let id = self.builder.uid_generator.new_uid();
        self.inner.lists.insert(id, list);
        ListRef(id)
    }

    pub fn insert_at(&mut self, point: InsertionPoint) -> InsertionPoint {
        std::mem::replace(&mut self.point, point)
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
        self.put_block(block.into());
    }

    fn put_block(&mut self, mut block: Block) {
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
            opcode: "control_forever",
            inputs: None,
        });
        self.point.place = Place::Substack1;
    }

    pub fn repeat(&mut self, times: Operand) -> InsertionPoint {
        self.put(block::Stacking {
            opcode: "control_repeat",
            inputs: Some([("TIMES", times.0)].into()),
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
        let id = variable.0;
        let name = self.inner.variables[&id].name.clone();
        self.put_block(Block {
            opcode: "control_for_each",
            parent: None,
            next: None,
            inputs: Some([("VALUE", times.0)].into()),
            fields: Some(Fields::Variable { name, id }),
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
        });
        self.insert_at(InsertionPoint {
            parent: self.point.parent,
            place: Place::Substack1,
        })
    }

    pub fn set_variable(&mut self, variable: VariableRef, to: Operand) {
        let id = variable.0;
        let name = self.inner.variables[&id].name.clone();
        self.put_block(Block {
            opcode: "data_setvariableto",
            parent: None,
            next: None,
            inputs: Some([("VALUE", to.0)].into()),
            fields: Some(Fields::Variable { name, id }),
        });
    }

    pub fn change_variable(&mut self, variable: VariableRef, by: Operand) {
        let id = variable.0;
        let name = self.inner.variables[&id].name.clone();
        self.put_block(Block {
            opcode: "data_changevariableby",
            parent: None,
            next: None,
            inputs: Some([("VALUE", by.0)].into()),
            fields: Some(Fields::Variable { name, id }),
        });
    }

    pub fn eq(&mut self, lhs: Operand, rhs: Operand) -> Operand {
        self.op(Block {
            opcode: "operator_equals",
            parent: None,
            next: None,
            inputs: Some([("OPERAND1", lhs.0), ("OPERAND2", rhs.0)].into()),
            fields: None,
        })
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

#[derive(Clone, Copy)]
pub struct VariableRef(Uid);

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

pub struct ListRef(Uid);
