//! This module provides the `ScheduledNote` struct, which represents a note's lifetime and parameters, 
//! and the `NoteState` enum, which indicates whether a note is on or off. The `ScheduledNote` 
//! struct can be extended as needed, as the triggering mechanism `ScheduledEvent` borrows each instance's data as a whole.
//! 
//! Responsibilities:
//! - Representing a scheduled note with its properties (start beat, end beat, note value, velocity, probability).
//! - Validating the properties of a scheduled note to ensure they are within acceptable ranges.
//! - Providing methods to access and modify the properties of a scheduled note.
//!
//! Invariants:
//! - start beat must be non-negative and finite
//! - end beat must be greater than start beat and finite
//! - note value must be between 0 and 127
//! - velocity must be between 0 and 127
//! 
//! Owns:
//! - ScheduledNote instances
//! 
//! Does Not Own:
//! - ScheduledEvent instances (which borrow ScheduledNote instances)

use uuid::Uuid;
use crate::clips::clip_router::{self, ClipRouter};
use crate::scheduler::enums::ScheduledNoteError;

const MAX_BEAT: f64 = 1.0e12; // Arbitrary large value to prevent overflow in calculations

/// Represents a note's lifetime and parameters, this can be extended as needed as the triggering mechanism ScheduledEvent borrows each instance's data as a whole.   
#[derive(Debug, PartialEq)]
pub struct ScheduledNote {
    id: Uuid,
    start_beat: f64,
    end_beat: f64,
    note: u8,
    velocity: u8,
    clip_router: ClipRouter, // can be used to route the note to different clips or tracks
}

impl ScheduledNote {
    pub fn new(start_beat: f64, note: u8, length: f64, clip_router: ClipRouter) -> Result<Self, ScheduledNoteError> {
        if start_beat < 0.0 {
            return Err(ScheduledNoteError::InvalidStartBeat);
        }
        if !start_beat.is_finite() {
            return Err(ScheduledNoteError::InvalidStartBeat);
        }
        if note > 127 {
            return Err(ScheduledNoteError::InvalidNoteValue);
        }
        if length <= 0.0 || !length.is_finite() {
            return Err(ScheduledNoteError::InvalidLength);
        }
        if start_beat + length <= 0.0 || !(start_beat + length).is_finite() {
            return Err(ScheduledNoteError::InvalidLength);
        }
        if start_beat + length > MAX_BEAT {
            return Err(ScheduledNoteError::InvalidLength);
        }
        let id = Uuid::new_v4();
        let end_beat = start_beat + length;

        Ok(ScheduledNote {
            id,
            start_beat,
            note,
            end_beat,
            velocity: 127,
            clip_router,
        })
    }
    /// Gets the unique identifier of the scheduled note.
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    /// Gets the start beat position of the scheduled note.
    pub fn start_beat(&self) -> f64 {
        self.start_beat
    }
    pub fn move_to(&mut self, start_beat: f64) -> Result<(), ScheduledNoteError> {
        if start_beat < 0.0 || !(start_beat + self.length()).is_finite() || start_beat + self.length() >= MAX_BEAT {
            return Err(ScheduledNoteError::InvalidStartBeat);
        }
        self.end_beat = start_beat + self.length();
        self.start_beat = start_beat;
        println!("start_beat: {}, end_beat: {}, length: {}", self.start_beat, self.end_beat, self.length());
        Ok(())
    }
    /// Gets the end beat position of the scheduled note.
    pub fn end_beat(&self) -> f64 {
        self.end_beat
    }
    /// Gets the length of the scheduled note.
    pub fn length(&self) -> f64 {
        self.end_beat - self.start_beat
    }
    /// Sets the length of the scheduled note by adjusting the end beat position based on the provided length. The length must be positive, non-zero, and finite.
    pub fn set_length(&mut self, length: f64) -> Result<(), ScheduledNoteError> {
        if length <= 0.0 || !(self.start_beat + length).is_finite() || self.start_beat + length >= MAX_BEAT {
            return Err(ScheduledNoteError::InvalidLength);
        }
        self.end_beat = self.start_beat + length;
        Ok(())
    }
    /// Gets the note value of the scheduled note.
    pub fn note(&self) -> u8 {
        self.note
    }
    /// Gets the velocity of the scheduled note.
    pub fn velocity(&self) -> u8 {
        self.velocity
    }
    /// Sets the velocity of the scheduled note. The velocity must not exceed 127.
    pub fn set_velocity(&mut self, velocity: u8) -> Result<(), ScheduledNoteError> {
        if velocity > 127 {
            return Err(ScheduledNoteError::InvalidVelocity);
        }
        self.velocity = velocity;
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let clip_router = ClipRouter::new(Uuid::new_v4());
        let mut note = ScheduledNote::new(0.0, 60, 2.0, clip_router).unwrap();
        assert_eq!(note.start_beat(), 0.0);
        assert_eq!(note.end_beat(), 2.0);
        assert_eq!(note.length(), 2.0);
        assert_eq!(note.note(), 60);
        assert_eq!(note.velocity(), 127);

        assert!(note.set_length(f64::NAN).is_err());
        assert!(note.set_velocity(128).is_err());
    }
    
    #[test]
    fn errors() {
        let clip_router = ClipRouter::new(Uuid::new_v4());
        let mut note = ScheduledNote::new(0.0, 60, 2.0, clip_router).unwrap();

        assert!(note.set_length(-1.0).is_err());
        assert!(note.set_length(f64::NAN).is_err());
        assert!(note.set_length(f64::NEG_INFINITY).is_err());
        assert!(note.set_length(MAX_BEAT).is_err());

        assert!(note.move_to(-1.0).is_err());
        assert!(note.move_to(f64::NAN).is_err());
        assert!(note.move_to(f64::INFINITY).is_err());   
        assert!(note.move_to(MAX_BEAT).is_err());
    }

}