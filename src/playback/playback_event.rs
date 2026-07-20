//! Playback-domain model for note events consumed by the playback layer.
//! Conversion from scheduler-owned types is handled in `playback_event_model`.

use crate::playback::enums::PlaybackEventType;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackEvent {
    pub beat: f64,
    pub note_id: Uuid,
    pub probability: u8, 
    pub channel: u8,
    pub playback_event_type: PlaybackEventType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_playback_event_fields() {
        let note_id = Uuid::new_v4();
        let playback_event = PlaybackEvent {
            beat: 0.0,
            note_id,
            probability: 127, 
            channel: 1,
            playback_event_type: PlaybackEventType::NoteOn{note: 100, velocity: 127},
        };

        assert_eq!(playback_event.beat, 0.0);
        assert_eq!(playback_event.note_id, note_id);
        match playback_event.playback_event_type {
            PlaybackEventType::NoteOn { note, velocity } => {
                assert_eq!(note, 100);
                assert_eq!(velocity, 127);
            },
            _ => panic!("Expected NoteOn event"),
        }
        assert_eq!(playback_event.channel, 1);
        assert_eq!(playback_event.playback_event_type, PlaybackEventType::NoteOn{note: 100, velocity: 127});
    }
}