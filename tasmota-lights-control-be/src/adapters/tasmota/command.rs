use crate::ports::bulb_controller::Command;

pub fn properties(
    dimmer: Option<u8>,
    color: Option<&str>,
    kelvin: Option<u16>,
    fade: Option<u8>,
    speed: Option<u8>,
) -> Command {
    Command::ApplyProperties {
        dimmer,
        color: color.map(str::to_owned),
        ct: kelvin.map(|value| {
            let value = u32::from(value);
            ((1_000_000 + value / 2) / value) as u16
        }),
        fade,
        speed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_color_is_sent_unchanged_before_ratio_preserving_dimmer() {
        let command = properties(Some(72), Some("#88C0D03719"), None, Some(1), Some(4));
        assert_eq!(
            command.text(),
            "Backlog0 Fade 1; Speed 4; Color #88C0D03719; Dimmer4 72"
        );
    }

    #[test]
    fn all_zero_mixed_color_is_not_brightened() {
        let command = properties(Some(72), Some("#0000000000"), None, None, None);
        assert_eq!(command.text(), "Backlog0 Color #0000000000");
    }
}
