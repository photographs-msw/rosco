use derive_builder::Builder;

use crate::common::constants::{FLOAT_EPSILON, SAMPLE_RATE};
use crate::common::float_utils::{float_eq, float_geq, float_leq};
use crate::note::playback_note;
use crate::note::playback_note::{PlaybackNoteBuilder, PlaybackNote, NoteType};
use crate::sequence::note_sequence_trait::{NextNotes, SetCurPosition};
use crate::track::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid<SequenceType: NextNotes + Iterator + SetCurPosition> {
    pub(crate) tracks: Vec<Track<SequenceType>>,

    #[builder(default = "0.0")]
    cur_position_ms: f32,
}

impl<SequenceType: NextNotes + Iterator + SetCurPosition> TrackGrid<SequenceType> {

    pub(crate) fn next_notes(&mut self) -> Vec<PlaybackNote> {

        fn note_ref_into_note(playback_note: &PlaybackNote, cur_notes_time_ms: f32,
                              window_end_time_ms: f32) -> PlaybackNote {
            let mut new_pb_note: PlaybackNote = playback_note.clone();
            new_pb_note.playback_start_time_ms = cur_notes_time_ms;
            new_pb_note.playback_end_time_ms = window_end_time_ms;
            
            // TODO BUG
            //  adjust playback_sample_start_time_ms and end_time_ms and sample_index if SampleNote
            if playback_note.note_type == NoteType::Sample {
                new_pb_note.playback_sample_start_time =
                    (new_pb_note.playback_start_time_ms * (SAMPLE_RATE / 1000.0)).floor() as u64;
                new_pb_note.playback_sample_end_time =
                    (new_pb_note.playback_end_time_ms * (SAMPLE_RATE / 1000.0)).floor() as u64;
                new_pb_note.sampled_note.sample_index = ((new_pb_note.playback_start_time_ms -
                    new_pb_note.sampled_note.start_time_ms) * (SAMPLE_RATE / 1000.0)) as usize;
            }

            new_pb_note
        }

        let mut track_playback_notes = Vec::new();

        for track in self.tracks.iter_mut() {
            
            // TEMP DEBUG
            println!("IN TrackGrid::next_notes() track");

            track.sequence.set_cur_position(self.cur_position_ms);
            
            for playback_note in track.sequence.next_notes() {
                
                // TEMP DEBUG
                println!("IN TrackGrid::next_notes() playback_note");
                println!("cur_position_ms: {:?}", self.cur_position_ms);

                let mut playback_note_builder = PlaybackNoteBuilder::default();
                    playback_note_builder
                        .envelopes(playback_note.envelopes.clone())
                        .lfos(playback_note.lfos.clone())
                        .flangers(playback_note.flangers.clone())
                        .delays(playback_note.delays.clone())
                        .playback_start_time_ms(playback_note.playback_start_time_ms)
                        .playback_end_time_ms(playback_note.playback_end_time_ms)
                        .playback_sample_start_time((playback_note.playback_start_time_ms *
                            (SAMPLE_RATE / 1000.0)).floor() as u64)
                        .playback_end_time_ms(playback_note.playback_end_time_ms)
                        .playback_sample_end_time((playback_note.playback_end_time_ms *
                            (SAMPLE_RATE / 1000.0)).floor() as u64);
                
                match playback_note.note_type {
                    NoteType::Oscillator => {
                        track_playback_notes.push(
                            playback_note_builder
                                .note_type(NoteType::Oscillator)
                                .note(playback_note.note)
                                .build().unwrap()
                        );
                    }
                    NoteType::Sample => {
                        
                        // TEMP DEBUG 
                        println!("IN TrackGrid::next_notes() Sample note");

                        track_playback_notes.push(
                            playback_note_builder
                                .note_type(NoteType::Sample)
                                .sampled_note(playback_note.sampled_note)
                                .build().unwrap()
                        );
                    }
                }
            }
        }

        let window_start_time_ms = get_frontier_min_start_time(&track_playback_notes);
        let window_end_time_ms = get_frontier_min_end_time(
            &track_playback_notes, self.cur_position_ms);

        // TEMP DEBUG
        println!("window_start_time_ms: {:?}", window_start_time_ms);
        println!("window_end_time_ms: {:?}", window_end_time_ms);

        // If the current note time is earlier than that, emit a rest note and increment
        // the current notes time to the frontier min start time + epsilon
        if self.cur_position_ms < window_start_time_ms {

            // TEMP DEBU
            println!("IN TrackGrid::next_notes() rest note");

            self.cur_position_ms = window_start_time_ms + FLOAT_EPSILON;
            return vec![playback_note::playback_rest_note(self.cur_position_ms,
                                                          window_start_time_ms)];
        }

        let mut out_playback_notes = Vec::new();

        // If the current note time is the same as the frontier min start time, emit all notes
        // in the frontier with the same start time and increment the current notes time to the
        // earliest end time in the frontier. This is the next window emit, note to end time.
        if float_eq(self.cur_position_ms, window_start_time_ms) {
            let playback_notes: Vec<PlaybackNote> = track_playback_notes
                .iter()
                .filter(|playback_note|
                        float_eq(playback_note.note_start_time_ms(), self.cur_position_ms))
                .map(|playback_note| note_ref_into_note(
                    playback_note, self.cur_position_ms, window_end_time_ms))
                .collect();

            out_playback_notes.extend_from_slice(&playback_notes);

        } else if self.cur_position_ms > window_start_time_ms {
            let playback_notes: Vec<PlaybackNote> = track_playback_notes
                .iter()
                .filter(|playback_note|
                    float_leq(playback_note.note_start_time_ms(), self.cur_position_ms) &&
                    float_geq(playback_note.note_end_time_ms(), self.cur_position_ms)
                )
                .filter(|playback_note| playback_note.note_duration_ms() > 0.0)
                .map(|playback_note|
                    note_ref_into_note(playback_note, self.cur_position_ms, window_end_time_ms)
                )
                .collect();


            out_playback_notes.extend_from_slice(&playback_notes);
        }

        self.cur_position_ms = window_end_time_ms + FLOAT_EPSILON;
        
        // TEMP DEBuG
        println!("self.cur_position_ms: {:?}", self.cur_position_ms);
        println!("out_playback_notes.len(): {:?}", out_playback_notes.len());

        out_playback_notes
    }
}

