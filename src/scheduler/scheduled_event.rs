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

use crate::scheduler::enums::NoteState;
use crate::scheduler::scheduled_note::ScheduledNote;

/// Represents a scheduled event for a note, either turning it on or off at a specific beat position, 
/// note consists of a reference to a `ScheduledNote` which contains the note's properties.
pub struct ScheduledEvent<'a> {
    note: &'a ScheduledNote,
    state: NoteState,
}

impl<'a> ScheduledEvent<'a> {
    /// Creates a new instance of `ScheduledEvent` with the given note and state (On or Off). 
    /// The note is borrowed from the `ScheduledNote` to avoid ownership issues, and the state indicates whether the event is turning the note on or off.
    pub fn new(note: &'a ScheduledNote, state: NoteState) -> Self {
        // invariant already checked in ScheduledNote creation, so no need to check here.
        ScheduledEvent { note, state }
    }
    /// Gets the reference to the associated `ScheduledNote`.
    pub fn note(&self) -> &ScheduledNote {
        self.note
    }
    /// Gets the beat position of the scheduled event by examining its state. This is required
    /// because ScheduledNote stores two beat positions requiring a ScheduledEvent for each occurrence, 
    /// which is why we need to know which one is for the start and which one is for the end. 
    pub fn beat(&self) -> f64 {
        match self.state {
            NoteState::On  => self.note.start_beat(), // if the event is an "On" event, return the start beat of the scheduled note
            NoteState::Off => self.note.end_beat(), // if the event is an "Off" event, return the end beat of the scheduled note
        }
    }
    /// Gets the state (On or Off) of the scheduled event.
    pub fn state(&self) -> NoteState {
        self.state
    }
    pub fn print(&self) {
        println!("ScheduledEvent: \n\tbeat: {}\n\tstate: {:?}\n\tnote: {:?}", self.beat(), self.state(), self.note());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::enums::NoteState;
    use crate::scheduler::scheduled_note::ScheduledNote;

    #[test]
    fn create() {
        let note = ScheduledNote::new(0.0, 60, 2.0).unwrap();
        let event = ScheduledEvent::new(&note, NoteState::On);
        assert_eq!(event.beat(), 0.0);
        assert_eq!(event.state(), NoteState::On);
        assert_eq!(event.note().start_beat(), 0.0);
    }
}