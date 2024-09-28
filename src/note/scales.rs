use crate::note::constants::PITCH_TO_FREQ_HZ;

#[allow(dead_code)]
pub(crate) enum WesternPitch{
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
}

#[allow(dead_code)]
pub(crate) enum WesternScale {
    Major,
    Minor,
    Pentatonic,
    Blues, // TODO CHATTY BROKEN
    Chromatic,
}

#[allow(dead_code)]
pub(crate) enum ArabicScale {
    Hijaz,
    Bayati,
    Rast,
    Saba,
}

#[allow(dead_code)]
impl WesternPitch {
    pub(crate) fn get_pitch_index(&self) -> u8 {
        match self {
            WesternPitch::C => 0,
            WesternPitch::CSharp => 1,
            WesternPitch::DFlat => 1,
            WesternPitch::D => 2,
            WesternPitch::DSharp => 3,
            WesternPitch::EFlat => 3,
            WesternPitch::E => 4,
            WesternPitch::F => 5,
            WesternPitch::FSharp => 6,
            WesternPitch::GFlat => 6,
            WesternPitch::G => 7,
            WesternPitch::GSharp => 8,
            WesternPitch::AFlat => 8,
            WesternPitch::A => 9,
            WesternPitch::ASharp => 10,
            WesternPitch::BFlat => 10,
            WesternPitch::B => 11,
        }
    }
    
    pub(crate) fn get_frequency(&self, octave: u8) -> f32 {
        PITCH_TO_FREQ_HZ[(octave * 12 + self.get_pitch_index()) as usize] as f32
    }
}

#[allow(dead_code)]
impl WesternScale {
    pub(crate) fn get_scale(&self, root_pitch: u8) -> Vec<f32> {
        let mut scale = Vec::new();
        let root_freq = PITCH_TO_FREQ_HZ[root_pitch as usize] as f32;
        match self {
            WesternScale::Major => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 5.0 / 4.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 5.0 / 3.0);
                scale.push(root_freq * 15.0 / 8.0);
            }
            WesternScale::Minor => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 9.0 / 5.0);
            }
            WesternScale::Pentatonic => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
            }
            WesternScale::Blues => {
                scale.push(root_freq);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 7.0 / 5.0);
                scale.push(root_freq * 7.0 / 6.0);
                scale.push(root_freq * 9.0 / 5.0);
            }
            WesternScale::Chromatic => {
                for i in 0..12 {
                    scale.push(root_freq * 2.0_f32.powf(i as f32 / 12.0));
                }
            }
        }
        
        scale
    }
}

// TODO ABSOLUTELY NO IDEA IF THIS IS CORRECT
#[allow(dead_code)]
impl ArabicScale {
    pub(crate) fn get_scale(&self, root_pitch: u8) -> Vec<f32> {
        let mut scale = Vec::new();
        let root_freq = PITCH_TO_FREQ_HZ[root_pitch as usize] as f32;
        match self {
            ArabicScale::Hijaz => {
                scale.push(root_freq);
                scale.push(root_freq * 16.0 / 15.0);
                scale.push(root_freq * 10.0 / 9.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 16.0 / 9.0);
            }
            ArabicScale::Bayati => {
                scale.push(root_freq);
                scale.push(root_freq * 16.0 / 15.0);
                scale.push(root_freq * 10.0 / 9.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 16.0 / 9.0);
            }
            ArabicScale::Rast => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 5.0 / 4.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 5.0 / 3.0);
                scale.push(root_freq * 15.0 / 8.0);
            }
            ArabicScale::Saba => {
                scale.push(root_freq);
                scale.push(root_freq * 9.0 / 8.0);
                scale.push(root_freq * 6.0 / 5.0);
                scale.push(root_freq * 4.0 / 3.0);
                scale.push(root_freq * 3.0 / 2.0);
                scale.push(root_freq * 8.0 / 5.0);
                scale.push(root_freq * 9.0 / 5.0);
            }
        }
        
        scale
    }
}
