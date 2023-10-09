use std::io::Write;

use bytes::{BufMut, Bytes, BytesMut};
use sha3::{Digest, Sha3_256};

use crate::bucket::BucketMeta;
use crate::page::PageID;

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct Meta {
    pub(crate) meta_page: u32,
    pub(crate) integrity_code: u32,
    pub(crate) version: u32,
    pub(crate) pagesize: u64,
    pub(crate) root: BucketMeta,
    pub(crate) num_pages: PageID,
    pub(crate) freelist_page: PageID,
    pub(crate) tx_id: u64,
    pub(crate) hash: [u8; 32],
}

impl Meta {
    pub(crate) fn hash_self(&self) -> [u8; 32] {
        let mut hash_result: [u8; 32] = [0; 32];
        let mut hasher = Sha3_256::new();
        hasher.update(self.bytes());
        let hash = hasher.finalize();
        assert_eq!(hash.len(), 32);
        hash_result.copy_from_slice(&hash[..]);
        hash_result
    }

    fn bytes(&self) -> Bytes {
        let buf = BytesMut::new();
        let mut w = buf.writer();
        let _ = w.write(&self.meta_page.to_be_bytes());
        let _ = w.write(&self.integrity_code.to_be_bytes());
        let _ = w.write(&self.version.to_be_bytes());
        let _ = w.write(&self.pagesize.to_be_bytes());
        let _ = w.write(&self.root.root_page.to_be_bytes());
        let _ = w.write(&self.root.next_int.to_be_bytes());
        let _ = w.write(&self.num_pages.to_be_bytes());
        let _ = w.write(&self.freelist_page.to_be_bytes());
        let _ = w.write(&self.tx_id.to_be_bytes());

        w.into_inner().freeze()
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_meta_hash() {
        let meta = Meta {
            meta_page: 0,
            integrity_code: 0,
            version: 0,
            pagesize: 0,
            root: BucketMeta {
                root_page: 0,
                next_int: 0,
            },
            num_pages: 0,
            freelist_page: 0,
            tx_id: 0,
            hash: [0; 32],
        };
        let hash = meta.hash_self();
        dbg!(hash);
    }
}