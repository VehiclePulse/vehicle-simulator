use crate::model::telemetry::{
    TelemetryState
};
use rand::RngExt;

pub fn update_starting_speed(
    telemetry: &mut TelemetryState,
    rng: &mut impl RngExt,
) {
    let change = rng.random_range(2.0..8.0);

    telemetry.speed += change;

    // Startup must remain below 40 km/h.
    telemetry.speed = telemetry.speed.min(39.0);
}


pub fn update_speed(
    telemetry: &mut TelemetryState,
    rng: &mut impl RngExt,
    elapsed_seconds: f64,
) {
    let event = rng.random_range(1..=20);

    let acceleration = match event {
        // Normal acceleration: +1 to +4 km/h/s
        1..=6 => {
            let value = rng.random_range(1.0..4.0);

            println!(
                "Accelerating: +{value:.2} km/h/s"
            );

            value
        }

        // Normal deceleration: -1 to -4 km/h/s
        7..=12 => {
            let value = rng.random_range(-4.0..-1.0);

            println!(
                "Decelerating: {value:.2} km/h/s"
            );

            value
        }

        // Cruising: small positive or negative change
        13..=16 => {
            let value = rng.random_range(-1.5..1.5);

            println!(
                "Cruising: {value:+.2} km/h/s"
            );

            value
        }

        // Harsh acceleration: +7 to +12 km/h/s
        17..=18 => {
            let value = rng.random_range(7.0..12.0);

            println!(
                "Harsh acceleration: +{value:.2} km/h/s"
            );

            value
        }

        // Harsh braking: -7 to -12 km/h/s
        19..=20 => {
            let value = rng.random_range(-12.0..-7.0);

            println!(
                "Harsh braking: {value:.2} km/h/s"
            );

            value
        }

        _ => unreachable!(),
    };

    telemetry.speed += acceleration * elapsed_seconds;

    telemetry.speed = telemetry.speed.clamp(0.0, 120.0);
}


pub fn update_stopping_speed(
    telemetry: &mut TelemetryState,
    stop_elapsed: &mut f64,
    elapsed_seconds: f64,
) {
    const STOP_DURATION: f64 = 10.0;

    *stop_elapsed += elapsed_seconds;

    let progress = (*stop_elapsed / STOP_DURATION).clamp(0.0, 1.0);

    telemetry.speed *= 1.0 - progress;

    if progress >= 1.0 {
        telemetry.speed = 0.0;
    }
}