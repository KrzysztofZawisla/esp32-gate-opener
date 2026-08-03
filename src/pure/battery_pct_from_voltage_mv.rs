pub fn battery_pct_from_voltage_mv(voltage_mv: f32, full_mv: f32, empty_mv: f32) -> u8 {
    if full_mv <= empty_mv {
        return 0;
    }
    let pct = (voltage_mv - empty_mv) / (full_mv - empty_mv) * 100.0;
    pct.clamp(0.0, 100.0) as u8
}