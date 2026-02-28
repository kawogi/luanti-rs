//! Contains `MediaRegistry`

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

/// Contains a list of media files and provides access to them
#[derive(Default)]
pub struct MediaRegistry {
    media: HashMap<SharedStr, MediaFile>,
}

impl MediaRegistry {
    /// # Errors
    ///
    /// Returns an error if the given directory could not be read.
    pub fn load_directory(&mut self, path: impl AsRef<Path>) -> Result<()> {
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

            // TODO switch to camino for UTF-8-only
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

            if let Some(duplicate) = self.media.insert(
                file_name.to_owned().into(),
                MediaFile { path: entry.path() },
            ) {
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
            #[expect(
                clippy::unwrap_used,
                reason = "// TODO(kawogi) the computation of hashes should be done at load time so that it cannot fail at run-time; also improves performance"
            )]
            hasher.update(fs::read(path).unwrap());
            let hash = hasher.finalize();
            STANDARD.encode(hash)
        };

        self.media
            .iter()
            .map(move |(name, file)| (name, hash_base64(&file.path)))
    }

    pub(crate) fn file_content(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let Some(file) = self.media.get(key) else {
            return Ok(None);
        };
        Ok(Some(fs::read(&file.path)?))
    }
}

struct MediaFile {
    path: PathBuf,
}
