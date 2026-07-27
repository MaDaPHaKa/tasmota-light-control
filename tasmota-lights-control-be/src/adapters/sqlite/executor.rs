use crate::{
    adapters::sqlite::migrations::migrate,
    error::{AppError, AppResult},
};
use rusqlite::Connection;
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
pub struct Db {
    sender: Mutex<Option<mpsc::Sender<DbJob>>>,
    readiness: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<Result<(), String>>>>,
}
struct DbJob {
    run: Box<dyn FnOnce(&mut Connection) + Send>,
}
impl Db {
    pub fn open(path: PathBuf, busy: u64) -> Result<Self, String> {
        let (sender, mut receiver) = mpsc::channel::<DbJob>(128);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let readiness = Arc::new(AtomicBool::new(false));
        let worker_readiness = readiness.clone();
        let worker = std::thread::Builder::new().name("sqlite-worker".into()).spawn(move || {
            struct ReadinessGuard(Arc<AtomicBool>);
            impl Drop for ReadinessGuard {
                fn drop(&mut self) {
                    self.0.store(false, Ordering::Release);
                }
            }
            let _readiness_guard = ReadinessGuard(worker_readiness.clone());
            let result = Connection::open(path).map_err(|error| error.to_string()).and_then(|mut connection| {
                connection.execute_batch(&format!("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout={busy};")).map_err(|error| error.to_string())?;
                migrate(&mut connection)?;
                let settings_count: i64 = connection
                    .query_row("SELECT COUNT(*) FROM reset_settings WHERE singleton=1", [], |row| row.get(0))
                    .map_err(|error| error.to_string())?;
                if settings_count == 0 {
                    connection.execute("INSERT INTO reset_settings(singleton,dimmer,mode,rgb_color,color_temperature_kelvin) VALUES(1,100,'color_temperature',NULL,3000)", [])
                        .map_err(|error| error.to_string())?;
                }
                crate::adapters::sqlite::settings_repository::get(&connection)
                    .map_err(|_| "Invalid persisted reset settings".to_owned())?;
                Ok(connection)
            });
            match result {
                Ok(mut connection) => {
                    worker_readiness.store(true, Ordering::Release);
                    let _ = ready_tx.send(Ok(()));
                    while let Some(job) = receiver.blocking_recv() {
                        (job.run)(&mut connection);
                    }
                }
                Err(error) => { let _ = ready_tx.send(Err(error)); }
            }
        }).map_err(|_| "Cannot start database worker")?;
        ready_rx.recv().map_err(|_| "Database worker stopped")??;
        let worker = tokio::task::spawn_blocking(move || {
            worker
                .join()
                .map_err(|_| "Database worker panicked".to_owned())
        });
        Ok(Self {
            sender: Mutex::new(Some(sender)),
            readiness,
            worker: Mutex::new(Some(worker)),
        })
    }
    pub async fn run<T: Send + 'static>(
        &self,
        action: impl FnOnce(&mut Connection) -> AppResult<T> + Send + 'static,
    ) -> AppResult<T> {
        let (tx, rx) = oneshot::channel();
        if !self.readiness.load(Ordering::Acquire) {
            tracing::error!("database unavailable before operation dispatch");
            return Err(AppError::Unavailable);
        }
        let sender = self
            .sender
            .lock()
            .map_err(|_| {
                tracing::error!("database sender lock failed");
                AppError::Unavailable
            })?
            .clone()
            .ok_or_else(|| {
                tracing::error!("database worker sender unavailable");
                AppError::Unavailable
            })?;
        sender
            .try_send(DbJob {
                run: Box::new(move |connection| {
                    let _ = tx.send(action(connection));
                }),
            })
            .map_err(|_| {
                tracing::error!("database operation queue unavailable");
                AppError::Unavailable
            })?;
        let result = rx.await.map_err(|_| {
            tracing::error!("database worker dropped operation result");
            AppError::Unavailable
        })?;
        if let Err(error @ (AppError::Internal | AppError::Unavailable)) = &result {
            tracing::error!(error = ?error, "database operation failed");
        }
        result
    }

    pub fn is_available(&self) -> bool {
        self.readiness.load(Ordering::Acquire)
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        self.readiness.store(false, Ordering::Release);
        self.sender
            .lock()
            .map_err(|_| "Database sender lock failed")?
            .take();
        let worker = self
            .worker
            .lock()
            .map_err(|_| "Database worker lock failed")?
            .take();
        if let Some(worker) = worker {
            worker
                .await
                .map_err(|error| format!("Database join task failed: {error}"))??;
        }
        Ok(())
    }
}
