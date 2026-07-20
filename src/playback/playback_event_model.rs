use crate::playback::{PlaybackEvent, PlaybackEventType};
use crate::scheduler::{NoteState, ScheduledEvent};

const DEFAULT_CHANNEL: u8 = 1;



/// Adapter for converting scheduler-owned models into playback-owned events.
pub struct PlaybackEventModel;

impl PlaybackEventModel {
	/// Converts a `ScheduledEvent` into a `PlaybackEvent`.
	pub fn from_scheduled_event(event: &ScheduledEvent) -> PlaybackEvent {
		let note_state = event.state();
		let note = event.note();
		let channel = DEFAULT_CHANNEL;

		match note_state {
			NoteState::On => PlaybackEvent {
				beat: event.beat(),
				note_id: *note.id(),
				probability: 127, // Default probability value, can be adjusted as needed
				channel,
				playback_event_type: PlaybackEventType::NoteOn {
					note: note.note(),
					velocity: note.velocity(),
				},
			},
			NoteState::Off => PlaybackEvent {
				beat: event.beat(),
				note_id: *note.id(),
				probability: 127, // Default probability value, can be adjusted as needed
				channel,
				playback_event_type: PlaybackEventType::NoteOff {
					note: note.note(),
				},
			},
		}
	}
}	