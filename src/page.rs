pub(crate) type PageID = u64;

pub(crate) type PageType = u8;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct Page {
    // id * pagesize is the offset from the beginning of the file
    pub(crate) id: PageID,
    pub(crate) page_type: PageType,
    // Number of elements on this page, the type of element depends on the pageType
    pub(crate) count: u64,
    // Number of additional pages after this one that are part of this block
    pub(crate) overflow: u64,
    // ptr serves as a reference to where the actual data starts
    pub(crate) ptr: u64,
}

impl Page {
    pub(crate) const TYPE_BRANCH: PageType = 0x01;
    pub(crate) const TYPE_LEAF: PageType = 0x02;
    pub(crate) const TYPE_META: PageType = 0x03;
    pub(crate) const TYPE_FREELIST: PageType = 0x04;
}