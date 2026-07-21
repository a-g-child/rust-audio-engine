use uuid::Uuid;
use crate::clips::enums::{ClipPlaybackMode, ClipPlaybackModeError};

/// reusable musical content local length and looping behaviour
#[derive(Debug)]
pub struct Clip {
    id: Uuid,
    length: f64,
    playback_mode: ClipPlaybackMode,
}

impl Clip {
    pub fn new(length: f64, playback_mode: ClipPlaybackMode) -> Result<Self, ClipPlaybackModeError> {
        if !length.is_finite() || length <= 0.0 {
            return Err(ClipPlaybackModeError::LoopBoundsNotFinite);
        }
        Self::validate_playback_mode(length, playback_mode)?;

        Ok(Self {
            id: Uuid::new_v4(),
            length,
            playback_mode,
        })
    }

    fn validate_playback_mode(
        clip_length: f64,
        playback_mode: ClipPlaybackMode,
    ) -> Result<(), ClipPlaybackModeError> {
        match playback_mode {
            ClipPlaybackMode::OneShot => Ok(()),

            ClipPlaybackMode::Loop {
                start_beat,
                end_beat,
            } => {
                if !start_beat.is_finite() || !end_beat.is_finite() {
                    return Err(ClipPlaybackModeError::LoopBoundsNotFinite);
                }

                if start_beat < 0.0 {
                    return Err(ClipPlaybackModeError::LoopStartNegative);
                }

                if end_beat <= start_beat {
                    return Err(ClipPlaybackModeError::LoopEndNotAfterStart);
                }

                if end_beat > clip_length {
                    return Err(ClipPlaybackModeError::LoopEndExceedsClipLength);
                }

                Ok(())
            }
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn playback_mode(&self) -> ClipPlaybackMode {
        self.playback_mode
    }
}




#[cfg(test)]
mod tests {
    use super::*;



}
