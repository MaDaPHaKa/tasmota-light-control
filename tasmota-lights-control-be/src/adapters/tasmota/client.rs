pub use crate::ports::bulb_controller::PolicyState;
use crate::{
    adapters::tasmota::parser::{rejected, status as parse_status},
    domain::control::ResultCode,
    error::{AppError, AppResult},
    ports::bulb_controller::{BulbController, Command, DeviceState, Endpoint},
};
use futures_util::StreamExt;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Semaphore, time::timeout};

pub fn endpoint_url(endpoint: &Endpoint, command: &Command) -> AppResult<url::Url> {
    let mut url = url::Url::parse("http://0.0.0.0/cm").map_err(|_| AppError::Internal)?;
    url.set_host(Some(&endpoint.ip.to_string()))
        .map_err(|_| AppError::Internal)?;
    url.set_port(Some(endpoint.port))
        .map_err(|_| AppError::Internal)?;
    url.query_pairs_mut().append_pair("cmnd", &command.text());
    Ok(url)
}

#[derive(Clone)]
pub struct TasmotaClient {
    pub policy: Arc<PolicyState>,
    pub client: reqwest::Client,
    pub outbound: Arc<Semaphore>,
    pub timeout: Duration,
    pub response_limit: usize,
}

impl TasmotaClient {
    async fn request(
        &self,
        endpoint: Endpoint,
        command: Command,
    ) -> Result<serde_json::Value, ResultCode> {
        if self
            .policy
            .endpoint(&endpoint.ip.to_string(), endpoint.port)
            .is_err()
        {
            return Err(ResultCode::DeviceError);
        }
        let url = endpoint_url(&endpoint, &command).map_err(|_| ResultCode::InvalidResponse)?;
        timeout(self.timeout, async {
            let _permit = self
                .outbound
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| ResultCode::Timeout)?;
            let response = self
                .client
                .get(url)
                .send()
                .await
                .map_err(|_| ResultCode::Unreachable)?;
            if !response.status().is_success() || response.status().is_redirection() {
                return Err(ResultCode::HttpError);
            }
            let mut bytes = Vec::new();
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|_| ResultCode::Unreachable)?;
                if bytes.len().saturating_add(chunk.len()) > self.response_limit {
                    return Err(ResultCode::InvalidResponse);
                }
                bytes.extend_from_slice(&chunk);
            }
            if bytes.is_empty() {
                return Err(ResultCode::InvalidResponse);
            }
            let value = serde_json::from_slice::<serde_json::Value>(&bytes)
                .map_err(|_| ResultCode::InvalidResponse)?;
            if !value.is_object() {
                return Err(ResultCode::InvalidResponse);
            }
            if rejected(&value) {
                return Err(ResultCode::DeviceError);
            }
            Ok(value)
        })
        .await
        .map_err(|_| ResultCode::Timeout)?
    }
}

impl BulbController for TasmotaClient {
    async fn invoke(&self, endpoint: Endpoint, command: Command) -> ResultCode {
        self.request(endpoint, command)
            .await
            .map_or_else(|code| code, |_| ResultCode::Success)
    }

    async fn status(&self, endpoint: Endpoint) -> Result<DeviceState, ResultCode> {
        timeout(self.timeout, async {
            let mut state = parse_status(&self.request(endpoint.clone(), Command::Status).await?);
            if state.rgb_color.is_none()
                && let Ok(value) = self.request(endpoint.clone(), Command::Color).await
            {
                state.rgb_color = parse_status(&value).rgb_color;
            }
            if state.ct.is_none()
                && let Ok(value) = self.request(endpoint, Command::ColorTemperature).await
            {
                state.ct = parse_status(&value).ct;
            }
            if state.mode.is_none() {
                state.mode = match (&state.rgb_color, state.ct) {
                    (Some(_), None) => Some("rgb".into()),
                    (None, Some(_)) => Some("color_temperature".into()),
                    _ => None,
                };
            }
            Ok(state)
        })
        .await
        .map_err(|_| ResultCode::Timeout)?
    }
}
