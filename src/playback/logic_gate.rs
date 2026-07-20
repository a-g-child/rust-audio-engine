//! Logic gate between probabilities and playback events
use crate::{playback::{PlaybackEvent,Probabilities}, scheduler::ScheduledEvent, scheduler::NoteState};
use rand::{Rng, RngExt};

/// Filters a `PlaybackEvent` based on the provided `Probabilities`.
/// If the event is a `NoteOn` event, it checks the probability associated with the note's UUID. 
/// If the random chance is less than the probability, the event is returned; otherwise, `None` is returned.
/// if 100% chance or 0% chance, it will always return the event or None respectively.
pub fn filter_event(event: &ScheduledEvent, probabilities: &Probabilities) -> Option<PlaybackEvent> {
    println!("starting Filter");
    if event.state() == NoteState::On {
        println!("Filtering event for note ID: {:?}\n", event.note().id());
        let note_id = event.note().id();
        if let Some(probability) = probabilities.get(note_id) {
            let chance = probability.chance();
            if chance == 100 {
                return Some(PlaybackEvent::from(event));
            } else if chance == 0 {
                return None;
            } else {
                let mut rng = rand::rng();
                let roll: u8 = rng.random_range(0..=100);
                println!("Roll: {}", roll);
                println!("Chance: {}", chance);
                if roll < chance {
                    return Some(PlaybackEvent::from(event));
                } else {
                    return None;
                }
            }
        }
    }
    Some(PlaybackEvent::from(event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::{Probabilities, ProbabilityTarget};
    use crate::scheduler::{ScheduledEvent, ScheduledNote, NoteState};

    #[test]
    fn test_filter_event_with_probability() {
        let mut probabilities = Probabilities::new();
        let note = ScheduledNote::new(0.0, 100, 1.0).unwrap();
        let scheduled_event = ScheduledEvent::new(&note, NoteState::On);
        let id = *scheduled_event.note().id();
        probabilities.add(id, 50, ProbabilityTarget::Note).unwrap();
        // Since the probability is 50%, we can't guarantee the outcome, but we can check that it returns Some or None.
        let result = filter_event(&scheduled_event, &probabilities);
        assert!(result.is_some() || result.is_none());
    }
}