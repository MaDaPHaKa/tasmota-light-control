use crate::ports::bulb_controller::Command;

pub fn properties(
    dimmer: Option<u8>,
    mode: Option<&str>,
    rgb: Option<&str>,
    kelvin: Option<u16>,
    fade: Option<u8>,
    speed: Option<u8>,
) -> Command {
    let color = (mode == Some("rgb")).then(|| rgb.unwrap_or("#000000").to_owned());
    let ct = (mode == Some("color_temperature")).then(|| {
        let kelvin = u32::from(kelvin.unwrap_or(2000));
        ((1_000_000 + kelvin / 2) / kelvin) as u16
    });
    Command::ApplyProperties {
        dimmer,
        color,
        ct,
        fade,
        speed,
    }
}
