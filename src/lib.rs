#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::unwrap_used)]

mod costume;
mod finish;
mod uid;

pub use costume::Costume;

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
        let stage = RealTarget {
            name: "Stage".to_owned(),
            costumes: Vec::new(),
            variables: Vec::new(),
            lists: Vec::new(),
        };
        Self {
            builder,
            targets: vec![stage],
        }
    }
}

impl Project {
    pub fn stage(&mut self) -> Target {
        Target {
            inner: &mut self.targets[0],
            builder: &mut self.builder,
        }
    }

    pub fn add_sprite(&mut self, name: String) -> Target {
        let target = RealTarget {
            name,
            costumes: Vec::new(),
            variables: Vec::new(),
            lists: Vec::new(),
        };
        self.targets.push(target);
        Target {
            inner: self.targets.last_mut().unwrap_or_else(|| unreachable!()),
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
    variables: Vec<(Variable, Uid)>,
    lists: Vec<(List, Uid)>,
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
        self.inner.variables.push((variable, id));
    }

    pub fn add_list(&mut self, list: List) {
        let id = self.builder.uid_generator.new_uid();
        self.inner.lists.push((list, id));
    }
}

pub struct Variable {
    pub name: String,
}

pub struct List {
    pub name: String,
}
