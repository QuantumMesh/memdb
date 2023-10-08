use std::{
    fs::{File, OpenOptions as FileOpenOptions},
    io::Write,
    path::Path,
};
use std::any::Any;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::Arc;

use fs2::FileExt;
use page_size::get as get_page_size;
use parking_lot::{Mutex, RwLock};

use crate::bucket::BucketMeta;
use crate::defaults::{DATABASE_INTEGRITY_CODE, DEFAULT_NUM_PAGES, VERSION};
use crate::errors::Result;
use crate::page::Page;

pub struct DBOpenOptions {
    read_only: bool,
    page_size: u64,
    num_pages: usize,
    flags: DBFlags,
}

impl DBOpenOptions {
    fn new() -> Self {
        Self::default()
    }
    pub fn page_size(mut self, pagesize: u64) -> Self {
        if pagesize < 1024 {
            panic!("Pagesize must be 1024 bytes minimum");
        }
        self.page_size = pagesize;
        self
    }

    /// Sets the number of pages to allocate for a new database file.
    /// The default is 32.
    ///
    /// So if page_size is 4096 (4Kb), then the initial size of the database file will be 128Kb.
    /// Setting `num_pages` when opening an existing database has no effect.
    pub fn num_pages(mut self, numpages: usize) -> Self {
        if numpages < 4 {
            panic!("Number of pages must be 4 minimum");
        }
        self.num_pages = numpages;
        self
    }

    /// Enables or disables "Strict Mode", where each transaction will check the database for errors before finalizing a write.
    ///
    /// The default is `true`, but you may disable this if you want an extra bit of performance. But at the cost of safety.
    pub fn strict_mode(mut self, strict_mode: bool) -> Self {
        self.flags.strict_mode = strict_mode;
        self
    }

    /// Enables or disables the [MAP_POPULATE flag](MAP_POPULATE) for the `mmap` call, which will cause Linux to eagerly load pages into memory.
    ///
    /// The default is `false`, but you may enable this if your database file will stay smaller than your available memory.
    /// It is not recommended to enable this unless you know what you are doing.
    ///
    /// This setting only works on Linux, and is a no-op on other platforms.
    ///
    /// # References
    /// - [mmap - Linux manual page](http://man7.org/linux/man-pages/man2/mmap.2.html)
    /// - [MAP_POPULATE - Linux kernel documentation](https://www.kernel.org/doc/Documentation/vm/mmap_populate.txt)
    /// - [MAP_POPULATE - libc crate](http://rust-lang.github.io/libc/arm-linux-androideabi/doc/libc/constant.MAP_POPULATE.html)
    /// - [mmap2 crate](https://docs.rs/mmap2/latest/mmap2/)
    /// - [memmap crate](https://docs.rs/memmap/latest/memmap/)
    /// - [rustix crate](https://docs.rs/rustix/latest/rustix/mm/struct.MapFlags.html)
    /// - [mmap-rs crate](https://docs.rs/mmap-rs/0.3.0/mmap_rs/struct.MmapFlags.html)
    /// - [nix crate](https://docs.rs/nix/latest/```
    pub fn mmap_populate(mut self, mmap_populate: bool) -> Self {
        self.flags.mmap_populate = mmap_populate;
        self
    }

    /// Enables or disables the O_DIRECT flag when opening the database file.
    /// This gives a hint to Linux to bypass any operarating system caches when writing to this file.
    ///
    /// The default is `false`, but you may enable this if your database is much larger
    /// than your available memory to avoid throttling the page cache.
    ///
    /// It is not recommended to enable this unless you know what you are doing.
    ///
    /// This setting only works on Linux, and is a no-op on other platforms.
    pub fn direct_writes(mut self, direct_writes: bool) -> Self {
        self.flags.direct_writes = direct_writes;
        self
    }

    pub fn open<P: AsRef<Path>>(self, path: P) -> Result<DB> {
        let path: &Path = path.as_ref();
        let file = if !path.exists() {
            init_file(
                path,
                self.page_size,
                self.num_pages,
                self.flags.direct_writes,
            )?
        } else {
            open_file(path, false, self.flags.direct_writes)?
        };
        let db = DBInner::open(file, self.page_size, self.flags)?;
        Ok(DB {
            inner: Arc::new(db),
        })
    }
}

