mod vehicle_simulation;
mod speed;
mod coordindate;
mod fuel;
mod temperature;

use std::{
    collections::HashMap,
    time::{ Duration, Instant }
};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time
};

use uuid::Uuid;

use crate::model::{
    TelemetryMessage,
    event::Event,
    variable::Variable,
    shared::state::VehicleMode,
    telemetry::{
        TelemetryState,
        InitialTelemetry
    }
};



use crate::simulator::{
    vehicle_simulation::VehicleSimulation
};

enum SimulatorCommand {
    AddVehicle {
        initial_telemetry: InitialTelemetry,
        response: oneshot::Sender<Result<(), String>>
    },
    RemoveVehicle {
        vehicle_id: Uuid,
        response: oneshot::Sender<Result<(), String>>
    }
}


pub struct Simulator {
    command_sender: mpsc::Sender<SimulatorCommand>,
    _manager_task: JoinHandle<()>   
}

struct VehicleTask {
    task: JoinHandle<()>,
    stop_sender: Option<oneshot::Sender<()>>
}

impl Simulator {
    pub fn new(
        interval: Duration
    ) -> (Self, mpsc::Receiver<TelemetryMessage>) {
        let (telemetry_sender, telemetry_receiver) = mpsc::channel(100);
        let (command_sender, command_receiver) = mpsc::channel(32);

        let _manager_task = tokio::spawn(
            async move {
                run_manager(command_receiver, telemetry_sender, interval).await;
            }
        );

        (
            Self {
                command_sender,
                _manager_task
            },
            telemetry_receiver
        )
    }

    pub async fn start_vehicle(
        &self,
        initial_telemetry: InitialTelemetry
    ) -> Result<(), String> {
        let (response_sender, response_receiver) = oneshot::channel();

        self.command_sender
            .send(SimulatorCommand::AddVehicle { 
                initial_telemetry, 
                response: response_sender 
            })
            .await
            .map_err(|_| "Simulator manager is unavailable".to_string())?;

        response_receiver
        .await
        .map_err(|_| "Simulator manager dropped the response".to_string())?
    }

    pub async fn stop_vehicle(
        &self,
        vehicle_id: Uuid
    ) -> Result<(), String> {
        let (response_sender, response_receiver) = oneshot::channel();

        self.command_sender
            .send(SimulatorCommand::RemoveVehicle { 
                vehicle_id, 
                response: response_sender 
            })
            .await
            .map_err(|_| "SImulator manager is unavailable.to_string()")?;

        response_receiver
            .await
            .map_err(|_| "SImulator manager dropped the response".to_string())?
    }
}

async fn run_manager(
    mut command_receiver: mpsc::Receiver<SimulatorCommand>,
    telemetry_sender: mpsc::Sender<TelemetryMessage>,
    interval: Duration
) {
    let mut vehicles: HashMap<Uuid, VehicleTask> = HashMap::new();

    while let Some(command) = command_receiver.recv().await {
        vehicles.retain(|_, vehicle| !vehicle.task.is_finished());

        match command {
            SimulatorCommand::AddVehicle { 
                initial_telemetry, 
                response 
            } => {
                let vehicle_id = initial_telemetry.vehicle_id;

                if vehicles.contains_key(&vehicle_id) {
                    let _ = response.send(Err(
                        format!("Vehicle {vehicle_id} is already running")
                    ));

                    continue;
                }

                let vehicle = VehicleSimulation::new(initial_telemetry);

                let (stop_sender, stop_receiver) = oneshot::channel();

                let task = tokio::spawn(run_vehicle_simulation(
                    vehicle,
                    interval,
                    telemetry_sender.clone(),
                    stop_receiver
                ));

                vehicles.insert(vehicle_id, VehicleTask { task, stop_sender: Some(stop_sender) });

                let _ = response.send(Ok(()));
            }

            SimulatorCommand::RemoveVehicle {
                vehicle_id,
                response,
            } => {
                match vehicles.get_mut(&vehicle_id) {
                    Some(vehicle) => {
                        // Tell the vehicle to begin decelerating.
                        if let Some(stop_sender) = vehicle.stop_sender.take() {
                            let _ = stop_sender.send(());
                        }

                        // Wait until the vehicle has stopped.
                        let task = vehicles.remove(&vehicle_id).unwrap().task;
                        let result = task.await;

                        if result.is_ok() {
                            let _ = response.send(Ok(()));
                        } else {
                            let _ = response.send(Err(
                                "Vehicle simulation task failed".to_string()
                            ));
                        }
                    }
                    None => {
                        let _ = response.send(Err(format!(
                            "Vehicle {vehicle_id} is not running"
                        )));
                    }
                }
            }
        }
    }


    for (_, vehicle) in vehicles {
        vehicle.task.abort();
        let _ = vehicle.task.await;
    }
}

