
use engine_lab::clips::{ArrangementView, Clip, ClipPlacement, ClipPlacements, ClipPlaybackMode, ClipRouter, Clips};
use engine_lab::playback::{MutationBatch, MutationBatchDecision, PlaybackEventKind, PlaybackRuntime, Probabilities, ProbabilityTarget};
use engine_lab::scheduler::ScheduledNote;
use engine_lab::tempo::Tempo;
use engine_lab::transport::Transport;

fn describe_kind(kind: PlaybackEventKind) -> String {
    match kind {
        PlaybackEventKind::NoteOn { note, velocity } => {
            format!("NoteOn(note={note}, velocity={velocity})")
        }
        PlaybackEventKind::NoteOff { note } => format!("NoteOff(note={note})"),
        PlaybackEventKind::ControlChange => "ControlChange".to_string(),
        PlaybackEventKind::ProgramChange => "ProgramChange".to_string(),
        PlaybackEventKind::ParameterChange => "ParameterChange".to_string(),
        PlaybackEventKind::PitchWheelChange => "PitchWheelChange".to_string(),
        PlaybackEventKind::Aftertouch => "Aftertouch".to_string(),
        PlaybackEventKind::ChannelPressure => "ChannelPressure".to_string(),
    }
}

fn main() {
    let tempo = Tempo::new(120.0, 44_100, (4, 4));

    let note_kick = ScheduledNote::new(0.0, 36, 0.25).expect("valid kick note");
    let note_hat = ScheduledNote::new(0.5, 42, 0.2).expect("valid hat note");
    let note_fill = ScheduledNote::new(2.75, 38, 0.3).expect("valid fill note");
    let notes = vec![note_kick, note_hat, note_fill];

    let mut clips = Clips::new();
    let groove_clip = Clip::new(
        4.0,
        ClipPlaybackMode::Loop {
            start_beat: 0.0,
            end_beat: 4.0,
        },
    )
    .expect("valid loop clip");
    let groove_clip_id = clips.add(groove_clip);

    let fill_clip = Clip::new(4.0, ClipPlaybackMode::OneShot).expect("valid one-shot clip");
    let fill_clip_id = clips.add(fill_clip);

    let mut placements = ClipPlacements::new();
    let groove_placement = placements
        .add(ClipPlacement::new(groove_clip_id, 8.0, 8.0).expect("valid groove placement"));
    let fill_placement = placements
        .add(ClipPlacement::new(fill_clip_id, 12.0, 4.0).expect("valid fill placement"));

    let mut router = ClipRouter::new();
    router.route_note_to_clip(*notes[0].id(), groove_clip_id);
    router.route_note_to_clip(*notes[1].id(), groove_clip_id);
    router.route_note_to_clip(*notes[2].id(), fill_clip_id);
    router.add_placement_to_clip(groove_clip_id, groove_placement);
    router.add_placement_to_clip(fill_clip_id, fill_placement);

    let arrangement = ArrangementView::new(&notes, &clips, &placements, &router);

    let mut probabilities = Probabilities::new();
    probabilities
        .add(*notes[0].id(), 100, ProbabilityTarget::Note)
        .expect("kick probability");
    probabilities
        .add(*notes[1].id(), 90, ProbabilityTarget::Note)
        .expect("hat probability");
    probabilities
        .add(*notes[2].id(), 100, ProbabilityTarget::Note)
        .expect("fill probability");

    let mut runtime = PlaybackRuntime::new();
    runtime
        .pipeline_mut()
        .set_lookahead(0.5)
        .expect("lookahead set");

    let mut transport = Transport::new();
    transport.play();
    transport.set_sample_position(tempo.beats_to_samples(8.0));

    println!("== Engine slice demo ==");
    for tick in 0..8 {
        let scheduled_count = runtime
            .schedule(&arrangement, &transport, &tempo, &probabilities)
            .expect("schedule succeeds");

        let block_end_sample = transport.sample_position() + tempo.beats_to_samples(0.25);
        let timed_events = runtime.process_until(block_end_sample);

        println!(
            "tick={tick:02} beat={:.2} scheduled={} executed={}",
            transport.beat_position(&tempo),
            scheduled_count,
            timed_events.len()
        );

        for event in timed_events {
            println!(
                "  sample={} note_id={} kind={}",
                event.sample_position,
                event.note_id,
                describe_kind(event.kind)
            );
        }

        transport.advance_b(0.25, &tempo);
    }

    if let Some(horizon) = runtime.pipeline().committed_horizon_beat() {
        println!("committed_horizon_beat={horizon:.3}");

        let mut risky_batch = MutationBatch::new();
        risky_batch
            .add("move_loop_earlier", horizon - 0.25)
            .add("retime_fill", horizon + 0.25);

        match runtime.pipeline().mutation_decision_for_named_batch(&risky_batch) {
            MutationBatchDecision::Allowed => {
                println!("risky batch unexpectedly allowed");
            }
            MutationBatchDecision::Rejected {
                label,
                from_beat,
                committed_horizon_beat,
            } => {
                println!(
                    "named batch rejected: op={label} from_beat={from_beat:.3} committed={committed_horizon_beat:.3}"
                );
            }
        }

        let mut safe_batch = MutationBatch::new();
        safe_batch
            .add("move_loop_later", horizon + 0.25)
            .add("retime_fill", horizon + 0.5);

        let applied = runtime
            .pipeline()
            .apply_if_mutable_named_batch(&safe_batch, || "safe batch applied")
            .expect("safe batch should apply");
        println!("{applied}");
    }

    let panic_offs = runtime.stop(transport.sample_position(), &mut transport);
    println!("panic note offs generated on reset: {}", panic_offs.len());
}

