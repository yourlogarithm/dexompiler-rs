
use std::{path::Path, fs, io::Read};
use dex::DexReader;
use zip::ZipArchive;
use serde::{Serialize, Deserialize};

use crate::{dex_parsing::DexParseModel, manifest_parsing::ManifestParseModel, utils::Error};


#[derive(Debug, Serialize, Deserialize)]
pub struct ApkParseModel {
    dex: DexParseModel,
    manifest: ManifestParseModel
}


impl ApkParseModel {
    pub fn try_from_path(path: &str, dex_sequence_cap: usize) -> Result<Self, Error> {
        let file = fs::File::open(Path::new(path))?;
        let mut zip_handler = ZipArchive::new(file)?; 

        let mut dexes = vec![];
        let mut manifest = None;

        for i in 0..zip_handler.len() {
            let (file_name, contents) = {
                let mut current_file = match zip_handler.by_index(i) {
                    Ok(file) => file,
                    _ => continue
                };
                let mut contents = Vec::new();
                if let Ok(_) = current_file.read_to_end(&mut contents) {
                    let is_xml = current_file.name().to_string();
                    (is_xml, contents)
                } else {
                    continue;
                }
            };

            if file_name == "AndroidManifest.xml" {
                manifest = Some(ManifestParseModel::try_from_bytes(&contents)?);
            } else if contents.starts_with(&[100, 101, 120, 10]) {
                if let Ok(dex) = DexReader::from_vec(contents) {
                    dexes.push(dex);
                }
            }
        }

        Ok(
            Self {
                dex: DexParseModel::try_from_dexes(dexes, dex_sequence_cap)?,
                manifest: manifest.unwrap_or_default()
            }
        )
    }
}