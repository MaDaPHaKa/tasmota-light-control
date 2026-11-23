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
pub type ValidLight = (
    String,
    String,
    u8,
    String,
    Option<String>,
    Option<u16>,
    Option<String>,
);
pub type ValidSettings = (
    u8,
    String,
    Option<String>,
    Option<u16>,
    Option<String>,
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
        rgb.map(str::to_ascii_uppercase),
        kelvin.and_then(|value| u16::try_from(value).ok()),
    )
}

fn validate_mixed_color(
    fields: &mut BTreeMap<String, String>,
    color: Option<&str>,
    has_legacy_fields: bool,
) -> Option<String> {
    let color = color?;
    if has_legacy_fields {
        fields.insert(
            "mixedColor".into(),
            "Mixed color cannot be combined with RGB color or color temperature".into(),
        );
        return None;
    }
    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() != 10 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        fields.insert(
            "mixedColor".into(),
            "Mixed color must be 10 hex digits (RGB, cold white, warm white)".into(),
        );
        return None;
    }
    Some(format!("#{}", hex.to_ascii_uppercase()))
}

fn legacy_mixed_color(rgb: &str, kelvin: u16) -> String {
    // Keep the legacy Kelvin interpolation, but emit the white bytes in Tasmota order.
    let warm = (6000 - u32::from(kelvin)) * 255 / 4000;
    let cold = 255 - warm;
    format!("{rgb}{cold:02X}{warm:02X}")
}

pub fn light(input: LightInput) -> AppResult<ValidLight> {
    let mut fields = BTreeMap::new();
    let display_name = input.name.trim().to_owned();
    if display_name.is_empty() || display_name.chars().count() > 80 {
        fields.insert("name".into(), "Name must contain 1 to 80 characters".into());
    }
    let (dimmer, rgb, kelvin) = validate_property_values(
        &mut fields,
        Some(input.dimmer),
        input.rgb_color.as_deref(),
        input.color_temperature_kelvin,
    );
    let mixed = validate_mixed_color(
        &mut fields,
        input.mixed_color.as_deref(),
        input.rgb_color.is_some() || input.color_temperature_kelvin.is_some(),
    );
    if input.mode != "mixed" && input.mixed_color.is_some() {
        fields.insert(
            "mixedColor".into(),
            "Mixed color requires mixed mode".into(),
        );
    }
    match input.mode.as_str() {
        "rgb" => {
            if rgb.is_none() {
                fields.insert("rgbColor".into(), "RGB color must match #RRGGBB".into());
            }
            if input.color_temperature_kelvin.is_some() {
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
            if input.color_temperature_kelvin.is_none() {
                fields.insert(
                    "colorTemperatureKelvin".into(),
                    "Color temperature must be between 2000 and 6000".into(),
                );
            }
        }
        "mixed" => {
            if input.mixed_color.is_none() && (rgb.is_none() || kelvin.is_none()) {
                fields.insert(
                    "mixedColor".into(),
                    "Provide a mixed color or both RGB color and color temperature".into(),
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
        let (rgb, kelvin, mixed) = if input.mode == "mixed" {
            (
                None,
                None,
                mixed.or_else(|| Some(legacy_mixed_color(rgb.as_deref()?, kelvin?))),
            )
        } else {
            (rgb, kelvin, None)
        };
        Ok((
            display_name.to_lowercase(),
            display_name,
            dimmer.ok_or(AppError::Internal)?,
            input.mode,
            rgb,
            kelvin,
            mixed,
        ))
    } else {
        Err(AppError::Validation(fields))
    }
}
pub fn settings(input: SettingsInput) -> AppResult<ValidSettings> {
    let (_, _, dimmer, mode, rgb, kelvin, mixed) = light(LightInput {
        name: "settings".into(),
        dimmer: input.dimmer,
        mode: input.mode,
        rgb_color: input.rgb_color,
        color_temperature_kelvin: input.color_temperature_kelvin,
        mixed_color: input.mixed_color,
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
        mixed,
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
    mixed_color: Option<String>,
) -> AppResult<ValidProperties> {
    let mut fields = BTreeMap::new();
    if dimmer.is_none()
        && rgb_color.is_none()
        && color_temperature_kelvin.is_none()
        && mixed_color.is_none()
    {
        fields.insert(
            "properties".into(),
            "At least one property is required".into(),
        );
    }
    let (dimmer, rgb, kelvin) = validate_property_values(
        &mut fields,
        dimmer,
        rgb_color.as_deref(),
        color_temperature_kelvin,
    );
    let mixed = validate_mixed_color(
        &mut fields,
        mixed_color.as_deref(),
        rgb_color.is_some() || color_temperature_kelvin.is_some(),
    );
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }
    let (color, kelvin) = match (mixed, rgb, kelvin) {
        (Some(color), _, _) => (Some(color), None),
        (None, Some(rgb), Some(kelvin)) => (Some(legacy_mixed_color(&rgb, kelvin)), None),
        (None, rgb, kelvin) => (rgb, kelvin),
    };
    Ok((dimmer, color, kelvin))
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

    #[test]
    fn light_reports_name_dimmer_and_mode_errors_together() {
        let result = light(LightInput {
            name: " ".into(),
            dimmer: 256,
            mode: "invalid".into(),
            rgb_color: None,
            color_temperature_kelvin: None,
            mixed_color: None,
        });

        let Err(AppError::Validation(fields)) = result else {
            panic!("expected validation error");
        };
        assert_eq!(
            fields.keys().cloned().collect::<Vec<_>>(),
            ["dimmer", "mode", "name"]
        );
    }

    #[test]
    fn properties_normalize_mixed_hex_without_losing_channel_values() {
        let valid = properties(Some(40), None, None, Some("#abcdef12a3".into())).unwrap();
        assert_eq!(valid, (Some(40), Some("#ABCDEF12A3".into()), None));
        for color in ["#88C0D0", "88C0D0123Z", "#88C0D012345", "88C0D01234;"] {
            assert!(matches!(
                properties(None, None, None, Some(color.into())),
                Err(AppError::Validation(fields)) if fields.contains_key("mixedColor")
            ));
        }
        assert!(matches!(
            properties(None, Some("#88C0D0".into()), Some(3000), Some("88C0D01234".into())),
            Err(AppError::Validation(fields)) if fields.contains_key("mixedColor")
        ));
    }

    #[test]
    fn legacy_properties_put_cold_white_before_warm_white() {
        for (kelvin, expected) in [(2000, "#88C0D000FF"), (6000, "#88C0D0FF00")] {
            let (_, color, ct) =
                properties(None, Some("#88C0D0".into()), Some(kelvin), None).unwrap();
            assert_eq!(color.as_deref(), Some(expected));
            assert_eq!(ct, None);
        }
    }
}