async fn run_vehicle_simulation(
    vehicle: VehicleSimulation,
    interval: Duration,
    telemetry_sender: mpsc::Sender<TelemetryMessage>,
    mut stop_receiver: oneshot::Receiver<()>
) {
    let VehicleSimulation {
        mut telemetry,
        mut alarm_generator,
        mut service_manager
    } = vehicle;

    let mut ticker = time::interval(interval);

    ticker.tick().await;

    let mut last_update = Instant::now();

    {
        let mut rng = rand::rng();

        speed::update_starting_speed(&mut telemetry, &mut rng);
    }

    let mut stopping = false;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let now = Instant::now();

                let elapsed = now.duration_since(last_update);
                last_update = now;

                let elapsed_seconds = elapsed.as_secs_f64();

                if stopping {
                    let stopped = speed::update_stopping_speed(
                        &mut telemetry,
                        elapsed_seconds,
                    );

                    telemetry.last_updated_at = chrono::Utc::now();

                    let variable = TelemetryMessage::Variable(
                        Variable::from(&telemetry)
                    );

                    if telemetry_sender
                        .send(variable)
                        .await
                        .is_err()
                    {
                        break;
                    }

                    if stopped {
                        break;
                    }

                    continue;
                }

                update_vehicle(
                    &mut telemetry,
                    elapsed_seconds,
                );

                telemetry.last_updated_at = chrono::Utc::now();

                let alarms = alarm_generator.generate(
                    elapsed_seconds,
                    &telemetry,
                );

                for alarm in alarms {
                    {
                        let mut rng = rand::rng();

                        service_manager.handle_alarm(
                            &alarm,
                            &mut rng,
                        );
                    }

                    if let Some(event) = Event::from_alarm(&alarm) {
                        if telemetry_sender
                            .send(TelemetryMessage::Event(event))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }

                    if telemetry_sender
                        .send(TelemetryMessage::Alarm(alarm))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }

                if service_manager.has_active_service() {
                    if telemetry.speed > 0.0 {
                        telemetry.mode = VehicleMode::Stopping;
                    } else {
                        if let Some(event) = service_manager.servicing(
                            &mut telemetry,
                            elapsed_seconds,
                        ) {
                            if telemetry_sender
                                .send(TelemetryMessage::Event(event))
                                .await
                                .is_err()
                                {
                                    return;
                                }
                        };

                        //let event = event::Event.from_alarm();
                    }
                } else {
                    service_manager.check_targets(
                        &telemetry,
                    );
                    //println!("is Active: {:?}", is_active);

                    // if !is_active {
                    //     telemetry_state.mode = VehicleMode::Driving;
                    // }
                }

                let variable =
                    TelemetryMessage::Variable(
                        Variable::from(&telemetry)
                    );

                if telemetry_sender
                    .send(variable)
                    .await
                    .is_err()
                {
                    break;
                }
            }

            _ = &mut stop_receiver, if !stopping => {
                stopping = true;
                telemetry.mode = VehicleMode::Stopping;
            }
        }
    }
}


fn update_vehicle(telemetry: &mut TelemetryState, elapsed_seconds: f64) {
    let mut rng = rand::rng();

    match telemetry.mode {
        VehicleMode::Driving => speed::update_speed(telemetry, &mut rng, elapsed_seconds),
        VehicleMode::Stopping => _ = speed::update_stopping_speed(telemetry, elapsed_seconds), 
    }

    coordindate::update_coordinates(
        telemetry.speed,
        elapsed_seconds,
        telemetry.heading,
        &mut telemetry.latitude,
        &mut telemetry.longitude,
    );

    fuel::update_fuel(&mut telemetry.fuel_level, telemetry.speed, elapsed_seconds);
    //println!("fuel level: {:?}", telemetry.fuel_level);

    temperature::update_engine_temperature(
        &mut telemetry.engine_temperature,
        telemetry.speed,
        elapsed_seconds,
    );

    update_battery(&mut telemetry.battery_level, elapsed_seconds);

    update_tire_pressure(&mut telemetry.tire_pressure, elapsed_seconds);
}

fn update_battery(battery_level: &mut f64, elapsed_seconds: f64) {
    let consumption_per_second = 0.0002;

    *battery_level -= consumption_per_second * elapsed_seconds;
    *battery_level = battery_level.clamp(0.0, 100.0);
}

fn update_tire_pressure(tire_pressure: &mut crate::model::TirePressure, elapsed_seconds: f64) {
    let pressure_change = 0.00001 * elapsed_seconds;

    tire_pressure.front_left = (tire_pressure.front_left - pressure_change).max(0.0);

    tire_pressure.front_right = (tire_pressure.front_right - pressure_change).max(0.0);

    tire_pressure.rear_left = (tire_pressure.rear_left - pressure_change).max(0.0);

    tire_pressure.rear_right = (tire_pressure.rear_right - pressure_change).max(0.0);
}
