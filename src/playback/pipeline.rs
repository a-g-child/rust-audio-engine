use crate::clips::ArrangementView;
use crate::playback::{PlaybackEvent, ProbabilityGate, Probabilities, TimedPlaybackEvent};
use crate::scheduler::enums::SchedulerError;
use crate::scheduler::Scheduler;
use crate::tempo::Tempo;
use crate::transport::Transport;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackPipelineError {
    Scheduler(SchedulerError),
}

pub struct PlaybackPipeline {
    scheduler: Scheduler,
    probability_gate: ProbabilityGate,
    scheduled_horizon_beat: Option<f64>,
    committed_horizon_sample: Option<u64>,
}

impl PlaybackPipeline {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler::new(),
            probability_gate: ProbabilityGate::new(),
            scheduled_horizon_beat: None,
            committed_horizon_sample: None,
        }
    }

    pub fn committed_horizon_beat(&self) -> Option<f64> {
        self.scheduled_horizon_beat
    }

    pub fn scheduled_horizon_beat(&self) -> Option<f64> {
        self.scheduled_horizon_beat
    }

    pub fn committed_horizon_sample(&self) -> Option<u64> {
        self.committed_horizon_sample
    }

    pub fn mark_committed_horizon_sample(&mut self, sample: u64) {
        self.committed_horizon_sample = Some(
            self.committed_horizon_sample
                .map_or(sample, |existing| existing.max(sample)),
        );
    }

    pub fn mutation_decision_from_beat(&self, from_beat: f64) -> MutationDecision {
        let Some(committed_horizon_beat) = self.scheduled_horizon_beat else {
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

    pub fn can_apply_mutation_batch(&self, from_beats: impl IntoIterator<Item = f64>) -> bool {
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

    pub fn mutation_decision_for_named_batch(&self, batch: &MutationBatch) -> MutationBatchDecision {
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

    pub fn can_apply_mutation_named_batch(&self, batch: &MutationBatch) -> bool {
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

    pub fn set_lookahead(&mut self, lookahead: f64) -> Result<(), SchedulerError> {
        self.scheduler.set_lookahead(lookahead)
    }

    pub fn reset(&mut self) {
        self.scheduler.reset();
        self.probability_gate.clear();
        self.scheduled_horizon_beat = None;
        self.committed_horizon_sample = None;
    }

    pub fn advance(
        &mut self,
        arrangement: &ArrangementView<'_>,
        transport: &Transport,
        tempo: &Tempo,
        probabilities: &Probabilities,
    ) -> Result<Vec<PlaybackEvent>, PlaybackPipelineError> {
        let routed_notes = arrangement.routed_notes();

        let scheduled_events = self
            .scheduler
            .advance_window(routed_notes, transport, tempo)
            .map_err(PlaybackPipelineError::Scheduler)?;

        self.scheduled_horizon_beat = self.scheduler.cursor();

        let out: Vec<PlaybackEvent> = scheduled_events
            .into_iter()
            .filter_map(|event| self.probability_gate.apply(&event, probabilities))
            .collect();

        Ok(out)
    }

    pub fn advance_timed(
        &mut self,
        arrangement: &ArrangementView<'_>,
        transport: &Transport,
        tempo: &Tempo,
        probabilities: &Probabilities,
    ) -> Result<Vec<TimedPlaybackEvent>, PlaybackPipelineError> {
        let timed: Vec<TimedPlaybackEvent> = self
            .advance(arrangement, transport, tempo, probabilities)?
            .iter()
            .map(|event| TimedPlaybackEvent::from_playback_event(event, tempo))
            .collect();

        if let Some(max_sample) = timed.iter().map(|event| event.sample_position).max() {
            self.mark_committed_horizon_sample(max_sample);
        }

        Ok(timed)
    }
}
