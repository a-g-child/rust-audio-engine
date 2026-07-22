# Scheduler Crate

The scheduler crate materializes note-edge events (On/Off) in a lookahead window using routing-resolved note context.

## Responsibilities

- Own lookahead window state (cursor and last observed transport beat).
- Convert routed note context into global scheduled events.
- Support one-shot and looped clip event expansion.
- Emit placement-aware occurrence identities for downstream gating.

## Main Types

- ScheduledNote: note-local timing and note data.
- ScheduledEvent: materialized global event with resolved beat and occurrence identity.
- NoteOccurrenceKey: unique identity for note occurrences (note_id + placement_id + loop_iteration).
- Scheduler: lookahead window engine.

## Key Functions

- ScheduledNote::new(start_beat, note, length)
- ScheduledNote::move_to(start_beat)
- ScheduledNote::set_length(length)
- ScheduledNote::set_velocity(velocity)
- ScheduledEvent::new(note, state, scheduled_beat, occurrence_key)
- ScheduledEvent::beat(), ScheduledEvent::state(), ScheduledEvent::occurrence_key()
- NoteOccurrenceKey::new(note_id, placement_id, loop_iteration)
- Scheduler::new()
- Scheduler::set_lookahead(lookahead)
- Scheduler::advance_window(routed_notes, transport, tempo)

## Boundary Notes

- The scheduler consumes RoutedNote inputs from the clips layer.
- It should not own or traverse clip registries directly.
