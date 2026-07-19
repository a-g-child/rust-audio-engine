//! Transport Module
//!
//! Responsibility:
//! - manage playback state (playing, paused, stopped)
//! - manage sample position
//! - calculate beat position based on tempo
//!
//! Invariants:
//! - sample position is always non-negative
//! 
//! Owns: 
//! - playback state
//! - sample position
//! 
//! Does Not Own:
//! - Tempo (BPM, beat position, time signature)
//! - Scheduler
//! - Audio
//! - Midi

use crate::tempo::Tempo;
use crate::transport::enums::TransportState;
/// Transport struct manages playback state and sample position.
pub struct Transport {
    state: TransportState,
    sample_position: u64,
}
/// Implementation of the Transport struct
impl Transport {
    pub fn new() -> Self {
        Transport {
            state: TransportState::Stopped,
            sample_position: 0,
        }
    }
    pub fn state(&self) -> TransportState {
        self.state
    }
    /// Starts playback
    pub fn play(&mut self) {
        if self.state == TransportState::Playing {
            println!("Transport is already playing");
            return;
        }
        self.state = TransportState::Playing;
    }
    /// Pauses playback
    pub fn pause(&mut self) {
        self.state = TransportState::Paused;
    }
    /// Stops playback and resets positions
    pub fn stop(&mut self) {
        self.state = TransportState::Stopped;
        self.sample_position = 0;
    }
    /// Advances the transport by a given number of samples
    pub fn advance_s(&mut self, samples: u64) {
        if self.state == TransportState::Playing {
            self.sample_position += samples;
        }
    }
    pub fn advance_b(&mut self, beats: f64, tempo: &Tempo) {
        if self.state == TransportState::Playing {
            let samples_to_advance = tempo.beats_to_samples(beats);
            self.sample_position += samples_to_advance;
        }
    }
    /// Gets the current playback state
    pub fn is_playing(&self) -> bool {
        self.state == TransportState::Playing
    }
    /// Gets the current sample position
    pub fn sample_position(&self) -> u64 {
        self.sample_position
    }
    /// Sets the sample position
    pub fn set_sample_position(&mut self, position: u64) {
        // No need to check for negative values since u64 is always non-negative
        self.sample_position = position;
    }
    /// Gets the current beat position based on the provided tempo
    pub fn beat_position(&self, tempo: &Tempo) -> f64 {
        tempo.samples_to_beats(self.sample_position)
        // self.sample_position as f64 / tempo.samples_per_beat() as f64
    }
    /// Gets the current bar position based on the provided tempo
    pub fn bar_position(&self, tempo: &Tempo) -> f64 {
        let beats_per_bar = tempo.time_signature().0 as f64;
        self.beat_position(tempo) / beats_per_bar
    }
    /// Gets the current time in milliseconds based on the sample position and sample rate
    pub fn time_now_ms(&self, sample_rate: u64) -> f64 {
        self.sample_position as f64 / sample_rate as f64 * 1000.0
    }
}   

#[cfg(test)]
mod tests {
    use crate::tempo::Tempo;
    use crate::transport::Transport;

    #[test]
    fn test_transport_create() {
        let transport = Transport::new();
        assert_eq!(transport.is_playing(), false);
        assert_eq!(transport.sample_position(), 0);
    }
    #[test]
    fn test_transport_play() {
        let mut transport = Transport::new();
        transport.play();
        assert_eq!(transport.is_playing(), true);
    }
    #[test]
    fn test_transport_play_pause() {
        let mut transport = Transport::new();
        transport.play();
        assert_eq!(transport.is_playing(), true);
        transport.pause();
        assert_eq!(transport.is_playing(), false);
    }
    #[test]
    fn test_transport_play_stop() {
        let mut transport = Transport::new();
        transport.play();
        assert_eq!(transport.is_playing(), true);
        transport.stop();
        assert_eq!(transport.is_playing(), false);
        assert_eq!(transport.sample_position(), 0);
    }
    #[test]
    fn test_transport_play_already_playing() {
        let mut transport = Transport::new();
        transport.play();
        transport.play(); // Should print a message but not change state
        assert_eq!(transport.is_playing(), true);
    }
    #[test]
    fn test_transport_pause() {
        let mut transport = Transport::new();
        transport.play();
        transport.pause();
        assert_eq!(transport.is_playing(), false);
    }
    #[test]
    fn test_transport_stop() {
        let mut transport = Transport::new();
        transport.play();
        transport.advance_s(44100);
        transport.stop();
        assert_eq!(transport.is_playing(), false);
        assert_eq!(transport.sample_position(), 0);
        // assert_eq!(transport.beat_position, 0.00);
    }
    #[test]
    fn test_transport_advance() {
        let mut transport = Transport::new();
        transport.play();
        transport.advance_s(512);
        assert_eq!(transport.sample_position(), 512);
    }   
    #[test]
    fn test_transport_advance_not_playing() {
        let mut transport = Transport::new();
        transport.advance_s(512);
        assert_eq!(transport.sample_position(), 0);
    }
    #[test]
    fn test_transport_advance_play_pause() {
        let mut transport = Transport::new();
        transport.play();
        transport.advance_s(512);
        assert_eq!(transport.sample_position(), 512);
        transport.pause();
        transport.advance_s(512); // Should not advance since paused
        assert_eq!(transport.sample_position(), 512);
    }
    #[test]
    fn test_transport_set_sample_position() {
        let mut transport = Transport::new();
        transport.set_sample_position(44100);
        assert_eq!(transport.sample_position(), 44100);
    }
      #[test]
    fn test_transport_set_sample_position_zero() {
        let mut transport = Transport::new();
        transport.set_sample_position(0);
        assert_eq!(transport.sample_position()  , 0);
    }
    // unable to add sample position negative test as u64 cannot be negative  
    #[test]
    fn test_transport_get_beat_position() {
        let mut transport = Transport::new();
        let tempo = Tempo::new(120.0, 44100, (4, 4));
        transport.set_sample_position(44100); // 1 second at 44100 Hz
        let beat_position = transport.beat_position(&tempo);
        assert_eq!(beat_position, 2.0); // At 120 BPM, 1 second is 2 beats
    }
 
    #[test]
    fn test_transport_get_time_now_ms() {
        let mut transport = Transport::new();
        transport.set_sample_position(44100);
        let sample_rate = 44100;
        let time_now_ms = transport.time_now_ms(sample_rate);
        assert_eq!(time_now_ms, 1000.0); // 44100 samples       
    }
  

}

