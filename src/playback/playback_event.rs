use crate::scheduler::ScheduledEvent;
use crate::scheduler::NoteState;
use uuid::Uuid;

pub struct PlaybackEvent {
    pub beat: f64,
    pub note_id: Uuid,
    pub note: u8,
    pub velocity: u8,
    pub probability: u8,
    pub state: NoteState,
}

impl From<&ScheduledEvent<'_>> for PlaybackEvent {
    fn from(event: &ScheduledEvent<'_>) -> Self {
        Self {
            beat: event.beat(),
            note_id: *event.note().id(),
            note: event.note().note(),
            velocity: event.note().velocity(),
            probability: event.note().probability(), 
            state: event.state(),
        }
    }
}