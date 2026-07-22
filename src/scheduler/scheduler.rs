//! Scheduler Module
//!
//! Responsibility:
//! - calculate scheduling windows from transport/tempo
//! - materialize note-edge events from routing-resolved note inputs
//! - return events sorted by beat/state/id

use crate::clips::{ClipPlaybackMode, RoutedNote};
use crate::scheduler::enums::{NoteState, SchedulerError};
use crate::scheduler::occurrence::NoteOccurrenceKey;
use crate::scheduler::scheduled_event::ScheduledEvent;
use crate::tempo::Tempo;
use crate::transport::Transport;
use crate::transport::TransportState;

/// Collects global note-edge events for a lookahead window.
pub struct Scheduler {
    lookahead: f64,
    cursor: Option<f64>,
    last_transport_beat: Option<f64>,
}

impl Scheduler {
    /// Creates a scheduler with a default 4-beat lookahead.
    pub fn new() -> Self {
        Scheduler {
            lookahead: 4.0,
            cursor: None,
            last_transport_beat: None,
        }
    }

    pub fn cursor(&self) -> Option<f64> {
        self.cursor
    }

    pub fn last_transport_beat(&self) -> Option<f64> {
        self.last_transport_beat
    }

    pub fn set_lookahead(&mut self, lookahead: f64) -> Result<(), SchedulerError> {
        if lookahead < 0.0 || !lookahead.is_finite() {
            return Err(SchedulerError::InvalidLookahead);
        }
        self.lookahead = lookahead;
        Ok(())
    }

