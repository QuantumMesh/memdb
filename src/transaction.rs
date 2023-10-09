use std::cell::RefCell;
use std::fs::File;
use std::rc::Rc;
use parking_lot::{MutexGuard, RwLockReadGuard};
use crate::bucket::InnerBucket;

use crate::db::DB;
use crate::errors::Result;
use crate::freelist::TxFreelist;
use crate::meta::Meta;
use crate::page::Pages;

pub(crate) enum TxLock<'tx> {
    Rw(MutexGuard<'tx, File>),
    Ro(RwLockReadGuard<'tx, ()>),
}

impl<'tx> TxLock<'tx> {
    fn writable(&self) -> bool {
        match self {
            Self::Rw(_) => true,
            Self::Ro(_) => false,
        }
    }
}

pub struct Tx<'tx> {
    pub(crate) inner: RefCell<TxInner<'tx>>,
}

pub(crate) struct TxInner<'tx> {
    pub(crate) db: &'tx DB,
    pub(crate) lock: TxLock<'tx>,
    pub(crate) root: Rc<RefCell<InnerBucket<'tx>>>,
    pub(crate) meta: Meta,
    pub(crate) freelist: Rc<RefCell<TxFreelist>>,
    pages: Pages,
    num_freelist_pages: u64,
}

impl <'tx> Tx<'tx> {
    pub(crate) fn new(db: &'tx DB, writable: bool) -> Result<Tx<'tx>> {
        let lock = match writable {
            true => TxLock::Rw(db.inner.file.lock()),
            false => TxLock::Ro(db.inner.mmap_lock.read()),
        };

        let mut freelist = db.inner.freelist.lock().clone();
        let mut meta = db.inner.meta();
        todo!()
    }
}