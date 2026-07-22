//! This module defines the `ScheduledEvent` struct, which represents a scheduled event for a note, either turning it on or off at a specific beat position.
//! 
//! Responsibilities:
//! - Representing a scheduled event for a note, either turning it on or off at a
//! - specific beat position.
//! 
//! Invariants:
//! - The `ScheduledEvent` struct must always have a valid reference to a `ScheduledNote` instance.
//! - The `ScheduledEvent` struct must always have a valid `NoteState` (On or Off) indicating the state of the note at the scheduled event.
//!
//! Owns:
//! - ScheduledEvent instances
//! 
//! Does Not Own:
//! - ScheduledNote instances (which are borrowed by ScheduledEvent instances) 

// use uuid::Uuid;

use crate::scheduler::enums::NoteState;
use crate::scheduler::scheduled_note::ScheduledNote;
use crate::scheduler::occurrence::NoteOccurrenceKey;

/// Represents a scheduled event for a note, either turning it on or off at a specific beat position, 
/// note consists of a reference to a `ScheduledNote` which contains the note's properties.
pub struct ScheduledEvent<'a> {
    note: &'a ScheduledNote,
    state: NoteState,
    scheduled_beat: f64,
    occurrence_key: NoteOccurrenceKey,
}

impl<'a> ScheduledEvent<'a> {
    /// Creates a new instance of `ScheduledEvent` with the given note and state (On or Off). 
    /// The note is borrowed from the `ScheduledNote` to avoid ownership issues, and the state indicates whether the event is turning the note on or off.
    pub fn new(
        note: &'a ScheduledNote,
        state: NoteState,
        scheduled_beat: f64,
        occurrence_key: NoteOccurrenceKey,
    ) -> Self {
        // invariant already checked in ScheduledNote creation, so no need to check here.
        ScheduledEvent {
            note,
            state,
            scheduled_beat,
            occurrence_key,
        }
    }
    /// Gets the reference to   the associated `ScheduledNote`.
    pub fn note(&self) -> &ScheduledNote {
        self.note
    }
    /// Gets the beat position of the scheduled event by examining its state. This is required
    /// because ScheduledNote stores two beat positions requiring a ScheduledEvent for each occurrence, 
    /// which is why we need to know which one is for the start and which one is for the end. 
    pub fn beat(&self) -> f64 {
        self.scheduled_beat
    }
    /// Gets the state (On or Off) of the scheduled event.
    pub fn state(&self) -> NoteState {
        self.state
    }

    pub fn loop_iteration(&self) -> u64 {
        self.occurrence_key.loop_iteration()
    }

    pub fn occurrence_key(&self) -> NoteOccurrenceKey {
        self.occurrence_key
    }

    pub fn print(&self) {
        println!("ScheduledEvent: \n\tbeat: {}\n\tstate: {:?}\n\tnote: {:?}", self.beat(), self.state(), self.note());
    }
}

// impl Probability for ScheduledEvent<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::enums::NoteState;
    use crate::scheduler::scheduled_note::ScheduledNote;

    #[test]
    fn create() {
        let note = ScheduledNote::new(0.0, 60, 2.0).unwrap();
        let occurrence_key = NoteOccurrenceKey::new(*note.id(), uuid::Uuid::new_v4(), 0);
        let event = ScheduledEvent::new(&note, NoteState::On, 8.0, occurrence_key);
        assert_eq!(event.beat(), 8.0);
        assert_eq!(event.state(), NoteState::On);
        assert_eq!(event.note().start_beat(), 0.0);
        assert_eq!(event.occurrence_key().note_id(), note.id());
        assert_eq!(event.loop_iteration(), 0);
        assert_eq!(event.occurrence_key().loop_iteration(), 0);
    }
}