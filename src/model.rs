pub mod alarm;
pub mod event;
pub mod shared;
pub mod telemetry;
pub mod variable;

pub use telemetry::{InitialTelemetry, TirePressure};

use alarm::Alarm;
use event::Event;
use variable::Variable;

#[derive(Debug)]
pub enum TelemetryMessage {
    Variable(Variable),
    Event(Event),
    Alarm(Alarm),
}
