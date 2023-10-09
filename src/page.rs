use std::slice::from_raw_parts;
use std::sync::Arc;

use memmap2::Mmap;

use crate::meta::Meta;

pub(crate) type PageID = u64;

pub(crate) type PageType = u8;

#[derive(Clone)]
pub(crate) struct Pages {
    pub(crate) data: Arc<Mmap>,
    pub(crate) pagesize: u64,
}

impl Pages {
    pub fn new(data: Arc<Mmap>, pagesize: u64) -> Pages {
        Pages { data, pagesize }
    }

    #[inline]
    pub fn page<'a>(&self, id: PageID) -> &'a Page {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &*(&self.data[(id * self.pagesize) as usize] as *const u8 as *const Page)
        }
    }
}

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

    pub(crate) fn meta(&self) -> &Meta {
        assert_eq!(self.page_type, Page::TYPE_META);
        unsafe { &*(&self.ptr as *const u64 as *const Meta) }
    }

    pub(crate) fn meta_mut(&mut self) -> &mut Meta {
        assert_eq!(self.page_type, Page::TYPE_META);
        unsafe { &mut *(&mut self.ptr as *mut u64 as *mut Meta) }
    }

    #[inline]
    pub(crate) fn from_buf(buf: &[u8], id: PageID, pagesize: u64) -> &Page {
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            &*(&buf[(id * pagesize) as usize] as *const u8 as *const Page)
        }
    }


    pub(crate) fn freelist(&self) -> &[PageID] {
        assert_eq!(self.page_type, Page::TYPE_FREELIST);
        let start = &self.ptr as *const u64 as *const PageID;
        unsafe {
            from_raw_parts(start, self.count as usize)
        }
    }
}

mod tests {
    use crate::sys::sys_limits;

    #[test]
    fn test_new_page() {
        dbg!(sys_limits::get_memory_limit());
    }
}