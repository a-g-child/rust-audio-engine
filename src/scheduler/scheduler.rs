//! Scheduler Module
//!
//! Responsibility:
//! - manage scheduling of events based on tempo and transport state
//! - calculate the next event time based on tempo and transport state
//! - provide a collection of events that occur within the lookahead window
//! - schedule events in chronological order based on their beat positions
//! - provide a way to set the lookahead value for scheduling events
//! - provide a way to collect events that occur within the lookahead window starting from a given beat position
//!
//! Invariants:
//! - events can be stored at any time
//! - scheduled events are always sorted by beat position
//! - all contained events are valid
//! - beat positions are always non-negative and finite
//! - lookahead is always finite and non-negative
//!
//! Owns:
//! - Event collection
//! - lookahead value
//! - schdeuler cursor position
//! - Model for notes
//!
//! Does Not Own:
//! - Tempo (BPM, beat position, time signature)
//! - Transport (playback state, sample position, playback speed)
//! - Audio
//! - Midi

use crate::scheduler::scheduled_event::ScheduledEvent;
use crate::scheduler::scheduled_note::ScheduledNote;
use crate::tempo::Tempo;
use crate::transport::Transport;
use crate::scheduler::enums::{NoteState, SchedulerError};
use crate::transport::TransportState;

/// Stores notes in scheduling order and emits note-edge events
/// for a configurable lookahead window.  
pub struct Scheduler {
    notes: Vec<ScheduledNote>,
    lookahead: f64,
    cursor: Option<f64>,
    last_transport_beat: Option<f64>,
}

impl Scheduler {
    /// Creates a new instance of the Scheduler with an empty event
    /// list and a default lookahead value of 4.0 beats.
    pub fn new() -> Self {
        Scheduler {
            notes: Vec::new(),
            lookahead: 4.0,
            cursor: None,
            last_transport_beat: None,
        }
    }

    /// Gets the cursor position of the scheduler, which represents
    /// the limit of scheduled events.
    pub fn cursor(&self) -> Option<f64> {
        self.cursor
    }

    pub fn last_transport_beat(&self) -> Option<f64> {
        self.last_transport_beat
    }

    /// Schedules a new note in the scheduler. The note is inserted
    /// into the notes vector in sorted order based on its start beat position.
    pub fn schedule_note(&mut self, event: ScheduledNote) {
        let position = self.notes.partition_point(|s_event| {
            Self::compare_notes_for_scheduling(s_event, &event) == std::cmp::Ordering::Less
        });
        self.notes.insert(position, event);
    }

    /// Sets the lookahead value for the scheduler. The lookahead determines
    /// how far ahead in beats the scheduler will consider events for processing.
    pub fn set_lookahead(&mut self, lookahead: f64) -> Result<(), SchedulerError> {
        if lookahead < 0.0 || !lookahead.is_finite() {
            return Err(SchedulerError::InvalidLookahead);
        }
        self.lookahead = lookahead;
        Ok(())
    }

    /// Returns the number of scheduled notes in the scheduler.
    pub fn notes_count(&self) -> usize {
        self.notes.len()
    }

    /// Advances the scheduling window based on the current transport state and tempo.
    pub fn advance_window(&mut self, transport: &Transport, tempo: &Tempo,) -> Result<Vec<ScheduledEvent<'_>>, SchedulerError> {

        let (window_start, window_end) = self.compute_window(transport, tempo);
        if window_end == window_start { return Ok(Vec::new()); } // If the window is zero width, return an empty vector
        self.validate_window(window_start, window_end)?;
        self.commit_window_progress(window_end, transport, tempo)?;
        let events = Self::collect_events_in_window(&self.notes, window_start, window_end);
       
        Ok(events)
    }

}

impl Scheduler{

