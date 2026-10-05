use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

use crate::model::shared::state::VehicleMode;

pub struct InitialTelemetry {
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub heading: f64,
    pub fuel_level: f64,
    pub engine_temperature: f64,
    pub battery_level: f64,
    pub tire_pressure: TirePressure,
    pub mode: VehicleMode
}

pub(crate) struct TelemetryState {
    pub vehicle_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub heading: f64,

    pub fuel_level: f64,
    pub engine_temperature: f64,
    pub battery_level: f64,

    pub tire_pressure: TirePressure,

    pub last_updated_at: DateTime<Utc>,
    pub mode: VehicleMode
}

pub struct TirePressure {
    pub front_left: f64,
    pub front_right: f64,
    pub rear_left: f64,
    pub rear_right: f64,
}

impl From<InitialTelemetry> for TelemetryState {
    fn from(initial: InitialTelemetry) -> Self {
        Self {
            vehicle_id: initial.vehicle_id,
            latitude: initial.latitude,
            longitude: initial.longitude,
            speed: initial.speed,
            heading: initial.heading,
            fuel_level: initial.fuel_level,
            engine_temperature: initial.engine_temperature,
            battery_level: initial.battery_level,
            tire_pressure: initial.tire_pressure,
            last_updated_at: Utc::now(),
            mode: initial.mode
        }
    }
}
