//! ClipRouter is responsible for resolving relationships between notes, clips, and clip placements.
//! 
//! Responsibilities:
//! - Map notes to clips
//! - Map clips to placements
//! 
//! Owns:
//! - HashMap<note_id, clip_id>
//! 
//! Does not own:
//! - Clips
//! - ClipPlacements
//! - Notes
//! - Note Events

use crate::clips::clip_placement::{ClipPlacement, ClipPlacements};
use crate::clips::enums::ClipPlaybackMode;
use crate::clips::{Clip, Clips};
use crate::scheduler::scheduled_note::ScheduledNote;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

// ClipRouter resolves note -> clip and clip -> placements relationships.
#[derive(Debug, PartialEq, Clone)]
pub struct ClipRouter {
    note_to_clip: HashMap<Uuid, Uuid>,
    clip_to_placements: HashMap<Uuid, HashSet<Uuid>>,
}

impl ClipRouter {
    pub fn new() -> Self {
        Self {
            note_to_clip: HashMap::new(),
            clip_to_placements: HashMap::new(),
        }
    }
    /// Map a note to a clip. A note can only be routed to one clip.
    pub fn route_note_to_clip(&mut self, note_id: Uuid, clip_id: Uuid) -> Option<Uuid> {
        self.note_to_clip.insert(note_id, clip_id)
    }
    /// Map a placement to a clip. A clip can have multiple placements.
    pub fn add_placement_to_clip(&mut self, clip_id: Uuid, placement_id: Uuid) {
        self.clip_to_placements
            .entry(clip_id)
            .or_default()
            .insert(placement_id);
    }

    /// Remove a note to clip routing if present.
    pub fn unroute_note(&mut self, note_id: &Uuid) -> Option<Uuid> {
        self.note_to_clip.remove(note_id)
    }

    /// Remove a placement from a clip if present.
    pub fn remove_placement_from_clip(&mut self, clip_id: &Uuid, placement_id: &Uuid) -> bool {
        let Some(placements) = self.clip_to_placements.get_mut(clip_id) else {
            return false;
        };

        let removed = placements.remove(placement_id);
        if placements.is_empty() {
            self.clip_to_placements.remove(clip_id);
        }
        removed
    }

    /// Remove all routing relationships for a clip and return removed placement IDs.
    pub fn remove_clip(&mut self, clip_id: &Uuid) -> Option<HashSet<Uuid>> {
        self.clip_to_placements.remove(clip_id)
    }
    // Resolve a note to its clip and placements. Returns an empty vector if the note is not routed to a clip or if the clip has no placements.
    pub fn clip_for_note(&self, note_id: &Uuid) -> Option<&Uuid> {
        self.note_to_clip.get(note_id)
    }
    // Get all placements for a clip. Returns an empty iterator if the clip has no placements.
    pub fn placements_for_clip(&self, clip_id: &Uuid) -> impl Iterator<Item = &Uuid> {
        self.clip_to_placements
            .get(clip_id)
            .into_iter()
            .flatten()
    }
    // Resolve a note to its clip and placements. Returns an empty vector if the note is not routed to a clip or if the clip has no placements.
    pub fn resolve_note<'a>(
        &'a self,
        note: &'a ScheduledNote,
        clips: &'a Clips,
        placements: &'a ClipPlacements,
    ) -> Vec<ResolvedClipNote<'a>> {
        let Some(clip_id) = self.clip_for_note(note.id()) else {
            return Vec::new();
        };

        let Some(clip) = clips.get(clip_id) else {
            return Vec::new();
        };

        self.placements_for_clip(clip_id)
            .filter_map(|placement_id| {
                let placement = placements.get(placement_id)?;
                if placement.clip_id() != clip_id {
                    return None;
                }

                Some(ResolvedClipNote {
                    note,
                    clip,
                    placement,
                })
            })
            .collect()
    }
    // Resolve a note to its clip and placements, and return a vector of RoutedNote structs. Returns an empty vector if the note is not routed to a clip or if the clip has no placements.
    pub fn resolve_routed_note<'a>(
        &'a self,
        note: &'a ScheduledNote,
        clips: &'a Clips,
        placements: &'a ClipPlacements,
    ) -> Vec<RoutedNote<'a>> {
        self.resolve_note(note, clips, placements)
            .into_iter()
            .map(RoutedNote::from)
            .collect()
    }
}

/// Return type for the `resolve_note` method, which contains references to the note, clip, and placement.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedClipNote<'a> {
    note: &'a ScheduledNote,
    clip: &'a Clip,
    placement: &'a ClipPlacement,
}

impl<'a> ResolvedClipNote<'a> {
    /// Get the note, clip, and placement for this resolved note.
    pub fn note(&self) -> &'a ScheduledNote {
        self.note
    }
    /// Get the clip for this resolved note.
    pub fn clip(&self) -> &'a Clip {
        self.clip
    }
    /// Get the placement for this resolved note.
    pub fn placement(&self) -> &'a ClipPlacement {
        self.placement
    }
    /// Get the start beat of the placement for this resolved note.
    pub fn placement_offset(&self) -> f64 {
        self.placement.start_beat()
    }
    /// Get the length of the placement for this resolved note.
    pub fn clip_length(&self) -> f64 {
        self.clip.length()
    }
    /// Get the playback mode of the clip for this resolved note.
    pub fn clip_playback_mode(&self) -> ClipPlaybackMode {
        self.clip.playback_mode()
    }
}

