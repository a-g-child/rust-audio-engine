//! Translation mappings from upstream domain events into playback-owned events.
//!
//! Future mappings may include MIDI, automation and clip events.ppings for other event types as needed, i.e, midi, automation, etc.

use crate::playback::{PlaybackEvent, PlaybackEventKind};
use crate::scheduler::{NoteState, ScheduledEvent};

const DEFAULT_CHANNEL: u8 = 1;

/// Converts a borrowed scheduler event into an owned playback event.
impl From<&ScheduledEvent<'_>> for PlaybackEvent {
    fn from(event: &ScheduledEvent<'_>) -> Self {
        let note = event.note();
        let channel = DEFAULT_CHANNEL;

        match event.state() {
            NoteState::On => PlaybackEvent {
                beat: event.beat(),
                note_id: *note.id(),
                probability: 127,
                channel,
                kind: PlaybackEventKind::NoteOn {
                    note: note.note(),
                    velocity: note.velocity(),
                },
            },
            NoteState::Off => PlaybackEvent {
                beat: event.beat(),
                note_id: *note.id(),
                probability: 127,
                channel,
                kind: PlaybackEventKind::NoteOff {
                    note: note.note(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

	#[test]
	fn test_mapper() {

		let note = crate::scheduler::ScheduledNote::new(0.0, 100, 1.0).unwrap();
		let scheduled_event = crate::scheduler::ScheduledEvent::new(&note, crate::scheduler::NoteState::On); // Create a mock ScheduledEvent
		let playback_event: PlaybackEvent = (&scheduled_event).into();

		assert_eq!(playback_event.beat, scheduled_event.beat());
		assert_eq!(playback_event.note_id, *scheduled_event.note().id());
		assert_eq!(playback_event.channel, 1);
		match playback_event.kind {
			PlaybackEventKind::NoteOn { note, velocity } => {
				assert_eq!(note, scheduled_event.note().note());
				assert_eq!(velocity, scheduled_event.note().velocity());
			},
			_ => panic!("Expected NoteOn event"),
		}
	}

}