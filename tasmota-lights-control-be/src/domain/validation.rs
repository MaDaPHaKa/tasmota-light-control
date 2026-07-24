use crate::{
    domain::{profile::LightInput, settings::Settings},
    error::{AppError, AppResult, validation},
};
use std::collections::BTreeMap;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
pub fn name(input: String) -> AppResult<(String, String)> {
    let display = input.trim().to_owned();
    if display.is_empty() || display.chars().count() > 80 {
        return Err(validation("name", "Name must contain 1 to 80 characters"));
    }
    Ok((display.to_lowercase(), display))
}
pub type ValidLight = (String, String, u8, String, Option<String>, Option<u16>);
pub fn light(input: LightInput) -> AppResult<ValidLight> {
    let (normalized, name) = name(input.name)?;
    let mut fields = BTreeMap::new();
    if !(1..=100).contains(&input.dimmer) {
        fields.insert("dimmer".into(), "Dimmer must be between 1 and 100".into());
    }
    let rgb = input.rgb_color.0.map(|value| value.to_uppercase());
    let valid_rgb = |value: &str| {
        value.len() == 7
            && value.starts_with('#')
            && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    };
    match input.mode.as_str() {
        "rgb"
            if rgb.as_deref().is_some_and(valid_rgb)
                && input.color_temperature_kelvin.0.is_none() => {}
        "color_temperature"
            if rgb.is_none()
                && input
                    .color_temperature_kelvin
                    .0
                    .is_some_and(|value| (3000..=6000).contains(&value)) => {}
        "rgb" => {
            fields.insert(
                "rgbColor".into(),
                "RGB mode requires #RRGGBB and null temperature".into(),
            );
        }
        "color_temperature" => {
            fields.insert(
                "colorTemperatureKelvin".into(),
                "Color temperature mode requires 3000 through 6000 and null RGB".into(),
            );
        }
        _ => {
            fields.insert(
                "mode".into(),
                "Mode must be rgb or color_temperature".into(),
            );
        }
    }
    if fields.is_empty() {
        Ok((
            normalized,
            name,
            input.dimmer,
            input.mode,
            rgb,
            input.color_temperature_kelvin.0,
        ))
    } else {
        Err(AppError::Validation(fields))
    }
}
pub fn settings(input: Settings) -> AppResult<(u8, String, Option<String>, Option<u16>)> {
    let (_, _, dimmer, mode, rgb, kelvin) = light(LightInput {
        name: "settings".into(),
        dimmer: input.dimmer,
        mode: input.mode,
        rgb_color: input.rgb_color,
        color_temperature_kelvin: input.color_temperature_kelvin,
    })?;
    Ok((dimmer, mode, rgb, kelvin))
}
pub fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

pub fn timestamp(value: &str) -> bool {
    OffsetDateTime::parse(value, &Rfc3339).is_ok()
}
