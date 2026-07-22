use crate::clips::ArrangementView;
use crate::playback::{
    PlaybackExecutor, PlaybackPipeline, PlaybackPipelineError, PlaybackQueue, Probabilities,
    TimedPlaybackEvent,
};
use crate::tempo::Tempo;
use crate::transport::Transport;

pub struct PlaybackRuntime {
    pipeline: PlaybackPipeline,
    queue: PlaybackQueue,
    executor: PlaybackExecutor,
}

impl PlaybackRuntime {
    pub fn new() -> Self {
        Self {
            pipeline: PlaybackPipeline::new(),
            queue: PlaybackQueue::new(),
            executor: PlaybackExecutor::new(),
        }
    }

    pub fn pipeline(&self) -> &PlaybackPipeline {
        &self.pipeline
    }

    pub fn pipeline_mut(&mut self) -> &mut PlaybackPipeline {
        &mut self.pipeline
    }

    pub fn queue(&self) -> &PlaybackQueue {
        &self.queue
    }

    pub fn schedule(
        &mut self,
        arrangement: &ArrangementView<'_>,
        transport: &Transport,
        tempo: &Tempo,
        probabilities: &Probabilities,
    ) -> Result<usize, PlaybackPipelineError> {
        let timed = self
            .pipeline
            .advance_timed(arrangement, transport, tempo, probabilities)?;

        let max_sample = timed.iter().map(|event| event.sample_position).max();
        let len = timed.len();
        self.queue.push_batch(timed);

        if let Some(sample) = max_sample {
            self.pipeline.mark_committed_horizon_sample(sample);
        }

        Ok(len)
    }

    pub fn process_until(&mut self, block_end_sample: u64) -> Vec<TimedPlaybackEvent> {
        let due = self.queue.drain_due(block_end_sample);
        for event in &due {
            self.executor.execute(event);
        }
        due
    }

    pub fn stop(&mut self, current_sample: u64, transport: &mut Transport) -> Vec<TimedPlaybackEvent> {
        let panic_offs = self.executor.panic_note_offs(current_sample);
        self.queue.clear();
        self.pipeline.reset();
        transport.stop();
        panic_offs
    }

    pub fn seek(&mut self, current_sample: u64, target_sample: u64, transport: &mut Transport) -> Vec<TimedPlaybackEvent> {
        let panic_offs = self.executor.panic_note_offs(current_sample);
        self.queue.clear();
        self.pipeline.reset();
        transport.set_sample_position(target_sample);
        panic_offs
    }

    pub fn rewind(&mut self, current_sample: u64, transport: &mut Transport) -> Vec<TimedPlaybackEvent> {
        self.seek(current_sample, 0, transport)
    }
}
