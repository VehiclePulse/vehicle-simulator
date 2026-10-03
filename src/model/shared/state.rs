#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug)]
pub enum Severity {
    Information,
    Warning,
    Critical
}

#[derive(Debug)]
pub enum Servicing {
    Refuel,
    Radiate,
    Recharge,
    NormalizePressure
}


impl IssueKind {
    pub fn servicing(self) -> Option<Servicing> {
        match self {
            IssueKind::FuelLevel => Some(Servicing::Refuel),
            IssueKind::EngineTemperature => Some(Servicing::Radiate),
            IssueKind::BatteryLevel => Some(Servicing::Recharge),
            IssueKind::TirePressure => Some(Servicing::NormalizePressure),
            _ => None
        }
    }
}