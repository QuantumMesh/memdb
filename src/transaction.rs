use std::cell::RefCell;
use std::fs::File;
use parking_lot::{MutexGuard, RwLockReadGuard};

use crate::db::DB;
use crate::errors::Result;

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
        todo!("")
    }
}