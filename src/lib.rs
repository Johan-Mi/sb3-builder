#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::unwrap_used)]

mod costume;
mod finish;

pub use costume::Costume;

pub struct Project {
    targets: Vec<Target>,
}

impl Default for Project {
    fn default() -> Self {
        let stage = Target {
            name: "Stage".to_owned(),
            costumes: Vec::new(),
        };
        Self {
            targets: vec![stage],
        }
    }
}

impl Project {
    pub fn stage(&mut self) -> &mut Target {
        &mut self.targets[0]
    }
}

pub struct Target {
    name: String,
    costumes: Vec<Costume>,
}

impl Target {
    pub fn add_costume(&mut self, costume: Costume) {
        self.costumes.push(costume);
    }
}
