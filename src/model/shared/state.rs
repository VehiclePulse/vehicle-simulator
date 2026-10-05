use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum IssueKind {
    Acceleration,
    Deceleration,
    //Cruising,
    HarshAcceleration,
    HarshBrake,

    FuelLevel,
    EngineTemperature,
    BatteryLevel,
    TirePressure,
}

#[derive(Debug, Serialize)]
pub enum Severity {
    Information,
    Warning,
    Critical,
}

#[derive(Debug, Serialize)]
pub enum Servicing {
    Refuel,
    Radiate,
    Recharge,
    NormalizePressure,
}

pub enum VehicleMode {
    //Starting,
    Driving,
    Stopping,
}

impl IssueKind {
    pub fn servicing(self) -> Option<Servicing> {
        match self {
            IssueKind::FuelLevel => Some(Servicing::Refuel),
            IssueKind::EngineTemperature => Some(Servicing::Radiate),
            IssueKind::BatteryLevel => Some(Servicing::Recharge),
            IssueKind::TirePressure => Some(Servicing::NormalizePressure),
            _ => None,
        }
    }
}

impl Servicing {
    pub fn get_issue(&self) -> IssueKind {
        match self {
            Servicing::Refuel => IssueKind::FuelLevel,
            Servicing::Recharge => IssueKind::BatteryLevel,
            Servicing::Radiate => IssueKind::EngineTemperature,
            Servicing::NormalizePressure => IssueKind::TirePressure
        }
    }
}