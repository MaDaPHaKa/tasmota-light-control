use crate::ports::bulb_controller::DeviceState;
use serde_json::{Map, Value};
use std::fmt::Write;

pub fn rejected(value: &Value) -> bool {
    match value {
        Value::Object(values) => {
            values.iter().any(|(key, value)| {
                (key.eq_ignore_ascii_case("error") || key.eq_ignore_ascii_case("command"))
                    && value.as_str().is_some_and(|text| {
                        let text = text.to_ascii_lowercase();
                        text.contains("unknown")
                            || text.contains("error")
                            || text.contains("unsupported")
                    })
            }) || values.values().any(rejected)
        }
        Value::Array(values) => values.iter().any(rejected),
        _ => false,
    }
}

pub fn status(value: &Value) -> DeviceState {
    let mut state = DeviceState::default();
    let root = match value.as_object() {
        Some(root) => root,
        None => return state,
    };
    let status = find_status_object(root).unwrap_or(root);
    state.power = text(status, &["POWER", "POWER1"]).and_then(power);
    state.dimmer = number(status, &["Dimmer"])
        .filter(|value| *value <= 100)
        .map(|value| value as u8);
    if let Some(channels) = text(status, &["Color"]).and_then(color) {
        if channels.len() >= 6 {
            state.rgb_color = Some(format!("#{}", &channels[..6]));
        }
        state.white_channels = match channels.len() {
            4 => Some(channels),
            10 => Some(channels[6..].to_owned()),
            _ => None,
        };
    }
    state.ct = number(status, &["CT", "ColorTemperature"])
        .filter(|value| (153..=500).contains(value))
        .map(|value| value as u16);
    state
}

fn find_status_object(values: &Map<String, Value>) -> Option<&Map<String, Value>> {
    if let Some(status) = object(values, "StatusSTS") {
        return Some(status);
    }
    if has_status_fields(values) {
        return Some(values);
    }
    values.values().find_map(|value| match value {
        Value::Object(nested) => find_status_object(nested),
        Value::Array(items) => items
            .iter()
            .find_map(|item| item.as_object().and_then(find_status_object)),
        _ => None,
    })
}

fn has_status_fields(values: &Map<String, Value>) -> bool {
    values.keys().any(|key| {
        [
            "POWER",
            "POWER1",
            "Dimmer",
            "Color",
            "CT",
            "ColorTemperature",
        ]
        .iter()
        .any(|alias| key.eq_ignore_ascii_case(alias))
    })
}

fn object<'a>(values: &'a Map<String, Value>, alias: &str) -> Option<&'a Map<String, Value>> {
    values.iter().find_map(|(key, value)| {
        key.eq_ignore_ascii_case(alias)
            .then(|| value.as_object())
            .flatten()
    })
}

fn value<'a>(values: &'a Map<String, Value>, aliases: &[&str]) -> Option<&'a Value> {
    aliases.iter().find_map(|alias| {
        values
            .iter()
            .find_map(|(key, value)| key.eq_ignore_ascii_case(alias).then_some(value))
    })
}

fn text<'a>(values: &'a Map<String, Value>, aliases: &[&str]) -> Option<&'a str> {
    value(values, aliases)?.as_str()
}

fn number(values: &Map<String, Value>, aliases: &[&str]) -> Option<u64> {
    value(values, aliases)?
        .as_u64()
        .or_else(|| value(values, aliases)?.as_str()?.parse().ok())
}

fn power(value: &str) -> Option<String> {
    match value.to_ascii_uppercase().as_str() {
        "ON" => Some("on".into()),
        "OFF" => Some("off".into()),
        _ => None,
    }
}

fn color(value: &str) -> Option<String> {
    let value = value.trim();
    if value.contains(',') {
        let mut hex = String::new();
        for channel in value.splitn(6, ',') {
            write!(hex, "{:02X}", channel.trim().parse::<u8>().ok()?).ok()?;
        }
        return matches!(hex.len(), 4 | 6 | 8 | 10).then_some(hex);
    }
    let hex = value.strip_prefix('#').unwrap_or(value);
    (matches!(hex.len(), 4 | 6 | 8 | 10) && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| hex.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn status_reads_fields_from_nested_fallback_object() {
        let state =
            status(&json!({"wrapper":{"light":{"POWER1":"ON","Dimmer":42,"Color":"ff8000"}}}));

        assert_eq!(
            (
                state.power,
                state.dimmer,
                state.rgb_color,
                state.white_channels
            ),
            (Some("on".into()), Some(42), Some("#FF8000".into()), None)
        );
    }

    #[test]
    fn status_splits_rgb_and_white_channels() {
        let state = status(&json!({
            "POWER": "OFF",
            "StatusSTS": {
                "POWER": "ON",
                "Dimmer": 89,
                "Color": "E38C5240BF",
                "CT": 500
            }
        }));

        assert_eq!(state.power.as_deref(), Some("on"));
        assert_eq!(state.rgb_color.as_deref(), Some("#E38C52"));
        assert_eq!(state.white_channels.as_deref(), Some("40BF"));
        assert_eq!(state.ct, Some(500));
    }

    #[test]
    fn status_accepts_decimal_color_and_preserves_zero_channels() {
        let state = status(&json!({"Color":"0,0,0,255,0", "CT":"153"}));
        assert_eq!(state.rgb_color.as_deref(), Some("#000000"));
        assert_eq!(state.white_channels.as_deref(), Some("FF00"));
        assert_eq!(state.ct, Some(153));

        let state = status(&json!({"Color":"#ff00000000", "CT":0}));
        assert_eq!(state.rgb_color.as_deref(), Some("#FF0000"));
        assert_eq!(state.white_channels.as_deref(), Some("0000"));
        assert_eq!(state.ct, None);
    }

    #[test]
    fn status_rejects_malformed_color_and_out_of_range_ct() {
        let state = status(&json!({"Color":"256,0,0,0,0", "CT":65536}));
        assert_eq!(
            (state.rgb_color, state.white_channels, state.ct),
            (None, None, None)
        );
        assert_eq!(status(&json!({"Color":"12345z6789"})).rgb_color, None);
        assert_eq!(status(&json!({"CT":333})).ct, Some(333));
    }
}
