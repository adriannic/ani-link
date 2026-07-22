use atomic_float::AtomicF32;
use std::sync::{Mutex, MutexGuard, mpsc::Sender};

#[derive(Clone, Debug)]
pub struct DownloadToken {
    pub name: String,
    pub slug: String,
    pub episode: f64,
}

pub struct Download {
    tx: Sender<DownloadToken>,
    current: Mutex<Option<DownloadToken>>,
    progress: AtomicF32,
}

impl Download {
    pub const fn new(tx: Sender<DownloadToken>) -> Self {
        Self {
            tx,
            current: Mutex::new(None),
            progress: AtomicF32::new(f32::NAN),
        }
    }

    pub fn tx(&self) -> Sender<DownloadToken> {
        self.tx.clone()
    }

    pub fn current(&self) -> MutexGuard<'_, Option<DownloadToken>> {
        self.current.lock().expect("Couldn't lock mutex")
    }

    pub const fn progress(&self) -> &AtomicF32 {
        &self.progress
    }
}
