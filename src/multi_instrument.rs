use derive_builder::Builder;
use crate::audio_gen;
use crate::track::{Track, TrackBuilder};
use crate::note::Note;
use crate::oscillator;
use crate::sequence::SequenceBuilder;

#[derive(Builder, Debug)]
pub(crate) struct MultiInstrument {
    track_waveforms: Vec<Vec<oscillator::Waveform>>,

    #[allow(dead_code)]
    num_tracks: usize,

    // user can call tracks() to build with empty tracks or add_tracks() to add tracks on build
    #[builder(public, setter(custom))]
    pub(crate) tracks: Vec<Track>,
}

impl MultiInstrumentBuilder {
    // builds with empty tracks, the default setter in the builder
    pub(crate) fn tracks(&mut self) -> &mut Self {
        let num_tracks = self.num_tracks.unwrap();
        self.tracks =
            Some(vec![TrackBuilder::default()
                          .sequence(SequenceBuilder::default().build().unwrap())
                          .volume(1.0 / num_tracks as f32)
                          .build().unwrap(); num_tracks]);
        self
    }

    // overriding setting in builder allowing the caller to add tracks on build
    pub (crate) fn add_tracks(&mut self, tracks: Vec<Track>) -> &mut Self {
        self.tracks = Some(tracks);
        self
    }
}

impl MultiInstrument {

    pub(crate) fn play_track_notes(&self) {
        audio_gen::gen_notes(self.get_next_notes(), self.track_waveforms.clone());
    }

    pub(crate) fn play_track_notes_and_advance(&mut self) {
        let notes = self.get_next_notes();
        audio_gen::gen_notes(notes, self.track_waveforms.clone());
        for channel in self.tracks.iter_mut() {
            channel.sequence.advance();
        }
    }

    pub(crate) fn reset_all_tracks(&mut self) {
        for channel in &mut self.tracks {
            channel.sequence.reset_index();
        }
    }

    pub(crate) fn loop_once(&mut self) {
        self.reset_all_tracks();
        while !self.tracks.iter().all(|channel| channel.sequence.at_end()) {
            self.play_track_notes_and_advance();
        }
    }

    pub(crate) fn loop_n(&mut self, n: u8) {
        self.reset_all_tracks();
        for _ in 0..n {
            self.loop_once();
        }
    }

    pub(crate) fn add_note_to_track(&mut self, track_num: usize, note: Note) {
        self.validate_track_num(track_num);

        self.tracks[track_num].sequence.add_note(note);
    }

    pub(crate) fn add_note_to_tracks(&mut self, note: Note) {
        self.validate_has_tracks();

        for track in &mut self.tracks {
            track.sequence.add_note(note);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn add_chord_to_tracks(&mut self, track_nums: Vec<usize>, chord: Vec<Note>) {
        for track_num in &track_nums {
            self.validate_track_num(*track_num);
        }
        if track_nums.len() != chord.len() {
            panic!("Number of tracks must match number of notes in chord");
        }
        let first_index: usize = self.tracks[track_nums[0]].sequence.get_index();
        for channel_num in &track_nums[1..] {
            if self.tracks[*channel_num].sequence.get_index() != first_index {
                panic!("Tracks must all be at the same index to add chord notes across \
                        channel sequences");
            }
        }

        for (track_num, note) in track_nums.iter().zip(chord) {
            self.tracks[*track_num].sequence.add_note(note);
        }
    }

    pub(crate) fn set_volume_for_tracks(&mut self, volume: f32) {
        self.validate_has_tracks();

        for track in &mut self.tracks {
            track.volume = volume;
        }
    }

    pub(crate) fn set_volume_for_track(&mut self, track_num: usize, volume: f32) {
        self.validate_track_num(track_num);

        self.tracks[track_num].volume = volume;
    }

    pub(crate) fn play_notes_direct(&self, notes: Vec<Note>) {
        audio_gen::gen_notes(notes, self.track_waveforms.clone());
    }

    fn get_next_notes(&self) -> Vec<Note> {
        let mut notes = Vec::new();
        for track in &self.tracks {
            if track.sequence.at_end() {
                continue;
            }
            let mut note = track.sequence.get_note();
            note.volume *= track.volume;
            notes.push(note);
        }
        notes
    }

    fn validate_track_num(&self, track_num: usize) {
        if track_num >= self.tracks.len() {
            panic!("Invalid track number");
        }
    }

    fn validate_has_tracks(&self) {
        if self.tracks.len() == 0 {
            panic!("No tracks available");
        }
    }
}
