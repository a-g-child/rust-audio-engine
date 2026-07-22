mod common;

use common::PlaybackFixture;
use engine_lab::playback::{PlaybackEventKind, PlaybackExecutor, PlaybackQueue, PlaybackRuntime, TimedPlaybackEvent};
use engine_lab::scheduler::NoteOccurrenceKey;
use uuid::Uuid;

#[test]
fn block_processing_tracks_active_notes_only_after_execution() {
    let note_id = Uuid::new_v4();
    let placement_id = Uuid::new_v4();
    let key = NoteOccurrenceKey::new(note_id, placement_id, 0);

    let on = TimedPlaybackEvent {
        sample_position: 100,
        note_id,
        occurrence_key: key,
        channel: 1,
        kind: PlaybackEventKind::NoteOn {
            note: 60,
            velocity: 100,
        },
    };

    let off = TimedPlaybackEvent {
        sample_position: 300,
        note_id,
        occurrence_key: key,
        channel: 1,
        kind: PlaybackEventKind::NoteOff { note: 60 },
    };

    let mut queue = PlaybackQueue::new();
    queue.push_batch([on, off]);

    let mut executor = PlaybackExecutor::new();

    let due_0 = queue.drain_due(128);
    assert_eq!(due_0.len(), 1);
    assert!(matches!(due_0[0].kind, PlaybackEventKind::NoteOn { .. }));
    for event in &due_0 {
        executor.execute(event);
    }

    let due_1 = queue.drain_due(256);
    assert!(due_1.is_empty());

    let panic_at_200 = executor.panic_note_offs(200);
    assert_eq!(panic_at_200.len(), 1);
    assert!(matches!(
        panic_at_200[0].kind,
        PlaybackEventKind::NoteOff { note: 60 }
    ));
    assert_eq!(panic_at_200[0].sample_position, 200);

    queue.clear();
    assert!(queue.drain_due(384).is_empty());
}

#[test]
fn runtime_stop_clears_queue_and_pipeline_state() {
    let mut fixture = PlaybackFixture::one_shot_note();

    let mut runtime = PlaybackRuntime::new();
    runtime.pipeline_mut().set_lookahead(0.5).unwrap();

    {
        let arrangement = fixture.arrangement();
        let _ = runtime
            .schedule(
                &arrangement,
                &fixture.transport,
                &fixture.tempo,
                &fixture.probabilities,
            )
            .unwrap();
    }

    // Execute the first block so NoteOn becomes genuinely active.
    let first_block_end = fixture.transport.sample_position() + 128;
    let _ = runtime.process_until(first_block_end);

    let panic_offs = runtime.stop(
        fixture.transport.sample_position() + 200,
        &mut fixture.transport,
    );
    assert!(!panic_offs.is_empty());
    assert!(runtime.queue().is_empty());
    assert_eq!(runtime.pipeline().committed_horizon_beat(), None);
}
