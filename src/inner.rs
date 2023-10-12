use std::fmt::Debug;
use std::path::PathBuf;

use crate::config::{Config, Mode};
use crate::config::flags::DBFlags;

const DEFAULT_PATH: &str = "default.db";

#[derive(Debug, Clone)]
pub(crate) struct Inner {
    pub cache_capacity: usize,
    pub flush_every_ms: Option<u64>,
    pub segment_size: usize,
    pub mode: Mode,
    pub path: PathBuf,
    pub temporary: bool,
    tmp_path: PathBuf,
    pub create_new: bool,
    pub snapshot_after_ops: u64,
    pub version: (usize, usize),
    // TODO: Event log handler for debugging
    pub(crate) flags: DBFlags,

}

impl Default for Inner {
    fn default() -> Self {
        Self {
            path: PathBuf::from(DEFAULT_PATH),
            tmp_path: Config::gen_temp_path(),
            cache_capacity: 1024 * 1024 * 1024, // 1gb
            mode: Mode::LowSpace,
            temporary: false,
            version: crate_version(),

            // useful in testing
            segment_size: 512 * 1024, // 512kb in bytes
            flush_every_ms: Some(500),
            snapshot_after_ops: if cfg!(feature = "for-internal-testing-only") {
                10
            } else {
                1_000_000
            },
            flags: DBFlags {
                strict_mode: false,
                mmap_populate: false,
                direct_writes: false,
            },
            create_new: false,
        }
    }
}


impl Inner {
    pub fn get_path(&self) -> PathBuf {
        if self.temporary && self.path == PathBuf::from(DEFAULT_PATH) {
            self.tmp_path.clone()
        } else {
            self.path.clone()
        }
    }

    pub(crate) fn db_path(&self) -> PathBuf {
        self.get_path().join("db")
    }

    fn config_path(&self) -> PathBuf {
        self.get_path().join("conf")
    }

    pub(crate) fn normalize<T>(&self, value: T) -> T
        where
            T: Copy
            + TryFrom<usize>
            + std::ops::Div<Output=T>
            + std::ops::Mul<Output=T>,
            <T as TryFrom<usize>>::Error: Debug,
    {
        let segment_size: T = T::try_from(self.segment_size).unwrap();
        value / segment_size * segment_size
    }
}


fn crate_version() -> (usize, usize) {
    let vsn = env!("CARGO_PKG_VERSION");
    let mut parts = vsn.split('.');
    let major = parts.next().unwrap().parse().unwrap();
    let minor = parts.next().unwrap().parse().unwrap();
    (major, minor)
}