pub(crate) struct DBFlags {
    pub(crate) strict_mode: bool,
    pub(crate) mmap_populate: bool,
    pub(crate) direct_writes: bool,
}

#[derive(Clone)]
pub struct DB {
    pub(crate) inner: Arc<dyn Any>,
}


impl Default for DBOpenOptions {
    fn default() -> DBOpenOptions {
        let page_size = get_page_size() as u64;
        if page_size < 1024 {
            panic!("Page size must be 1024 bytes minimum");
        }
        DBOpenOptions {
            read_only: false,
            page_size,
            num_pages: DEFAULT_NUM_PAGES,
            flags: DBFlags {
                strict_mode: true,
                mmap_populate: false,
                direct_writes: false,
            },
        }
    }
}

pub(crate) struct DBInner {
    pub(crate) data: Mutex<Arc<Mmap>>,
    pub(crate) mmap_lock: RwLock<()>,
    pub(crate) freelist: Mutex<Freelist>,
    pub(crate) file: Mutex<File>,
    pub(crate) open_ro_txs: Mutex<Vec<u64>>,
    pub(crate) flags: DBFlags,

    pub(crate) pagesize: u64,
}



/// The `O_DIRECT` flag is a hint to the kernel that the mapped pages should be faulted in (i.e., loaded into memory) immediately, rather than being loaded later on-demand.
/// This can improve performance by reducing the number of page faults that occur when the file is accessed.
/// However, this function should be used with caution. If the file is larger than the available memory, enabling `O_DIRECT` can cause the system to run out of memory.
/// Therefore, this function should only be used if you're certain that the file will fit into memory.
///
///
/// # References
///
/// - [mmap - Linux manual page](http://man7.org/linux/man-pages/man2/mmap.2.html)
/// - [O_DIRECT - Linux manual page](http://man7.org/linux/man-pages/man2/open.2.html)
/// - [OpenOptions - Rust standard library](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)
/// - [File - Rust standard library](https://doc.rust-lang.org/std/fs/struct.File.html)
#[cfg(any(target_os = "linux"))]
const O_DIRECT: libc::c_int = libc::O_DIRECT;

#[cfg(unix)]
fn open_file<P: AsRef<Path>>(path: P, create: bool, direct_write: bool) -> Result<File> {
    let open_options = set_open_options(create, direct_write);
    Ok(open_options.open(path)?)
}

#[cfg(unix)]
fn set_open_options(create: bool, direct_write: bool) -> FileOpenOptions {
    let mut open_options = FileOpenOptions::new();
    open_options.write(true).read(true);
    if create {
        open_options.create_new(true);
    }
    if direct_write {
        open_options.custom_flags(O_DIRECT);
    }
    open_options
}

fn init_file(path: &Path, pagesize: u64, num_pages: usize, direct_write: bool) -> Result<File> {
    let mut file = open_file(path, true, direct_write)?;
    file.allocate(
        pagesize * (num_pages as u64)
    )?;

    let mut buf = vec![0; (pagesize * 4) as usize];
    let mut get_page = |index: u64| {
        let ptr = &mut buf[(index * pagesize) as usize] as *mut u8;
        let page_ptr = ptr as *mut Page;
        unsafe { &mut *page_ptr }
    };

    for i in 0..2 {
        let page = get_page(i);
        page.id = 1;
        page.page_type = Page::TYPE_META;
        let m = page.meta_mut();
        m.meta_page = i as u32;
        m.integrity_code = DATABASE_INTEGRITY_CODE;
        m.version = VERSION;
        m.pagesize = pagesize;
        m.freelist_page = 2;
        m.root = BucketMeta {
            root_page: 3,
            next_int: 0,
        };
        m.num_pages = 4;
        m.hash = m.hash_self();
    }

    let p = get_page(2);

    p.id = 2;
    p.page_type = Page::TYPE_FREELIST;
    p.count = 0;

    let p = get_page(3);
    p.id = 3;
    p.page_type = Page::TYPE_LEAF;
    p.count = 0;

    file.write_all(&buf[..])?;
    file.flush()?;
    file.sync_all()?;
    Ok(file)
}