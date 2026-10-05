use std::time::Duration;

use uuid::Uuid;
use vehicle_simulator::{
    model::{InitialTelemetry, TelemetryMessage, TirePressure},
    simulator::Simulator,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let initial_telemetry = InitialTelemetry {
        vehicle_id: Uuid::new_v4(),
        latitude: 12.9716,
        longitude: 77.5946,
        speed: 0.0,
        heading: 90.0,
        fuel_level: 30.01,
        engine_temperature: 85.0,
        battery_level: 80.0,
        tire_pressure: TirePressure {
            front_left: 32.0,
            front_right: 32.0,
            rear_left: 32.0,
            rear_right: 32.0,
        },
        mode: vehicle_simulator::model::shared::state::VehicleMode::Driving
    };

    let mut simulator = Simulator::new(Duration::from_secs(1));

    let mut receiver = simulator
        .start(initial_telemetry)
        .expect("failed to start simulator");

    for _ in 0..100 {
        match receiver.recv().await {
            Some(TelemetryMessage::Variable(variable)) => {
                println!("VARIABLE: {variable:#?}");
            }

            Some(TelemetryMessage::Alarm(alarm)) => {
                println!("ALARM: {alarm:#?}");
            }

            Some(TelemetryMessage::Event(event)) => {
                println!("EVENT: {event:#?}");
            }

            None => {
                println!("Telemetry channel closed.");
                break;
            }
        }
    }

    simulator.stop().await.expect("failed to stop simulator");

    while let Ok(message) = receiver.try_recv() {
        println!("{message:#?}");
    }

    println!("Simulator stopped.");
}
