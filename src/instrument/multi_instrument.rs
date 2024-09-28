use derive_builder::Builder;

use crate::audio_gen::audio_gen;
use crate::audio_gen::oscillator::Waveform;
use crate::note::playback_note::{PlaybackNote, PlaybackNoteBuilder};
use crate::sequence::grid_note_sequence::{GridNoteSequence, GridNoteSequenceBuilder};
use crate::sequence::note_sequence_trait::AppendNote;
use crate::track::track::{Track, TrackBuilder};

#[allow(dead_code)]
#[derive(Builder, Debug)]
pub(crate) struct MultiInstrument {
    track_waveforms: Vec<Vec<Waveform>>,

    #[allow(dead_code)]
    num_tracks: usize,

    // user can call tracks() to build with empty tracks or add_tracks() to add tracks on build
    #[builder(public, setter(custom))]
    pub(crate) tracks: Vec<Track<GridNoteSequence>>,
}

#[allow(dead_code)]
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
    pub(crate) fn add_tracks(&mut self, tracks: Vec<Track<GridNoteSequence>>) -> &mut Self {
        self.tracks = Some(tracks);
        self
    }
}

#[allow(dead_code)]
impl MultiInstrument {

    pub(crate) fn play_track_notes(&self) {
        let (playback_note_kinds, max_note_duration_ms) =
            self.get_next_playback_notes();
        audio_gen::gen_notes_stream(playback_note_kinds, max_note_duration_ms);
    }

    pub(crate) fn play_track_notes_and_advance(&mut self) {
        let (playback_note_kinds, max_note_duration_ms) =
            self.get_next_playback_notes();
        for channel in self.tracks.iter_mut() {
            channel.sequence.increment();
        }
        audio_gen::gen_notes_stream(playback_note_kinds, max_note_duration_ms);
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

    pub(crate) fn add_note_to_track(&mut self, track_num: usize, playback_note: PlaybackNote) {
        self.validate_track_num(track_num);

        self.tracks[track_num].sequence.append_note(playback_note);
    }

    pub(crate) fn add_note_to_tracks(&mut self, playback_note: PlaybackNote) {
        self.validate_has_tracks();
        self.tracks.iter_mut().for_each(
            |track| track.sequence.append_note(playback_note.clone())
        );
    }

    #[allow(dead_code)]
    pub(crate) fn add_chord_to_tracks(&mut self, track_nums: Vec<usize>, chord: Vec<PlaybackNote>) {
        for track_num in &track_nums {
            self.validate_track_num(*track_num);
        }
        if track_nums.len() != chord.len() {
            panic!("Number of tracks must match number of notes in chord");
        }
        let first_index= self.tracks[track_nums[0]].sequence.get_index();
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

    pub(crate) fn play_notes_direct(&self, playback_notes: Vec<PlaybackNote>) {
        let (playback_note_kinds, max_note_duration_ms) =
            self.get_playback_notes_direct(playback_notes);

        audio_gen::gen_notes_stream(playback_note_kinds, max_note_duration_ms);
    }

    fn get_next_notes(&self) -> Vec<PlaybackNote> {
        self.tracks.iter()
            .filter(|track| !track.sequence.at_end())
            .map(|track| {
                let mut playback_note = track.sequence.get_note();
                playback_note.set_note_volume(playback_note.note_volume() * track.volume);
                playback_note
            })
            .collect()
    }
    
    fn get_next_playback_notes(&self) -> (Vec<PlaybackNote>, f32) {
        let mut max_note_duration_ms = 0.0;
        let playback_notes = self.tracks.iter()
            .filter(|track| !track.sequence.at_end())
            .map(|track| {
                let playback_note = track.sequence.get_note();
                if playback_note.note_duration_ms() > max_note_duration_ms {
                    max_note_duration_ms = playback_note.note_duration_ms();
                }
                PlaybackNoteBuilder::default()
                    .note_type(playback_note.note_type)
                    .note(playback_note.note)
                    .sampled_note(playback_note.sampled_note)
                    // TODO MOVE WAVEFORMS TO NOTE
                    .waveforms(self.track_waveforms[track.sequence.get_index()].clone())
                    .build().unwrap()
            })
            .collect();

        (playback_notes, max_note_duration_ms)
    }
    
    fn get_playback_notes_direct(&self, playback_notes: Vec<PlaybackNote>) -> (Vec<PlaybackNote>, f32) {
        if playback_notes.len() != self.track_waveforms.len() {
            panic!("Number of notes must match number of waveforms");
        }
        let mut max_note_duration_ms = 0.0;
        let ret_playback_notes = playback_notes.iter().enumerate()
            .map(|(i, playback_note)| {
                if playback_note.note_duration_ms() > max_note_duration_ms {
                    max_note_duration_ms = playback_note.note_duration_ms();
                }
                let ret_playback_note= playback_note.clone();
                PlaybackNoteBuilder::default()
                    .note_type(ret_playback_note.note_type)
                    .note(ret_playback_note.note)
                    .sampled_note(ret_playback_note.sampled_note)
                    // TODO MOVE WAVEFORMS TO NOTE
                    .waveforms(self.track_waveforms[i].clone())
                    .build().unwrap()
            })
            .collect();

        (ret_playback_notes, max_note_duration_ms)
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
