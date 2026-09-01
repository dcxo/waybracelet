use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct FrecencyScore {
    score: f64,
    last_use: SystemTime,
}

const EPSILON: f64 = 1e-12;
const HALF_LIFE: Duration = if cfg!(debug_assertions) {
    Duration::from_mins(1)
} else {
    Duration::from_hours(24 * 5)
};

impl FrecencyScore {
    pub fn new() -> Self {
        Self {
            score: 0.0,
            last_use: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn replace(&mut self, new_data: FrecencyScore) {
        let _ = std::mem::replace(self, new_data);
    }

    pub fn score_at(&self, now: SystemTime) -> f64 {
        let elapsed = now.duration_since(self.last_use).unwrap().as_secs_f64();

        let half_lives = elapsed / HALF_LIFE.as_secs_f64();

        let score = self.score * f64::powf(0.5, half_lives);

        if score < EPSILON { 0.0 } else { score }
    }

    pub fn update_at(&mut self, last_use: SystemTime) {
        self.score = self.score_at(last_use) + 1.;
        self.last_use = last_use;
    }
}

impl FrecencyScore {
    pub fn to_bytes(&self) -> [u8; 16] {
        let score_bytes = self.score.to_be_bytes();
        let last_use_bytes = self
            .last_use
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_be_bytes();

        let mut bytes = [0; 16];
        bytes[..8].copy_from_slice(&score_bytes);
        bytes[8..].copy_from_slice(&last_use_bytes);

        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let score = f64::from_be_bytes(data[..8].try_into().unwrap());
        let timestamp = u64::from_be_bytes(data[8..].try_into().unwrap());
        let duration = Duration::from_secs(timestamp);
        let last_use = SystemTime::UNIX_EPOCH + duration;

        Self { score, last_use }
    }
}
