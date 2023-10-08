use crate::page::PageID;

pub(crate) struct Freelist {
    free_pages: BTreeSet<PageID>,
    pending_pages: BTreeMap<u64, Vec<PageID>>,
}