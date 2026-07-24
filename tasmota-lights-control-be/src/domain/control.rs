use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ids {
    pub bulb_ids: Vec<Uuid>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Apply {
    pub profile_id: Uuid,
    pub bulb_ids: Vec<Uuid>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Link {
    pub bulb_id: Uuid,
    pub profile_id: Uuid,
}
#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ResultCode {
    Success,
    Timeout,
    Unreachable,
    InvalidResponse,
    DeviceError,
    HttpError,
}
impl ResultCode {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Success => "Command accepted",
            Self::Timeout => "Bulb request timed out",
            Self::Unreachable => "Bulb is unreachable",
            Self::InvalidResponse => "Bulb returned an invalid response",
            Self::DeviceError => "Bulb rejected the command",
            Self::HttpError => "Bulb returned an HTTP error",
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub status: ResultCode,
    pub compatible: Option<bool>,
    pub message: String,
    pub checked_at: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveState {
    pub bulb_id: Uuid,
    pub reachability: String,
    pub power: Option<String>,
    pub dimmer: Option<u8>,
    pub mode: Option<String>,
    pub rgb_color: Option<String>,
    pub color_temperature_kelvin: Option<u16>,
    pub applied_profile_id: Option<Uuid>,
    pub checked_at: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetResult {
    pub bulb_id: Uuid,
    pub bulb_name: String,
    pub status: ResultCode,
    pub message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub operation: String,
    pub profile_id: Option<Uuid>,
    pub started_at: String,
    pub completed_at: String,
    pub summary: Summary,
    pub results: Vec<TargetResult>,
}
#[derive(Serialize)]
pub struct Summary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}
