use crate::playback::{PlaybackEvent, PlaybackEventKind};
use crate::scheduler::NoteOccurrenceKey;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct ActiveNote {
    occurrence_key: NoteOccurrenceKey,
    note_id: uuid::Uuid,
    note: u8,
    channel: u8,
}

#[derive(Debug, Default)]
pub struct ActiveNotes {
    active: HashMap<NoteOccurrenceKey, ActiveNote>,
}

impl ActiveNotes {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
        }
    }

    pub fn track_event(&mut self, event: &PlaybackEvent) {
        match event.kind {
            PlaybackEventKind::NoteOn { note, .. } => {
                let active_note = ActiveNote {
                    occurrence_key: event.occurrence_key,
                    note_id: event.note_id,
                    note,
                    channel: event.channel,
                };
                self.active.insert(event.occurrence_key, active_note);
            }
            PlaybackEventKind::NoteOff { .. } => {
                self.active.remove(&event.occurrence_key);
            }
            _ => {}
        }
    }

    pub fn panic_note_offs(&mut self, beat: f64) -> Vec<PlaybackEvent> {
        let mut out: Vec<PlaybackEvent> = self
            .active
            .values()
            .map(|active| PlaybackEvent {
                beat,
                note_id: active.note_id,
                occurrence_key: active.occurrence_key,
                probability: 127,
                channel: active.channel,
                kind: PlaybackEventKind::NoteOff { note: active.note },
            })
            .collect();

        out.sort_by(|a, b| {
            a.occurrence_key
                .placement_id()
                .cmp(b.occurrence_key.placement_id())
                .then_with(|| a.note_id.cmp(&b.note_id))
                .then_with(|| {
                    a.occurrence_key
                        .loop_iteration()
                        .cmp(&b.occurrence_key.loop_iteration())
                })
        });

        self.active.clear();
        out
    }

    pub fn clear(&mut self) {
        self.active.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_on_off_and_flushes_panic_offs() {
        let key = NoteOccurrenceKey::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), 0);
        let note_id = *key.note_id();

        let on = PlaybackEvent {
            beat: 1.0,
            note_id,
            occurrence_key: key,
            probability: 127,
            channel: 1,
            kind: PlaybackEventKind::NoteOn {
                note: 64,
                velocity: 100,
            },
        };

        let mut active = ActiveNotes::new();
        active.track_event(&on);

        let panic_offs = active.panic_note_offs(2.0);
        assert_eq!(panic_offs.len(), 1);
        assert_eq!(panic_offs[0].beat, 2.0);
        assert_eq!(panic_offs[0].kind, PlaybackEventKind::NoteOff { note: 64 });

        // Flush should clear state.
        assert!(active.panic_note_offs(3.0).is_empty());
    }
}
