use std::io::{self, Write as _};

impl crate::Project {
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
        write!(zip, r#"{{"meta":{{"semver":"3.0.0"}},"targets":["#)?;
        for (i, target) in self.targets.iter().enumerate() {
            if i != 0 {
                write!(zip, ",")?;
            }
            target.serialize(&mut zip)?;
        }
        write!(zip, "]}}")?;

        Ok(())
    }
}
