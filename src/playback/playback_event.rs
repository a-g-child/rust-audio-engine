//! Playback-domain model for note events consumed by the playback layer.
//! Conversion from scheduler-owned types is handled in `mapper`.

use crate::playback::enums::PlaybackEventKind;
// use crate::playback::mapper::SchedulerEventMapper;
use uuid::Uuid;
/// Represents a playback event, which is a note event that can be consumed by the playback layer.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackEvent {
    pub beat: f64,
    pub note_id: Uuid,
    pub probability: u8, 
    pub channel: u8,
    pub kind: PlaybackEventKind,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::scheduler::{ScheduledEvent, NoteState, ScheduledNote};
    
    #[test]
    fn test_playback_event_fields() {
        let note_id = Uuid::new_v4();
        let playback_event = PlaybackEvent {
            beat: 0.0,
            note_id,
            probability: 127, 
            channel: 1,
            kind: PlaybackEventKind::NoteOn{note: 100, velocity: 127},
        };

        assert_eq!(playback_event.beat, 0.0);
        assert_eq!(playback_event.note_id, note_id);
        match playback_event.kind {
            PlaybackEventKind::NoteOn { note, velocity } => {
                assert_eq!(note, 100);
                assert_eq!(velocity, 127);
            },
            _ => panic!("Expected NoteOn event"),
        }
        assert_eq!(playback_event.channel, 1);
        assert_eq!(playback_event.kind, PlaybackEventKind::NoteOn{note: 100, velocity: 127});
    }

    #[test]
    fn test_mapper() {

        let note = ScheduledNote::new(0.0, 100, 1.0).unwrap();

        let scheduled_event = ScheduledEvent::new(&note, NoteState::On); // Create a mock ScheduledEvent
        let playback_event = PlaybackEvent::from(&scheduled_event);

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

    #[test]
    fn map_note_off_event() {
        let note = ScheduledNote::new(0.0, 100, 1.0).unwrap();
        let scheduled_event = ScheduledEvent::new(&note, NoteState::Off);

        let playback_event = PlaybackEvent::from(&scheduled_event);

        assert_eq!(
            playback_event.kind,
            PlaybackEventKind::NoteOff { note: 100 }
        );
    }
    #[test]
    fn map_note_on_event() {
        let note = ScheduledNote::new(0.0, 100, 1.0).unwrap();
        let scheduled_event = ScheduledEvent::new(&note, NoteState::On);    
        let playback_event = PlaybackEvent::from(&scheduled_event);
    
        assert_eq!(
            playback_event.kind,
            PlaybackEventKind::NoteOn { note: 100, velocity: 127 }
        );
    }
}

