use crate::ports::bulb_controller::Command;

pub fn command(dimmer: u8, mode: &str, rgb: Option<&str>, kelvin: Option<u16>) -> Command {
    match mode {
        "rgb" => Command::ApplyRgb {
            dimmer,
            color: rgb.unwrap_or("#000000").to_owned(),
        },
        _ => {
            let kelvin = u32::from(kelvin.unwrap_or(3000));
            Command::ApplyColorTemperature {
                dimmer,
                ct: ((1_000_000 + kelvin / 2) / kelvin) as u16,
            }
        }
    }
}
