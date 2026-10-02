use std::time::Duration;    

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueKind {
    Acceleration,
    Deceleration,
    Cruising,
    HarshAcceleration,
    HarshBrake,

    FuelLevel,
    EngineTemperature,
    BatteryLevel,
    TirePressure,
}

#[derive(Debug)]
pub enum Severity {
    Information,
    Warning,
    Critical
}

pub enum Servicing {
    refuled,
    radiated,
    recharged,
    Normalize
}

#[derive(Debug, PartialEq)]
enum VehicleState {
    Starting,
    Driving,
    Stopping,
    Stopped,
}

struct VehicleSimulator {
    state: VehicleState,
    elapsed: Duration,
    stop_elapsed: Duration,
}
