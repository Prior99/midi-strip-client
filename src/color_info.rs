use std::fmt::{Debug, Result, Formatter};
use std::time::{Instant, Duration};
use palette::Hsla;

#[derive(Debug)]
pub struct ColorInfo {
    pub note: u8,
    pub velocity: u8,
    pub deleted: Option<Instant>,
    pub created: Instant,
    pub release: Duration,
    pub attack: Duration,
}

impl ColorInfo {
    pub fn new(note: u8, velocity: u8, attack: Duration, release: Duration) -> ColorInfo {
        ColorInfo {
            note,
            velocity,
            attack,
            release,
            deleted: None,
            created: Instant::now(),
        }
    }

    pub fn to_hsla(&self, now: &Instant) -> Hsla {
        let hue = (self.note as f32) * 3_f32;
        let saturation = 1_f32;
        let lightness = (self.velocity as f32) / 127_f32 * 0.7_f32;
        let created_since = now.duration_since(self.created);
        let alpha_attack = if created_since > self.attack {
            1_f32
        } else {
            let attack_ms = self.attack.as_millis() as f32;
            let created_ms = created_since.as_millis() as f32;
            created_ms / attack_ms
        };
        let alpha_deleted = match self.deleted {
            Some(deleted) => {
                let deleted_since = now.duration_since(deleted);
                if deleted_since > self.release {
                    1_f32
                } else {
                    let release_ms = self.release.as_millis() as f32;
                    let deleted_ms = deleted_since.as_millis() as f32;
                    deleted_ms / release_ms
                }
            },
            None => 0_f32,
        };
        let alpha = alpha_attack - alpha_deleted * alpha_attack;
        Hsla::new(hue, saturation, lightness, alpha)
    }

    pub fn is_gone(&self, now: &Instant) -> bool {
        match self.deleted {
            Some(deleted) => now.duration_since(deleted) > self.release,
            None => false,
        }
    }
}
