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

    pub fn route_note_to_clip(&mut self, note_id: Uuid, clip_id: Uuid) {
        self.note_to_clip.insert(note_id, clip_id);
    }

    pub fn add_placement_to_clip(&mut self, clip_id: Uuid, placement_id: Uuid) {
        self.clip_to_placements
            .entry(clip_id)
            .or_default()
            .insert(placement_id);
    }

    pub fn clip_for_note(&self, note_id: &Uuid) -> Option<&Uuid> {
        self.note_to_clip.get(note_id)
    }

    pub fn placements_for_clip(&self, clip_id: &Uuid) -> impl Iterator<Item = &Uuid> {
        self.clip_to_placements
            .get(clip_id)
            .into_iter()
            .flatten()
    }

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
                placements.get(placement_id).map(|placement| ResolvedClipNote {
                    note,
                    clip,
                    placement,
                })
            })
            .collect()
    }

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

#[derive(Debug, Clone, Copy)]
pub struct ResolvedClipNote<'a> {
    note: &'a ScheduledNote,
    clip: &'a Clip,
    placement: &'a ClipPlacement,
}

impl<'a> ResolvedClipNote<'a> {
    pub fn note(&self) -> &'a ScheduledNote {
        self.note
    }

    pub fn clip(&self) -> &'a Clip {
        self.clip
    }

    pub fn placement(&self) -> &'a ClipPlacement {
        self.placement
    }

    pub fn placement_offset(&self) -> f64 {
        self.placement.start_beat()
    }

    pub fn clip_length(&self) -> f64 {
        self.clip.length()
    }

    pub fn clip_playback_mode(&self) -> ClipPlaybackMode {
        self.clip.playback_mode()
    }
}

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
    pub fn note(&self) -> &'a ScheduledNote {
        self.note
    }

    pub fn clip_id(&self) -> Uuid {
        self.clip_id
    }

    pub fn placement_id(&self) -> Uuid {
        self.placement_id
    }

    pub fn placement_start_beat(&self) -> f64 {
        self.placement_start_beat
    }

    pub fn placement_length(&self) -> f64 {
        self.placement_length
    }

    pub fn placement_end_beat(&self) -> f64 {
        self.placement_start_beat + self.placement_length
    }

    pub fn clip_length(&self) -> f64 {
        self.clip_length
    }

    pub fn clip_playback_mode(&self) -> ClipPlaybackMode {
        self.clip_playback_mode
    }
}

impl<'a> From<ResolvedClipNote<'a>> for RoutedNote<'a> {
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
}