# Transport Crate

The transport crate owns global playback state and position.

## Responsibilities

- Track playback state (Stopped, Playing, Paused, Rewinding).
- Track sample position and expose beat/bar/time views.
- Advance position in samples or beats while playing.

## Main Types

- Transport: global playback position/state model.
- TransportState: playback state enum.

## Key Functions

- Transport::new()
- Transport::state(), Transport::is_playing()
- Transport::play(), Transport::pause(), Transport::stop()
- Transport::advance_s(samples)
- Transport::advance_b(beats, tempo)
- Transport::sample_position(), Transport::set_sample_position(position)
- Transport::beat_position(tempo)
- Transport::bar_position(tempo)
- Transport::time_now_ms(sample_rate)
- Transport::loop_iteration()

## Boundary Notes

- Transport does not manage clip-local timing.
- Clips and scheduler map arrangement/clip context onto this global transport timeline.
