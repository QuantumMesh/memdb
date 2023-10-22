use std::fs::File;

use memmap2::Mmap;

use crate::errors::Result;

#[cfg(unix)]
pub(crate) fn mmap(file: &File, populate: bool) -> Result<Mmap> {
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
    use crate::utils::mmap;

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