use crate::{Costume, Project, Target};
use serde::Serialize;
use std::{collections::HashMap, io};

impl Project {
    pub fn finish(
        self,
        writer: impl io::Write + io::Seek,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut zip = zip::ZipWriter::new(writer);

        for costume in self.targets.iter().flat_map(|target| &target.costumes) {
            costume.add_to_archive(&mut zip)?;
        }

        zip.start_file("project.json", Default::default())?;
        let targets = self
            .targets
            .into_iter()
            .map(Target::finish)
            .collect::<Box<_>>();
        let finished_project = FinishedProject {
            meta: Meta { semver: "3.0.0" },
            targets: &targets,
        };
        serde_json::to_writer(zip, &finished_project)?;

        Ok(())
    }
}

#[derive(Serialize)]
struct FinishedProject<'a> {
    meta: Meta,
    targets: &'a [FinishedTarget],
}

#[derive(Serialize)]
struct Meta {
    semver: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FinishedTarget {
    name: String,
    is_stage: bool,
    current_costume: usize,
    costumes: Vec<Costume>,
    sounds: &'static [()],
    variables: HashMap<(), ()>,
    lists: HashMap<(), ()>,
    blocks: HashMap<(), ()>,
}

impl Target {
    fn finish(self) -> FinishedTarget {
        let is_stage = self.name == "Stage";
        FinishedTarget {
            name: self.name,
            is_stage,
            current_costume: 0,
            costumes: self.costumes,
            sounds: &[],
            variables: HashMap::new(),
            lists: HashMap::new(),
            blocks: HashMap::new(),
        }
    }
}
