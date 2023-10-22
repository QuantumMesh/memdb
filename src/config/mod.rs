use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::SystemTime;

use log::error;
use page_size::get as get_page_size;
use parking_lot::lock_api::Mutex;
use parking_lot::RwLock;

use crate::config::running_config::RunningConfig;
use crate::db::DB;
use crate::errors::{Error, Result};
use crate::freelist::Freelist;
use crate::inner::Inner;
use crate::maybe_fsync_directory;
use crate::sys::sys_limits;
use crate::utils::mmap;

pub(crate) mod flags;
pub(crate) mod running_config;


macro_rules! supported {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err(Error::Unsupported($msg));
        }
    };
}

macro_rules! builder {
    ($(($name:ident, $t:ty, $desc:expr)),*) => {
        $(
            #[doc=$desc]
            pub fn $name(mut self, to: $t) -> Self {
                if Arc::strong_count(&self.0) != 1 {
                    error!(
                        "config has already been used to start \
                         the system and probably should not be \
                         mutated",
                    );
                }
                let m = Arc::make_mut(&mut self.0);
                m.$name = to;
                self
            }
        )*
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    /// In this mode, the database will make
    /// decisions that favor using less space
    /// instead of supporting the highest possible
    /// write throughput. This mode will also
    /// rewrite data more frequently as it
    /// strives to reduce fragmentation.
    LowSpace,
    /// In this mode, the database will try
    /// to maximize write throughput while
    /// potentially using more disk space.
    HighThroughput,
}


#[derive(Default, Debug, Clone)]
pub struct Config(Arc<Inner>);

impl Deref for Config {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.0
    }
}


impl Config {
    pub fn new() -> Config {
        Config::default()
    }

    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Config {
        let m = Arc::get_mut(&mut self.0).unwrap();
        m.path = path.as_ref().to_path_buf();
        self
    }

    pub fn open(&self) -> Result<DB> {
        self.validate()?;
        let mut config = self.clone();
        config.limit_cache_max_memory();

        let file = config.open_file()?;
        let data = mmap(&file, self.flags.mmap_populate)?;
        let pagesize = get_page_size() as u64;
        if pagesize < 1024 {
            panic!("Pagesize must be 1024 bytes minimum");
        }

        let config = RunningConfig {
            inner: config,
            file: Mutex::new(Arc::new(file)),
            data: Mutex::new(Arc::new(data)),
            freelist: Mutex::new(Freelist::new()),
            open_ro_txs: Mutex::new(Vec::new()),
            mmap_lock: RwLock::new(()),
            pagesize,
        };
        DB::start_inner(config)
    }

    pub fn open_file(&self) -> Result<File> {
        let heap_dir: PathBuf = self.get_path().join("heap");
        if !heap_dir.exists() {
            fs::create_dir_all(heap_dir)?;
        }

        self.verify_config()?;
        let mut options = fs::OpenOptions::new();
        let _ = options.create(true);
        let _ = options.read(true);
        let _ = options.write(true);

        let _ = File::create(
            self.get_path().join("DO_NOT_USE_THIS_DIRECTORY_FOR_ANYTHING"),
        );

        let file = self.try_lock(options.open(&self.db_path())?)?;
        maybe_fsync_directory(self.get_path())?;
        Ok(file)
    }

    pub fn flush_every_ms(mut self, every_ms: Option<u64>) -> Self {
        if Arc::strong_count(&self.0) != 1 {
            error!(
                "config has already been used to start \
                 the system and probably should not be \
                 mutated",
            );
        }
        let m = Arc::make_mut(&mut self.0);
        m.flush_every_ms = every_ms;
        self
    }

    fn limit_cache_max_memory(&mut self) {
        if let Some(limit) = sys_limits::get_memory_limit() {
            if self.cache_capacity > limit {
                let m = Arc::make_mut(&mut self.0);
                m.cache_capacity = limit;
                error!(
                    "cache capacity is limited to the cgroup memory \
                 limit: {} bytes",
                    self.cache_capacity
                );
            }
        }
    }

    fn try_lock(&self, file: File) -> Result<File> {
        #[cfg(all(
        any(target_os = "linux", target_os = "macos")
        ))]
        {
            use fs2::FileExt;
            let try_lock = if cfg!(any(feature = "for-internal-testing-only", feature = "light_testing")) {
                file.lock_exclusive()
            } else {
                file.try_lock_exclusive()
            };

            if try_lock.is_err() {
                return Err(Error::Io(
                    ErrorKind::Other,
                    "could not acquire database file lock",
                ));
            }
        }
        Ok(file)
    }
    pub(crate) fn gen_temp_path() -> PathBuf {
        static SALT_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let seed = SALT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u128;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() << 48;


        let pid = u128::from(std::process::id());
        let salt = (pid << 16) + now + seed;
        if cfg!(target_os = "linux") {
            // use shared memory for temporary linux files
            format!("/dev/shm/pagecache.tmp.{}", salt).into()
        } else {
            std::env::temp_dir().join(format!("pagecache.tmp.{}", salt))
        }
    }

    fn validate(&self) -> Result<()> {
        supported!(
            self.segment_size.count_ones() == 1,
            "segment_size should be a power of 2"
        );
        supported!(
            self.segment_size >= 256,
            "segment_size should be hundreds of kb at minimum, and we won't start if below 256"
        );
        supported!(
            self.segment_size <= 1 << 24,
            "segment_size should be <= 16mb"
        );
        Ok(())
    }

    builder!(
        (
            cache_capacity,
            usize,
            "maximum size in bytes for the system page cache"
        ),
        (
            mode,
            Mode,
            "specify whether the system should run in \"small\" or \"fast\" mode"
        ),
        (
            temporary,
            bool,
            "deletes the database after drop. if no path is set, uses /dev/shm on linux"
        ),
        (
            create_new,
            bool,
            "attempts to exclusively open the database, failing if it already exists"
        ),
        (
            snapshot_after_ops,
            u64,
            "take a fuzzy snapshot of pagecache metadata after this many ops"
        )
    );
    fn verify_config(&self) -> Result<()> {
        todo!()
    }
}


