use std::{borrow::Cow, collections::HashMap};

use crate::varint;

/// Interns the strings of the key table or the value table during serialization.
/// Ids are assigned in first-use order.
#[derive(Debug, Default)]
pub(crate) struct StringTable {
    ids: HashMap<Cow<'static, str>, u32>,
}

impl StringTable {
    /// Look up `s`, inserting it with the next free id if it is new. `None` if the table is full.
    /// `owned` is only called on a miss, so `&'static str` names are never copied.
    pub(crate) fn intern(
        &mut self,
        s: &str,
        owned: impl FnOnce() -> Cow<'static, str>,
    ) -> Option<u32> {
        if let Some(&id) = self.ids.get(s) {
            return Some(id);
        }
        // Ids stay below `u32::MAX` so the count also fits in a `u32`.
        let id = u32::try_from(self.ids.len())
            .ok()
            .filter(|&id| id < u32::MAX)?;
        self.ids.insert(owned(), id);
        Some(id)
    }

    /// The string with the given id. Only used to build error paths, so a linear scan is fine.
    pub(crate) fn get(&self, id: u32) -> Option<&str> {
        self.ids
            .iter()
            .find_map(|(s, &s_id)| (s_id == id).then_some(s.as_ref()))
    }

    /// The strings ordered by id, or `None` if there are none, since empty tables are never written.
    pub(crate) fn into_ordered(self) -> Option<Vec<Cow<'static, str>>> {
        if self.ids.is_empty() {
            return None;
        }
        let mut strings = self.ids.into_iter().collect::<Vec<_>>();
        strings.sort_unstable_by_key(|&(_, id)| id);
        Some(strings.into_iter().map(|(s, _)| s).collect())
    }
}

/// The size of a table on the wire: the count, then each string's length and bytes.
pub(crate) fn encoded_len(strings: &[Cow<'_, str>]) -> usize {
    varint::encode(strings.len() as u32).1
        + strings
            .iter()
            .map(|s| varint::encode(s.len() as u32).1 + s.len())
            .sum::<usize>()
}

/// Write a table. Counts and lengths were checked to fit in a `u32` when the strings were interned.
pub(crate) fn write(doc: &mut Vec<u8>, strings: &[Cow<'_, str>]) {
    varint::write(doc, strings.len() as u32);
    for s in strings {
        varint::write(doc, s.len() as u32);
        doc.extend_from_slice(s.as_bytes());
    }
}
