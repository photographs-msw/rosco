// use derive_builder::Builder;
// 
// use crate::audio_gen::audio_gen;
// use crate::audio_gen::oscillator::Waveform;
// use crate::sequence::grid_note_sequence::{GridNoteSequence, GridNoteSequenceBuilder};
// use crate::note::playback_note::PlaybackNote;
// use crate::track::track::{Track, TrackBuilder};
// 
// static DEFAULT_TRACK_VOLUME: f32 = 1.0;
// 
// #[allow(dead_code)]
// #[derive(Builder, Debug)]
// pub(crate) struct Instrument<> {
//     waveforms: Vec<Waveform>,
// 
//     #[builder(default = "DEFAULT_TRACK_VOLUME")]
//     #[allow(dead_code)]
//     volume: f32,
// 
//     #[builder(public, setter(custom))]
//     track: Track<GridNoteSequence>,
// }
// 
// #[allow(dead_code)]
// impl InstrumentBuilder {
//     pub(crate) fn track(&mut self) -> &mut Self {
//         self.track = Some(TrackBuilder::default()
//             .sequence(GridNoteSequenceBuilder::default().build().unwrap())
//             .volume(self.volume.unwrap())
//             .build().unwrap());
//         self
//     }
// }
// 
// #[allow(dead_code)]
// impl Instrument {
// 
//     pub(crate) fn add_note(&mut self, playback_note: PlaybackNote) {
//         self.track.sequence.insert_note(playback_note);
//     }
// 
//     pub(crate) fn play_note(&self) {
//         audio_gen::gen_note_stream(self.track.sequence.get_note());
//     }
// 
//     pub(crate) fn play_note_and_advance(&mut self, index: usize) {
//         audio_gen::gen_note_stream(self.track.sequence.get_note_at(index));
//         self.track.sequence.increment();
//     }
// 
//     pub(crate) fn reset(&mut self) {
//         self.track.sequence.reset_index();
//     }
// 
//     pub(crate) fn loop_once(&self) {
//         for playback_note in self.track.sequence.notes_iter() {
//             audio_gen::gen_note_stream(playback_note.clone());
//         }
//     }
// 
//     pub(crate) fn loop_n(&self, n: u8) {
//         for _ in 0..n {
//             for playback_note in self.track.sequence.notes_iter() {
//                 audio_gen::gen_note_stream(playback_note.clone());
//             }
//         }
//     }
// 
//     pub(crate) fn set_volume(&mut self, volume: f32) {
//         self.track.volume = volume;
//     }
// 
//     pub(crate) fn play_note_direct(&self, playback_note: PlaybackNote) {
//         audio_gen::gen_note_stream(playback_note);
//     }
// }
