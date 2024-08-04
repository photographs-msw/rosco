use derive_builder::Builder;

use crate::audio_gen;
use crate::track::{Track, TrackBuilder};
use crate::note::Note;
use crate::sequence::SequenceBuilder;

static DEFAULT_TRACK_VOLUME: f32 = 1.0;

#[derive(Builder)]
pub(crate) struct Instrument<> {
    waveforms: Vec<u8>,

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
        let note = self.track.sequence.get_note();
        audio_gen::gen_note_audio(note.volume, note.frequency, note.duration_ms, &self.waveforms);
    }

    pub(crate) fn play_note_and_advance(&mut self) {
        let note = self.track.sequence.get_note_and_advance();
        audio_gen::gen_note_audio(note.volume, note.frequency, note.duration_ms, &self.waveforms);
    }

    pub(crate) fn reset(&mut self) {
        self.track.sequence.reset_index();
    }

    pub(crate) fn loop_once(&self) {
        for note in self.track.sequence.iter() {
            audio_gen::gen_note_audio(note.volume, note.frequency, note.duration_ms,
                                      &self.waveforms);
        }
    }

    pub(crate) fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.track.sequence.iter() {
                audio_gen::gen_note_audio(note.volume, note.frequency, note.duration_ms,
                                          &self.waveforms);
            }
        }
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.track.volume = volume;
    }

    pub(crate) fn play_note_direct(&self, note: &Note) {
        audio_gen::gen_note_audio(note.volume, note.frequency, note.duration_ms, &self.waveforms);
    }
}