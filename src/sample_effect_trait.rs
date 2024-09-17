pub(crate) trait ApplyEffect {
    fn apply_effect(&self, sample: f32, frequency: f32, sample_clock: f32) -> f32;
}

pub(crate) trait NoOpEffect {
    fn no_op(&self, sample: f32, _frequency: f32, _sample_clock: f32) -> f32 {
        sample
    }
}

pub(crate) trait Clone<EffectType> {
    fn clone(&self) -> EffectType;
}

pub(crate) trait BuilderWrapper<EffectType> {
    fn new() -> EffectType;
} 
