use crate::{domain::control::ResultCode, error::AppResult};
use ipnet::IpNet;
use std::{collections::BTreeMap, net::Ipv4Addr};

#[derive(Clone)]
pub struct PolicyState {
    pub cidrs: Vec<IpNet>,
}

#[derive(Clone)]
pub struct Endpoint {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl PolicyState {
    pub fn endpoint(&self, address: &str, port: i64) -> AppResult<Endpoint> {
        let mut fields = BTreeMap::new();
        let ip = address.parse::<Ipv4Addr>().ok();
        if ip.is_none() {
            fields.insert("ipAddress".into(), "IPv4 address is required".into());
        }
        let port = u16::try_from(port).ok().filter(|port| *port != 0);
        if port.is_none() {
            fields.insert("port".into(), "Port must be between 1 and 65535".into());
        }
        if ip.is_some_and(|ip| {
            ip.is_unspecified()
                || ip.is_loopback()
                || ip.is_multicast()
                || ip.is_link_local()
                || ip == Ipv4Addr::BROADCAST
                || !self.cidrs.iter().any(|network| matches!(network, IpNet::V4(network) if network.contains(&ip) && ip != network.network() && ip != network.broadcast()))
        })
        {
            fields.insert("ipAddress".into(), "Address is not an allowed host".into());
        }
        if !fields.is_empty() {
            return Err(crate::error::AppError::Validation(fields));
        }
        Ok(Endpoint {
            ip: ip.ok_or(crate::error::AppError::Internal)?,
            port: port.ok_or(crate::error::AppError::Internal)?,
        })
    }
}

#[derive(Clone)]
pub enum Command {
    Test,
    Status,
    Color,
    ColorTemperature,
    ApplyProperties {
        dimmer: Option<u8>,
        color: Option<String>,
        ct: Option<u16>,
        fade: Option<u8>,
        speed: Option<u8>,
    },
}

impl Command {
    pub fn text(&self) -> String {
        match self {
            Self::Test => "Status 0".into(),
            Self::Status => "Status 11".into(),
            Self::Color => "Color".into(),
            Self::ColorTemperature => "CT".into(),
            Self::ApplyProperties {
                dimmer,
                color,
                ct,
                fade,
                speed,
            } => {
                let mut commands = Vec::new();
                if let Some(fade) = fade {
                    commands.push(format!("Fade {fade}"));
                }
                if let Some(speed) = speed {
                    commands.push(format!("Speed {speed}"));
                }
                if let Some(color) = color {
                    commands.push(format!("Color {color}"));
                }
                if let Some(ct) = ct {
                    commands.push(format!("CT {ct}"));
                }
                if let Some(dimmer) = dimmer {
                    commands.push(format!("Dimmer {dimmer}"));
                }
                format!("Backlog0 {}", commands.join("; "))
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct DeviceState {
    pub power: Option<String>,
    pub dimmer: Option<u8>,
    pub mode: Option<String>,
    pub rgb_color: Option<String>,
    pub ct: Option<u16>,
}

pub trait BulbController: Send + Sync {
    fn invoke(
        &self,
        endpoint: Endpoint,
        command: Command,
    ) -> impl Future<Output = ResultCode> + Send;
    fn status(
        &self,
        endpoint: Endpoint,
    ) -> impl Future<Output = Result<DeviceState, ResultCode>> + Send;
}
