use crate::ports::bulb_controller::Command;

pub fn properties(
    dimmer: Option<u8>,
    mode: Option<&str>,
    rgb: Option<&str>,
    kelvin: Option<u16>,
    fade: Option<u8>,
    speed: Option<u8>,
) -> Command {
    let mixed = mode == Some("mixed") || (mode.is_none() && rgb.is_some() && kelvin.is_some());
    let color = (matches!(mode, Some("rgb" | "mixed")) || mixed)
        .then(|| rgb.unwrap_or("#000000").to_owned());
    let ct = (mode == Some("color_temperature") || (mode.is_none() && !mixed && kelvin.is_some()))
        .then(|| {
            let kelvin = u32::from(kelvin.unwrap_or(2000));
            ((1_000_000 + kelvin / 2) / kelvin) as u16
        });
    let color = if mixed {
        Some(mixed_color(
            rgb.unwrap_or("#000000"),
            kelvin.unwrap_or(3000),
        ))
    } else {
        color
    };
    Command::ApplyProperties {
        dimmer,
        color,
        ct,
        fade,
        speed,
    }
}

fn mixed_color(rgb: &str, kelvin: u16) -> String {
    let kelvin = u32::from(kelvin.clamp(2000, 6000));
    let cold = ((6000 - kelvin) * 255 / 4000) as u8;
    let warm = 255 - cold;
    format!("{rgb}{cold:02X}{warm:02X}")
}
