use std::collections::HashMap;
use derive_builder::Builder;
use nodi::midly::num::u28;

use crate::note::Note;
use crate::{midi, oscillator};
use crate::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid {
    pub(crate) tracks: Vec<Track>,
    pub(crate) track_waveforms: Vec<Vec<oscillator::Waveform>>,
    pub(crate) sample_clock_index: f32,
    pub(crate) bpm: u8,
}

// Notes from all tracks, with associated waveforms for each note
pub(crate) struct NotesData {
    pub(crate) notes: Vec<Note>,
    pub(crate) notes_waveforms: Vec<Vec<oscillator::Waveform>>,
}

// Basically a copy of the tracks data with a global start_time, end_time over all the notes
// from all the tracks. The notes_data is vector of a struct which itself is a vector of
// notes from a track and associated data used to generate the notes, e.g. the track_waveforms
// from that track. The client is expected to walk the Vector to retrieve notes and generate their
// audio polyphonically
#[allow(dead_code)]
pub(crate) struct NotesWindow {
    // TODO Experiment with returning an immutable reference
    pub(crate) notes_data: NotesData,
    pub(crate) start_time_ms: f32,
    pub(crate) end_time_ms: f32,
}

#[allow(dead_code)]
impl TrackGrid {
    pub(crate) fn get_sample_clock_index(&self) -> f32 {
        self.sample_clock_index % oscillator::SAMPLE_RATE
    }

    pub(crate) fn get_current_time_ms(&self) -> f32 {
        (self.sample_clock_index / oscillator::SAMPLE_RATE) * 1000.0
    }

    pub(crate) fn advance_sample_clock_index_by_ms(&mut self, ms: f32) {
        self.sample_clock_index += ms * oscillator::SAMPLE_RATE / 1000.0;
    }
    
    pub(crate) fn reset_clock(&mut self) {
        self.sample_clock_index = 0.0;
    }

    // TODO DEFINITELY NEED UNIT TESTING OF THE END TIME WINDOW LOGIC
    // If any note ends or any note is added, then the window ends on that event,
    // because a window is a time range where one set of one or more notes are playing.
    // So the window end time is either the minimum end time of any note playing as of the
    // start of the window, or any new note that starts playing after the start of the window
    // but before the end of the notes initially playing.
    pub(crate) fn next_notes_window(&mut self) -> NotesWindow {
        let start_time_ms = self.get_current_time_ms();

        // NOTE: mutable bindings in this loop because notes_playing_at_start is used mutably
        // in the load_notes_and_waveforms and calculate_end_time_ms functions
        let notes_playing_at_start = self.tracks.iter()
            .flat_map(|track| track.sequence.iter())
            .filter(|note| note.is_playing(start_time_ms))
            .collect::<Vec<&Note>>();
        if notes_playing_at_start.is_empty() {
           return NotesWindow::empty_notes_window(); 
        }

        // Calculate the end of the window in two passes
        // This first value is the end of the note ending the soonest after the start of the window
        // This is the end of the window unless we find a note that starts before this time
        // by walking forward in time from the start of the window to this time, checked next
        let mut end_time_ms = f32::INFINITY;
        if notes_playing_at_start.len() == 1 {
            end_time_ms = notes_playing_at_start.first().unwrap().end_time_ms;
        } else {
            notes_playing_at_start.iter()
                .map(|note| note.end_time_ms)
                .min_by(|a, b| a.partial_cmp(&b)
                    .unwrap()).unwrap();
        }
        // Now do the second pass, walk the range of ticks from start to end and see if any
        // new note starts before the earliest end of the notes in the starting set.
        // That new note would change the set so its start would mark the end of the window.
        // +1 to skip the first tick, which we already checked above in getting the notes at start
        let start_time_tick =
            (midi::milliseconds_to_ticks(self.bpm, start_time_ms) + u28::from(1)).as_int();
        // +1 because this is an open range so we include the last tick of the note
        let cur_end_time_tick =
            (midi::milliseconds_to_ticks(self.bpm, end_time_ms) + u28::from(1)).as_int();
        let mut found = false;
        for tick in start_time_tick..cur_end_time_tick {
            if found {
                break;
            }
            let time_ms = midi::ticks_to_milliseconds(self.bpm, u28::from(tick));
            // TODO THIS IS A PAINFUL CLONE too
            for track in self.tracks.clone() {
                for note in track.sequence.iter() {
                    // If we hit a new note starting for the first time that wasn't in the
                    // set of notes at the start of the window, and we haven't reached end time
                    // yet then this is the end of the window, so break
                    if note.is_playing(time_ms) && !notes_playing_at_start.contains(&note) {
                        end_time_ms = time_ms;
                        found = true;
                    }
                }
            }
        }

        // Walk the notes found at the start and set their volume adjusted by the track volume
        // and push a clone of each note and its waveforms into the output
        let mut window_notes_data = Vec::new();
        let mut window_notes_waveforms = Vec::new();
        // self.load_notes_data_and_waveforms(&mut notes_playing_at_start, &mut window_notes_data,
        //                                    &mut window_notes_waveforms);
        let track_num_track_map: HashMap<i16, &Track> =
            self.tracks.iter().map(|track| (track.num, track)).collect();
        for (i, note) in notes_playing_at_start.clone().iter_mut().enumerate() {
            let track_for_note = track_num_track_map.get(&note.track_num).unwrap();
            // TODO This still assumes even volume for entire note duration, no envelope support
            // let out_note: &mut Note = note.clone();
            let new_volume = (**note).volume * (**track_for_note).volume;
            let mut note = (**note).clone();
            note.volume = new_volume;
            window_notes_data.push(note);
            window_notes_waveforms.push(self.track_waveforms[i].clone());
        }
        
        self.advance_sample_clock_index_by_ms(end_time_ms - start_time_ms);

        NotesWindow {
            notes_data: NotesData {
                notes: window_notes_data,
                notes_waveforms: window_notes_waveforms,
            },
            start_time_ms,
            end_time_ms,
        }
    }
    
