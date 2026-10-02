mod model;
mod simulator;

use std::{
    thread, time::{Duration, Instant}
};

use chrono::Utc;
use rand::RngExt;
use uuid::Uuid;

use crate::model::{alarm, telemetry::{ TelemetryState, TirePressure }};
use crate::model::variable;
use crate::simulator::{
    speed,
    coordindate,
    fuel,
    temperature
};

fn main() {
    let tick_rate = Duration::from_secs(1);

    let mut telemetry_state = TelemetryState {
        vehicle_id: Uuid::new_v4(),
        latitude: 45.1234,
        longitude: 74.1263,
        speed: 0.0,
        heading: 45.6,
        fuel_level: 30.01,
        engine_temperature: 45.6,
        battery_level: 75.5,
        tire_pressure: TirePressure {
            front_left: 42.6,
            front_right: 58.9,
            rear_left: 456.8,
            rear_right: 47.2,
        },
        last_updated_at: Utc::now(),
    };

    let mut rng = rand::rng();
    let mut last_tick = Instant::now();

    let startup_duration = rng.random_range(10..=30);
    let mut vehicle_elapsed = 0.0;
    let mut stopping = false;
    let mut iteration = 0;
    let mut stop_elapsed = 0.0;
    let mut alarm = alarm::AlarmGenerator::new();

    loop {
        let elapsed_seconds = last_tick.elapsed().as_secs_f64();
        last_tick = Instant::now();
        if iteration > 35 {
            stopping = true;
        }
        iteration +=1;

        if stopping && telemetry_state.speed == 0.0 {
            break;
        }

        vehicle_elapsed += elapsed_seconds;

        if stopping {
            speed::update_stopping_speed(
                &mut telemetry_state,
                &mut stop_elapsed,
                elapsed_seconds,
            );
        } else if vehicle_elapsed < startup_duration as f64 {
            speed::update_starting_speed(
                &mut telemetry_state,
                &mut rng,
            );
        } else {
            speed::update_speed(
                &mut telemetry_state,
                &mut rng,
                elapsed_seconds
            );
        }

        coordindate::update_coordinates(
            telemetry_state.speed,
            elapsed_seconds,
            telemetry_state.heading,
            &mut telemetry_state.latitude,
            &mut telemetry_state.longitude,
        );

        fuel::update_fuel(
            &mut telemetry_state.fuel_level,
            telemetry_state.speed,
            elapsed_seconds,
        );
        
        temperature::update_engine_temperature(
            &mut telemetry_state.engine_temperature,
            telemetry_state.speed,
            elapsed_seconds,
        );

        telemetry_state.last_updated_at = Utc::now();

        println!("variable: {:#?}", variable::Variable::from(&telemetry_state));
        println!("alarm: {:#?}", alarm.generate(elapsed_seconds, &telemetry_state));
        

        thread::sleep(tick_rate);
    }
}
