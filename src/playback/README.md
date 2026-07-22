# Playback Crate

The playback crate maps scheduler events into playback-domain events and applies probability gating logic.

## Responsibilities

- Define executable playback events.
- Define and store probability models for targets.
- Gate note events probabilistically while preserving On/Off pairing behavior.
- Translate scheduler events into playback events.

## Main Types

- PlaybackEvent: playback-layer event with beat, note identity, and event kind.
- PlaybackEventKind: event payload enum (NoteOn, NoteOff, and future event kinds).
- Probability and Probabilities: chance values and keyed collection.
- ProbabilityGate: gating stage that accepts or rejects occurrences.

## Key Functions

- Probabilities::new()
- Probabilities::add(note_id, chance, target)
- Probabilities::update(note_id, chance, target)
- Probabilities::get(note_id)
- Probabilities::remove(note_id)
- Probabilities::set_applied(note_id, applied)
- Probabilities::clear(), Probabilities::len(), Probabilities::is_empty(), Probabilities::contains(note_id)
- ProbabilityGate::new()
- ProbabilityGate::apply(event, probabilities)
- From<&ScheduledEvent> for PlaybackEvent

## Boundary Notes

- This crate consumes scheduler output.
- It does not resolve clip relationships or calculate schedule windows.
