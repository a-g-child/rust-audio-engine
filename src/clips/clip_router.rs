use uuid::Uuid;
// clip router sits betweeen notes and the associated clips, it is responsible for routing note events to the correct clip, which allows flexibility.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ClipRouter {
    router_id: Uuid,
    clip_id: Uuid,
}

impl ClipRouter {
    pub fn new(clip_id: Uuid) -> Self {
        Self { router_id: Uuid::new_v4(), clip_id }
    }

    pub fn router_id(&self) -> &Uuid {
        &self.router_id
    }

    pub fn clip_id(&self) -> &Uuid {
        &self.clip_id
    }

    pub fn migrate_clip(&mut self, new_clip_id: Uuid) {
        self.clip_id = new_clip_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_clip_router() {
        let clip_id = Uuid::new_v4();
        let router = ClipRouter::new(clip_id);
        assert_eq!(router.clip_id(), &clip_id);
    }
}