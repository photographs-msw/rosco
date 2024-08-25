use derive_builder::Builder;
use crate::audio_gen;
use crate::track::{Track, TrackBuilder};
use crate::note;
use crate::note::Note;
use crate::oscillator;
use crate::grid_note_sequence::GridNoteSequenceBuilder;

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
                          .sequence(GridNoteSequenceBuilder::default().build().unwrap())
                          .volume(1.0 / num_tracks as f32)
                          .build().unwrap(); num_tracks]);
        self
    }

    // overriding setting in builder allowing the caller to add tracks on build
    #[allow(dead_code)]
    pub (crate) fn add_tracks(&mut self, tracks: Vec<Track>) -> &mut Self {
        self.tracks = Some(tracks);
        self
    }
}

impl MultiInstrument {

    pub(crate) fn play_track_notes(&self) {
        let notes = self.get_next_notes();
        let max_note_duration_ms = note::max_note_duration_ms(&notes);
        audio_gen::gen_notes(notes, self.track_waveforms.clone(), max_note_duration_ms);
    }

    pub(crate) fn play_track_notes_and_advance(&mut self) {
        let notes = self.get_next_notes();
        let max_note_duration_ms = note::max_note_duration_ms(&notes);
        audio_gen::gen_notes(notes, self.track_waveforms.clone(), max_note_duration_ms);
        for channel in self.tracks.iter_mut() {
            channel.sequence.increment();
        }
    }

    pub(crate) fn reset_all_tracks(&mut self) {
        self.tracks.iter_mut().for_each(
            |channel| channel.sequence.reset_index()
        );
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

        self.tracks[track_num].sequence.append_note(note);
    }

    pub(crate) fn add_note_to_tracks(&mut self, note: Note) {
        self.validate_has_tracks();
        self.tracks.iter_mut().for_each(
            |track| track.sequence.append_note(note)
        );
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
            self.tracks[*track_num].sequence.append_note(note);
        }
    }

    pub(crate) fn set_volume_for_tracks(&mut self, volume: f32) {
        self.validate_has_tracks();

        self.tracks.iter_mut().for_each(
            |track| track.volume = volume
        );
    }

    pub(crate) fn set_volume_for_track(&mut self, track_num: usize, volume: f32) {
        self.validate_track_num(track_num);

        self.tracks[track_num].volume = volume;
    }

    pub(crate) fn play_notes_direct(&self, notes: Vec<Note>) {
        let max_note_duration_ms = note::max_note_duration_ms(&notes);
        audio_gen::gen_notes(notes, self.track_waveforms.clone(), max_note_duration_ms);
    }

    // TODO THIS IS ALL WRONG
    //  NEED ACTUAL TIME BASED GRID AND CURRENT TICK AND CURRENT SET OF NOTES BEING TURNED ON/OFF
    // deprecated
    fn get_next_notes(&self) -> Vec<Note> {
        self.tracks.iter()
            .filter(|track| !track.sequence.at_end())
            .map(|track| {
                let mut note = track.sequence.get_note();
                note.volume *= track.volume;
                note
            })
            .collect()
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
