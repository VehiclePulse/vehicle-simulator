use crate::model::{
    event::Event,
    alarm::Alarm,
    telemetry::TelemetryState,
    shared::state::{IssueKind, Severity, Servicing}
};

use rand::RngExt;

#[derive(Debug)]
struct PendingService {
    issue_kind: IssueKind,
    target_value: f64
}

pub struct ServiceManager {
    pending_services: Vec<PendingService>,
    active_service: Option<PendingService>,
    remaining_seconds: f64,
}


impl ServiceManager {
    pub fn new() -> Self {
        Self {
            pending_services: Vec::new(),
            active_service: None,
            remaining_seconds: 0.0
        }
    }

    pub fn has_active_service(&self) -> bool {
        self.active_service.is_some()
    }

    pub fn is_servicing(&self) -> bool {
        self.active_service.is_some()
        //&& self.waiting_seconds <= 0.0
        && self.remaining_seconds > 0.0
    }

    pub fn handle_alarm(
        &mut self,
        alarm: &Alarm,
        rng: &mut impl RngExt
    ) {
        if !matches! (
            alarm.severity, 
            Severity::Warning | Severity::Critical
        ) {
            return;
        }

        if Event::from_alarm(alarm).is_none() {
            return;
        }


        // 1. Unwrap the Option safely using `if let`
        if self.pending_services.iter()
            .any(|service| service.issue_kind == alarm.alarm_kind)
        {
            return;
        }

        //do we need it ?
        if self
            .active_service
            .as_ref()
            .is_some_and(|service| service.issue_kind == alarm.alarm_kind)
        {
            return;
        }


        let target_value = match alarm.alarm_kind {
            IssueKind::FuelLevel => {
                rng.random_range(29.98..=30.0)
            }

            IssueKind::BatteryLevel => {
                rng.random_range(1.0..=30.0)
            }

            IssueKind::EngineTemperature => {
                rng.random_range(110.0..=120.0)
            }

            IssueKind::TirePressure => {
                rng.random_range(18.0..=24.0)
            }

            _ => return
        };

        let service = PendingService {
            issue_kind: alarm.alarm_kind,
            target_value
        };

        println!(
            "Service scheduled: {:?} | alarm={} | target={:.2}",
            service.issue_kind,
            alarm.alarm_value,
            target_value
        );

        self.pending_services.push(service);
    }

    fn current_value(
        telemetry: &TelemetryState,
        issue_kind: IssueKind,
    ) -> f64 {
        match issue_kind {
            IssueKind::FuelLevel => telemetry.fuel_level,

            IssueKind::BatteryLevel => telemetry.battery_level,

            IssueKind::EngineTemperature => telemetry.engine_temperature,

            IssueKind::TirePressure => {
                // For now, use the lowest tire pressure.
                telemetry.tire_pressure.front_left
                    .min(telemetry.tire_pressure.front_right)
                    .min(telemetry.tire_pressure.rear_left)
                    .min(telemetry.tire_pressure.rear_right)
            }

            _ => 0.0,
        }
    }

    fn target_reached(
        issue_kind: IssueKind,
        current: f64,
        target: f64,
    ) -> bool {
        match issue_kind {
            IssueKind::FuelLevel => current <= target,

            IssueKind::BatteryLevel => current <= target,

            IssueKind::EngineTemperature => current >= target,

            IssueKind::TirePressure => current <= target,

            _ => false,
        }
    }

}


impl ServiceManager {
    pub fn check_targets(
        &mut self,
        telemetry: &TelemetryState,
    ) -> bool {
        if self.active_service.is_some() {
            return false;
        }

        let reached_index = self
            .pending_services
            .iter()
            .position(|service| {
                let current = Self::current_value(
                    telemetry,
                    service.issue_kind,
                );

                Self::target_reached(
                    service.issue_kind,
                    current,
                    service.target_value,
                )
            });

        let Some(index) = reached_index else {
            return false;
        };

        let service = self.pending_services.remove(index);

        println!(
            "Service target reached: {:?} | target={:.2}",
            service.issue_kind,
            service.target_value,
        );

        self.active_service = Some(service);
        self.remaining_seconds = 10.0;

        true
    }
}

impl ServiceManager {
    pub fn update(
        &mut self,
        telemetry: &mut TelemetryState,
        elapsed_seconds: f64,
    ) -> bool {
        let Some(service) = self.active_service.as_ref() else {
            return false;
        };

        // Safety: servicing only happens while stopped.
        if telemetry.speed > 0.0 {
            return false;
        }

        let Some(servicing_kind) = service.issue_kind.servicing() else {
            return false;
        };

        match servicing_kind {
            Servicing::Refuel => {
                telemetry.fuel_level = (
                    telemetry.fuel_level
                    + 10.0 * elapsed_seconds
                ).min(100.0);
            }

            Servicing::Radiate => {
                telemetry.engine_temperature = (
                    telemetry.engine_temperature
                    - 5.0 * elapsed_seconds
                ).max(70.0);
            }

            Servicing::Recharge => {
                telemetry.battery_level = (
                    telemetry.battery_level
                    + 8.0 * elapsed_seconds
                ).min(100.0);
            }

            Servicing::NormalizePressure => {
                let amount = 3.0 * elapsed_seconds;

                telemetry.tire_pressure.front_left = (
                    telemetry.tire_pressure.front_left + amount
                ).min(36.0);

                telemetry.tire_pressure.front_right = (
                    telemetry.tire_pressure.front_right + amount
                ).min(36.0);

                telemetry.tire_pressure.rear_left = (
                    telemetry.tire_pressure.rear_left + amount
                ).min(36.0);

                telemetry.tire_pressure.rear_right = (
                    telemetry.tire_pressure.rear_right + amount
                ).min(36.0);
            }
        }

        self.remaining_seconds -= elapsed_seconds;

        println!(
            "Servicing {:?} | remaining={:.1}s",
            service.issue_kind.servicing(),
            self.remaining_seconds.max(0.0),
        );

        if self.remaining_seconds <= 0.0 {
            println!(
                "Service completed: {:?}",
                service.issue_kind.servicing(),
            );

            self.active_service = None;
            self.remaining_seconds = 0.0;

            return true;
        }

        false
    }
}