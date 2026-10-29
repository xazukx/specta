use std::{borrow::Cow, collections::HashMap};

use crate::{Error, ErrorKind, varint};

/// Interns object keys during serialization. Ids are assigned in first-use order.
#[derive(Debug, Default)]
pub(crate) struct KeyTable {
    ids: HashMap<Cow<'static, str>, u32>,
}

impl KeyTable {
    /// Look up `key`, inserting it with the next free id if it is new.
    /// `owned` is only called on a miss, so `&'static str` field names are never copied.
    pub(crate) fn intern(
        &mut self,
        key: &str,
        owned: impl FnOnce() -> Cow<'static, str>,
    ) -> Result<u32, Error> {
        if let Some(&id) = self.ids.get(key) {
            return Ok(id);
        }
        // Ids stay below `u32::MAX` so the key count also fits in a `u32`.
        let id = u32::try_from(self.ids.len())
            .ok()
            .filter(|&id| id < u32::MAX)
            .ok_or(ErrorKind::TooManyKeys)?;
        self.ids.insert(owned(), id);
        Ok(id)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// The key with the given id. Only used to build error paths, so a linear scan is fine.
    pub(crate) fn get(&self, id: u32) -> Option<&str> {
        self.ids
            .iter()
            .find_map(|(key, &key_id)| (key_id == id).then_some(key.as_ref()))
    }

    /// Order the keys by id, returning them with the encoded size of the table.
    pub(crate) fn into_ordered(self) -> (Vec<Cow<'static, str>>, usize) {
        let mut keys = self.ids.into_iter().collect::<Vec<_>>();
        keys.sort_unstable_by_key(|&(_, id)| id);
        let keys = keys.into_iter().map(|(key, _)| key).collect::<Vec<_>>();
        let size = varint::encode(keys.len() as u32).1
            + keys
                .iter()
                .map(|key| varint::encode(key.len() as u32).1 + key.len())
                .sum::<usize>();
        (keys, size)
    }
}
