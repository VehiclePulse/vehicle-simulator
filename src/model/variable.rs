use chrono::DateTime;
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::model::telemetry::TelemetryState;

#[derive(Debug, Serialize)]
pub struct Variable {
    vehicle_id: Uuid,
    speed: f64,
    heading: f64,
    latitude: f64,
    longitude: f64,
    timestamp: DateTime<Utc>,
}

impl From<&TelemetryState> for Variable {
    fn from(telemetry: &TelemetryState) -> Self {
        Self {
            vehicle_id: telemetry.vehicle_id,
            speed: telemetry.speed,
            heading: telemetry.heading,
            latitude: telemetry.latitude,
            longitude: telemetry.longitude,
            timestamp: telemetry.last_updated_at,
        }
    }
}
