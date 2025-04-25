//! Contains `ContentIdMap`

use std::{borrow::Borrow, collections::HashMap, hash::Hash, ops::Index};

use anyhow::{Result, bail};
use flexstr::SharedStr;
use luanti_core::ContentId;

/// Translates between numeric content ids and their names
pub struct ContentIdMap {
    to_name: Vec<SharedStr>,
    to_id: HashMap<NameKey, ContentId>,
}

impl ContentIdMap {
    const EMPTY: &SharedStr = &SharedStr::EMPTY;

    /// Create a new id map containing the default mappings for `UNKNOWN`, `AIR` and `IGNORE`.
    /// This is what you normally need.
    #[expect(
        clippy::new_without_default,
        reason = "a Default implementation is expected to create something less complex"
    )]
    #[must_use]
    pub fn new() -> Self {
        let mut result = Self::empty();

        result.insert(ContentId::UNKNOWN, SharedStr::from_static("unknown"));
        result.insert(ContentId::AIR, SharedStr::from_static("air"));
        result.insert(ContentId::IGNORE, SharedStr::from_static("ignore"));

        result
    }

    fn empty() -> Self {
        Self {
            to_name: Vec::with_capacity(128),
            to_id: HashMap::with_capacity(128),
        }
    }

    pub(crate) fn insert(&mut self, id: ContentId, name: SharedStr) {
        self.insert_to_id(id, name.clone());
        self.insert_to_name(id, name);
    }

    /// Add a new entry to this map and automatically assign a new id.
    /// Return the assigned content id.
    ///
    /// # Errors
    ///
    /// This function will return an error if there's no space left.
    pub fn push(&mut self, name: SharedStr) -> Result<ContentId> {
        let Some(id) = self.find_free_id() else {
            bail!("cannot create more content ids");
        };
        self.insert(id, name);
        Ok(id)
    }

    fn find_free_id(&self) -> Option<ContentId> {
        self.to_name
            .iter()
            .position(SharedStr::is_empty)
            .unwrap_or(self.to_name.len())
            .try_into()
            .ok()
    }

    fn insert_to_id(&mut self, id: ContentId, name: SharedStr) {
        self.to_id.insert(NameKey(name), id);
    }

    fn insert_to_name(&mut self, id: ContentId, name: SharedStr) {
        if let Some(entry) = self.to_name.get_mut(usize::from(id)) {
            *entry = name;
        } else {
            self.to_name
                .resize(usize::from(id).saturating_sub(1), SharedStr::EMPTY);
            self.to_name.push(name);
        }
    }
}

impl Index<ContentId> for ContentIdMap {
    type Output = SharedStr;

    fn index(&self, index: ContentId) -> &Self::Output {
        self.to_name.get(usize::from(index)).unwrap_or(Self::EMPTY)
    }
}

impl Index<&str> for ContentIdMap {
    type Output = ContentId;

    fn index(&self, index: &str) -> &Self::Output {
        &self[index.as_bytes()]
    }
}

impl Index<&[u8]> for ContentIdMap {
    type Output = ContentId;

    fn index(&self, index: &[u8]) -> &Self::Output {
        self.to_id.get(index).unwrap_or(&ContentId::UNKNOWN)
    }
}

/// This new-type permits using a regular (cheap and displayable) `SharedStr` while still being able to look up keys
/// by `&[u8]` which is necessary because some storage providers contain these.
#[derive(Eq, PartialEq)]
struct NameKey(SharedStr);

impl Hash for NameKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state);
    }
}

impl Borrow<[u8]> for NameKey {
    fn borrow(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
