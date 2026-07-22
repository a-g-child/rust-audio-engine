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

    pub fn reset(&mut self) {
        self.cursor = None;
        self.last_transport_beat = None;
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

    fn note_on_is_in_range(beat: f64, start: f64, end: f64) -> bool {
        beat >= start && beat < end
    }

    fn note_off_is_in_range(beat: f64, start: f64, end: f64) -> bool {
        beat >= start && beat <= end
    }

    fn loop_iteration_bounds(
        window_start: f64,
        window_end: f64,
        placement_start: f64,
        loop_length: f64,
        max_iterations: u64,
    ) -> Option<(u64, u64)> {
        if max_iterations == 0 || loop_length <= 0.0 || !loop_length.is_finite() {
            return None;
        }

        let first_iteration = if window_start <= placement_start {
            0
        } else {
            ((window_start - placement_start) / loop_length).floor() as u64
        };

        let last_iteration_exclusive = if window_end <= placement_start {
            0
        } else {
            ((window_end - placement_start) / loop_length).ceil() as u64
        };

        let first_iteration = first_iteration.min(max_iterations);
        let last_iteration_exclusive = last_iteration_exclusive.min(max_iterations);

        if first_iteration >= last_iteration_exclusive {
            return None;
        }

        Some((first_iteration, last_iteration_exclusive))
    }

    fn push_one_shot_events<'a>(
        window_start: f64,
        window_end: f64,
        events: &mut Vec<ScheduledEvent<'a>>,
        routed: RoutedNote<'a>,
    ) {
        let note = routed.note();
        let placement_start = routed.placement_start_beat();
        let placement_end = routed.placement_end_beat();

        let note_on = routed.placement_start_beat() + note.start_beat();
        let natural_note_off = routed.placement_start_beat() + note.end_beat();
        let note_off = natural_note_off.min(placement_end);
        let occurrence_key = NoteOccurrenceKey::new(*note.id(), routed.placement_id(), 0);

        if Self::note_on_is_in_range(note_on, window_start, window_end)
            && Self::note_on_is_in_range(note_on, placement_start, placement_end)
        {
            events.push(ScheduledEvent::new(
                note,
                NoteState::On,
                note_on,
                occurrence_key,
            ));
        }

        if Self::note_off_is_in_range(note_off, window_start, window_end)
            && Self::note_off_is_in_range(note_off, placement_start, placement_end)
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
        // Simple loop model: only notes that start within the loop section repeat.
        if note.start_beat() < loop_start_beat || note.start_beat() >= loop_end_beat {
            return;
        }

        let local_on = note.start_beat() - loop_start_beat;
        let local_off = note.end_beat() - loop_start_beat;
        let placement_start = routed.placement_start_beat();
        let placement_end = routed.placement_end_beat();

        let max_iterations = (routed.placement_length() / loop_length).ceil() as u64;
        let Some((first_iteration, last_iteration_exclusive)) = Self::loop_iteration_bounds(
            window_start,
            window_end,
            placement_start,
            loop_length,
            max_iterations,
        ) else {
            return;
        };

        for iteration in first_iteration..last_iteration_exclusive {
            let loop_offset = iteration as f64 * loop_length;
            let iteration_start = placement_start + loop_offset;
            let iteration_end = iteration_start + loop_length;

            let note_on = iteration_start + local_on;
            let natural_note_off = iteration_start + local_off;
            let note_off = natural_note_off.min(iteration_end).min(placement_end);

            let occurrence_key =
                NoteOccurrenceKey::new(*note.id(), routed.placement_id(), iteration);

            if Self::note_on_is_in_range(note_on, window_start, window_end)
                && Self::note_on_is_in_range(note_on, placement_start, placement_end)
            {
                events.push(ScheduledEvent::new(
                    note,
                    NoteState::On,
                    note_on,
                    occurrence_key,
                ));
            }

            if Self::note_off_is_in_range(note_off, window_start, window_end)
                && Self::note_off_is_in_range(note_off, placement_start, placement_end)
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

    fn event_order(event: &ScheduledEvent<'_>) -> u8 {
        match event.state() {
            NoteState::Off => 0,
            NoteState::On => 1,
        }
    }

    fn sort_events(events: &mut [ScheduledEvent<'_>]) {
        events.sort_by(|a, b| {
            a.beat()
                .total_cmp(&b.beat())
                .then_with(|| Self::event_order(a).cmp(&Self::event_order(b)))
                .then_with(|| a.occurrence_key().placement_id().cmp(b.occurrence_key().placement_id()))
                .then_with(|| a.note().id().cmp(b.note().id()))
                .then_with(|| a.occurrence_key().loop_iteration().cmp(&b.occurrence_key().loop_iteration()))
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

    #[test]
    fn note_off_exactly_at_one_shot_placement_end_is_emitted() {
        let note = ScheduledNote::new(3.5, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 16.0, 4.0).unwrap());

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

        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::On && e.beat() == 19.5));
        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::Off && e.beat() == 20.0));
    }

    #[test]
    fn one_shot_tail_crossing_placement_end_is_clamped() {
        let note = ScheduledNote::new(3.5, 60, 1.0).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(8.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 16.0, 4.0).unwrap());

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

        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::On && e.beat() == 19.5));
        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::Off && e.beat() == 20.0));
        assert!(!events
            .iter()
            .any(|e| e.state() == NoteState::Off && e.beat() > 20.0));
    }

    #[test]
    fn note_crossing_loop_end_is_clamped_before_next_iteration() {
        let note = ScheduledNote::new(3.75, 60, 0.5).unwrap();

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
        let placement_id = placements.add(ClipPlacement::new(clip_id, 16.0, 8.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);
        let routed_notes = arrangement.routed_notes();

        let mut scheduler = Scheduler::new();
        scheduler.set_lookahead(16.0).unwrap();

        let mut transport = Transport::new();
        transport.play();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.set_sample_position(tempo.beats_to_samples(16.0));

        let events = scheduler
            .advance_window(routed_notes, &transport, &tempo)
            .unwrap();

        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::On && e.beat() == 19.75));
        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::Off && e.beat() == 20.0));
        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::On && e.beat() == 23.75));
        assert!(events
            .iter()
            .any(|e| e.state() == NoteState::Off && e.beat() == 24.0));
        assert!(!events
            .iter()
            .any(|e| e.state() == NoteState::Off && e.beat() == 20.25));
    }

    #[test]
    fn note_before_non_zero_loop_start_does_not_repeat() {
        let note = ScheduledNote::new(1.0, 60, 0.25).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(
            8.0,
            ClipPlaybackMode::Loop {
                start_beat: 2.0,
                end_beat: 6.0,
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

        assert!(events.is_empty());
    }

    #[test]
    fn same_clip_two_placements_produce_distinct_occurrence_keys() {
        let note = ScheduledNote::new(0.5, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_a = placements.add(ClipPlacement::new(clip_id, 0.0, 4.0).unwrap());
        let placement_b = placements.add(ClipPlacement::new(clip_id, 16.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_a);
        router.add_placement_to_clip(clip_id, placement_b);

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

        let on_keys: Vec<_> = events
            .iter()
            .filter(|e| e.state() == NoteState::On)
            .map(|e| e.occurrence_key())
            .collect();

        assert_eq!(on_keys.len(), 2);
        assert_ne!(on_keys[0].placement_id(), on_keys[1].placement_id());
    }

    #[test]
    fn late_window_uses_correct_loop_iteration_indices() {
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
        let placement_id = placements.add(ClipPlacement::new(clip_id, 0.0, 120.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);
        let routed_notes = arrangement.routed_notes();

        let mut scheduler = Scheduler::new();
        scheduler.set_lookahead(4.0).unwrap();

        let mut transport = Transport::new();
        transport.play();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.set_sample_position(tempo.beats_to_samples(100.0));

        let events = scheduler
            .advance_window(routed_notes, &transport, &tempo)
            .unwrap();

        let on_event = events
            .iter()
            .find(|e| e.state() == NoteState::On)
            .expect("expected NoteOn in late scheduling window");

        assert_eq!(on_event.beat(), 101.0);
        assert_eq!(on_event.occurrence_key().loop_iteration(), 25);
    }

    #[test]
    fn loop_iteration_bounds_focus_on_window_slice_for_long_placements() {
        let loop_length = 4.0;
        let placement_start = 0.0;
        let max_iterations = (10_000_000.0_f64 / loop_length).ceil() as u64;

        let (first, last_exclusive) = Scheduler::loop_iteration_bounds(
            9_999_992.0,
            9_999_996.0,
            placement_start,
            loop_length,
            max_iterations,
        )
        .expect("expected non-empty iteration bounds");

        assert_eq!(first, 2_499_998);
        assert_eq!(last_exclusive, 2_499_999);
        assert_eq!(last_exclusive - first, 1);
    }

    #[test]
    fn loop_iteration_bounds_are_none_when_window_is_before_placement() {
        let bounds = Scheduler::loop_iteration_bounds(0.0, 4.0, 16.0, 4.0, 10);
        assert!(bounds.is_none());
    }
}
