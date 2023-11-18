#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::unwrap_used)]

pub mod block;
mod costume;
mod finish;
mod operand;
mod uid;

pub use costume::Costume;
pub use operand::Operand;

use block::Input;
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
            parent: None,
            place: Place::Next,
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
    sounds: &'static [()],
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
            sounds: &[],
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
        }
    }
}

pub struct Target<'a> {
    inner: &'a mut RealTarget,
    builder: &'a mut Builder,
    parent: Option<Uid>,
    place: Place,
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

    pub fn start_script(&mut self, hat: block::Hat) {
        let id = self.builder.uid_generator.new_uid();
        self.inner.blocks.insert(id, hat.into());
        self.parent = Some(id);
        self.place = Place::Next;
    }

    pub fn put(&mut self, block: block::Stacking) {
        let mut block = block::Block::from(block);
        block.parent = self.parent;
        let id = self.builder.uid_generator.new_uid();
        self.inner.blocks.insert(id, block);
        self.set_next(id);
        self.parent = Some(id);
        self.place = Place::Next;
    }

    pub fn forever(&mut self) {
        self.put(block::Stacking {
            opcode: "control_forever",
            inputs: None,
        });
        self.place = Place::Substack1;
    }

    fn set_next(&mut self, next: Uid) {
        let Some(parent) = self.parent else { return };
        let parent = self
            .inner
            .blocks
            .get_mut(&parent)
            .expect("parent block does not exist");
        match self.place {
            Place::Next => parent.next = Some(next),
            Place::Substack1 => {
                parent
                    .inputs
                    .get_or_insert_with(HashMap::default)
                    .insert("SUBSTACK", Input::Substack(next));
            }
        }
    }
}

enum Place {
    Next,
    Substack1,
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
