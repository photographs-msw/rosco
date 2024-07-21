use derive_builder::Builder;

use crate::audio_gen;
use crate::channel::{Channel, ChannelBuilder};
use crate::note::Note;
use crate::oscillator;
use crate::sequence::SequenceBuilder;

static DEFAULT_CHANNEL_VOLUME: f32 = 1.0;

#[derive(Builder, Clone)]
#[allow(dead_code)]
pub(crate) struct Instrument<> {
    oscillators: Vec<oscillator::OscType>,

    #[builder(default = "DEFAULT_CHANNEL_VOLUME")]
    volume: f32,

    #[builder(public, setter(custom))]
    channel: Channel,
}

impl InstrumentBuilder {
    pub(crate) fn channel(&mut self) -> &mut Self {
        self.channel = Some(ChannelBuilder::default()
            .sequence(SequenceBuilder::default().build().unwrap())
            .volume(self.volume.unwrap())
            .build().unwrap());
        self
    }
}

#[allow(dead_code)]
impl Instrument {

    pub(crate) fn add_note(&mut self, note: Note) {
        self.channel.sequence.add_note(note);
    }

    pub(crate) fn play_note(&self) {
        audio_gen::gen_note(&self.channel.sequence.get_note(), self.oscillators.clone());
    }

    pub(crate) fn play_note_and_advance(&mut self) {
        audio_gen::gen_note(&self.channel.sequence.get_note_and_advance(), self.oscillators.clone());
    }

    pub(crate) fn reset(&mut self) {
        self.channel.sequence.reset_index();
    }

    pub(crate) fn loop_once(&self) {
        for note in self.channel.sequence.iter() {
            audio_gen::gen_note(note, self.oscillators.clone());
        }
    }

    pub(crate) fn loop_n(&self, n: u8) {
        for _ in 0..n {
            for note in self.channel.sequence.iter() {
                audio_gen::gen_note(note, self.oscillators.clone());
            }
        }
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.channel.volume = volume;
    }

    pub(crate) fn play_note_direct(&self, note: &Note) {
        audio_gen::gen_note(note, self.oscillators.clone());
    }
}