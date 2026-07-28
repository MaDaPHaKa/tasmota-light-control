use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use ipnet::IpNet;
use serde::Deserialize;

#[derive(Parser)]
pub struct Arguments {
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    bind_address: Option<String>,
    database_path: Option<PathBuf>,
    allowed_cidrs: Option<Vec<String>>,
    bulb_timeout_ms: Option<u64>,
    fanout_concurrency: Option<usize>,
    max_control_operations: Option<usize>,
    database_busy_timeout_ms: Option<u64>,
    request_body_limit_bytes: Option<usize>,
    tasmota_body_limit_bytes: Option<usize>,
    shutdown_grace_ms: Option<u64>,
    log_level: Option<String>,
}

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database: PathBuf,
    pub cidrs: Vec<IpNet>,
    pub bulb_timeout: Duration,
    pub fanout: usize,
    pub operations: usize,
    pub body_limit: usize,
    pub response_limit: usize,
    pub shutdown: Duration,
    pub busy_timeout: u64,
    pub log_level: String,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self, String> {
        let file = match path {
            Some(path) => load_file_config(path, true)?,
            None => match default_config_path() {
                Ok(path) if path.is_file() => load_file_config(path, false)?,
                Ok(_) | Err(_) => FileConfig::default(),
            },
        };
        let get = |key: &str, value: Option<String>| std::env::var(key).ok().or(value);
        let log_level = get("TLC_LOG_LEVEL", file.log_level).unwrap_or_else(|| {
            if cfg!(debug_assertions) {
                "trace"
            } else {
                "info"
            }
            .into()
        });
        if !matches!(
            log_level.as_str(),
            "error" | "warn" | "info" | "debug" | "trace"
        ) {
            return Err("Invalid TLC_LOG_LEVEL".into());
        }
        let bind = get("TLC_BIND_ADDRESS", file.bind_address)
            .unwrap_or_else(|| "127.0.0.1:8080".into())
            .parse()
            .map_err(|_| "Invalid TLC_BIND_ADDRESS")?;
        let database = get(
            "TLC_DATABASE_PATH",
            file.database_path.map(|path| path.display().to_string()),
        )
        .map(PathBuf::from)
        .map_or_else(default_database_path, Ok)?;
        if !database.is_absolute() {
            return Err("Database path must be absolute".into());
        }
        let cidrs: Vec<IpNet> = get(
            "TLC_ALLOWED_CIDRS",
            file.allowed_cidrs.map(|values| values.join(",")),
        )
        .unwrap_or_else(|| "192.168.178.0/24".into())
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| "Invalid TLC_ALLOWED_CIDRS")?;
        if cidrs.is_empty() || cidrs.iter().any(|cidr| matches!(cidr, IpNet::V6(_))) {
            return Err("Allowed CIDRs must be non-empty IPv4 CIDRs".into());
        }
        let number = |key: &str,
                      file_value: Option<u64>,
                      default: u64,
                      min: u64,
                      max: u64|
         -> Result<u64, String> {
            let value = get(key, file_value.map(|value| value.to_string()))
                .unwrap_or_else(|| default.to_string())
                .parse()
                .map_err(|_| format!("Invalid {key}"))?;
            if !(min..=max).contains(&value) {
                return Err(format!("Invalid {key}"));
            }
            Ok(value)
        };
        let bulb_timeout = number(
            "TLC_BULB_TIMEOUT_MS",
            file.bulb_timeout_ms,
            3000,
            100,
            30000,
        )?;
        let fanout = usize::try_from(number(
            "TLC_FANOUT_CONCURRENCY",
            file.fanout_concurrency.map(|value| value as u64),
            8,
            1,
            32,
        )?)
        .map_err(|_| "Invalid TLC_FANOUT_CONCURRENCY")?;
        let operations = usize::try_from(number(
            "TLC_MAX_CONTROL_OPERATIONS",
            file.max_control_operations.map(|value| value as u64),
            4,
            1,
            16,
        )?)
        .map_err(|_| "Invalid TLC_MAX_CONTROL_OPERATIONS")?;
        let body_limit = usize::try_from(number(
            "TLC_REQUEST_BODY_LIMIT_BYTES",
            file.request_body_limit_bytes.map(|value| value as u64),
            16384,
            1024,
            1048576,
        )?)
        .map_err(|_| "Invalid TLC_REQUEST_BODY_LIMIT_BYTES")?;
        let response_limit = usize::try_from(number(
            "TLC_TASMOTA_BODY_LIMIT_BYTES",
            file.tasmota_body_limit_bytes.map(|value| value as u64),
            65536,
            1024,
            1048576,
        )?)
        .map_err(|_| "Invalid TLC_TASMOTA_BODY_LIMIT_BYTES")?;
        Ok(Self {
            bind,
            database,
            cidrs,
            bulb_timeout: Duration::from_millis(bulb_timeout),
            fanout,
            operations,
            body_limit,
            response_limit,
            shutdown: Duration::from_millis(number(
                "TLC_SHUTDOWN_GRACE_MS",
                file.shutdown_grace_ms,
                10000,
                1000,
                60000,
            )?),
            busy_timeout: number(
                "TLC_DATABASE_BUSY_TIMEOUT_MS",
                file.database_busy_timeout_ms,
                5000,
                100,
                30000,
            )?,
            log_level,
        })
    }
}

fn load_file_config(path: PathBuf, require_absolute: bool) -> Result<FileConfig, String> {
    if require_absolute && !path.is_absolute() {
        return Err("Config path must be absolute".into());
    }
    toml::from_str(&std::fs::read_to_string(path).map_err(|_| "Cannot read config file")?)
        .map_err(|_| "Invalid config file".into())
}

fn default_database_path() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .map(|home| home.join(".config/tasmota-lights-control/data/app.sqlite3"))
        .ok_or_else(|| "Cannot determine absolute default database path".into())
}

fn default_config_path() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| home.is_absolute())
        .map(|home| home.join(".config/tasmota-lights-control/config/config.toml"))
        .ok_or_else(|| "Cannot determine absolute default config path".into())
}