    pub fn advance_window<'a, I>(
        &mut self,
        routed_notes: I,
        transport: &Transport,
        tempo: &Tempo,
    ) -> Result<Vec<ScheduledEvent<'a>>, SchedulerError>
    where
        I: IntoIterator<Item = RoutedNote<'a>>,
    {
        let (window_start, window_end) = self.compute_window(transport, tempo);
        if window_end == window_start {
            return Ok(Vec::new());
        }

        self.validate_window(window_start, window_end)?;
        self.commit_window_progress(window_end, transport, tempo)?;

        let mut events = Self::collect_events_in_window(routed_notes, window_start, window_end);
        Self::sort_events(&mut events);
        Ok(events)
    }

    fn collect_events_in_window<'a, I>(
        routed_notes: I,
        window_start: f64,
        window_end: f64,
    ) -> Vec<ScheduledEvent<'a>>
    where
        I: IntoIterator<Item = RoutedNote<'a>>,
    {
        let mut events: Vec<ScheduledEvent<'a>> = Vec::new();

        for routed in routed_notes {
            Self::collect_routed_note_events(window_start, window_end, &mut events, routed);
        }

        events
    }

    fn collect_routed_note_events<'a>(
        window_start: f64,
        window_end: f64,
        events: &mut Vec<ScheduledEvent<'a>>,
        routed: RoutedNote<'a>,
    ) {
        match routed.clip_playback_mode() {
            ClipPlaybackMode::OneShot => {
                Self::push_one_shot_events(window_start, window_end, events, routed);
            }
            ClipPlaybackMode::Loop {
                start_beat,
                end_beat,
            } => {
                Self::push_loop_events(
                    window_start,
                    window_end,
                    events,
                    routed,
                    start_beat,
                    end_beat,
                );
            }
        }
    }

    fn push_one_shot_events<'a>(
        window_start: f64,
        window_end: f64,
        events: &mut Vec<ScheduledEvent<'a>>,
        routed: RoutedNote<'a>,
    ) {
        let note = routed.note();
        let note_on = routed.placement_start_beat() + note.start_beat();
        let note_off = routed.placement_start_beat() + note.end_beat();
        let placement_end = routed.placement_end_beat();
        let occurrence_key = NoteOccurrenceKey::new(*note.id(), routed.placement_id(), 0);

        if note_on >= window_start
            && note_on < window_end
            && note_on >= routed.placement_start_beat()
            && note_on < placement_end
        {
            events.push(ScheduledEvent::new(
                note,
                NoteState::On,
                note_on,
                occurrence_key,
            ));
        }

        if note_off >= window_start
            && note_off < window_end
            && note_off >= routed.placement_start_beat()
            && note_off < placement_end
        {
            events.push(ScheduledEvent::new(
                note,
                NoteState::Off,
                note_off,
                occurrence_key,
            ));
        }
    }

    fn push_loop_events<'a>(
        window_start: f64,
        window_end: f64,
        events: &mut Vec<ScheduledEvent<'a>>,
        routed: RoutedNote<'a>,
        loop_start_beat: f64,
        loop_end_beat: f64,
    ) {
        let loop_length = loop_end_beat - loop_start_beat;
        if loop_length <= 0.0 || !loop_length.is_finite() {
            return;
        }

        let note = routed.note();
        let local_on = note.start_beat() - loop_start_beat;
        let local_off = note.end_beat() - loop_start_beat;
        let placement_start = routed.placement_start_beat();
        let placement_end = routed.placement_end_beat();

        let max_iterations = (routed.placement_length() / loop_length).ceil() as u64 + 1;
        for iteration in 0..max_iterations {
            let loop_offset = iteration as f64 * loop_length;
            let note_on = placement_start + loop_offset + local_on;
            let note_off = placement_start + loop_offset + local_off;
            let occurrence_key =
                NoteOccurrenceKey::new(*note.id(), routed.placement_id(), iteration);

            if note_on >= window_start
                && note_on < window_end
                && note_on >= placement_start
                && note_on < placement_end
            {
                events.push(ScheduledEvent::new(
                    note,
                    NoteState::On,
                    note_on,
                    occurrence_key,
                ));
            }

            if note_off >= window_start
                && note_off < window_end
                && note_off >= placement_start
                && note_off < placement_end
            {
                events.push(ScheduledEvent::new(
                    note,
                    NoteState::Off,
                    note_off,
                    occurrence_key,
                ));
            }
        }
    }

    fn sort_events(events: &mut [ScheduledEvent<'_>]) {
        events.sort_by(|a, b| {
            a.beat()
                .total_cmp(&b.beat())
                .then_with(|| a.state().cmp(&b.state()))
                .then_with(|| a.note().id().cmp(b.note().id()))
        });
    }

    fn compute_window(&mut self, transport: &Transport, tempo: &Tempo) -> (f64, f64) {
        let current_beat = transport.beat_position(tempo);
        let mut window_start = self.cursor().unwrap_or(current_beat);
        if transport.state() == TransportState::Rewinding {
            window_start = current_beat;
        }
        let window_end = current_beat + self.lookahead;
        (window_start, window_end)
    }

    fn validate_position(position: f64, error: SchedulerError) -> Result<(), SchedulerError> {
        if position < 0.0 || !position.is_finite() {
            return Err(error);
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

    fn commit_window_progress(
        &mut self,
        window_end: f64,
        transport: &Transport,
        tempo: &Tempo,
    ) -> Result<(), SchedulerError> {
        self.update_last_transport_beat(transport.beat_position(tempo))?;
        self.update_cursor(window_end)?;
        Ok(())
    }

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
    use crate::clips::{
        ArrangementView, Clip, ClipPlacement, ClipPlaybackMode, ClipPlacements, ClipRouter, Clips,
    };
    use crate::scheduler::ScheduledNote;

    #[test]
    fn creation_and_setters() {
        let mut scheduler = Scheduler::new();
        assert_eq!(scheduler.lookahead, 4.0);

        scheduler.set_lookahead(2.0).unwrap();
        assert_eq!(scheduler.lookahead, 2.0);
        assert!(scheduler.set_lookahead(-1.0).is_err());
        assert!(scheduler.set_lookahead(f64::NAN).is_err());
        assert!(scheduler.set_lookahead(f64::INFINITY).is_err());
    }

    #[test]
    fn one_shot_placement_resolves_global_beats() {
        let note = ScheduledNote::new(1.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 16.0, 8.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);
        let routed_notes = arrangement.routed_notes();

        let mut scheduler = Scheduler::new();
        scheduler.set_lookahead(8.0).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(16.0));

        let events = scheduler
            .advance_window(routed_notes, &transport, &tempo)
            .unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].beat(), 17.0);
        assert_eq!(events[1].beat(), 17.5);
    }

    #[test]
    fn looped_clip_emits_per_iteration_occurrences() {
        let note = ScheduledNote::new(1.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(
            4.0,
            ClipPlaybackMode::Loop {
                start_beat: 0.0,
                end_beat: 4.0,
            },
        )
        .unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 16.0, 12.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);
        let routed_notes = arrangement.routed_notes();

        let mut scheduler = Scheduler::new();
        scheduler.set_lookahead(32.0).unwrap();

        let mut transport = Transport::new();
        transport.play();

        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        let events = scheduler
            .advance_window(routed_notes, &transport, &tempo)
            .unwrap();

        assert!(events.iter().any(|e| e.beat() == 17.0));
        assert!(events.iter().any(|e| e.beat() == 21.0));
        assert!(events.iter().any(|e| e.beat() == 25.0));

        let on_events: Vec<_> = events
            .iter()
            .filter(|e| e.state() == NoteState::On)
            .collect();

        assert_eq!(on_events.len(), 3);
        assert_ne!(
            *on_events[0].occurrence_key().placement_id(),
            uuid::Uuid::nil()
        );
        assert_eq!(on_events[0].occurrence_key().loop_iteration(), 0);
        assert_eq!(on_events[1].occurrence_key().loop_iteration(), 1);
        assert_eq!(on_events[2].occurrence_key().loop_iteration(), 2);
    }
}
