use serde_derive::Serialize;
use std::{
    fs,
    io::{Seek, Write},
    path::Path,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    name: String,
    data_format: String,
    asset_id: String,
    md5ext: String,
    #[serde(skip)]
    content: Vec<u8>,
}

impl Costume {
    /// Creates a [`Costume`] with the image file at the given [`Path`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the path has no extension
    /// or it fails to read the file.
    pub fn from_file(
        name: String,
        path: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
        zip: &mut zip::ZipWriter<impl Write + Seek>,
    ) -> zip::result::ZipResult<()> {
        zip.start_file(&self.md5ext, zip::write::FileOptions::default())?;
        zip.write_all(&self.content)?;
        Ok(())
    }
}
