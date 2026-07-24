use crate::{
    domain::control::ResultCode,
    error::{AppResult, validation},
};
use ipnet::IpNet;
use std::net::Ipv4Addr;

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
    pub fn endpoint(&self, address: &str, port: u16) -> AppResult<Endpoint> {
        let ip: Ipv4Addr = address
            .parse()
            .map_err(|_| validation("ipAddress", "IPv4 address is required"))?;
        if port == 0 {
            return Err(validation("port", "Port must not be zero"));
        }
        if ip.is_unspecified()
            || ip.is_loopback()
            || ip.is_multicast()
            || ip.is_link_local()
            || ip == Ipv4Addr::BROADCAST
            || !self.cidrs.iter().any(|network| matches!(network, IpNet::V4(network) if network.contains(&ip) && ip != network.network() && ip != network.broadcast()))
        {
            return Err(validation("ipAddress", "Address is not an allowed host"));
        }
        Ok(Endpoint { ip, port })
    }
}

#[derive(Clone)]
pub enum Command {
    Test,
    Status,
    Color,
    ColorTemperature,
    ApplyRgb { dimmer: u8, color: String },
    ApplyColorTemperature { dimmer: u8, ct: u16 },
}

impl Command {
    pub fn text(&self) -> String {
        match self {
            Self::Test => "Status 0".into(),
            Self::Status => "Status 11".into(),
            Self::Color => "Color".into(),
            Self::ColorTemperature => "CT".into(),
            Self::ApplyRgb { dimmer, color } => {
                format!("Backlog0 Power1 On; Dimmer {dimmer}; Color {color}")
            }
            Self::ApplyColorTemperature { dimmer, ct } => {
                format!("Backlog0 Power1 On; Dimmer {dimmer}; CT {ct}")
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
