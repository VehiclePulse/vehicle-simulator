use crate::model::{shared::state, telemetry::TelemetryState};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Alarm {
    pub vehicle_id: Uuid,
    pub alarm_kind: state::IssueKind,
    pub alarm_value: String,
    pub severity: state::Severity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub information: Option<f64>,
    pub warning: Option<f64>,
    pub critical: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct DrivingThresholds {
    //Units: km/h/s
    //pub cruising: f64,
    pub acceleration: f64,
    pub warning_acceleration: f64,
    pub critical_acceleration: f64,
    pub deceleration: f64,
    pub warning_deceleration: f64,
    pub critical_deceleration: f64,
}

impl Default for DrivingThresholds {
    fn default() -> Self {
        Self {
            // Absolute acceleration below this is cruising.
            //cruising: 1.5,

            // Positive acceleration.
            acceleration: 4.0,
            warning_acceleration: 7.0,
            critical_acceleration: 10.0,

            // Negative acceleration magnitude.
            deceleration: 4.0,
            warning_deceleration: 7.0,
            critical_deceleration: 10.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThresholdDirection {
    Below,
    Above,
}

pub struct AlarmGenerator {
    previous_values: std::collections::HashMap<state::IssueKind, f64>,
    previous_speed: Option<f64>,
    driving_threshold: DrivingThresholds,
}

impl AlarmGenerator {
    pub fn new() -> Self {
        Self {
            previous_values: std::collections::HashMap::new(),
            previous_speed: None,
            driving_threshold: DrivingThresholds::default(),
        }
    }

    pub(crate) fn generate(
        &mut self,
        elasped_second: f64,
        telemetry: &TelemetryState,
    ) -> Vec<Alarm> {
        let mut alarms = Vec::new();

        //fuel
        self.check_threshold(
            telemetry,
            state::IssueKind::FuelLevel,
            telemetry.fuel_level,
            Thresholds {
                information: Some(100.0),
                warning: Some(30.0),
                critical: Some(10.0),
            },
            ThresholdDirection::Below,
            &mut alarms,
        );

        // Battery
        self.check_threshold(
            telemetry,
            state::IssueKind::BatteryLevel,
            telemetry.battery_level,
            Thresholds {
                information: Some(50.0),
                warning: Some(30.0),
                critical: Some(10.0),
            },
            ThresholdDirection::Below,
            &mut alarms,
        );

        // Engine temperature
        self.check_threshold(
            telemetry,
            state::IssueKind::EngineTemperature,
            telemetry.engine_temperature,
            Thresholds {
                information: Some(90.0),
                warning: Some(105.0),
                critical: Some(120.0),
            },
            ThresholdDirection::Above,
            &mut alarms,
        );

        // Tire pressure
        for tire in [
            telemetry.tire_pressure.front_left,
            telemetry.tire_pressure.front_right,
            telemetry.tire_pressure.rear_left,
            telemetry.tire_pressure.rear_right,
        ] {
            self.check_threshold(
                telemetry,
                state::IssueKind::TirePressure,
                tire,
                Thresholds {
                    information: Some(32.0),
                    warning: Some(28.0),
                    critical: Some(24.0),
                },
                ThresholdDirection::Below,
                &mut alarms,
            );
        }

        self.check_driving_behavior(telemetry, elasped_second, &mut alarms);

        self.previous_speed = Some(telemetry.speed);

        alarms
    }

    fn check_threshold(
        &mut self,
        telemetry: &TelemetryState,
        issue_kind: state::IssueKind,
        current: f64,
        thresholds: Thresholds,
        direction: ThresholdDirection,
        alarms: &mut Vec<Alarm>,
    ) {
        let previous = self.previous_values.insert(issue_kind, current);

        let Some(previous) = previous else {
            return;
        };

        let severity = match direction {
            ThresholdDirection::Below => {
                if crossed_down(previous, current, thresholds.critical) {
                    Some(state::Severity::Critical)
                } else if crossed_down(previous, current, thresholds.warning) {
                    Some(state::Severity::Warning)
                } else if crossed_down(previous, current, thresholds.information) {
                    Some(state::Severity::Information)
                } else {
                    None
                }
            }

            ThresholdDirection::Above => {
                if crossed_up(previous, current, thresholds.critical) {
                    Some(state::Severity::Critical)
                } else if crossed_up(previous, current, thresholds.warning) {
                    Some(state::Severity::Warning)
                } else if crossed_up(previous, current, thresholds.information) {
                    Some(state::Severity::Information)
                } else {
                    None
                }
            }
        };

        if let Some(severity) = severity {
            alarms.push(Alarm {
                vehicle_id: telemetry.vehicle_id,
                alarm_kind: issue_kind,
                alarm_value: format!("{current:.2}"),
                severity,
                timestamp: telemetry.last_updated_at,
            });
        }
    }

    fn check_driving_behavior(
        &self,
        telemetry: &TelemetryState,
        elapsed_second: f64,
        alarms: &mut Vec<Alarm>,
    ) {
        let Some(previous_speed) = self.previous_speed else {
            return;
        };

        // Protect against an extremely small or invalid time interval.
        if elapsed_second <= 0.001 {
            return;
        }

        let thresholds = self.driving_threshold;

        let speed_change = telemetry.speed - previous_speed;
        let acceleration = if elapsed_second > 0.0 {
            speed_change / elapsed_second
        } else {
            0.0
        };

        let (issue_kind, severity) = if acceleration >= thresholds.critical_acceleration {
            (
                state::IssueKind::HarshAcceleration,
                state::Severity::Critical,
            )
        } else if acceleration >= thresholds.warning_acceleration {
            (
                state::IssueKind::HarshAcceleration,
                state::Severity::Warning,
            )
        } else if acceleration >= thresholds.acceleration {
            (state::IssueKind::Acceleration, state::Severity::Information)
        } else if acceleration <= -thresholds.critical_deceleration {
            (state::IssueKind::HarshBrake, state::Severity::Critical)
        } else if acceleration <= -thresholds.warning_deceleration {
            (state::IssueKind::HarshBrake, state::Severity::Warning)
        } else if acceleration <= -thresholds.deceleration {
            (state::IssueKind::Deceleration, state::Severity::Information)
        } else {
            // Small speed changes are simply normal driving.
            return;
        };

        alarms.push(Alarm {
            vehicle_id: telemetry.vehicle_id,
            alarm_kind: issue_kind,
            alarm_value: format!("{:.2} km/h/s", acceleration.abs()),
            severity,
            timestamp: telemetry.last_updated_at,
        });
    }
}

fn crossed_down(previous: f64, current: f64, threshold: Option<f64>) -> bool {
    let Some(threshold_val) = threshold else {
        return false;
    };

    previous >= threshold_val && current < threshold_val
}

fn crossed_up(previous: f64, current: f64, threshold: Option<f64>) -> bool {
    let Some(threshold_val) = threshold else {
        return false;
    };

    previous <= threshold_val && current > threshold_val
}