/// RoutedNote is a struct that contains references to a ScheduledNote, Clip, and ClipPlacement, along with the placement's start beat and length, and the clip's length and playback mode.
#[derive(Debug, Clone, Copy)]
pub struct RoutedNote<'a> {
    note: &'a ScheduledNote,
    clip_id: Uuid,
    placement_id: Uuid,
    placement_start_beat: f64,
    placement_length: f64,
    clip_length: f64,
    clip_playback_mode: ClipPlaybackMode,
}

impl<'a> RoutedNote<'a> {
    /// Create a new RoutedNote from a ResolvedClipNote.
    pub fn note(&self) -> &'a ScheduledNote {
        self.note
    }
    /// Get the clip ID for this routed note.
    pub fn clip_id(&self) -> Uuid {
        self.clip_id
    }
    /// Get the placement ID for this routed note.
    pub fn placement_id(&self) -> Uuid {
        self.placement_id
    }
    /// Get the start beat of the clip's placement for this routed note.
    pub fn placement_start_beat(&self) -> f64 {
        self.placement_start_beat
    }
    /// Get the length of the clip's placement for this routed note.
    pub fn placement_length(&self) -> f64 {
        self.placement_length
    }
    /// Get the end beat of the clip's placement for this routed note.
    pub fn placement_end_beat(&self) -> f64 {
        self.placement_start_beat + self.placement_length
    }

    /// Get the length of the clip for this routed note.
    pub fn clip_length(&self) -> f64 {
        self.clip_length
    }

    /// Get the playback mode of the clip for this routed note.
    pub fn clip_playback_mode(&self) -> ClipPlaybackMode {
        self.clip_playback_mode
    }
}

impl<'a> From<ResolvedClipNote<'a>> for RoutedNote<'a> {
    /// Create a new RoutedNote from a ResolvedClipNote.
    fn from(value: ResolvedClipNote<'a>) -> Self {
        Self {
            note: value.note,
            clip_id: *value.clip.id(),
            placement_id: *value.placement.id(),
            placement_start_beat: value.placement.start_beat(),
            placement_length: value.placement.length(),
            clip_length: value.clip.length(),
            clip_playback_mode: value.clip.playback_mode(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clips::enums::ClipPlaybackMode;

    #[test]
    fn resolves_one_item_per_placement() {
        let note = ScheduledNote::new(1.0, 64, 0.5).unwrap();

        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_id = *clip.id();

        let placement_a = ClipPlacement::new(clip_id, 0.0, 8.0).unwrap();
        let placement_b = ClipPlacement::new(clip_id, 16.0, 8.0).unwrap();

        let mut clips = Clips::new();
        clips.add(clip);

        let mut placements = ClipPlacements::new();
        let placement_a_id = placements.add(placement_a);
        let placement_b_id = placements.add(placement_b);

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_id);
        router.add_placement_to_clip(clip_id, placement_a_id);
        router.add_placement_to_clip(clip_id, placement_b_id);

        let routed = router.resolve_routed_note(&note, &clips, &placements);
        assert_eq!(routed.len(), 2);
    }

    #[test]
    fn route_note_to_clip_returns_previous_clip() {
        let note_id = Uuid::new_v4();
        let clip_a = Uuid::new_v4();
        let clip_b = Uuid::new_v4();

        let mut router = ClipRouter::new();
        assert!(router.route_note_to_clip(note_id, clip_a).is_none());
        assert_eq!(router.route_note_to_clip(note_id, clip_b), Some(clip_a));
        assert_eq!(router.clip_for_note(&note_id), Some(&clip_b));
    }

    #[test]
    fn can_remove_note_and_clip_placement_routes() {
        let note_id = Uuid::new_v4();
        let clip_id = Uuid::new_v4();
        let placement_id = Uuid::new_v4();

        let mut router = ClipRouter::new();
        router.route_note_to_clip(note_id, clip_id);
        router.add_placement_to_clip(clip_id, placement_id);

        assert_eq!(router.unroute_note(&note_id), Some(clip_id));
        assert!(router.clip_for_note(&note_id).is_none());

        assert!(router.remove_placement_from_clip(&clip_id, &placement_id));
        assert_eq!(router.placements_for_clip(&clip_id).count(), 0);
    }

    #[test]
    fn remove_clip_returns_removed_placements() {
        let clip_id = Uuid::new_v4();
        let placement_a = Uuid::new_v4();
        let placement_b = Uuid::new_v4();

        let mut router = ClipRouter::new();
        router.add_placement_to_clip(clip_id, placement_a);
        router.add_placement_to_clip(clip_id, placement_b);

        let removed = router.remove_clip(&clip_id).unwrap();
        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&placement_a));
        assert!(removed.contains(&placement_b));
        assert_eq!(router.placements_for_clip(&clip_id).count(), 0);
    }

    #[test]
    fn wrong_clip_placement_route_is_not_resolved() {
        let note = ScheduledNote::new(1.0, 64, 0.5).unwrap();

        let clip_a = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_a_id = *clip_a.id();
        let clip_b = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let clip_b_id = *clip_b.id();

        let mut clips = Clips::new();
        clips.add(clip_a);
        clips.add(clip_b);

        // Placement belongs to clip_b but we route it under clip_a in the router.
        let mismatched_placement = ClipPlacement::new(clip_b_id, 0.0, 8.0).unwrap();
        let mut placements = ClipPlacements::new();
        let mismatched_placement_id = placements.add(mismatched_placement);

        let mut router = ClipRouter::new();
        router.route_note_to_clip(*note.id(), clip_a_id);
        router.add_placement_to_clip(clip_a_id, mismatched_placement_id);

        let routed = router.resolve_routed_note(&note, &clips, &placements);
        assert!(routed.is_empty());
    }
}