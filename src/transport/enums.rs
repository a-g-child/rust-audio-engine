#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub enum TransportState {
    Playing,
    Paused,
    Stopped,
    Rewinding,
    Seeking,
    SeekingBackward,
    FastForwarding,
    FastForwardingBackward,
}