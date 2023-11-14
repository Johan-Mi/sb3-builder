#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::unwrap_used)]

mod costume;
mod finish;
mod uid;

pub use costume::Costume;

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
        let stage = RealTarget::new("Stage".to_owned());
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
        self.targets.push(RealTarget::new(name));
        self.target(self.targets.len() - 1)
    }

    fn target(&mut self, index: usize) -> Target {
        Target {
            inner: &mut self.targets[index],
            builder: &mut self.builder,
        }
    }
}

struct Builder {
    uid_generator: uid::Generator,
}

struct RealTarget {
    name: String,
    costumes: Vec<Costume>,
    variables: HashMap<Uid, Variable>,
    lists: HashMap<Uid, List>,
}

impl RealTarget {
    fn new(name: String) -> Self {
        Self {
            name,
            costumes: Vec::new(),
            variables: HashMap::new(),
            lists: HashMap::new(),
        }
    }
}

pub struct Target<'a> {
    inner: &'a mut RealTarget,
    builder: &'a mut Builder,
}

impl Target<'_> {
    pub fn add_costume(&mut self, costume: Costume) {
        self.inner.costumes.push(costume);
    }

    pub fn add_variable(&mut self, variable: Variable) {
        let id = self.builder.uid_generator.new_uid();
        self.inner.variables.insert(id, variable);
    }

    pub fn add_list(&mut self, list: List) {
        let id = self.builder.uid_generator.new_uid();
        self.inner.lists.insert(id, list);
    }
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
