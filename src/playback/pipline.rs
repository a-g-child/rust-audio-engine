use crate::clips::ArrangementView;
use crate::playback::{
    ActiveNotes, PlaybackEvent, ProbabilityGate, Probabilities, TimedPlaybackEvent,
};
use crate::scheduler::Scheduler;
use crate::transport::Transport;
use crate::tempo::Tempo;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MutationDecision {
    Allowed,
    Rejected { committed_horizon_beat: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutationBatchItem {
    pub label: String,
    pub from_beat: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MutationBatch {
    items: Vec<MutationBatchItem>,
}

impl MutationBatch {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add<S: Into<String>>(&mut self, label: S, from_beat: f64) -> &mut Self {
        self.items.push(MutationBatchItem {
            label: label.into(),
            from_beat,
        });
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &MutationBatchItem> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationBatchDecision {
    Allowed,
    Rejected {
        label: String,
        from_beat: f64,
        committed_horizon_beat: f64,
    },
}

pub struct PlaybackPipeline {   
    scheduler: Scheduler,
    probability_gate: ProbabilityGate,
    active_notes: ActiveNotes,
    committed_horizon_beat: Option<f64>,
}

impl PlaybackPipeline {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(),
            probability_gate: ProbabilityGate::new(),
            active_notes: ActiveNotes::new(),
            committed_horizon_beat: None,
        }
    }

    pub fn committed_horizon_beat(&self) -> Option<f64> {
        self.committed_horizon_beat
    }

    pub fn mutation_decision_from_beat(&self, from_beat: f64) -> MutationDecision {
        let Some(committed_horizon_beat) = self.committed_horizon_beat else {
            return MutationDecision::Allowed;
        };

        if !from_beat.is_finite() || from_beat < 0.0 {
            return MutationDecision::Rejected {
                committed_horizon_beat,
            };
        }

        if from_beat < committed_horizon_beat {
            MutationDecision::Rejected {
                committed_horizon_beat,
            }
        } else {
            MutationDecision::Allowed
        }
    }

    pub fn can_apply_mutation_from_beat(&self, from_beat: f64) -> bool {
        matches!(
            self.mutation_decision_from_beat(from_beat),
            MutationDecision::Allowed
        )
    }

    pub fn apply_if_mutable_from_beat<T, F>(
        &self,
        from_beat: f64,
        op: F,
    ) -> Result<T, MutationDecision>
    where
        F: FnOnce() -> T,
    {
        match self.mutation_decision_from_beat(from_beat) {
            MutationDecision::Allowed => Ok(op()),
            decision @ MutationDecision::Rejected { .. } => Err(decision),
        }
    }

    pub fn mutation_decision_for_batch(
        &self,
        from_beats: impl IntoIterator<Item = f64>,
    ) -> MutationDecision {
        for from_beat in from_beats {
            let decision = self.mutation_decision_from_beat(from_beat);
            if !matches!(decision, MutationDecision::Allowed) {
                return decision;
            }
        }
        MutationDecision::Allowed
    }

    pub fn can_apply_mutation_batch(
        &self,
        from_beats: impl IntoIterator<Item = f64>,
    ) -> bool {
        matches!(
            self.mutation_decision_for_batch(from_beats),
            MutationDecision::Allowed
        )
    }

    pub fn apply_if_mutable_batch<T, F>(
        &self,
        from_beats: impl IntoIterator<Item = f64>,
        op: F,
    ) -> Result<T, MutationDecision>
    where
        F: FnOnce() -> T,
    {
        match self.mutation_decision_for_batch(from_beats) {
            MutationDecision::Allowed => Ok(op()),
            decision @ MutationDecision::Rejected { .. } => Err(decision),
        }
    }

    pub fn mutation_decision_for_named_batch(
        &self,
        batch: &MutationBatch,
    ) -> MutationBatchDecision {
        for item in batch.iter() {
            if let MutationDecision::Rejected {
                committed_horizon_beat,
            } = self.mutation_decision_from_beat(item.from_beat)
            {
                return MutationBatchDecision::Rejected {
                    label: item.label.clone(),
                    from_beat: item.from_beat,
                    committed_horizon_beat,
                };
            }
        }

        MutationBatchDecision::Allowed
    }

    pub fn can_apply_mutation_named_batch(
        &self,
        batch: &MutationBatch,
    ) -> bool {
        matches!(
            self.mutation_decision_for_named_batch(batch),
            MutationBatchDecision::Allowed
        )
    }

    pub fn apply_if_mutable_named_batch<T, F>(
        &self,
        batch: &MutationBatch,
        op: F,
    ) -> Result<T, MutationBatchDecision>
    where
        F: FnOnce() -> T,
    {
        match self.mutation_decision_for_named_batch(batch) {
            MutationBatchDecision::Allowed => Ok(op()),
            decision @ MutationBatchDecision::Rejected { .. } => Err(decision),
        }
    }

    pub fn set_lookahead(&mut self, lookahead: f64) -> Result<(), crate::scheduler::enums::SchedulerError> {
        self.scheduler.set_lookahead(lookahead)
    }

    pub fn reset(&mut self) {
        self.scheduler.reset();
        self.probability_gate.clear();
        self.active_notes.clear();
        self.committed_horizon_beat = None;
    }

    pub fn reset_with_panic_note_offs(&mut self, beat: f64) -> Vec<PlaybackEvent> {
        let panic_note_offs = self.active_notes.panic_note_offs(beat);
        self.scheduler.reset();
        self.probability_gate.clear();
        self.committed_horizon_beat = None;
        panic_note_offs
    }

    pub fn reset_with_panic_note_offs_timed(
        &mut self,
        beat: f64,
        tempo: &Tempo,
    ) -> Vec<TimedPlaybackEvent> {
        self.reset_with_panic_note_offs(beat)
            .iter()
            .map(|event| TimedPlaybackEvent::from_playback_event(event, tempo))
            .collect()
    }

    pub fn advance(
        &mut self,
        arrangement: &ArrangementView<'_>,
        transport: &Transport,
        tempo: &Tempo,
        probabilities: &Probabilities,
    ) -> Vec<PlaybackEvent> {
        let routed_notes = arrangement.routed_notes();

        let scheduled_events = match self
            .scheduler
            .advance_window(routed_notes, transport, tempo)
        {
            Ok(events) => events,
            Err(_) => return Vec::new(),
        };

        self.committed_horizon_beat = self.scheduler.cursor();

        let out: Vec<PlaybackEvent> = scheduled_events
            .into_iter()
            .filter_map(|event| {
                self.probability_gate.apply(&event, probabilities)
            })
            .collect();

        for event in &out {
            self.active_notes.track_event(event);
        }

        out
    }

    pub fn advance_timed(
        &mut self,
        arrangement: &ArrangementView<'_>,
        transport: &Transport,
        tempo: &Tempo,
        probabilities: &Probabilities,
    ) -> Vec<TimedPlaybackEvent> {
        self.advance(arrangement, transport, tempo, probabilities)
            .iter()
            .map(|event| TimedPlaybackEvent::from_playback_event(event, tempo))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clips::{ArrangementView, Clip, ClipPlacement, ClipPlaybackMode, ClipPlacements, ClipRouter, Clips};
    use crate::playback::{PlaybackEventKind, Probabilities, ProbabilityTarget};
    use crate::scheduler::ScheduledNote;

    #[test]
    fn looping_clip_pipeline_emits_expected_playback_slice() {
        let note_a = ScheduledNote::new(0.0, 60, 0.5).unwrap();
        let note_b = ScheduledNote::new(1.0, 62, 0.5).unwrap();
        let note_c = ScheduledNote::new(3.75, 64, 0.5).unwrap();

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
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 8.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note_a.id(), clip_id);
        router.route_note_to_clip(*note_b.id(), clip_id);
        router.route_note_to_clip(*note_c.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note_a, note_b, note_c];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();
        probabilities.add(*notes[1].id(), 100, ProbabilityTarget::Note).unwrap();
        probabilities.add(*notes[2].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let mut events = Vec::new();
        for _ in 0..=30 {
            events.extend(pipeline.advance(&arrangement, &transport, &tempo, &probabilities));
            transport.advance_b(0.25, &tempo);
        }

        let actual: Vec<(f64, &'static str)> = events
            .iter()
            .map(|e| {
                let state = match e.kind {
                    PlaybackEventKind::NoteOn { .. } => "On",
                    PlaybackEventKind::NoteOff { .. } => "Off",
                    _ => "Other",
                };
                (e.beat, state)
            })
            .collect();

        let expected = vec![
            (8.0, "On"),
            (8.5, "Off"),
            (9.0, "On"),
            (9.5, "Off"),
            (11.75, "On"),
            (12.0, "Off"),
            (12.0, "On"),
            (12.5, "Off"),
            (13.0, "On"),
            (13.5, "Off"),
            (15.75, "On"),
            (16.0, "Off"),
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_clears_pipeline_state_for_rewinds_or_seeks() {
        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

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
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let first_pass = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        assert!(!first_pass.is_empty());

        pipeline.reset();
        transport.set_sample_position(tempo.beats_to_samples(8.0));
        let second_pass = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);

        assert_eq!(first_pass.len(), second_pass.len());
        assert_eq!(first_pass[0].beat, second_pass[0].beat);
    }

    #[test]
    fn reset_with_panic_note_offs_flushes_active_notes() {
        let note = ScheduledNote::new(0.0, 60, 2.0).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let first = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        assert!(first.iter().any(|e| matches!(e.kind, PlaybackEventKind::NoteOn { .. })));

        let panic_offs = pipeline.reset_with_panic_note_offs(8.25);
        assert_eq!(panic_offs.len(), 1);
        assert_eq!(panic_offs[0].beat, 8.25);
        assert!(matches!(panic_offs[0].kind, PlaybackEventKind::NoteOff { note: 60 }));

        let panic_offs_again = pipeline.reset_with_panic_note_offs(8.5);
        assert!(panic_offs_again.is_empty());
    }

    #[test]
    fn advance_timed_converts_beats_to_samples_and_preserves_identity() {
        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let timed = pipeline.advance_timed(&arrangement, &transport, &tempo, &probabilities);
        assert_eq!(timed.len(), 2);

        // 8 beats at 120 BPM and 44.1kHz => 8 * 22050 = 176400 samples.
        assert_eq!(timed[0].sample_position, 176_400);
        assert_eq!(timed[1].sample_position, tempo.beats_to_samples(8.5));
        assert_eq!(timed[0].note_id, *notes[0].id());
        assert_eq!(timed[1].note_id, *notes[0].id());
        assert!(matches!(timed[0].kind, PlaybackEventKind::NoteOn { note: 60, .. }));
        assert!(matches!(timed[1].kind, PlaybackEventKind::NoteOff { note: 60 }));
    }

    #[test]
    fn reset_with_panic_note_offs_timed_converts_deadlines() {
        let note = ScheduledNote::new(0.0, 60, 2.0).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);

        let timed_offs = pipeline.reset_with_panic_note_offs_timed(8.25, &tempo);
        assert_eq!(timed_offs.len(), 1);
        assert_eq!(timed_offs[0].sample_position, tempo.beats_to_samples(8.25));
        assert!(matches!(timed_offs[0].kind, PlaybackEventKind::NoteOff { note: 60 }));
    }

    #[test]
    fn committed_horizon_advances_with_successful_windows() {
        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        assert_eq!(pipeline.committed_horizon_beat(), None);

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let horizon_1 = pipeline.committed_horizon_beat().unwrap();
        assert!((horizon_1 - 8.5).abs() < 1e-3);

        transport.advance_b(0.25, &tempo);
        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let horizon_2 = pipeline.committed_horizon_beat().unwrap();
        assert!((horizon_2 - 8.75).abs() < 1e-3);
    }

    #[test]
    fn committed_horizon_clears_on_reset_variants() {
        let note = ScheduledNote::new(0.0, 60, 2.0).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        assert!(pipeline.committed_horizon_beat().is_some());

        pipeline.reset();
        assert_eq!(pipeline.committed_horizon_beat(), None);

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        assert!(pipeline.committed_horizon_beat().is_some());
        let _ = pipeline.reset_with_panic_note_offs(8.25);
        assert_eq!(pipeline.committed_horizon_beat(), None);
    }

    #[test]
    fn mutation_guard_blocks_mutations_before_committed_horizon() {
        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let committed = pipeline.committed_horizon_beat().unwrap();

        assert!(pipeline.can_apply_mutation_from_beat(committed));
        assert!(matches!(
            pipeline.mutation_decision_from_beat(committed - 0.25),
            MutationDecision::Rejected { .. }
        ));
        assert!(pipeline.can_apply_mutation_from_beat(committed + 0.01));
    }

    #[test]
    fn apply_if_mutable_from_beat_executes_only_when_allowed() {
        use std::cell::Cell;

        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let committed = pipeline.committed_horizon_beat().unwrap();

        let ran_rejected = Cell::new(false);
        let rejected = pipeline.apply_if_mutable_from_beat(committed - 0.1, || {
            ran_rejected.set(true);
            1usize
        });
        assert!(matches!(rejected, Err(MutationDecision::Rejected { .. })));
        assert!(!ran_rejected.get());

        let ran_allowed = Cell::new(false);
        let allowed = pipeline.apply_if_mutable_from_beat(committed + 0.1, || {
            ran_allowed.set(true);
            2usize
        });
        assert_eq!(allowed.unwrap(), 2);
        assert!(ran_allowed.get());
    }

    #[test]
    fn mutation_batch_rejects_if_any_beat_is_before_horizon() {
        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let committed = pipeline.committed_horizon_beat().unwrap();

        assert!(!pipeline.can_apply_mutation_batch([
            committed + 0.1,
            committed - 0.01,
            committed + 0.25,
        ]));

        assert!(matches!(
            pipeline.mutation_decision_for_batch([
                committed + 0.1,
                committed - 0.01,
                committed + 0.25,
            ]),
            MutationDecision::Rejected { .. }
        ));
    }

    #[test]
    fn apply_if_mutable_batch_executes_only_when_all_allowed() {
        use std::cell::Cell;

        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let committed = pipeline.committed_horizon_beat().unwrap();

        let ran_rejected = Cell::new(false);
        let rejected = pipeline.apply_if_mutable_batch(
            [committed + 0.1, committed - 0.1],
            || {
                ran_rejected.set(true);
                7usize
            },
        );
        assert!(matches!(rejected, Err(MutationDecision::Rejected { .. })));
        assert!(!ran_rejected.get());

        let ran_allowed = Cell::new(false);
        let allowed = pipeline.apply_if_mutable_batch(
            [committed + 0.1, committed + 0.2],
            || {
                ran_allowed.set(true);
                9usize
            },
        );
        assert_eq!(allowed.unwrap(), 9);
        assert!(ran_allowed.get());
    }

    #[test]
    fn named_batch_rejection_reports_offending_operation() {
        let note = ScheduledNote::new(0.0, 60, 0.5).unwrap();

        let mut clips = Clips::new();
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_id = placements.add(ClipPlacement::new(clip_id, 8.0, 4.0).unwrap());

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        let notes = vec![note];
        let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

        let mut probabilities = Probabilities::new();
        probabilities.add(*notes[0].id(), 100, ProbabilityTarget::Note).unwrap();

        let mut pipeline = PlaybackPipeline::new();
        pipeline.set_lookahead(0.5).unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        let _ = pipeline.advance(&arrangement, &transport, &tempo, &probabilities);
        let committed = pipeline.committed_horizon_beat().unwrap();

        let mut batch = MutationBatch::new();
        batch
            .add("update_clip_length", committed + 0.1)
            .add("move_note_earlier", committed - 0.01)
            .add("retime_probability", committed + 0.25);

        match pipeline.mutation_decision_for_named_batch(&batch) {
            MutationBatchDecision::Allowed => {
                panic!("expected named batch to be rejected");
            }
            MutationBatchDecision::Rejected {
                label,
                from_beat,
                committed_horizon_beat,
            } => {
                assert_eq!(label, "move_note_earlier");
                assert_eq!(from_beat, committed - 0.01);
                assert!((committed_horizon_beat - committed).abs() < 1e-9);
            }
        }
    }
}