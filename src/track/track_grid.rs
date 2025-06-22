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
        println!("TrackGrid::next_notes() called at position: {}", self.cur_position_ms);

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
        let mut all_tracks_done = true;

        for (i, track) in self.tracks.iter_mut().enumerate() {
            track.sequence.set_cur_position(self.cur_position_ms);
            let notes = track.sequence.next_notes();
            println!("Track {} returned {} notes", i, notes.len());
            if !notes.is_empty() {
                all_tracks_done = false;
            }
            for playback_note in notes {
                let mut playback_note_builder = PlaybackNoteBuilder::default();
                    playback_note_builder
                        .playback_start_time_ms(playback_note.playback_start_time_ms)
                        .playback_end_time_ms(playback_note.playback_end_time_ms)
                        .playback_sample_start_time((playback_note.playback_start_time_ms *
                            (SAMPLE_RATE / 1000.0)).floor() as u64)
                        .playback_end_time_ms(playback_note.playback_end_time_ms)
                        .playback_sample_end_time((playback_note.playback_end_time_ms *
                            (SAMPLE_RATE / 1000.0)).floor() as u64)
                        .envelopes(playback_note.envelopes.clone())
                        .lfos(playback_note.lfos.clone())
                        .flangers(playback_note.flangers.clone())
                        .delays(playback_note.delays.clone())
                        .track_effects(track.effects.clone());
                
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

        println!("all_tracks_done: {}, track_playback_notes.len(): {}", all_tracks_done, track_playback_notes.len());

        // If all tracks are done, return empty vector to end the iterator
        if all_tracks_done && track_playback_notes.is_empty() {
            println!("All tracks done, returning empty vector");
            return vec![];
        }

        let window_start_time_ms = get_frontier_min_start_time(&track_playback_notes);
        let window_end_time_ms = get_frontier_min_end_time(
            &track_playback_notes, self.cur_position_ms);

        println!("window_start_time_ms: {}, window_end_time_ms: {}", window_start_time_ms, window_end_time_ms);

        if self.cur_position_ms < window_start_time_ms {
            self.cur_position_ms = window_start_time_ms + FLOAT_EPSILON;
            println!("Emitting rest note from {} to {}", self.cur_position_ms, window_start_time_ms);
            return vec![playback_note::playback_rest_note(self.cur_position_ms,
                                                          window_start_time_ms)];
        }

        let mut out_playback_notes = Vec::new();

        // If the current note time is the same as the frontier min start time, emit all notes
        // that start in the current window (from cur_position_ms to window_end_time_ms)
        if float_eq(self.cur_position_ms, window_start_time_ms) {
            let playback_notes: Vec<PlaybackNote> = track_playback_notes
                .iter()
                .filter(|playback_note| {
                    let start_time = playback_note.note_start_time_ms();
                    float_geq(start_time, self.cur_position_ms) &&
                    float_leq(start_time, window_end_time_ms)
                })
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
        println!("Returning {} notes, advancing to position: {}", out_playback_notes.len(), self.cur_position_ms);

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
    if playback_notes.is_empty() {
        // If there are no notes, return a reasonable end time
        return note_time_ms + 5_000.0; // 5 seconds window
    }

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

    // If we still have f32::MAX, it means no valid end time was found
    // Return a reasonable fallback
    if end_time_ms == f32::MAX {
        end_time_ms = note_time_ms + 10_000.0; // 10 seconds window
    }

    end_time_ms
}

impl<SequenceType: NextNotes + Iterator + SetCurPosition> Iterator for TrackGrid<SequenceType> {
    type Item = Vec<PlaybackNote>;

    fn next(&mut self) -> Option<Self::Item> {
        let playback_notes= self.next_notes();
        if playback_notes.is_empty() {
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
