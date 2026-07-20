//!! Module for managing probabilities associated with playback events.
use uuid::Uuid;
use std::collections::HashMap;
use crate::playback::enums::ProbabilityTarget;
/// Represents a probability associated with a specific target (e.g., note, parameter, clip).
pub struct Probability {
    chance: u8,
    target: ProbabilityTarget,
}
// Implement methods for Probability
impl  Probability {
    pub fn new(chance: u8, target: ProbabilityTarget) -> Self {
        Probability { chance, target }
    }

    pub fn chance(&self) -> u8 {
        self.chance
    }

    pub fn target(&self) -> ProbabilityTarget {
        self.target
    }
    
}
/// Represents a collection of probabilities associated with unique identifiers (UUIDs).
pub struct Probabilities {
    probabilities: HashMap<Uuid, Probability>,
}
// Implement methods for Probabilities
impl Probabilities {
    pub fn new() -> Self {
        Probabilities {
            probabilities: HashMap::new(),
        }
    }
    /// Validates the chance value to ensure it is within the acceptable range (0-100).
    fn validate_chance(chance: u8) -> Result<(), String> {
        if chance > 100 {
            Err(format!("Chance value {} is out of range (0-100)", chance))
        } else {
            Ok(())
        }
    }
    /// Validates the UUID to ensure it is not nil and exists in the probabilities map.
    fn validate_uuid(&self, note_id: &Uuid) -> Result<(), String> {
        if note_id.is_nil() {
            Err("UUID cannot be nil".to_string())
        }else {
            println!("UUID {} is valid and exists in the probabilities map", note_id);
            Ok(())
        }
    }
    /// Validates both the chance value and the UUID for a given probability entry.
    fn validate(&self, note_id: &Uuid, chance: u8) -> Result<(), String> { 
        Self::validate_chance(chance)?;
        println!("Validating UUID: {:?}", note_id);
        self.validate_uuid(note_id)?;
        Ok(())
    }
    /// Adds a new probability entry to the collection, validating the chance and UUID. 
    pub fn add(&mut self, note_id: Uuid, chance: u8, target: ProbabilityTarget) -> Result<(), String> {
        println!("Adding probability");
        self.validate(&note_id , chance)?;
        println!("Adding probability ");
        
        self.probabilities.insert(note_id, Probability { chance, target });
        Ok(())
    }
    /// Updates an existing probability entry, validating the chance and UUID.
    pub fn update(&mut self, note_id: &Uuid, chance: u8, target: ProbabilityTarget) -> Result<(), String> {
        Self::validate_chance(chance)?;
        if let Some(probability) = self.probabilities.get_mut(note_id) {
            probability.chance = chance;
            probability.target = target;
            Ok(())
        } else {
            Err(format!("UUID {} does not exist in the probabilities map", note_id))
        }
    }
    /// Retrieves a probability entry by its UUID, returning an Option.
    pub fn get(&self, note_id: &Uuid) -> Option<&Probability> {
        self.probabilities.get(note_id)
    }
    /// Removes a probability entry by its UUID, if it exists.
    pub fn remove(&mut self, note_id: &Uuid) {
        self.probabilities.remove(note_id);
    }
    /// Clears all probability entries from the collection.
    pub fn clear(&mut self) {
        self.probabilities.clear();
    }
    /// Returns the number of probability entries in the collection.
    pub fn len(&self) -> usize {
        self.probabilities.len()
    }
    /// Checks if the collection of probabilities is empty.
    pub fn is_empty(&self) -> bool {
        self.probabilities.is_empty()
    }
    /// Checks if a probability entry exists for the given UUID.
    pub fn contains(&self, note_id: &Uuid) -> bool {
        self.probabilities.contains_key(note_id)
    }

}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn create() {
        let probability = Probability::new(75, ProbabilityTarget::Note);
        assert_eq!(probability.chance(), 75);
        assert_eq!(probability.target(), ProbabilityTarget::Note);
    }
    #[test]
    fn add_and_get() {
        let mut probabilities = Probabilities::new();
        let note_id = Uuid::new_v4();
        probabilities.add(note_id, 50, ProbabilityTarget::Note).unwrap();

        let retrieved_probability = probabilities.get(&note_id).unwrap();
        assert_eq!(retrieved_probability.chance(), 50);
        assert_eq!(retrieved_probability.target(), ProbabilityTarget::Note);
    }

    #[test]
    fn update() {
        let mut probabilities = Probabilities::new();
        let note_id = Uuid::new_v4();
        probabilities.add(note_id, 50, ProbabilityTarget::Note).unwrap();

        probabilities.update(&note_id, 75, ProbabilityTarget::Parameter).unwrap();
        let updated_probability = probabilities.get(&note_id).unwrap();
        assert_eq!(updated_probability.chance(), 75);
        assert_eq!(updated_probability.target(), ProbabilityTarget::Parameter);
    }
    #[test]
    fn remove() {
        let mut probabilities = Probabilities::new();
        let note_id = Uuid::new_v4();
        probabilities.add(note_id, 50, ProbabilityTarget::Note).unwrap();

        probabilities.remove(&note_id);
        assert!(probabilities.get(&note_id).is_none());
    }

    #[test]
    fn clear() {
        let mut probabilities = Probabilities::new();
        let note_id1 = Uuid::new_v4();
        let note_id2 = Uuid::new_v4();
        probabilities.add(note_id1, 50, ProbabilityTarget::Note).unwrap();
        probabilities.add(note_id2, 75, ProbabilityTarget::Parameter).unwrap(); 
        probabilities.clear();
        assert!(probabilities.is_empty());
    }

    #[test]
    fn len_and_is_empty() {
        let mut probabilities = Probabilities::new();
        assert_eq!(probabilities.len(), 0);
        assert!(probabilities.is_empty());
        let note_id = Uuid::new_v4();
        probabilities.add(note_id, 50, ProbabilityTarget::Note).unwrap();
        assert_eq!(probabilities.len(), 1);
        assert!(!probabilities.is_empty());
    }

    #[test]
    fn contains() {
        let mut probabilities = Probabilities::new();
        let note_id = Uuid::new_v4();
        probabilities.add(note_id, 50, ProbabilityTarget::Note).unwrap();   
        assert!(probabilities.contains(&note_id));
        let non_existent_id = Uuid::new_v4();
        assert!(!probabilities.contains(&non_existent_id));
    }


}

