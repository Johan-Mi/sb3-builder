use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub struct Costume<'strings> {
    name: &'strings str,
    data_format: Box<str>,
    digest: md5::Digest,
    content: Vec<u8>,
}

impl Costume<'_> {
    pub(crate) fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(
            writer,
            r#"{{"name":{:?},"dataFormat":{:?},"assetId":"{:x}","md5ext":"{:x}.{}"}}"#,
            self.name,
            self.data_format,
            self.digest,
            self.digest,
            self.data_format.escape_debug()
        )
    }
}

impl<'strings> Costume<'strings> {
    /// Creates a [`Costume`] with the image file at the given [`Path`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the path has no extension
    /// or it fails to read the file.
    pub fn from_file(name: &'strings str, path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data_format = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or("costume path must have an extension")?
            .into();
        let content = fs::read(path)?;
        let digest = md5::compute(&content);
        Ok(Self {
            name,
            data_format,
            digest,
            content,
        })
    }

    pub(crate) fn add_to_archive(
        &self,
        archive: &mut rawzip::ZipArchiveWriter<impl io::Write>,
    ) -> Result<(), rawzip::Error> {
        let file_name = format!("{:x}.{}", self.digest, self.data_format);
        let (mut entry, config) = archive
            .new_file(&file_name)
            .compression_method(rawzip::CompressionMethod::Store)
            .start()?;
        let mut file = config.wrap(&mut entry);
        file.write_all(&self.content)?;
        let (_, descriptor) = file.finish()?;
        let _: u64 = entry.finish(descriptor)?;
        Ok(())
    }
}
