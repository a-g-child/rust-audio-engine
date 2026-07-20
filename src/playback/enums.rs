#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum PlaybackEventType {
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