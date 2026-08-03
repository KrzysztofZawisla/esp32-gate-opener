use anyhow::Result;

use super::GatePins;

pub fn set_lamp(pins: &mut GatePins, green: bool, red: bool) -> Result<()> {
    if green {
        pins.lamp_green.set_high()?;
        pins.lamp_red.set_low()?;
    } else if red {
        pins.lamp_red.set_high()?;
        pins.lamp_green.set_low()?;
    } else {
        pins.lamp_green.set_low()?;
        pins.lamp_red.set_low()?;
    }
    Ok(())
}