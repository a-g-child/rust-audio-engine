use uuid::Uuid;



/// where that clip appears globally how long that placement exists
#[derive(Debug)]
pub struct ClipPlacement {
    id: Uuid,
    clip_id: Uuid,
    start_beat: f64,
    length: f64,
}

impl ClipPlacement {
    pub fn new(
        clip_id: Uuid,
        start_beat: f64,
        length: f64,
    ) -> Result<Self, String> {
        if !start_beat.is_finite() || start_beat < 0.0 {
            return Err(
                "Placement start must be finite and non-negative".to_string(),
            );
        }

        if !length.is_finite() || length <= 0.0 {
            return Err(
                "Placement length must be finite and greater than zero".to_string(),
            );
        }

        Ok(Self {
            id: Uuid::new_v4(),
            clip_id,
            start_beat,
            length,
        })
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn clip_id(&self) -> &Uuid {
        &self.clip_id
    }

    pub fn start_beat(&self) -> f64 {
        self.start_beat
    }

    pub fn end_beat(&self) -> f64 {
        self.start_beat + self.length()
    }

    pub fn length(&self) -> f64 {
        self.length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let clip_id = Uuid::new_v4();
        let placement = ClipPlacement::new(clip_id, 0.0, 4.0).unwrap(); 
        assert!(placement.start_beat() >= 0.0);
        assert_eq!(placement.end_beat(), placement.start_beat() + placement.length());
    }

    use crate::clips::Clip;
    use crate::clips::enums::ClipPlaybackMode;

    #[test]
    fn create_with_clip() {
        let clip = Clip::new(4.0, ClipPlaybackMode::OneShot).unwrap();
        let placement = ClipPlacement::new(*clip.id(), 0.0, 4.0).unwrap();
        assert_eq!(placement.clip_id(), clip.id()); 
        let clip = Clip::new(8.0, ClipPlaybackMode::Loop { start_beat: 2.0, end_beat: 6.0 }).unwrap();
        let placement = ClipPlacement::new(*clip.id(), 1.0, 4.0).unwrap();
        assert_eq!(placement.clip_id(), clip.id()); 
    }
}