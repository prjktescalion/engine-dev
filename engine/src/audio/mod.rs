//! Audio. Backed by `rodio` once the dep is enabled.

pub struct Audio {
    pub master_volume: f32,
}

impl Audio {
    pub fn new() -> Self {
        Self { master_volume: 1.0 }
    }

    pub fn play(&mut self, _path: &str) {
        // TODO: decode + play via rodio
    }
}

impl Default for Audio {
    fn default() -> Self {
        Self::new()
    }
}
