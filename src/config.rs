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

/// A persisted configuration about high-level
/// storage file information
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct StorageParameters {
    pub segment_size: usize,
    pub use_compression: bool,
    pub version: (usize, usize),
}

pub(crate) struct DBFlags {
    pub(crate) strict_mode: bool,
    pub(crate) mmap_populate: bool,
    pub(crate) direct_writes: bool,
}

