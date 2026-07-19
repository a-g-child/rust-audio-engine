//! Musical timing primitives.
//!
//! Responsibilities:
//! - Represent a tempo in beats per minute (BPM), sample rate, and time signature.
//! - Calculate the number of seconds per beat based on the BPM.
//! - Convert between beats and samples.
//! 
//! Invariants:
//! - BPM is always positive and non-zero
//! - Sample rate is always positive and non-zero
//! 
//! Owns:
//! - BPM
//! - Sample rate
//! - Time signature
//! 
//! Does Not Own:
//! - Transport (playback state, sample position, playback speed)
//! - Scheduler
//! - Audio
//! - Midi

// Implementation of the Tempo struct
pub struct Tempo {
    bpm: f64,
    sample_rate: u64,
    time_signature: (u64, u64),
}
/// Implementation of the Tempo struct
impl Tempo {
    // Creates a new Tempo instance with the given BPM, sample rate, and time signature.
    pub fn new(bpm: f64, sample_rate: u64, time_signature: (u64, u64)) -> Self {
        if bpm <= 0.0 {
            panic!("BPM must be greater than zero");
        }
        if sample_rate == 0 {
            panic!("Sample rate must be greater than zero");
        }
        Tempo {
            bpm,
            sample_rate,
            time_signature,
        }
    }
    /// Gets the current BPM.
    pub fn bpm(&self) -> f64 {
        self.bpm
    }
    /// Sets the BPM to a new value.
    pub fn set_bpm(&mut self, bpm: f64) {
        if bpm <= 0.0 {
            panic!("BPM must be greater than zero");
        }
        self.bpm = bpm;
    }
    /// Gets the current sample rate.
    pub fn sample_rate(&self) -> u64 {
        self.sample_rate
    }
    /// Sets the sample rate to a new value.
    pub fn set_sample_rate(&mut self, sample_rate: u64) {
        if sample_rate == 0 {
            panic!("Sample rate must be greater than zero");
        }
        self.sample_rate = sample_rate;
    }
    /// Gets the current time signature.
    pub fn time_signature(&self) -> (u64, u64) {
        self.time_signature
    }
    /// Sets the time signature to a new value.
    pub fn set_time_signature(&mut self, time_signature: (u64, u64)) {
        self.time_signature = time_signature;
    }
    /// Gets the number of seconds per beat based on the current BPM.
    pub fn seconds_per_beat(&self) -> f64 {
        60.0 / self.bpm
    }
    /// Gets the number of samples per beat based on the current BPM and sample rate.
    pub fn samples_per_beat(&self) -> u64 {
        (self.seconds_per_beat() * self.sample_rate as f64) as u64
    }
    /// Converts a duration in seconds to the equivalent number of beats based on the current BPM.
    pub fn beats_from_seconds(&self, seconds: f64) -> f64 {
        seconds * self.bpm / 60.0
    }
    /// Converts seconds to samples based on the current sample rate.
    pub fn seconds_to_samples(&self, seconds: f64) -> u64 {
        (seconds * self.sample_rate as f64) as u64
    }

    /// Converts samples to seconds based on the current sample rate.
    pub fn samples_to_seconds(&self, samples: u64) -> f64 {
        samples as f64 / self.sample_rate as f64
    }
    /// Converts a duration in beats to the equivalent number of samples based on the current BPM and sample rate.
    pub fn beats_to_samples(&self, beats: f64) -> u64 {
        (beats * self.samples_per_beat() as f64).round() as u64
    }
    /// Converts a duration in samples to the equivalent number of beats based on the current BPM and sample rate.
    pub fn samples_to_beats(&self, samples: u64) -> f64 {
        samples as f64 / self.samples_per_beat() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_creation() {
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        assert_eq!(tempo.bpm(), 120.0);
        assert_eq!(tempo.sample_rate(), 44_100);
        assert_eq!(tempo.time_signature(), (4, 4));
    }
    #[test]
    fn get_and_set_bpm() {
        let mut tempo = Tempo::new(120.0, 44_100, (4, 4));
        tempo.set_bpm(90.0);
        assert_eq!(tempo.bpm(), 90.0);
    }
    #[test]
    fn get_and_set_sample_rate() {
        let mut tempo = Tempo::new(120.0, 44_100, (4, 4));
        tempo.set_sample_rate(48_000);
        assert_eq!(tempo.sample_rate(), 48_000);
    }
    #[test]
    fn test_seconds_per_beat() {
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        assert_eq!(tempo.seconds_per_beat(), 0.5);  
    }
    #[test]
    fn test_samples_per_beat() {
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        assert_eq!(tempo.samples_per_beat(), 22_050);
    }
    #[test]
    fn test_beats_from_seconds() {
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        assert_eq!(tempo.beats_from_seconds(1.0), 2.0);  
    }
    #[test]
    fn test_seconds_to_samples() {
        let tempo = Tempo::new(120.0, 44_100, (4, 4));
        assert_eq!(tempo.seconds_to_samples(1.0), 44_100);
    }
    #[test]
    fn test_samples_to_seconds() {
        let tempo = Tempo::new(120.0, 44_100, (4    , 4));
        assert_eq!(tempo.samples_to_seconds(44_100), 1.0);  
    }
}