use derive_builder::Builder;

use crate::audio_gen;
use crate::grid_note_sequence::{GridNoteSequence, GridNoteSequenceBuilder};
use crate::note::Note;
use crate::oscillator;
use crate::playback_note::{PlaybackNote, PlaybackNoteBuilder};
use crate::sample_effect_trait::{ApplyEffect, BuilderWrapper};
use crate::track::{Track, TrackBuilder};

#[allow(dead_code)]
#[derive(Builder, Debug)]
pub(crate) struct MultiInstrument<
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Clone + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Clone + Send,
> {
    track_waveforms: Vec<Vec<oscillator::Waveform>>,

    #[allow(dead_code)]
    num_tracks: usize,

    // user can call tracks() to build with empty tracks or add_tracks() to add tracks on build
    #[builder(public, setter(custom))]
    pub(crate) tracks: Vec<Track<GridNoteSequence>>,
    
    #[builder(default = "vec![EnvelopeType::new(); self.tracks.clone().unwrap().len()]")]
    pub(crate) track_envelopes: Vec<EnvelopeType>,

    #[builder(default = "vec![vec![LFOType::new()]; self.tracks.clone().unwrap().len()]")]
    pub(crate) track_lfos: Vec<Vec<LFOType>>,
}

#[allow(dead_code)]
impl<
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Clone + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Clone + Send,
> MultiInstrumentBuilder<EnvelopeType, LFOType> {
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
    pub (crate) fn add_tracks(&mut self, tracks: Vec<Track<GridNoteSequence>>) -> &mut Self {
        self.tracks = Some(tracks);
        self
    }
}

#[allow(dead_code)]
impl<
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Clone + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Clone + Send
>
MultiInstrument<EnvelopeType, LFOType> {

    pub(crate) fn play_track_notes(&self) {
        let (mut playback_notes, max_note_duration_ms) =
            self.get_next_playback_notes();
        audio_gen::gen_notes(&mut playback_notes, max_note_duration_ms as u64);
    }

    pub(crate) fn play_track_notes_and_advance(&mut self) {
        let (mut playback_notes, max_note_duration_ms) =
            self.get_next_playback_notes();
        for channel in self.tracks.iter_mut() {
            channel.sequence.increment();
        }
        audio_gen::gen_notes(&mut playback_notes, max_note_duration_ms as u64);
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

        self.tracks[track_num].sequence.append_notes(&vec![note]);
    }

    pub(crate) fn add_note_to_tracks(&mut self, note: Note) {
        self.validate_has_tracks();
        self.tracks.iter_mut().for_each(
            |track| track.sequence.append_notes(&vec![note])
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
        let first_index= self.tracks[track_nums[0]].sequence.get_index();
        for channel_num in &track_nums[1..] {
            if self.tracks[*channel_num].sequence.get_index() != first_index {
                panic!("Tracks must all be at the same index to add chord notes across \
                        channel sequences");
            }
        }

        for (track_num, note) in track_nums.iter().zip(chord) {
            self.tracks[*track_num].sequence.append_notes(&vec![note]);
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
        let (mut playback_notes, max_note_duration_ms) =
            self.get_playback_notes_direct(notes);
        audio_gen::gen_notes(&mut playback_notes, max_note_duration_ms as u64);
    }

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
    
    fn get_next_playback_notes(&self) -> (Vec<PlaybackNote<EnvelopeType, LFOType>>, f32)
    {
        let mut max_note_duration_ms = 0.0;
        let playback_notes: Vec<PlaybackNote<EnvelopeType, LFOType>> = self.tracks.iter()
            .filter(|track| !track.sequence.at_end())
            .map(|track| {
                let note = track.sequence.get_note();
                if note.duration_ms > max_note_duration_ms {
                    max_note_duration_ms = note.duration_ms;
                }
                PlaybackNoteBuilder::default()
                    .note(note)
                    .waveforms(self.track_waveforms[track.sequence.get_index()].clone())
                    .envelope(self.track_envelopes[track.sequence.get_index()].clone())
                    .lfos(self.track_lfos[track.sequence.get_index()].clone())
                    .build().unwrap()
            })
            .collect();
        
        (playback_notes, max_note_duration_ms)
    }
    
    fn get_playback_notes_direct(&self, notes: Vec<Note>)
        -> (Vec<PlaybackNote<EnvelopeType, LFOType>>, f32)
    {
        if notes.len() != self.track_waveforms.len() {
            panic!("Number of notes must match number of waveforms");
        }
        let mut max_note_duration_ms = 0.0;
        let playback_notes: Vec<PlaybackNote<EnvelopeType, LFOType>> = notes.iter().enumerate()
            .map(|(i, note)| {
                if note.duration_ms > max_note_duration_ms {
                    max_note_duration_ms = note.duration_ms;
                }
                PlaybackNoteBuilder::default()
                    .note(*note)
                    .waveforms(self.track_waveforms[i].clone())
                    .build().unwrap()
            })
            .collect();
        (playback_notes, max_note_duration_ms)
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
