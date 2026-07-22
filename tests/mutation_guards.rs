mod common;

use common::PlaybackFixture;
use engine_lab::playback::{MutationBatch, MutationBatchDecision, MutationDecision, PlaybackPipeline};

#[test]
fn mutation_guard_blocks_mutations_before_committed_horizon() {
    let fixture = PlaybackFixture::one_shot_note();
    let arrangement = fixture.arrangement();

    let mut pipeline = PlaybackPipeline::new();
    pipeline.set_lookahead(0.5).unwrap();

    let _ = pipeline
        .advance(
            &arrangement,
            &fixture.transport,
            &fixture.tempo,
            &fixture.probabilities,
        )
        .unwrap();
    let committed = pipeline.committed_horizon_beat().unwrap();

    assert!(pipeline.can_apply_mutation_from_beat(committed));
    assert!(matches!(
        pipeline.mutation_decision_from_beat(committed - 0.25),
        MutationDecision::Rejected { .. }
    ));
    assert!(pipeline.can_apply_mutation_from_beat(committed + 0.01));
}

#[test]
fn apply_if_mutable_batch_executes_only_when_all_allowed() {
    use std::cell::Cell;

    let fixture = PlaybackFixture::one_shot_note();
    let arrangement = fixture.arrangement();

    let mut pipeline = PlaybackPipeline::new();
    pipeline.set_lookahead(0.5).unwrap();

    let _ = pipeline
        .advance(
            &arrangement,
            &fixture.transport,
            &fixture.tempo,
            &fixture.probabilities,
        )
        .unwrap();
    let committed = pipeline.committed_horizon_beat().unwrap();

    let ran_rejected = Cell::new(false);
    let rejected = pipeline.apply_if_mutable_batch([committed + 0.1, committed - 0.1], || {
        ran_rejected.set(true);
        7usize
    });
    assert!(matches!(rejected, Err(MutationDecision::Rejected { .. })));
    assert!(!ran_rejected.get());

    let ran_allowed = Cell::new(false);
    let allowed = pipeline.apply_if_mutable_batch([committed + 0.1, committed + 0.2], || {
        ran_allowed.set(true);
        9usize
    });
    assert_eq!(allowed.unwrap(), 9);
    assert!(ran_allowed.get());
}

#[test]
fn named_batch_rejection_reports_offending_operation() {
    let fixture = PlaybackFixture::one_shot_note();
    let arrangement = fixture.arrangement();

    let mut pipeline = PlaybackPipeline::new();
    pipeline.set_lookahead(0.5).unwrap();

    let _ = pipeline
        .advance(
            &arrangement,
            &fixture.transport,
            &fixture.tempo,
            &fixture.probabilities,
        )
        .unwrap();
    let committed = pipeline.committed_horizon_beat().unwrap();

    let mut batch = MutationBatch::new();
    batch
        .add("update_clip_length", committed + 0.1)
        .add("move_note_earlier", committed - 0.01)
        .add("retime_probability", committed + 0.25);

    match pipeline.mutation_decision_for_named_batch(&batch) {
        MutationBatchDecision::Allowed => panic!("expected named batch rejection"),
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
