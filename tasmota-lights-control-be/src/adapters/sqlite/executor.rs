use crate::{
    adapters::sqlite::migrations::migrate,
    error::{AppError, AppResult},
};
use rusqlite::Connection;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
pub struct Db {
    sender: mpsc::Sender<DbJob>,
    readiness: Arc<AtomicBool>,
    _worker: JoinHandle<()>,
}
struct DbJob {
    run: Box<dyn FnOnce(&mut Connection) + Send>,
}
impl Db {
    pub fn open(path: PathBuf, busy: u64) -> Result<Self, String> {
        let (sender, mut receiver) = mpsc::channel::<DbJob>(128);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let readiness = Arc::new(AtomicBool::new(true));
        let worker_readiness = readiness.clone();
        let worker = std::thread::Builder::new().name("sqlite-worker".into()).spawn(move || {
            let result = Connection::open(path).map_err(|error| error.to_string()).and_then(|mut connection| {
                connection.execute_batch(&format!("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout={busy};")).map_err(|error| error.to_string())?;
                migrate(&mut connection)?;
                Ok(connection)
            });
            match result {
                Ok(mut connection) => {
                    let _ = ready_tx.send(Ok(()));
                    while let Some(job) = receiver.blocking_recv() {
                        (job.run)(&mut connection);
                    }
                }
                Err(error) => { let _ = ready_tx.send(Err(error)); }
            }
            worker_readiness.store(false, Ordering::Release);
        }).map_err(|_| "Cannot start database worker")?;
        ready_rx.recv().map_err(|_| "Database worker stopped")??;
        Ok(Self {
            sender,
            readiness,
            _worker: tokio::task::spawn_blocking(move || {
                let _ = worker.join();
            }),
        })
    }
    pub async fn run<T: Send + 'static>(
        &self,
        action: impl FnOnce(&mut Connection) -> AppResult<T> + Send + 'static,
    ) -> AppResult<T> {
        let (tx, rx) = oneshot::channel();
        if !self.readiness.load(Ordering::Acquire) {
            return Err(AppError::Unavailable);
        }
        self.sender
            .try_send(DbJob {
                run: Box::new(move |connection| {
                    let _ = tx.send(action(connection));
                }),
            })
            .map_err(|_| AppError::Unavailable)?;
        rx.await.map_err(|_| AppError::Unavailable)?
    }

    pub fn is_available(&self) -> bool {
        self.readiness.load(Ordering::Acquire)
    }
}
