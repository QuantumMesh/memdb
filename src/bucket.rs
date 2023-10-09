use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::bytes::Bytes;
use crate::page::{PageID, Pages};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct BucketMeta {
    pub(crate) root_page: PageID,
    pub(crate) next_int: u64,
}

pub(crate) struct InnerBucket<'b> {
    pub(crate) meta: BucketMeta,
    // root: PageNodeID,
    pub(crate) deleted: bool,
    dirty: bool,
    buckets: HashMap<Bytes<'b>, Rc<RefCell<InnerBucket<'b>>>>,
    // pub(crate) nodes: Vec<Rc<RefCell<Node<'b>>>>,
    // Maps a PageID to it's NodeID, so we don't create multiple nodes for a single page
    // page_node_ids: HashMap<PageID, NodeID>,
    // Maps PageIDs to their parent's PageID
    page_parents: HashMap<PageID, PageID>,
    pages: Pages,
}