fn get_frontier_min_start_time(playback_notes: &Vec<PlaybackNote>) -> f32 {
    let mut start_time_ms = f32::MAX;
    for playback_note in playback_notes.iter() {
        if playback_note.note_start_time_ms() < start_time_ms {
            start_time_ms = playback_note.note_start_time_ms();
        }
    }
    start_time_ms
}

fn get_frontier_min_end_time(playback_notes: &Vec<PlaybackNote>, note_time_ms: f32) -> f32 {
    let mut end_time_ms = f32::MAX;

    // First pass, is what is the earliest end time in the future, after note_time_ms
    // for a note that starts on or before note_time_ms and ends after it
    for playback_note in playback_notes.iter() {
        if float_leq(playback_note.note_start_time_ms(), note_time_ms) &&
            playback_note.note_end_time_ms() > note_time_ms &&
            playback_note.note_end_time_ms() < end_time_ms {
            end_time_ms = playback_note.note_end_time_ms();
        }
    }

    // Second pass, is there a note that starts after note_time_ms earlier than the
    // earliest end time. Because if there is then that is the end time of this window
    for playback_note in playback_notes.iter() {
        if playback_note.note_start_time_ms() > note_time_ms &&
            playback_note.note_start_time_ms() < end_time_ms {
            end_time_ms = playback_note.note_start_time_ms();
        }
    }

    end_time_ms
}

impl<SequenceType: NextNotes + Iterator + SetCurPosition> Iterator for TrackGrid<SequenceType> {
    type Item = Vec<PlaybackNote>;

    fn next(&mut self) -> Option<Self::Item> {
        let playback_notes= self.next_notes();
        
        // TEMP DEBUG
        println!("IN TrackGrid::next(), playback_notes.len(): {:?}", playback_notes.len());
        println!("IN TrackGrid::next(), playback_notes.is_empty(): {:?}", playback_notes.is_empty());

        if playback_notes.is_empty() {
            
            // TEMP DEBUG
            println!("IN TrackGrid::next(), playback_notes.is_empty() returning None");
            return None;
        }

        Some(playback_notes)
    }
}

#[cfg(test)]
mod test_sequence_grid {
    use crate::effect::{flanger, lfo};
    use crate::envelope::envelope;
    use crate::note::note::NoteBuilder;
    use crate::note::playback_note::PlaybackNoteBuilder;
    use crate::sequence::grid_note_sequence::GridNoteSequenceBuilder;
    use crate::track::track::TrackBuilder;
    use crate::track::track_effects::TrackEffectsBuilder;
    use crate::track::track_grid::TrackGridBuilder;

    #[test]
    fn test_active_notes_grid_sequence() {
        // Create a sequence grid with a sequence with two notes, one on and one off
        let mut track_grid = TrackGridBuilder::default()
            .tracks(
                vec![
                    TrackBuilder::default()
                        .num(1)
                        .sequence(
                            GridNoteSequenceBuilder::default()
                                .sequence(
                                    vec![vec![
                                    PlaybackNoteBuilder::default()
                                        .note(
                                            setup_note()
                                                .start_time_ms(0.0)
                                                .end_time_ms(1000.0)
                                                .build().unwrap()
                                        )
                                        .playback_start_time_ms(0.0)
                                        .playback_end_time_ms(1000.0)
                                        .build().unwrap(),
                                    PlaybackNoteBuilder::default()
                                        .note(
                                            setup_note()
                                                .start_time_ms(1.0)
                                                .end_time_ms(1000.0)
                                                .build().unwrap()
                                        )
                                        .playback_start_time_ms(0.0)
                                        .playback_end_time_ms(1000.0)
                                        .build().unwrap()
                                ]])
                                .build().unwrap()
                        )
                        .volume(0.9)
                        .effects(
                            TrackEffectsBuilder::default()
                                .envelopes(vec![envelope::default_envelope()])
                                .lfos(vec![lfo::default_lfo()])
                                .flangers(vec![flanger::no_op_flanger()])
                                .build().unwrap()
                        )
                        .build().unwrap()
                ])
            .build().unwrap();

        let playback_notes = track_grid.next_notes();
        assert_eq!(playback_notes.len(), 2);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default().clone()
    }
}
