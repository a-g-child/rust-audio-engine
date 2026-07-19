
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum SchedulerError {
    InvalidLookahead,
    InvalidBeatStart,
    InvalidBeatEnd,
    InvalidNegativeWindow,
    InvalidCursorPosition,
    InvalidTransportPosition,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum NoteState {
    Off,
    On,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum ScheduledNoteError {
    InvalidStartBeat,
    InvalidLength,
    InvalidNoteValue,
    InvalidVelocity,
    InvalidProbability,
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum PlaybackEventType {
    NoteOff,
    NoteOn,
    ControlChange,
    ProgramChange,
    ParameterChange,
    PitchWheelChange,
    Aftertouch,
    ChannelPressure,
}