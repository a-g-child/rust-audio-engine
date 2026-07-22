# Playback Crate

The playback crate maps scheduler events into playback-domain events and applies probability gating logic.

## Responsibilities

- Define executable playback events.
- Define and store probability models for targets.
- Gate note events probabilistically while preserving On/Off pairing behavior.
- Translate scheduler events into playback events.

## Main Types

- PlaybackEvent: playback-layer event with beat, note identity, and event kind.
- TimedPlaybackEvent: playback event with an absolute sample_position for realtime execution.
- PlaybackEventKind: event payload enum (NoteOn, NoteOff, and future event kinds).
- Probability and Probabilities: chance values and keyed collection.
- ProbabilityGate: gating stage that accepts or rejects occurrences.
- ActiveNotes: tracks active note occurrences and can emit panic NoteOff events on resets/discontinuities.

## Key Functions

- Probabilities::new()
- Probabilities::add(target_id, chance, target)
- Probabilities::update(target_id, chance, target)
- Probabilities::get(target_id)
- Probabilities::remove(target_id)
- Probabilities::clear(), Probabilities::len(), Probabilities::is_empty(), Probabilities::contains(target_id)
- ProbabilityGate::new()
- ProbabilityGate::clear()
- ProbabilityGate::apply(event, probabilities)
- From<&ScheduledEvent> for PlaybackEvent
- TimedPlaybackEvent::from_playback_event(event, tempo)

## Discontinuity Handling

- PlaybackPipeline::reset() clears scheduler cursor state, probability gate runtime state, and active-note tracking.
- PlaybackPipeline::reset_with_panic_note_offs(beat) returns synthetic NoteOff events for currently active notes before clearing state.
- PlaybackPipeline::advance_timed(...) returns events with sample deadlines.
- PlaybackPipeline::reset_with_panic_note_offs_timed(beat, tempo) returns panic NoteOff events with sample deadlines.
- This is intended for stop/seek/rewind-style discontinuities to prevent held notes.

## Committed Horizon

- PlaybackPipeline tracks committed_horizon_beat as the scheduler cursor after each successful advance.
- This value represents the furthest beat currently considered committed by this pipeline instance.
- reset() and reset_with_panic_note_offs(...) clear the committed horizon.
- mutation_decision_from_beat(from_beat) reports whether a mutation can be applied without rewriting committed time.
- can_apply_mutation_from_beat(from_beat) is a convenience boolean wrapper.
- apply_if_mutable_from_beat(from_beat, op) executes a closure only when mutation is allowed and otherwise returns Rejected.
- mutation_decision_for_batch(from_beats) evaluates an all-or-nothing decision for a mutation batch.
- can_apply_mutation_batch(from_beats) is a boolean helper for batch checks.
- apply_if_mutable_batch(from_beats, op) executes only when every beat in the batch is allowed.
- MutationBatch stores named operations and source beats for batch guard checks.
- mutation_decision_for_named_batch(batch) returns the exact operation label and beat that violated the committed horizon.
- can_apply_mutation_named_batch(batch) and apply_if_mutable_named_batch(batch, op) provide all-or-nothing named batch wrappers.

## Boundary Notes

- This crate consumes scheduler output.
- It does not resolve clip relationships or calculate schedule windows.