    fn calculate_end_time_ms (&mut self,
                              start_time_ms: &f32,
                              notes_playing_at_start: &Vec<&mut Note>) -> f32 {
        // Calculate the end of the window in two passes
        // This first value is the end of the note ending the soonest after the start of the window
        // This is the end of the window unless we find a note that starts before this time
        // by walking forward in time from the start of the window to this time, checked next
        let mut end_time_ms =
            notes_playing_at_start.iter()
                .map(|note| note.end_time_ms)
                .min_by(|a, b| a.partial_cmp(&b)
                    .unwrap()).unwrap();
        // Now do the second pass, walk the range of ticks from start to end and see if any
        // new note starts before the earliest end of the notes in the starting set.
        // That new note would change the set so its start would mark the end of the window.
        // +1 to skip the first tick, which we already checked above in getting the notes at start
        let start_time_tick =
            (midi::milliseconds_to_ticks(self.bpm, *start_time_ms) + u28::from(1)).as_int();
        // +1 because this is an open range so we include the last tick of the note
        let cur_end_time_tick =
            (midi::milliseconds_to_ticks(self.bpm, end_time_ms) + u28::from(1)).as_int();
        'outer: for tick in start_time_tick..cur_end_time_tick {
            let time_ms = midi::ticks_to_milliseconds(self.bpm, u28::from(tick));
            for track in &mut self.tracks.iter_mut() {
                for note in track.sequence.iter_mut() {
                    // If we hit a new note starting for the first time that wasn't in the
                    // set of notes at the start of the window, and we haven't reached end time
                    // yet then this is the end of the window, so break
                    if note.is_playing(time_ms) && !notes_playing_at_start.contains(&note) {
                        end_time_ms = time_ms;
                        break 'outer;
                    }
                }
            }
        }
        end_time_ms
    }
}

// Custom iterator for TrackGrid over the note_windows in the grid
impl<'a> Iterator for TrackGrid {
    type Item = NotesWindow;

    fn next(&mut self) -> Option<Self::Item> {
        let notes_window = self.next_notes_window();
        if notes_window.is_empty() {
            return None;
        }

        Some(notes_window)
    }
}

impl NotesData {
    pub(crate) fn empty_notes_data() -> NotesData {
        NotesData {
            notes: Vec::new(),
            notes_waveforms: Vec::new(),
        }
    }
}

impl NotesWindow {
    pub(crate) fn is_empty(&self) -> bool {
        self.notes_data.notes.is_empty()
    }

    pub(crate) fn window_duration_ms(&self) -> f32 {
        self.end_time_ms - self.start_time_ms
    }
    
    pub(crate) fn empty_notes_window() -> NotesWindow {
        NotesWindow {
            notes_data: NotesData::empty_notes_data(),
            start_time_ms: 0.0,
            end_time_ms: f32::INFINITY,
        }
    }
}

#[cfg(test)]
mod test_sequence_grid {
    use crate::track::TrackBuilder;
    use crate::track_grid::TrackGridBuilder;
    use crate::note::NoteBuilder;
    use crate::oscillator;
    use crate::sequence::SequenceBuilder;

    #[test]
    fn test_next_notes_window_boundaries() {
        // Create a sequence grid with a sequence with two notes, one on and one off
        let mut track_grid = TrackGridBuilder::default()
            .tracks(
                vec![
                    TrackBuilder::default()
                        .num(1)
                        .sequence(
                            SequenceBuilder::default()
                                .notes(vec![
                                    // See comment below in setup_note(), we set start_time_ms there
                                    // because otherwise builder fails because end_time_ms depends on it
                                    // Now set again here to set up the logic under test
                                    setup_note()
                                        .start_time_ms(0.0)
                                        .build().unwrap(),
                                    setup_note()
                                        .start_time_ms(1.0)
                                        .build().unwrap(),
                                ]).build().unwrap()
                        )
                        .volume(0.9)
                        .build().unwrap()
                ])
            .track_waveforms(vec![vec![oscillator::Waveform::Sine]])
            .sample_clock_index(0.0)
            .bpm(120)
            .build().unwrap();

        // expect one note to be active when sample_clock_index is 0.0
        let note_window = track_grid.next_notes_window();
        assert_eq!(note_window.notes_data.notes.len(), 1);
        assert_eq!(note_window.start_time_ms, 0.0);

        // Now advance the sample_clock_index past both notes and expect no active notes
        track_grid.sample_clock_index = 2.0 * oscillator::SAMPLE_RATE;
        let notes_window = track_grid.next_notes_window();
        assert_eq!(notes_window.notes_data.notes.len(), 0);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            // Unfortunately because end_time_ms() custom builder unwraps values from
            // start_time_ms and duration_ms we have to set them before end_time_ms
            .start_time_ms(0.0)
            .duration_ms(1000.0)
            .end_time_ms()
            .frequency(440.0)
            .volume(1.0)
            .default_envelope()
            .track_num(1)
            .clone()
    }
}