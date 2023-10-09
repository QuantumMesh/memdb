use std::collections::{BTreeMap, BTreeSet};
use std::ptr::NonNull;

use crate::meta::Meta;
use crate::page::PageID;

#[derive(Clone)]
pub(crate) struct Freelist {
    free_pages: BTreeSet<PageID>,
    pending_pages: BTreeMap<u64, Vec<PageID>>,
}


impl Freelist {
    pub(crate) fn new() -> Freelist {
        Freelist {
            free_pages: BTreeSet::new(),
            pending_pages: BTreeMap::new(),
        }
    }

    pub(crate) fn init(&mut self, free_pages: &[PageID]) {
        free_pages.iter().for_each(|id| {
            self.free_pages.insert(*id);
        });
    }
}

pub(crate) struct TxFreelist {
    pub(crate) meta: Meta,
    pub(crate) inner: Freelist,
    pub(crate) pages: BTreeMap<u64, (NonNull<u8>, usize)>,
    // pub(crate) arena: Bump,
}