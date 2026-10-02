use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;

use crate::model::shared::state;

pub struct Event {
    pub vehicle_id: Uuid,
    pub event_kind: state::IssueKind,
    pub event_value: String,
    pub servicing: state::Servicing,
    pub timestamp: DateTime<Utc>
}