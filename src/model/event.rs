use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;

use crate::model::alarm::Alarm;
use crate::model::shared::state;
use crate::model::shared::state::Severity;

#[derive(Debug)]
pub struct Event {
    pub vehicle_id: Uuid,
    pub event_kind: state::IssueKind,
    pub event_value: String,
    pub servicing: state::Servicing,
    pub timestamp: DateTime<Utc>
}

impl Event {
    pub fn from_alarm(alarm: &Alarm) -> Option<Self> {
        if !matches!(
            alarm.severity,
            Severity::Warning | Severity::Critical
        ) {
            return None;
        }

        let servicing = alarm.alarm_kind.servicing()?;

        Some(Self {
            vehicle_id: alarm.vehicle_id,
            event_kind: alarm.alarm_kind,
            event_value: alarm.alarm_value.clone(),
            servicing,
            timestamp: Utc::now()
        })
    }
}