use crate::ports::bulb_controller::Command;

pub fn properties(dimmer: Option<u8>, color: Option<&str>, kelvin: Option<u16>) -> Command {
    Command::ApplyProperties {
        dimmer,
        color: color.map(str::to_owned),
        ct: kelvin.map(|value| {
            let value = u32::from(value);
            ((1_000_000 + value / 2) / value) as u16
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_color_is_sent_unchanged_before_ratio_preserving_dimmer() {
        let command = properties(Some(72), Some("#88C0D03719"), None);
        assert_eq!(command.text(), "Backlog0 Color #88C0D03719; Dimmer4 72");
    }

    #[test]
    fn all_zero_mixed_color_is_not_brightened() {
        let command = properties(Some(72), Some("#0000000000"), None);
        assert_eq!(command.text(), "Backlog0 Color #0000000000");
    }

    #[test]
    fn rgb_and_white_commands_only_set_color_and_brightness() {
        let rgb = properties(Some(20), Some("#88C0D0"), None);
        let white = properties(Some(20), None, Some(2000));
        assert_eq!(rgb.text(), "Backlog0 Color #88C0D0; Dimmer 20");
        assert_eq!(white.text(), "Backlog0 CT 500; Dimmer 20");
    }
}
