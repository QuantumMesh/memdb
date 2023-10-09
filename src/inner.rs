use std::fs::File;
use std::sync::Arc;

use fs2::FileExt;
use memmap2::Mmap;
use parking_lot::{Mutex, RwLock};

use crate::config::DBFlags;
use crate::errors::Result;
use crate::freelist::Freelist;
use crate::meta::Meta;
use crate::page::Page;

pub(crate) struct Inner {
    pub(crate) data: Mutex<Arc<Mmap>>,
    pub(crate) mmap_lock: RwLock<()>,
    pub(crate) freelist: Mutex<Freelist>,
    pub(crate) file: Mutex<File>,
    pub(crate) open_ro_txs: Mutex<Vec<u64>>,
    pub(crate) flags: DBFlags,

    pub(crate) pagesize: u64,
}


impl Inner {
    pub(crate) fn open(file: File, pagesize: u64, flags: DBFlags) -> Result<Inner> {
        file.lock_exclusive()?;
        let mmap = mmap(&file, flags.mmap_populate)?;

        let mmap = Mutex::new(Arc::new(mmap));
        let db = Inner {
            data: mmap,
            mmap_lock: RwLock::new(()),
            freelist: Mutex::new(Freelist::new()),
            file: Mutex::new(file),
            open_ro_txs: Mutex::new(Vec::new()),
            flags,
            pagesize,
        };

        {
            let meta = db.meta()?;
            let data = db.data.lock();
            let free_pages = Page::from_buf(
                &data,
                meta.freelist_page,
                pagesize,
            ).freelist();

            if !free_pages.is_empty() {
                db.freelist.lock().init(free_pages);
            }
        }

        Ok(db)
    }

    pub(crate) fn resize(&self, file: &File, new_size: u64) -> Result<Arc<Mmap>> {
        file.allocate(new_size)?;
        let lock = self.mmap_lock.write();
        let mut data = self.data.lock();
        let mmap = mmap(file, self.flags.mmap_populate)?;
        *data = Arc::new(mmap);
        drop(lock);
        Ok(data.clone())
    }

    pub(crate) fn meta(&self) -> Result<Meta> {
        let data = self.data.lock();

        // meta_1 and meta_2 are the two meta pages. We only need to read two
        // and identify which one is the most recent.
        let meta_1 = Page::from_buf(&data, 0, self.pagesize).meta();
        if meta_1.valid() && meta_1.pagesize != self.pagesize {
            assert_eq!(meta_1.pagesize, self.pagesize, "Invalid pagesize from meta_1 {}. Expected {}.", meta_1.pagesize, self.pagesize);
        }
        let meta_2 = Page::from_buf(&data, 1, self.pagesize).meta();
        let meta = match (meta_1.valid(), meta_2.valid()) {
            (true, true) => {
                assert_eq!(
                    meta_1.pagesize, self.pagesize,
                    "Invalid pagesize from meta1 {}. Expected {}.",
                    meta_1.pagesize, self.pagesize
                );

                assert_eq!(
                    meta_2.pagesize, self.pagesize,
                    "Invalid pagesize from meta2 {}. Expected {}.",
                    meta_2.pagesize, self.pagesize
                );

                if meta_1.tx_id > meta_2.tx_id {
                    meta_1
                } else {
                    meta_2
                }
            }
            (true, false) => {
                assert_eq!(
                    meta_1.pagesize, self.pagesize,
                    "Invalid pagesize from meta1 {}. Expected {}.",
                    meta_1.pagesize, self.pagesize
                );
                meta_1
            }
            (false, true) => {
                assert_eq!(
                    meta_2.pagesize, self.pagesize,
                    "Invalid pagesize from meta2 {}. Expected {}.",
                    meta_2.pagesize, self.pagesize
                );
                meta_2
            }
            (false, false) => panic!("Invalid meta pages"),
        };
        Ok(meta.clone())
    }
}

#[cfg(unix)]
fn mmap(file: &File, populate: bool) -> Result<Mmap> {
    use memmap2::MmapOptions;
    let mut opts = MmapOptions::new();
    if populate {
        opts.populate();
    }
    let mmap = unsafe { opts.map(file)? };
    // On Unix we advice the OS that page access will be random.
    mmap.advise(memmap2::Advice::Random)?;
    Ok(mmap)
}

mod tests {
    use crate::inner::mmap;

    #[test]
    fn test_mmap() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();
        let mmap = mmap(&file, true).unwrap();
        dbg!(mmap);

        dbg!(file.metadata().unwrap().len());
    }
}