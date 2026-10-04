pub fn update_fuel(fuel_level: &mut f64, speed: f64, elapsed_seconds: f64) {
    let fuel_rate = if (40.0..=70.0).contains(&speed) {
        // Efficient speed range.
        0.0005
    } else if speed < 40.0 {
        // Higher consumption below 40 km/h.
        let speed_deviation = 40.0 - speed;

        0.0005 + speed_deviation * 0.00003
    } else {
        // Higher consumption above 70 km/h.
        let speed_deviation = speed - 70.0;

        0.0005 + speed_deviation * 0.00005
    };

    *fuel_level -= fuel_rate * elapsed_seconds;
    *fuel_level = fuel_level.clamp(0.0, 100.0);
}
