use crate::ports::bulb_controller::DeviceState;
use serde_json::{Map, Value};

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
    state.rgb_color = text(status, &["Color"]).and_then(color);
    state.ct = number(status, &["CT", "ColorTemperature"])
        .filter(|value| *value > 0)
        .map(|value| value as u16);
    let explicit = text(status, &["Mode", "ColorMode", "LightMode"]).and_then(mode);
    state.mode = explicit.or_else(|| match (&state.rgb_color, state.ct) {
        (Some(_), None) => Some("rgb".into()),
        (None, Some(_)) => Some("color_temperature".into()),
        _ => None,
    });
    if state.mode.is_none() {
        state.rgb_color = None;
        state.ct = None;
    }
    state
}

fn find_status_object(values: &Map<String, Value>) -> Option<&Map<String, Value>> {
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
            "StatusSTS",
            "POWER",
            "POWER1",
            "Dimmer",
            "Color",
            "CT",
            "ColorTemperature",
            "Mode",
            "ColorMode",
            "LightMode",
        ]
        .iter()
        .any(|alias| key.eq_ignore_ascii_case(alias))
    }) && object(values, "StatusSTS").is_none()
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
    let value = value.trim().strip_prefix('#').unwrap_or(value.trim());
    (value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| format!("#{}", value.to_ascii_uppercase()))
}

fn mode(value: &str) -> Option<String> {
    match value.to_ascii_lowercase().as_str() {
        "rgb" | "color" => Some("rgb".into()),
        "ct" | "color_temperature" | "temperature" => Some("color_temperature".into()),
        _ => None,
    }
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
            (state.power, state.dimmer, state.mode, state.rgb_color),
            (
                Some("on".into()),
                Some(42),
                Some("rgb".into()),
                Some("#FF8000".into())
            )
        );
    }
}
