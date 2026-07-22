//! Arrangement view is a read-only view of the current arrangement, which resolves note/clip/placement relationships before handing data to the scheduler. It is used to provide a convenient way to access routed notes for scheduling.
//! It is a borrowed view, meaning it does not own the data it references, and is intended to be used for scheduling purposes only. It is not intended to be used for editing or modifying the arrangement.
//! 
//! Responsibilities:
//! - Provide a read-only view of the current arrangement
//! - Resolve note/clip/placement relationships before handing data to the scheduler
//! 
//! Owns:
//! - None

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
	/// Create a new ArrangementView with the given notes, clips, placements, and router.
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
	/// Get all routed notes for the current arrangement view. This method resolves note/clip/placement relationships before returning the routed notes.
	pub fn routed_notes(&'a self) -> Vec<RoutedNote<'a>> {
		self.notes
			.iter()
			.flat_map(|note| self.router.resolve_routed_note(note, self.clips, self.placements))
			.collect()
	}
}
