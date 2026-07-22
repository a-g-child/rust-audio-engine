use crate::playback::{PlaybackEvent, PlaybackEventKind};
use crate::scheduler::NoteOccurrenceKey;
use crate::tempo::Tempo;
use uuid::Uuid;

/// A playback event with an absolute sample deadline for realtime execution.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedPlaybackEvent {
    pub sample_position: u64,
    pub note_id: Uuid,
    pub occurrence_key: NoteOccurrenceKey,
    pub channel: u8,
    pub kind: PlaybackEventKind,
}

impl TimedPlaybackEvent {
    pub fn from_playback_event(event: &PlaybackEvent, tempo: &Tempo) -> Self {
        Self {
            sample_position: tempo.beats_to_samples(event.beat),
            note_id: event.note_id,
            occurrence_key: event.occurrence_key,
            channel: event.channel,
            kind: event.kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_beat_to_sample_deadline() {
        let note_id = Uuid::new_v4();
        let key = NoteOccurrenceKey::new(note_id, Uuid::new_v4(), 2);
        let event = PlaybackEvent {
            beat: 1.5,
            note_id,
            occurrence_key: key,
            probability: 127,
            channel: 1,
            kind: PlaybackEventKind::NoteOn {
                note: 64,
                velocity: 100,
            },
        };

        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        let timed = TimedPlaybackEvent::from_playback_event(&event, &tempo);

        // At 120 BPM, 1 beat is 22050 samples.
        assert_eq!(timed.sample_position, 33_075);
        assert_eq!(timed.note_id, note_id);
        assert_eq!(timed.occurrence_key, key);
        assert_eq!(timed.channel, 1);
        assert_eq!(timed.kind, event.kind);
    }
}
