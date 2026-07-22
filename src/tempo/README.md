# Tempo Crate

The tempo crate contains musical timing primitives and beat/sample conversion utilities.

## Responsibilities

- Represent BPM, sample rate, and time signature.
- Convert among beats, samples, and seconds.
- Provide timing math used by transport and scheduler windows.

## Main Types

- Tempo: immutable-style timing model with mutation setters for BPM/sample rate/time signature.

## Key Functions

- Tempo::new(bpm, sample_rate, time_signature)
- Tempo::bpm(), Tempo::set_bpm(bpm)
- Tempo::sample_rate(), Tempo::set_sample_rate(sample_rate)
- Tempo::time_signature(), Tempo::set_time_signature(signature)
- Tempo::seconds_per_beat()
- Tempo::samples_per_beat()
- Tempo::beats_from_seconds(seconds)
- Tempo::seconds_to_samples(seconds)
- Tempo::samples_to_seconds(samples)
- Tempo::beats_to_samples(beats)
- Tempo::samples_to_beats(samples)

## Boundary Notes

- Tempo does not own playback state.
- It is used by transport and scheduler for position and window calculations.
