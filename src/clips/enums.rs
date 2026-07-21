

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipPlaybackMode {
    OneShot,
    Loop { start_beat: f64, end_beat: f64 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipPlaybackModeError {
    LoopBoundsNotFinite,
    LoopStartNegative,
    LoopEndNotAfterStart,
    LoopEndExceedsClipLength,
}