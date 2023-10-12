use std::path::Path;
use std::sync::Arc;

use crate::config::running_config::RunningConfig;
use crate::context::Context;
use crate::errors::Result;
use crate::inner::Inner;
use crate::options::Options;
use crate::transaction::Tx;

#[derive(Clone)]
#[doc(alias = "database")]
pub struct DB {
    pub context: Context,
    pub(crate) inner: Arc<Inner>,
}


impl DB {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<DB> {
        Options::new().open(path)
    }

    pub fn tx(&self, writable: bool) -> Result<Tx> {
        Tx::new(self, writable)
    }
    pub fn pagesize(&self) -> u64 {
        todo!()
    }

    pub(crate) fn start_inner(config: RunningConfig) -> Result<Self> {
        todo!()
    }
}