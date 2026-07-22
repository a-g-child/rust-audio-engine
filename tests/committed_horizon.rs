mod common;

use common::PlaybackFixture;
use engine_lab::playback::PlaybackPipeline;

#[test]
fn committed_horizon_advances_and_clears_on_reset() {
    let mut fixture = PlaybackFixture::one_shot_note();

    let mut pipeline = PlaybackPipeline::new();
    pipeline.set_lookahead(0.5).unwrap();

    assert_eq!(pipeline.committed_horizon_beat(), None);

    {
        let arrangement = fixture.arrangement();
        let _ = pipeline
            .advance(
                &arrangement,
                &fixture.transport,
                &fixture.tempo,
                &fixture.probabilities,
            )
            .unwrap();
    }
    let horizon_1 = pipeline.committed_horizon_beat().unwrap();
    assert!((horizon_1 - 8.5).abs() < 1e-3);

    fixture.transport.advance_b(0.25, &fixture.tempo);
    {
        let arrangement = fixture.arrangement();
        let _ = pipeline
            .advance(
                &arrangement,
                &fixture.transport,
                &fixture.tempo,
                &fixture.probabilities,
            )
            .unwrap();
    }
    let horizon_2 = pipeline.committed_horizon_beat().unwrap();
    assert!((horizon_2 - 8.75).abs() < 1e-3);

    pipeline.reset();
    assert_eq!(pipeline.committed_horizon_beat(), None);
    assert_eq!(pipeline.committed_horizon_sample(), None);
}

#[test]
fn advance_timed_updates_sample_horizon() {
    let fixture = PlaybackFixture::one_shot_note();

    let mut pipeline = PlaybackPipeline::new();
    pipeline.set_lookahead(0.5).unwrap();

    let timed = {
        let arrangement = fixture.arrangement();
        pipeline
            .advance_timed(
                &arrangement,
                &fixture.transport,
                &fixture.tempo,
                &fixture.probabilities,
            )
            .unwrap()
    };

    assert_eq!(timed.len(), 2);
    assert_eq!(
        pipeline.committed_horizon_sample(),
        Some(fixture.tempo.beats_to_samples(8.5))
    );
}
