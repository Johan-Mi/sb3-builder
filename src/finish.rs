use crate::{Project, RealTarget};
use serde::ser::SerializeStruct;
use std::io;

impl Project {
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
        let mut zip = zip::ZipWriter::new(writer);

        for costume in self.targets.iter().flat_map(|target| &target.costumes) {
            costume.add_to_archive(&mut zip)?;
        }

        zip.start_file("project.json", zip::write::FileOptions::default())?;
        let targets = &self.targets;
        serde_json::to_writer(zip, &FinishedProject { targets })?;

        Ok(())
    }
}

struct FinishedProject<'a> {
    targets: &'a [RealTarget],
}

impl serde::Serialize for FinishedProject<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("FinishedProject", 2)?;
        s.serialize_field("meta", &Meta)?;
        s.serialize_field("targets", &self.targets)?;
        s.end()
    }
}

struct Meta;

impl serde::Serialize for Meta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Meta", 1)?;
        s.serialize_field("semver", "3.0.0")?;
        s.end()
    }
}
