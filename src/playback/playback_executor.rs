use crate::playback::{ActiveNotes, TimedPlaybackEvent};

#[derive(Debug, Default)]
pub struct PlaybackExecutor {
    active_notes: ActiveNotes,
}

impl PlaybackExecutor {
    pub fn new() -> Self {
        Self {
            active_notes: ActiveNotes::new(),
        }
    }

    pub fn execute(&mut self, event: &TimedPlaybackEvent) {
        self.active_notes.track_timed_event(event);
    }

    pub fn panic_note_offs(&mut self, sample_position: u64) -> Vec<TimedPlaybackEvent> {
        self.active_notes.panic_note_offs_timed(sample_position)
    }

    pub fn clear(&mut self) {
        self.active_notes.clear();
    }
}
