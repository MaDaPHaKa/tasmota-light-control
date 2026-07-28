use crate::{
    adapters::{
        sqlite::executor::Db,
        tasmota::client::{PolicyState, TasmotaClient},
    },
    application::{
        bulb_service::BulbService, control_service::ControlService,
        direct_link_service::DirectLinkService, profile_service::ProfileService,
        settings_service::SettingsService, status_service::StatusService,
    },
    config::{Arguments, Config},
    http::{dto::AppState, router::router},
};
use clap::Parser;
use reqwest::redirect::Policy;
use std::{
    future::{Future, IntoFuture},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
pub async fn run() {
    let config = match Config::load(Arguments::parse().config) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("backend failed: {error}");
            std::process::exit(1);
        }
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.parse().expect("validated log level")),
        )
        .init();
    if let Err(error) = serve(config).await {
        error!(%error, "backend failed");
        std::process::exit(1);
    }
}

async fn serve(config: Config) -> Result<(), String> {
    if let Some(parent) = config.database.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("database directory failed: {error}"))?;
    }
    let db = Arc::new(Db::open(config.database.clone(), config.busy_timeout)?);
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .no_proxy()
        .timeout(config.bulb_timeout)
        .connect_timeout(config.bulb_timeout)
        .build()
        .map_err(|error| format!("http client startup failed: {error}"))?;
    let policy = Arc::new(PolicyState {
        cidrs: config.cidrs,
    });
    let bulbs = Arc::new(BulbService {
        db: db.clone(),
        policy: policy.clone(),
    });
    let profiles = Arc::new(ProfileService { db: db.clone() });
    let settings = Arc::new(SettingsService { db: db.clone() });
    let tasmota = Arc::new(TasmotaClient {
        policy: policy.clone(),
        client,
        outbound: Arc::new(Semaphore::new(config.fanout)),
        timeout: config.bulb_timeout,
        response_limit: config.response_limit,
    });
    let control = Arc::new(ControlService {
        bulbs: bulbs.clone(),
        profiles: profiles.clone(),
        settings: settings.clone(),
        client: tasmota.clone(),
        admission: Arc::new(Semaphore::new(config.operations)),
    });
    let shutting_down = Arc::new(AtomicBool::new(false));
    let state = AppState {
        bulbs: bulbs.clone(),
        profiles: profiles.clone(),
        control,
        status: Arc::new(StatusService {
            bulbs: bulbs.clone(),
            client: tasmota,
        }),
        settings,
        direct_links: Arc::new(DirectLinkService {
            bulbs,
            profiles,
            policy,
        }),
        shutting_down: shutting_down.clone(),
    };
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .map_err(|error| format!("bind failed: {error}"))?;
    let shutdown_signal = shutdown_signal()?;
    let (shutdown_started, shutdown_started_rx) = tokio::sync::oneshot::channel();
    info!(
        version=env!("CARGO_PKG_VERSION"),
        address=%config.bind,
        database_initialized=true,
        migration_version=1,
        "backend ready"
    );
    let shutdown = async move {
        let signal = shutdown_signal.await;
        info!(%signal, "shutdown signal received");
        shutting_down.store(true, Ordering::Release);
        let _ = shutdown_started.send(());
    };
    let server_result = {
        let server = axum::serve(listener, router(state, config.body_limit))
            .with_graceful_shutdown(shutdown)
            .into_future();
        tokio::pin!(server);
        tokio::select! {
            result = &mut server => result.map_err(|error| format!("server failed: {error}")),
            _ = shutdown_started_rx => match tokio::time::timeout(config.shutdown, &mut server).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(error)) => Err(format!("server failed: {error}")),
                Err(_) => {
                    error!(grace_ms = config.shutdown.as_millis(), "shutdown grace elapsed");
                    warn!(grace_ms = config.shutdown.as_millis(), "remaining server tasks cancelled");
                    Ok(())
                }
            }
        }
    };
    db.shutdown().await?;
    info!(completed = server_result.is_ok(), "shutdown completed");
    server_result
}

#[cfg(unix)]
fn shutdown_signal() -> Result<impl Future<Output = &'static str>, String> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut interrupt = signal(SignalKind::interrupt()).map_err(|error| error.to_string())?;
    let mut terminate = signal(SignalKind::terminate()).map_err(|error| error.to_string())?;
    Ok(async move {
        tokio::select! {
            _ = interrupt.recv() => "SIGINT",
            _ = terminate.recv() => "SIGTERM",
        }
    })
}
