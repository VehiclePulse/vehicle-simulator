use std::time::Duration;

use tokio::time;
use uuid::Uuid;
use vehicle_simulator::{
    model::{
        shared::state::VehicleMode,
        InitialTelemetry,
        TelemetryMessage,
        TirePressure,
    },
    simulator::Simulator,
};

fn create_vehicle(vehicle_id: Uuid, latitude: f64, longitude: f64) -> InitialTelemetry {
    InitialTelemetry {
        vehicle_id,
        registration_number: String::from("KA01AB1001"),
        latitude,
        longitude,
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
        mode: VehicleMode::Driving,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (simulator, mut receiver) = Simulator::new(Duration::from_secs(1));

    // Start the first vehicle.
    let vehicle1_id = Uuid::new_v4();

    simulator
        .start_vehicle(create_vehicle(vehicle1_id, 12.9716, 77.5946))
        .await
        .expect("Failed to start vehicle 1");

    println!("Vehicle 1 started: {vehicle1_id}");

    // Print telemetry while the simulation runs.
    let reader = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            match message {
                TelemetryMessage::Variable(variable) => {
                    println!("VARIABLE: {variable:#?}");
                }
                TelemetryMessage::Alarm(alarm) => {
                    println!("ALARM: {alarm:#?}");
                }
                TelemetryMessage::Event(event) => {
                    println!("EVENT: {event:#?}");
                }
            }
        }

        println!("Telemetry channel closed.");
    });

    // Wait 10 seconds before starting the second vehicle.
    time::sleep(Duration::from_secs(10)).await;

    let vehicle2_id = Uuid::new_v4();

    simulator
        .start_vehicle(create_vehicle(vehicle2_id, 13.0827, 80.2707))
        .await
        .expect("Failed to start vehicle 2");

    println!("Vehicle 2 started: {vehicle2_id}");

    // Continue until 30 seconds have elapsed since the first vehicle started.
    time::sleep(Duration::from_secs(20)).await;

    println!("Stopping vehicle 1...");
    simulator
        .stop_vehicle(vehicle1_id)
        .await
        .expect("Failed to stop vehicle 1");

    println!("Stopping vehicle 2...");
    simulator
        .stop_vehicle(vehicle2_id)
        .await
        .expect("Failed to stop vehicle 2");

    println!("Both vehicles stopped.");

    reader.abort();
}