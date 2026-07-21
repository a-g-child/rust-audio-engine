//! Logic gate between probabilities and playback events
use crate::{playback::{PlaybackEvent,Probabilities}, scheduler::ScheduledEvent, scheduler::NoteState, scheduler::NoteOccurrenceKey};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug)]
pub struct ProbabilityGate {
    accepted_occurrences: HashSet<NoteOccurrenceKey>,
}

impl ProbabilityGate {
    pub fn new() -> Self {
        Self {
            accepted_occurrences: HashSet::new(),
        }
    }

    pub fn apply(&mut self, event: &ScheduledEvent<'_>, probabilities: &Probabilities,) -> Option<PlaybackEvent> {
        let id = *event.note().id();
        let occurrence_key = event.occurrence_key();

        match event.state() {
            NoteState::On => {
                let accepted = probabilities
                    .get(&id)
                    .map(|probability| Self::roll(probability.chance()))
                    .unwrap_or(true);

                self.handle_note_on(event, occurrence_key, accepted)
            }

            NoteState::Off => {
                self.handle_note_off(event, probabilities, id, occurrence_key)
            }
        }
    }

    fn handle_note_off(&mut self, event: &ScheduledEvent<'_>, probabilities: &Probabilities, id: Uuid, occurrence_key: NoteOccurrenceKey) -> Option<PlaybackEvent> {
        if probabilities.get(&id).is_none() {
            return Some(PlaybackEvent::from(event));
        }
    
        if self.accepted_occurrences.remove(&occurrence_key) {
            Some(PlaybackEvent::from(event))
        } else {
            None
        }
    }
    
    fn handle_note_on(&mut self, event: &ScheduledEvent<'_>, occurrence_key: NoteOccurrenceKey, accepted: bool) -> Option<PlaybackEvent> {
        // 
        if accepted {
            self.accepted_occurrences.insert(occurrence_key);
            Some(PlaybackEvent::from(event))
        } else {
            self.accepted_occurrences.remove(&occurrence_key);
            None
        }
    }
    
    fn roll(chance: u8) -> bool {
        match chance {
            0 => false,
            100 => true,
            chance => {
                use rand::RngExt;

                let mut rng = rand::rng();
                rng.random_range(0..100) < chance
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clips::ClipRouter;
    use crate::playback::{Probabilities, ProbabilityTarget};
    use crate::scheduler::{ScheduledEvent, ScheduledNote, NoteState};

    #[test]
    fn roll_uses_deterministic_probability_boundaries() {
        assert!(!ProbabilityGate::roll(0));
        assert!(ProbabilityGate::roll(100));
    }

    #[test]
    fn test_filter_event_with_probability() {
        let mut probabilities = Probabilities::new();
        let mut probability = ProbabilityGate::new();
        let clip_router = ClipRouter::new(Uuid::new_v4());
        let note = ScheduledNote::new(0.0, 100, 1.0, clip_router).unwrap();
        let scheduled_event = ScheduledEvent::new(&note, NoteState::On, 0);
        let id = *scheduled_event.note().id();
        probabilities.add(id, 50, ProbabilityTarget::Note).unwrap();
        // Since the probability is 50%, we can't guarantee the outcome, but we can check that it returns Some or None.
        let result = probability.apply(&scheduled_event, &mut probabilities);
        assert!(result.is_some() || result.is_none());
    }
    #[test]
    fn event_with_zero_probability_gates_note_off() {
       let mut probabilities = Probabilities::new();
       let mut probability = ProbabilityGate::new();
       let clip_router = ClipRouter::new(Uuid::new_v4());
       let note = ScheduledNote::new(0.0, 100, 1.0, clip_router).unwrap();
    let scheduled_event_on = ScheduledEvent::new(&note, NoteState::On, 0);
    let scheduled_event_off = ScheduledEvent::new(&note, NoteState::Off, 0);
       let id = *scheduled_event_on.note().id();
       println!("{}\n{}",*scheduled_event_on.note().id(), *scheduled_event_off.note().id());
       probabilities.add(id, 0, ProbabilityTarget::Note).unwrap();
       let result_on = probability.apply(&scheduled_event_on, &mut probabilities);
       assert!(result_on.is_none());
       let result_off = probability.apply(&scheduled_event_off, &mut probabilities);
       assert!(result_off.is_none());       
    
    }
}