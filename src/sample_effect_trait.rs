pub(crate) trait ApplyEffect {
    fn apply_effect(&self, volume: f32, frequency: f32, sample_clock: f32) -> f32;
}