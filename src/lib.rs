mod db;
pub mod defaults;
pub mod errors;
mod page;
mod bucket;
mod meta;
mod freelist;
mod options;
mod transaction;
mod inner;
mod bytes;
mod sys;
mod context;
mod pagecache;
mod config;
mod event_log;
mod utils;


#[cfg(all(unix))]
fn maybe_fsync_directory<P: AsRef<std::path::Path>>(
    path: P,
) -> std::io::Result<()> {
    std::fs::File::open(path)?.sync_all()
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
