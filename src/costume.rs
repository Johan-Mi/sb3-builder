use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub struct Costume {
    name: String,
    data_format: String,
    asset_id: String,
    md5ext: String,
    content: Vec<u8>,
}

impl Costume {
    pub(crate) fn serialize(&self, writer: &mut dyn io::Write) -> io::Result<()> {
        write!(
            writer,
            r#"{{"name":{:?},"dataFormat":{:?},"assetId":{:?},"md5ext":{:?}}}"#,
            self.name, self.data_format, self.asset_id, self.md5ext
        )
    }
}

impl Costume {
    /// Creates a [`Costume`] with the image file at the given [`Path`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the path has no extension
    /// or it fails to read the file.
    pub fn from_file(name: String, path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data_format = path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or("costume path must have an extension")?
            .to_owned();
        let content = fs::read(path)?;
        let digest = md5::compute(&content);
        let asset_id = format!("{digest:?}");
        let md5ext = format!("{asset_id}.{data_format}");
        Ok(Self {
            name,
            data_format,
            asset_id,
            md5ext,
            content,
        })
    }

    pub(crate) fn add_to_archive(
        &self,
        archive: &mut rawzip::ZipArchiveWriter<impl io::Write>,
    ) -> Result<(), rawzip::Error> {
        let (mut entry, config) = archive
            .new_file(&self.md5ext)
            .compression_method(rawzip::CompressionMethod::Deflate)
            .start()?;
        let encoder =
            flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
        let mut file = config.wrap(encoder);
        file.write_all(&self.content)?;
        let (_, descriptor) = file.finish()?;
        let _: u64 = entry.finish(descriptor)?;
        Ok(())
    }
}
