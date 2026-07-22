//! This module defines the 'NoteOccurrenceKey' struct, which is used to uniquely identify a note occurrence in the scheduler.
//! this is a helpful indicator for the probability gate to know which note occurrences have been accepted or rejected based on their probabilities.
//! this is also important for clip based playback where a clip may loop indefinitely over 4 bars while the global transport plays forward with increasing beats

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteOccurrenceKey {
    note_id: Uuid,
    placement_id: Uuid,
    loop_iteration: u64,
}

impl NoteOccurrenceKey {
    /// Creates a new instance of `NoteOccurrenceKey` with the given note ID and loop iteration, this should be linked to a 'ScheduledEvent' note pair.
    pub fn new(note_id: Uuid, placement_id: Uuid, loop_iteration: u64) -> Self {
        Self { note_id, placement_id, loop_iteration }
    }
    /// Gets the note 'ID' associated with this occurrence key.
    pub fn note_id(&self) -> &Uuid {
        &self.note_id
    }
    /// Gets the placement 'ID' associated with this occurrence key.
    pub fn placement_id(&self) -> &Uuid {
        &self.placement_id
    }
    /// Gets the loop iteration associated with this occurrence key.
    pub fn loop_iteration(&self) -> u64 {
        self.loop_iteration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_stores_note_id_and_iteration() {
        let id = Uuid::new_v4();
        let placement_id = Uuid::new_v4();
        let key = NoteOccurrenceKey::new(id, placement_id, 3);

        assert_eq!(key.note_id(), &id);
        assert_eq!(key.placement_id(), &placement_id);
        assert_eq!(key.loop_iteration(), 3);
    }
}