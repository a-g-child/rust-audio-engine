# Clips Crate

The clips crate models reusable musical content and where that content appears on the global arrangement timeline.

## Responsibilities

- Represent clip-local musical content metadata.
- Represent clip placements in global arrangement beat space.
- Own relationship routing between notes, clips, and placements.
- Provide an arrangement view that resolves scheduler-ready routed notes.

## Main Types

- Clip: clip-local content metadata with length and playback mode.
- Clips: clip registry keyed by UUID.
- ClipPlacement: one global placement for a clip.
- ClipPlacements: placement registry keyed by UUID.
- ClipRouter: note-to-clip and clip-to-placement routing.
- ResolvedClipNote: borrowed resolved view (note + clip + placement).
- RoutedNote: lightweight scheduling input with placement context.
- ArrangementView: orchestration helper that resolves all routed notes.

## Key Functions

- Clip::new(length, playback_mode)
- Clips::new(), Clips::add(clip), Clips::get(id), Clips::iter()
- ClipPlacement::new(clip_id, start_beat, length)
- ClipPlacements::new(), ClipPlacements::add(placement), ClipPlacements::get(id), ClipPlacements::iter()
- ClipRouter::new()
- ClipRouter::route_note_to_clip(note_id, clip_id)
- ClipRouter::add_placement_to_clip(clip_id, placement_id)
- ClipRouter::resolve_note(note, clips, placements)
- ClipRouter::resolve_routed_note(note, clips, placements)
- ArrangementView::new(notes, clips, placements, router)
- ArrangementView::routed_notes()

## Boundary Notes

- This crate resolves relationships and context.
- It does not schedule windows or materialize final playback events.
- The scheduler should consume RoutedNote inputs instead of traversing clip registries directly.