use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;


pub struct TelemetryState {
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
}

pub struct TirePressure {
    pub front_left: f64,
    pub front_right: f64,
    pub rear_left: f64,
    pub rear_right: f64,
}