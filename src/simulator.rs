mod coordindate;
mod fuel;
mod speed;
mod temperature;

use std::time::{Duration, Instant};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time,
};

use crate::{
    model::{
        TelemetryMessage, alarm::AlarmGenerator, event::Event, shared::state::VehicleMode, telemetry::{InitialTelemetry, TelemetryState}, variable::Variable,
    }, service_manager::service::ServiceManager,
};

pub struct Simulator {
    interval: Duration,
    stop_sender: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl Simulator {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            stop_sender: None,
            task: None,
        }
    }

    pub fn start(
        &mut self,
        initial_telemetry: InitialTelemetry,
    ) -> Result<mpsc::Receiver<TelemetryMessage>, &'static str> {
        if self.task.is_some() {
            return Err("simulation is already running");
        }

        let (telemetry_sender, telemetry_receiver) = mpsc::channel(100);
        let (stop_sender, mut stop_receiver) = oneshot::channel();

        let interval = self.interval;

        let telemetry_state = TelemetryState::from(initial_telemetry);

        let task = tokio::spawn(async move {
            run_simulation(
                telemetry_state,
                interval,
                telemetry_sender,
                &mut stop_receiver,
            )
            .await;
        });

        self.stop_sender = Some(stop_sender);
        self.task = Some(task);

        Ok(telemetry_receiver)
    }

    pub async fn stop(&mut self) -> Result<(), &'static str> {
        if let Some(task) = self.task.as_ref() {
            if task.is_finished() {
                self.task.take();
                self.stop_sender.take();
                return Ok(());
            }
        }

        let Some(stop_sender) = self.stop_sender.take() else {
            return Err("simulation not running");
        };

        let _ = stop_sender.send(());

        if let Some(task) = self.task.take() {
            let _ = task.await;
        }

        Ok(())
    }
}

async fn run_simulation(
    mut telemetry_state: TelemetryState,
    interval: Duration,
    telemetry_sender: mpsc::Sender<TelemetryMessage>,
    stop_receiver: &mut oneshot::Receiver<()>,
) {
    let mut ticker = time::interval(interval);

    ticker.tick().await;

    let mut last_update = Instant::now();

    let mut alarm_generator = AlarmGenerator::new();
    let mut service_manager = ServiceManager::new();

    {
        let mut rng = rand::rng();

        speed::update_starting_speed(&mut telemetry_state, &mut rng);
    }

    let mut stopping = false;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let now = Instant::now();

                let elapsed = now.duration_since(last_update);
                last_update = now;

                let elapsed_seconds = elapsed.as_secs_f64();

                if stopping && telemetry_state.speed == 0.0 {
                    let stopped = speed::update_stopping_speed(
                        &mut telemetry_state,
                        elapsed_seconds,
                    );

                    telemetry_state.last_updated_at = chrono::Utc::now();

                    let variable = TelemetryMessage::Variable(
                        Variable::from(&telemetry_state)
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
                    &mut telemetry_state,
                    elapsed_seconds,
                );

                telemetry_state.last_updated_at = chrono::Utc::now();

                let alarms = alarm_generator.generate(
                    elapsed_seconds,
                    &telemetry_state,
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
                    if telemetry_state.speed > 0.0 {
                        telemetry_state.mode = VehicleMode::Stopping;
                    } else {
                        service_manager.servicing(
                            &mut telemetry_state,
                            elapsed_seconds,
                        );
                    }
                } else {
                    service_manager.check_targets(
                        &telemetry_state,
                    );
                }

                let variable =
                    TelemetryMessage::Variable(
                        Variable::from(&telemetry_state)
                    );

                if telemetry_sender
                    .send(variable)
                    .await
                    .is_err()
                {
                    break;
                }
            }

            _ = &mut *stop_receiver, if !stopping => {
                stopping = true;
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
