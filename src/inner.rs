use std::fs::File;
use std::sync::Arc;

use fs2::FileExt;
use memmap2::Mmap;
use parking_lot::{Mutex, RwLock};

use crate::config::DBFlags;
use crate::errors::Result;
use crate::freelist::Freelist;

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
        todo!("")
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
    }
}