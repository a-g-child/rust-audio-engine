use crate::clips::{ClipPlacements, ClipRouter, Clips, RoutedNote};
use crate::scheduler::ScheduledNote;

// Borrowed orchestration view that resolves note/clip/placement relationships
// before handing data to the scheduler.
pub struct ArrangementView<'a> {
	notes: &'a [ScheduledNote],
	clips: &'a Clips,
	placements: &'a ClipPlacements,
	router: &'a ClipRouter,
}

impl<'a> ArrangementView<'a> {
	pub fn new(
		notes: &'a [ScheduledNote],
		clips: &'a Clips,
		placements: &'a ClipPlacements,
		router: &'a ClipRouter,
	) -> Self {
		Self {
			notes,
			clips,
			placements,
			router,
		}
	}

	pub fn routed_notes(&'a self) -> Vec<RoutedNote<'a>> {
		self.notes
			.iter()
			.flat_map(|note| self.router.resolve_routed_note(note, self.clips, self.placements))
			.collect()
	}
}
