use crate::{Project, RealTarget};
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
        serialize(&self.targets, &mut zip)?;

        Ok(())
    }
}

fn serialize(targets: &[RealTarget], writer: &mut dyn io::Write) -> io::Result<()> {
    write!(writer, r#"{{"meta":{{"semver":"3.0.0"}},"targets":["#)?;
    for (i, target) in targets.iter().enumerate() {
        if i != 0 {
            write!(writer, ",")?;
        }
        target.serialize(writer)?;
    }
    write!(writer, "]}}")
}
