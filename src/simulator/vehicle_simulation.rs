use crate::{
    model::telemetry::{InitialTelemetry, TelemetryState},
    model::alarm::AlarmGenerator,
    service_manager::service::ServiceManager,
};

pub(crate) struct VehicleSimulation {
    pub(crate) telemetry: TelemetryState,
    pub(crate) alarm_generator: AlarmGenerator,
    pub(crate) service_manager: ServiceManager,
}

impl VehicleSimulation {
    pub(crate) fn new(initial_telemetry: InitialTelemetry) -> Self {
        Self {
            telemetry: TelemetryState::from(initial_telemetry),
            alarm_generator: AlarmGenerator::new(),
            service_manager: ServiceManager::new(),
        }
    }
}

