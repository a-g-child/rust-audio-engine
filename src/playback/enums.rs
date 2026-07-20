
use uuid::Uuid;
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum PlaybackEventKind {
    NoteOff{
        note: u8,
    },
    NoteOn{
        note: u8,
        velocity: u8,
    },
    ControlChange,
    ProgramChange,
    ParameterChange,
    PitchWheelChange,
    Aftertouch,
    ChannelPressure,
}
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum ProbabilityError {
    ChanceOutOfRange(u8),
    NilTargetId,
    TargetNotFound(Uuid),
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum ProbabilityTarget {
    Note,
    Parameter,
    Clip,    
}