    /// Collects all scheduled events that occur within the specified scheduling window.
    fn collect_events_in_window<'a>( notes: &'a [ScheduledNote], window_start: f64, window_end: f64,) -> Vec<ScheduledEvent<'a>> {
        
        let mut events: Vec<ScheduledEvent<'_>> = Vec::new();

        for note in notes {
            Self::collect_note_events(window_start, window_end, &mut events, note);
        }
        // Additional sort by state then id in event of same beat position.
        Self::sort_events(&mut events);

        events
    }

    /// Inserts a note event into the scheduler's notes vector in sorted order based on its start beat position.
    fn collect_note_events<'a>( window_start: f64,
        window_end: f64,
        events: &mut Vec<ScheduledEvent<'a>>,
        note: &'a ScheduledNote,
    ) {
        let s = note.start_beat();
        let e = note.end_beat();

        // Start occurrence in window => NoteState::On
        if s >= window_start && s < window_end {
            // if the start beat of the note is within the scheduling window,
            // create a ScheduledEvent with NoteState::On and add it to the output vector
            events.push(ScheduledEvent::new(note, NoteState::On));
        }

        // End occurrence in window => NoteState::Off
        if e >= window_start && e < window_end {
            // if the end beat of the note is within the scheduling window,
            // create a ScheduledEvent with NoteState::Off and add it to the output vector
            events.push(ScheduledEvent::new(note, NoteState::Off));
        }
    }

    /// additonal sorting function to sort events by beat position, then by state (Off before On),
    /// and finally by note ID for handling notes that share the same beat position and state.
    fn sort_events(events: &mut [ScheduledEvent<'_>]) {
        events.sort_by(|a, b| {
            a.beat()
                .total_cmp(&b.beat())
                .then_with(|| a.state().cmp(&b.state()))
                .then_with(|| a.note().id().cmp(b.note().id()))
        });
    }

    /// Gets the scheduling window boundaries based on the current transport state and tempo.
    fn compute_window(&mut self, transport: &Transport, tempo: &Tempo) -> (f64, f64) {

        let current_beat = transport.beat_position(tempo);
        let mut window_start = self.cursor().unwrap_or(current_beat);
        if transport.state() == TransportState::Rewinding {
            window_start = current_beat;
        }
        let window_end = current_beat + self.lookahead;
        (window_start, window_end)
    }

    // Compares two scheduled notes based on their start
    /// beat positions and IDs for sorting purposes.
    fn compare_notes_for_scheduling(a: &ScheduledNote, b: &ScheduledNote) -> std::cmp::Ordering {
        a.start_beat()
            .total_cmp(&b.start_beat())
            .then_with(|| a.id().cmp(b.id()))
    }

    /// Sets the cursor position of the scheduler, which represents
    /// the limit of scheduled events. The cursor must be non-negative and finite.
    fn validate_position(position: f64, error: SchedulerError) -> Result<(), SchedulerError> {
        let err = error;
        if position < 0.0 || !position.is_finite() {
            return Err(err);
        }
        Ok(())
    }

    fn update_cursor(&mut self, cursor: f64) -> Result<(), SchedulerError> {
        Self::validate_position(cursor, SchedulerError::InvalidCursorPosition)?;
        self.cursor = Some(cursor);
        Ok(())
    }

    fn update_last_transport_beat(&mut self, last_beat: f64) -> Result<(), SchedulerError> {
        Self::validate_position(last_beat, SchedulerError::InvalidTransportPosition)?;
        self.last_transport_beat = Some(last_beat);
        Ok(())
    }
    /// Commits the progress of the scheduling window by updating the last transport beat and cursor position.
    fn commit_window_progress(&mut self, window_end: f64, transport: &Transport, tempo: &Tempo) -> Result<(), SchedulerError> {
        self.update_last_transport_beat(transport.beat_position(tempo))?;
        self.update_cursor(window_end)?;
        Ok(())
    }

    /// Validates the Sceduliing Window
    fn validate_window(&self, window_start: f64, window_end: f64) -> Result<(), SchedulerError> {
        if window_start < 0.0 || !window_start.is_finite() {
            return Err(SchedulerError::InvalidBeatStart);
        }
        if window_end < 0.0 || !window_end.is_finite() {
            return Err(SchedulerError::InvalidBeatEnd);
        }
        if window_start > window_end {
            return Err(SchedulerError::InvalidNegativeWindow);
        }
        Ok(())
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_and_setters() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.notes_count(), 0);
        assert_eq!(scheduler.lookahead, 4.0);

        scheduler.set_lookahead(2.0).unwrap();
        assert_eq!(scheduler.lookahead, 2.0);
        assert!(scheduler.set_lookahead(-1.0).is_err());
        assert!(scheduler.set_lookahead(f64::NAN).is_err());
        assert!(scheduler.set_lookahead(f64::INFINITY).is_err());

        assert!(scheduler.set_lookahead(-1.0).is_err());
        assert!(scheduler.set_lookahead(f64::NAN).is_err());
        assert!(scheduler.set_lookahead(f64::NEG_INFINITY).is_err());
        assert!(scheduler.set_lookahead(f64::INFINITY).is_err());
    }

    #[test]
    fn adds_event() {
        let mut scheduler = Scheduler::new();

        let event = ScheduledNote::new(0.0, 60, 2.0).unwrap();
        scheduler.schedule_note(event);
        assert_eq!(scheduler.notes.len(), 1);
        assert_eq!(scheduler.notes[0].start_beat(), 0.0);
        assert_eq!(scheduler.notes[0].note(), 60);
    }

    #[test]
    fn count() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule_note(ScheduledNote::new(0.0, 60, 2.0).unwrap());
        scheduler.schedule_note(ScheduledNote::new(1.0, 62, 2.0).unwrap());
        assert_eq!(scheduler.notes_count(), 2);
    }

    #[test]
    fn advance_window() {
        let mut scheduler = Scheduler::new();
        scheduler.schedule_note(ScheduledNote::new(0.0, 60, 1.0).unwrap());
        scheduler.schedule_note(ScheduledNote::new(1.0, 62, 1.0).unwrap());
        scheduler.schedule_note(ScheduledNote::new(4.1, 64, 0.9).unwrap());
        scheduler.schedule_note(ScheduledNote::new(4.2, 64, 1.0).unwrap());
        scheduler.schedule_note(ScheduledNote::new(4.2, 64, 2.0).unwrap());

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));

        transport.play();
        println!(
            "Transport is playing: {}, cursor at {:?}",
            transport.is_playing(),
            scheduler.cursor()
        );
        let events = scheduler.advance_window( &transport, &tempo).unwrap();
        assert_eq!(events.len(), 4); // Should include events for notes starting at beat 0 and beat <4
        println!("events = {}", events.len());
        for event in &events {
            event.print();
        }
        transport.advance_b(0.25, &tempo); // Advance by 1 second (120 BPM => 2 beats) 2-6, cursor at 4
        println!(
            "Transport is playing: {}, cursor at {:?}",
            transport.is_playing(),
            scheduler.cursor()
        );
        let events = scheduler.advance_window( &transport, &tempo).unwrap();
        assert_eq!(events.len(), 3); // Should include events for notes starting at beat 4 and beat <6
        println!("events = {}", events.len());
        for event in &events {
            event.print();
        }
        transport.advance_b(0.25, &tempo); // Advance by 1 second (120 BPM => 2 beats) 4-8, cursor at 6
        println!(
            "Transport is playing: {}, cursor at {:?}",
            transport.is_playing(),
            scheduler.cursor()
        );
        let events = scheduler.advance_window( &transport, &tempo).unwrap();
        assert_eq!(events.len(), 0); // Should include events for notes starting at beat 4 and beat 1
        println!("events = {}", events.len());
        for event in &events {
            event.print();
        }
    }
}
