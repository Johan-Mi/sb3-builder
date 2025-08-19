use serde::ser::SerializeStruct;
use std::{
    fs,
    io::{Seek, Write},
    path::Path,
};

pub struct Costume {
    name: String,
    data_format: String,
    asset_id: String,
    md5ext: String,
    content: Vec<u8>,
}

impl serde::Serialize for Costume {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Costume", 4)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("dataFormat", &self.data_format)?;
        s.serialize_field("assetId", &self.asset_id)?;
        s.serialize_field("md5ext", &self.md5ext)?;
        s.end()
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
        zip: &mut zip::ZipWriter<impl Write + Seek>,
    ) -> zip::result::ZipResult<()> {
        zip.start_file(&self.md5ext, zip::write::FileOptions::default())?;
        zip.write_all(&self.content)?;
        Ok(())
    }
}
