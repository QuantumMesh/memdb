use std::fs::File;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use memmap2::Mmap;
use parking_lot::{Mutex, RwLock};

use crate::config::Config;
use crate::freelist::Freelist;

pub struct RunningConfig {
    pub(crate) inner: Config,
    pub(crate) file: Mutex<Arc<File>>,
    pub(crate) data: Mutex<Arc<Mmap>>,
    pub(crate) freelist: Mutex<Freelist>,
    pub(crate) open_ro_txs: Mutex<Vec<u64>>,

    pub(crate) mmap_lock: RwLock<()>,
    pub(crate) pagesize: u64,
}

impl Deref for RunningConfig {
    type Target = Config;

    fn deref(&self) -> &Config {
        &self.inner
    }
}

impl Drop for RunningConfig {
    fn drop(&mut self) {
        use fs2::FileExt;
        let file = self.file.try_lock();

        match file {
            None => {}
            Some(file) => {
                if Arc::strong_count(&file) == 1 {
                    let _ = file.unlock();
                }
            }
        }
    }
}


impl RunningConfig {
    pub fn get_snapshot_files(&self) -> io::Result<Vec<PathBuf>> {
        let config_path = self.get_path().join("snap.");
        todo!()
    }
}