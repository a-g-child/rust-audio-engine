use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteOccurrenceKey {
    note_id: Uuid,
    loop_iteration: u64,
}

impl NoteOccurrenceKey {
    pub fn new(note_id: Uuid, loop_iteration: u64) -> Self {
        Self { note_id, loop_iteration }
    }

    pub fn note_id(&self) -> &Uuid {
        &self.note_id
    }

    pub fn loop_iteration(&self) -> u64 {
        self.loop_iteration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_stores_note_id_and_iteration() {
        let id = Uuid::new_v4();
        let key = NoteOccurrenceKey::new(id, 3);

        assert_eq!(key.note_id(), &id);
        assert_eq!(key.loop_iteration(), 3);
    }
}