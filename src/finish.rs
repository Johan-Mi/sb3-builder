use crate::{uid::Uid, Costume, List, Project, RealTarget, Variable};
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

        zip.start_file("project.json", zip::write::FileOptions::default())?;
        let targets = self
            .targets
            .into_iter()
            .map(RealTarget::finish)
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
    variables: HashMap<Uid, Variable>,
    lists: HashMap<Uid, List>,
    blocks: HashMap<(), ()>,
}

impl RealTarget {
    fn finish(self) -> FinishedTarget {
        let is_stage = self.name == "Stage";
        FinishedTarget {
            name: self.name,
            is_stage,
            current_costume: 0,
            costumes: self.costumes,
            sounds: &[],
            variables: self.variables,
            lists: self.lists,
            blocks: HashMap::new(),
        }
    }
}
