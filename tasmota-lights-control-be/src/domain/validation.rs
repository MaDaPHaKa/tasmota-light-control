use crate::{
    domain::{profile::LightInput, settings::SettingsInput},
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
pub type ValidSettings = (
    u8,
    String,
    Option<String>,
    Option<u16>,
    Option<u8>,
    Option<u8>,
);
pub type ValidProperties = (Option<u8>, Option<String>, Option<u16>);

fn validate_property_values(
    fields: &mut BTreeMap<String, String>,
    dimmer: Option<i64>,
    rgb: Option<&str>,
    kelvin: Option<i64>,
) -> ValidProperties {
    if dimmer.is_some_and(|value| !(1..=100).contains(&value)) {
        fields.insert("dimmer".into(), "Dimmer must be between 1 and 100".into());
    }
    if rgb.is_some_and(|value| {
        value.len() != 7
            || !value.starts_with('#')
            || !value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    }) {
        fields.insert("rgbColor".into(), "RGB color must match #RRGGBB".into());
    }
    if kelvin.is_some_and(|value| !(2000..=6000).contains(&value)) {
        fields.insert(
            "colorTemperatureKelvin".into(),
            "Color temperature must be between 2000 and 6000".into(),
        );
    }
    (
        dimmer.and_then(|value| u8::try_from(value).ok()),
        rgb.map(str::to_uppercase),
        kelvin.and_then(|value| u16::try_from(value).ok()),
    )
}

pub fn light(input: LightInput) -> AppResult<ValidLight> {
    let mut fields = BTreeMap::new();
    let display_name = input.name.trim().to_owned();
    if display_name.is_empty() || display_name.chars().count() > 80 {
        fields.insert("name".into(), "Name must contain 1 to 80 characters".into());
    }
    let rgb = input.rgb_color.0.map(|value| value.to_uppercase());
    let (_, _, _) = validate_property_values(
        &mut fields,
        Some(input.dimmer),
        rgb.as_deref(),
        input.color_temperature_kelvin.0,
    );
    match input.mode.as_str() {
        "rgb" => {
            if rgb.is_none() {
                fields.insert("rgbColor".into(), "RGB color must match #RRGGBB".into());
            }
            if input.color_temperature_kelvin.0.is_some() {
                fields.insert(
                    "colorTemperatureKelvin".into(),
                    "RGB mode requires null color temperature".into(),
                );
            }
        }
        "color_temperature" => {
            if rgb.is_some() {
                fields.insert(
                    "rgbColor".into(),
                    "Color temperature mode requires null RGB color".into(),
                );
            }
            if input.color_temperature_kelvin.0.is_none() {
                fields.insert(
                    "colorTemperatureKelvin".into(),
                    "Color temperature must be between 2000 and 6000".into(),
                );
            }
        }
        "mixed" => {
            if rgb.is_none() {
                fields.insert("rgbColor".into(), "RGB color must match #RRGGBB".into());
            }
            if input.color_temperature_kelvin.0.is_none() {
                fields.insert(
                    "colorTemperatureKelvin".into(),
                    "Color temperature must be between 2000 and 6000".into(),
                );
            }
        }
        _ => {
            fields.insert(
                "mode".into(),
                "Mode must be rgb, color_temperature, or mixed".into(),
            );
        }
    }
    if fields.is_empty() {
        let dimmer = u8::try_from(input.dimmer).map_err(|_| AppError::Internal)?;
        let kelvin = input
            .color_temperature_kelvin
            .0
            .map(u16::try_from)
            .transpose()
            .map_err(|_| AppError::Internal)?;
        Ok((
            display_name.to_lowercase(),
            display_name,
            dimmer,
            input.mode,
            rgb,
            kelvin,
        ))
    } else {
        Err(AppError::Validation(fields))
    }
}
pub fn settings(input: SettingsInput) -> AppResult<ValidSettings> {
    let (_, _, dimmer, mode, rgb, kelvin) = light(LightInput {
        name: "settings".into(),
        dimmer: input.dimmer,
        mode: input.mode,
        rgb_color: input.rgb_color,
        color_temperature_kelvin: input.color_temperature_kelvin,
    })?;
    let mut fields = BTreeMap::new();
    if input.fade.is_some_and(|value| !(0..=1).contains(&value)) {
        fields.insert("fade".into(), "Fade must be 0 or 1".into());
    }
    if input.speed.is_some_and(|value| !(1..=40).contains(&value)) {
        fields.insert("speed".into(), "Speed must be between 1 and 40".into());
    }
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }
    Ok((
        dimmer,
        mode,
        rgb,
        kelvin,
        input
            .fade
            .map(u8::try_from)
            .transpose()
            .map_err(|_| AppError::Internal)?,
        input
            .speed
            .map(u8::try_from)
            .transpose()
            .map_err(|_| AppError::Internal)?,
    ))
}

pub fn properties(
    dimmer: Option<i64>,
    rgb_color: Option<String>,
    color_temperature_kelvin: Option<i64>,
) -> AppResult<(Option<u8>, Option<String>, Option<u16>)> {
    let mut fields = BTreeMap::new();
    if dimmer.is_none() && rgb_color.is_none() && color_temperature_kelvin.is_none() {
        fields.insert(
            "properties".into(),
            "At least one property is required".into(),
        );
    }
    let rgb = rgb_color.map(|value| value.to_uppercase());
    let (dimmer_value, rgb_value, kelvin_value) = validate_property_values(
        &mut fields,
        dimmer,
        rgb.as_deref(),
        color_temperature_kelvin,
    );
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }
    Ok((dimmer_value, rgb_value, kelvin_value))
}
pub fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

pub fn timestamp(value: &str) -> bool {
    OffsetDateTime::parse(value, &Rfc3339).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::RequiredOption;

    #[test]
    fn light_reports_name_dimmer_and_mode_errors_together() {
        let result = light(LightInput {
            name: " ".into(),
            dimmer: 256,
            mode: "invalid".into(),
            rgb_color: RequiredOption(None),
            color_temperature_kelvin: RequiredOption(None),
        });

        let Err(AppError::Validation(fields)) = result else {
            panic!("expected validation error");
        };
        assert_eq!(
            fields.keys().cloned().collect::<Vec<_>>(),
            ["dimmer", "mode", "name"]
        );
    }
}
