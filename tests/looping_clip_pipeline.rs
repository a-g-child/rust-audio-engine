use engine_lab::clips::{ArrangementView, Clip, ClipPlacement, ClipPlaybackMode, ClipPlacements, ClipRouter, Clips};
use engine_lab::playback::{PlaybackEventKind, PlaybackPipeline, Probabilities, ProbabilityTarget};
use engine_lab::scheduler::ScheduledNote;
use engine_lab::tempo::Tempo;
use engine_lab::transport::Transport;

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
        events.extend(
            pipeline
                .advance(&arrangement, &transport, &tempo, &probabilities)
                .unwrap(),
        );
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
