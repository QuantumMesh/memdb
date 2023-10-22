use crate::config::running_config::RunningConfig;

#[derive(Clone)]
pub struct Context {
    config: RunningConfig,

    // #[cfg(not(miri))]
    // pub(crate) flusher: Arc<Mutex<Option<flusher::Flusher>>>,
    // #[doc(hidden)]
    // pub pagecache: PageCache,
}

impl std::ops::Deref for Context {
    type Target = RunningConfig;

    fn deref(&self) -> &RunningConfig {
        &self.config
    }
}

impl Context {}