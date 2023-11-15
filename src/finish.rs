use crate::{Project, RealTarget};
use serde::Serialize;
use std::io;

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
        let finished_project = FinishedProject {
            meta: Meta { semver: "3.0.0" },
            targets: &self.targets,
        };
        serde_json::to_writer(zip, &finished_project)?;

        Ok(())
    }
}

#[derive(Serialize)]
struct FinishedProject<'a> {
    meta: Meta,
    targets: &'a [RealTarget],
}

#[derive(Serialize)]
struct Meta {
    semver: &'static str,
}
