pub fn update_engine_temperature(
    temperature: &mut f64,
    speed: f64,
    elapsed_seconds: f64,
) {
    let temperature_change = if (40.0..=70.0).contains(&speed) {
        // Gradual temperature change.
        0.005
    } else if speed < 40.0 {
        // Faster change below 40 km/h.
        let speed_deviation = 40.0 - speed;

        0.005 + speed_deviation * 0.002
    } else {
        // Faster change above 70 km/h.
        let speed_deviation = speed - 70.0;

        0.005 + speed_deviation * 0.003
    };

    *temperature += temperature_change * elapsed_seconds;

    *temperature = temperature.clamp(20.0, 120.0);
}