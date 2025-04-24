use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
use flexstr::SharedStr;
use log::{debug, warn};
use sha2::Digest;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub(crate) struct MediaRegistry {
    media: HashMap<SharedStr, MediaFile>,
}

impl MediaRegistry {
    pub(crate) fn load_directory(&mut self, path: impl AsRef<Path>) -> Result<()> {
        for entry in path.as_ref().read_dir()? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let entry_path = entry.path();
            if file_type.is_dir() {
                debug!("skipping subdirectory {}", entry_path.display());
                continue;
            }
            if file_type.is_symlink() {
                debug!(
                    "skipping symlink {} for security reasons",
                    entry_path.display()
                );
                continue;
            }
            #[expect(
                clippy::filetype_is_file,
                reason = "this is ok as we already check for all other flags"
            )]
            if !file_type.is_file() {
                debug!("skipping non-file {}", entry_path.display());
                continue;
            }

            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                debug!(
                    "Skipping file with non-UTF-8 name: '{}'",
                    entry_path.display()
                );
                continue;
            };

            if !file_name
                .chars()
                .all(|char| char.is_ascii_alphanumeric() || ['.', '_'].contains(&char))
            {
                debug!(
                    "Skipping file with illegal characters in its name: '{}'",
                    entry_path.display()
                );
                continue;
            }

            if let Some(duplicate) = self
                .media
                .insert(file_name.into(), MediaFile { path: entry.path() })
            {
                warn!(
                    "the media file {} was overloaded by {}",
                    duplicate.path.display(),
                    entry_path.display()
                );
            } else {
                debug!("added {} to the media library", entry_path.display());
            }
        }

        Ok(())
    }

    pub(crate) fn hashes(&self) -> impl Iterator<Item = (&SharedStr, String)> {
        let hash_base64 = |path| {
            let mut hasher = sha1::Sha1::new();
            hasher.update(fs::read(path).unwrap());
            let hash = hasher.finalize();
            STANDARD.encode(hash)
        };

        self.media
            .iter()
            .map(move |(name, file)| (name, hash_base64(&file.path)))
    }
}

struct MediaFile {
    path: PathBuf,
}
