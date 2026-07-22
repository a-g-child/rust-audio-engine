use crate::playback::{PlaybackEventKind, TimedPlaybackEvent};
use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct PlaybackQueue {
    events: VecDeque<TimedPlaybackEvent>,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub fn push_batch(&mut self, events: impl IntoIterator<Item = TimedPlaybackEvent>) {
        self.events.extend(events);

        let mut sorted: Vec<TimedPlaybackEvent> = self.events.drain(..).collect();
        sorted.sort_by(compare_timed_events);
        self.events = sorted.into();
    }

    pub fn drain_due(&mut self, block_end_sample: u64) -> Vec<TimedPlaybackEvent> {
        let mut due = Vec::new();

        loop {
            let is_due = self
                .events
                .front()
                .map(|event| event.sample_position <= block_end_sample)
                .unwrap_or(false);

            if !is_due {
                break;
            }

            if let Some(event) = self.events.pop_front() {
                due.push(event);
            }
        }

        due
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

fn compare_timed_events(a: &TimedPlaybackEvent, b: &TimedPlaybackEvent) -> Ordering {
    a.sample_position
        .cmp(&b.sample_position)
        .then_with(|| event_kind_priority(a.kind).cmp(&event_kind_priority(b.kind)))
        .then_with(|| a.occurrence_key.placement_id().cmp(b.occurrence_key.placement_id()))
        .then_with(|| a.note_id.cmp(&b.note_id))
        .then_with(|| {
            a.occurrence_key
                .loop_iteration()
                .cmp(&b.occurrence_key.loop_iteration())
        })
}

fn event_kind_priority(kind: PlaybackEventKind) -> u8 {
    match kind {
        PlaybackEventKind::NoteOff { .. } => 0,
        PlaybackEventKind::NoteOn { .. } => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::NoteOccurrenceKey;
    use uuid::Uuid;

    #[test]
    fn queue_orders_by_deadline_then_noteoff_then_identity() {
        let placement = Uuid::new_v4();
        let note_a = Uuid::new_v4();
        let note_b = Uuid::new_v4();

        let on = TimedPlaybackEvent {
            sample_position: 128,
            note_id: note_a,
            occurrence_key: NoteOccurrenceKey::new(note_a, placement, 0),
            channel: 1,
            kind: PlaybackEventKind::NoteOn {
                note: 60,
                velocity: 100,
            },
        };

        let off = TimedPlaybackEvent {
            sample_position: 128,
            note_id: note_b,
            occurrence_key: NoteOccurrenceKey::new(note_b, placement, 0),
            channel: 1,
            kind: PlaybackEventKind::NoteOff { note: 62 },
        };

        let mut queue = PlaybackQueue::new();
        queue.push_batch([on, off]);

        let due = queue.drain_due(128);
        assert_eq!(due.len(), 2);
        assert!(matches!(due[0].kind, PlaybackEventKind::NoteOff { .. }));
        assert!(matches!(due[1].kind, PlaybackEventKind::NoteOn { .. }));
    }
}
