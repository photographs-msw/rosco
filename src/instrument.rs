use derive_builder::Builder;

use crate::audio_gen;
use crate::frequency_callback::{InstrumentGetFreqCallback};
use crate::track::{Track, TrackBuilder};
use crate::note::Note;
use crate::oscillator;
use crate::sequence::SequenceBuilder;

static DEFAULT_TRACK_VOLUME: f32 = 1.0;

#[derive(Builder, Debug)]
pub(crate) struct Instrument<> {
    waveforms: Vec<oscillator::Waveform>,

    #[builder(default = "DEFAULT_TRACK_VOLUME")]
    #[allow(dead_code)]
    volume: f32,

    #[builder(public, setter(custom))]
    track: Track,
}

impl InstrumentBuilder {
    pub(crate) fn track(&mut self) -> &mut Self {
        self.track = Some(TrackBuilder::default()
            .sequence(SequenceBuilder::default().build().unwrap())
            .volume(self.volume.unwrap())
            .build().unwrap());
        self
    }
}

impl Instrument {
    pub(crate) fn add_note(&mut self, note: Note) {
        self.track.sequence.add_note(note);
    }

    pub(crate) fn play_note(&self) {
        let callback = InstrumentGetFreqCallback { waveforms: &self.waveforms };
        audio_gen::gen_note(&self.track.sequence.get_note(), Box::new(callback));
    }

    pub(crate) fn play_note_and_advance(&mut self) {
        let callback = InstrumentGetFreqCallback { waveforms: &self.waveforms };
        audio_gen::gen_note(&self.track.sequence.get_note_and_advance(), Box::new(callback));
    }

    pub(crate) fn reset(&mut self) {
        self.track.sequence.reset_index();
    }

    pub(crate) fn loop_once(&self) {
        for note in self.track.sequence.iter() {
            let callback = InstrumentGetFreqCallback { waveforms: &self.waveforms };
            audio_gen::gen_note(note, Box::new(callback));
        }
    }

    pub(crate) fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.track.sequence.iter() {
                let callback = InstrumentGetFreqCallback { waveforms: &self.waveforms };
                audio_gen::gen_note(note, Box::new(callback));
            }
        }
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.track.volume = volume;
    }

    pub(crate) fn play_note_direct(&self, note: &Note) {
        let callback = InstrumentGetFreqCallback { waveforms: &self.waveforms };
        audio_gen::gen_note(note, Box::new(callback));
    }
}