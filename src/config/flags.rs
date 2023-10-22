/// A persisted configuration about high-level
/// storage file information
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
struct StorageParameters {
    pub segment_size: usize,
    pub use_compression: bool,
    pub version: (usize, usize),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DBFlags {
    pub(crate) strict_mode: bool,
    pub(crate) mmap_populate: bool,
    pub(crate) direct_writes: bool,
}

