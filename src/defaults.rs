pub const DATABASE_INTEGRITY_CHECK: u32 = 0x00ABCDEF;
pub const VERSION: u32 = 1;
// Minimum number of bytes to allocate when growing the databse
pub(crate) const MIN_ALLOC_SIZE: u64 = 8 * 1024 * 1024;

// Number of pages to allocate when creating the database
pub const DEFAULT_NUM_PAGES: usize = 32;