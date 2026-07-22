use engine_lab::clips::{ArrangementView, Clip, ClipPlacement, ClipPlacements, ClipPlaybackMode, ClipRouter, Clips};
use engine_lab::playback::{Probabilities, ProbabilityTarget};
use engine_lab::scheduler::ScheduledNote;
use engine_lab::tempo::Tempo;
use engine_lab::transport::Transport;

pub struct PlaybackFixture {
    pub notes: Vec<ScheduledNote>,
    pub clips: Clips,
    pub placements: ClipPlacements,
    pub router: ClipRouter,
    pub probabilities: Probabilities,
    pub transport: Transport,
    pub tempo: Tempo,
}

impl PlaybackFixture {
    pub fn one_shot_note() -> Self {
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

        let mut probabilities = Probabilities::new();
        probabilities
            .add(*notes[0].id(), 100, ProbabilityTarget::Note)
            .unwrap();

        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        transport.play();
        transport.set_sample_position(tempo.beats_to_samples(8.0));

        Self {
            notes,
            clips,
            placements,
            router,
            probabilities,
            transport,
            tempo,
        }
    }

    pub fn arrangement(&self) -> ArrangementView<'_> {
        ArrangementView::new(&self.notes, &self.clips, &self.placements, &self.router)
    }
}
