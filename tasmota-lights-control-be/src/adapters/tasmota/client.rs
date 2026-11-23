pub use crate::ports::bulb_controller::PolicyState;
use crate::{
    adapters::tasmota::parser,
    domain::{control::ResultCode, settings::BulbSettings},
    error::{AppError, AppResult},
    ports::bulb_controller::{BulbController, Command, DeviceState, Endpoint},
};
use futures_util::StreamExt;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::Semaphore,
    time::{Instant, timeout, timeout_at},
};
use tracing::{debug, error};

pub fn endpoint_url(endpoint: &Endpoint, command: &Command) -> AppResult<url::Url> {
    let mut url = url::Url::parse("http://0.0.0.0/cm").map_err(|_| AppError::Internal)?;
    url.set_host(Some(&endpoint.ip.to_string()))
        .map_err(|_| AppError::Internal)?;
    url.set_port(Some(endpoint.port))
        .map_err(|_| AppError::Internal)?;
    let encoded_command = url::form_urlencoded::byte_serialize(command.text().as_bytes())
        .collect::<String>()
        .replace('+', "%20");
    url.set_query(Some(&format!("cmnd={encoded_command}")));
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
        let command_text = command.text();
        let endpoint_ip = endpoint.ip;
        let endpoint_port = endpoint.port;
        if self
            .policy
            .endpoint(&endpoint_ip.to_string(), i64::from(endpoint_port))
            .is_err()
        {
            return Err(ResultCode::DeviceError);
        }
        let url = endpoint_url(&endpoint, &command).map_err(|_| ResultCode::InvalidResponse)?;
        debug!(
            url = %url,
            command = %command_text,
            "tasmota request"
        );
        let result = timeout(self.timeout, async {
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
            if parser::rejected(&value) {
                return Err(ResultCode::DeviceError);
            }
            Ok(value)
        })
        .await
        .map_err(|_| ResultCode::Timeout);
        if let Err(status) = &result {
            error!(
                endpoint = %format!("{endpoint_ip}:{endpoint_port}"),
                command = %command_text,
                status = ?status,
                "tasmota request failed"
            );
        }
        result?
    }

    pub async fn bulb_settings(
        &self,
        endpoint: Endpoint,
        update: Option<BulbSettings>,
    ) -> Result<BulbSettings, ResultCode> {
        let Some(settings) = update else {
            let value = self.request(endpoint, Command::Status).await?;
            return parser::bulb_settings(&value).ok_or(ResultCode::InvalidResponse);
        };

        // Backlog acknowledges queueing, not the individual setting changes.
        let value = self
            .request(endpoint.clone(), Command::SetFade(settings.fade))
            .await?;
        let fade = value.as_object().and_then(parser::fade);
        if fade != Some(settings.fade) {
            return Err(ResultCode::InvalidResponse);
        }
        let value = self
            .request(endpoint, Command::SetSpeed(settings.speed))
            .await?;
        let speed = value.as_object().and_then(parser::speed);
        if speed != Some(settings.speed) {
            return Err(ResultCode::InvalidResponse);
        }
        Ok(settings)
    }
}

impl BulbController for TasmotaClient {
    async fn invoke(&self, endpoint: Endpoint, command: Command) -> ResultCode {
        self.request(endpoint, command)
            .await
            .map_or_else(|code| code, |_| ResultCode::Success)
    }

    async fn status(&self, endpoint: Endpoint) -> Result<DeviceState, ResultCode> {
        let deadline = Instant::now() + self.timeout;
        let mut state = parser::status(&self.request(endpoint.clone(), Command::Status).await?);
        if state.rgb_color.is_none()
            && state.white_channels.is_none()
            && let Ok(Ok(value)) =
                timeout_at(deadline, self.request(endpoint.clone(), Command::Color)).await
        {
            let color = parser::status(&value);
            state.rgb_color = color.rgb_color;
            state.white_channels = color.white_channels;
        }
        if state.ct.is_none()
            && Instant::now() < deadline
            && let Ok(Ok(value)) =
                timeout_at(deadline, self.request(endpoint, Command::ColorTemperature)).await
        {
            state.ct = parser::status(&value).ct;
        }
        Ok(state)
    }
}
