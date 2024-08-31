use derive_builder::Builder;

use crate::note::Note;
use crate::note_sequence_trait::NextNotes;
use crate::oscillator;
use crate::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid<SequenceType: NextNotes + Iterator> {
    pub(crate) tracks: Vec<Track<SequenceType>>,
    pub(crate) track_waveforms: Vec<Vec<oscillator::Waveform>>,

    // TODO THIS IS EITHER PAREMETERIZED OR WE HAVE BOTH TIME AND INDEX AND IGNORE ONE
    #[builder(default = "0.0")]
    pub(crate) sample_clock_index: f32,

    #[builder(default = "0")]
    pub(crate) index: usize,
}

// Notes from all tracks, with associated waveforms for each note
#[derive(Clone, Debug)]
pub(crate) struct NotesData {
    pub(crate) notes: Vec<Note>,
    pub(crate) notes_waveforms: Vec<Vec<oscillator::Waveform>>,
}

impl<SequenceType: NextNotes + Iterator> TrackGrid<SequenceType> {

    pub(crate) fn next_notes(&mut self) -> NotesData {
        let mut notes = Vec::new();
        let mut notes_waveforms = Vec::new();

        for (i, track) in self.tracks.iter_mut().enumerate() {
            let mut note_count = 0;
            for note in track.sequence.next_notes() {
                notes.push(note);
                note_count += 1;
            }
            for _ in 0..note_count {
                notes_waveforms.push(self.track_waveforms[i].clone());
            }
        }

        NotesData {
            notes,
            notes_waveforms
        }
    }
}

impl<SequenceType: NextNotes + Iterator> Iterator for TrackGrid<SequenceType> {
    type Item = NotesData;

    fn next(&mut self) -> Option<Self::Item> {
        println!("TrackGrid::next() called");
        
        let notes_window = self.next_notes();
        if notes_window.notes.is_empty() {
            return None;
        }

        Some(notes_window)
    }
}


// Basically a copy of the tracks data with a global start_time, end_time over all the notes
// from all the tracks. The notes_data is vector of a struct which itself is a vector of
// notes from a track and associated data used to generate the notes, e.g. the track_waveforms
// from that track. The client is expected to walk the Vector to retrieve notes and generate their
// audio polyphonically
// #[allow(dead_code)]
// pub(crate) struct NotesWindow {
//     // TODO Experiment with returning an immutable reference
//     pub(crate) notes_data: NotesData,
//     pub(crate) start_time_ms: f32,
//     pub(crate) end_time_ms: f32,
// }
//
// #[allow(dead_code)]
// impl TrackGrid {
//     pub(crate) fn get_sample_clock_index(&self) -> f32 {
//         self.sample_clock_index % oscillator::SAMPLE_RATE
//     }
//
//     pub(crate) fn get_current_time_ms(&self) -> f32 {
//         (self.sample_clock_index / oscillator::SAMPLE_RATE) * 1000.0
//     }
//
//     pub(crate) fn advance_sample_clock_index_by_ms(&mut self, ms: f32) {
//         self.sample_clock_index += ms * oscillator::SAMPLE_RATE / 1000.0;
//     }
//
//     pub(crate) fn next_notes_window(&mut self) -> NotesWindow {
//         let start_time_ms = self.get_current_time_ms();
//
//         let mut end_time_ms = f32::INFINITY;
//         let mut window_notes_data = Vec::new();
//         let mut window_notes_waveforms = Vec::new();
//         for (i, track) in &mut self.tracks.iter_mut().enumerate() {
//             for note in track.sequence.notes_iter_mut()  {
//                 if note.is_playing(start_time_ms) {
//                     end_time_ms = f32::min(end_time_ms, note.end_time_ms);
//                     note.volume *= track.volume;
//                     window_notes_data.push(note.clone());
//                     window_notes_waveforms.push(self.track_waveforms[i].clone());
//                 }
//             }
//         }
//
//         self.advance_sample_clock_index_by_ms(end_time_ms - start_time_ms);
//
//         NotesWindow {
//             notes_data: NotesData {
//                 notes: window_notes_data,
//                 notes_waveforms: window_notes_waveforms,
//             },
//             start_time_ms,
//             end_time_ms,
//         }
//     }
// }

// Custom iterator for TrackGrid over the note_windows in the grid
// impl<'a, SequenceType: Iterator> Iterator for TrackGrid<SequenceType> {
//     type Item = NotesData;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let mut next_notes = Vec::new();
//         for track in &mut self.tracks {
//             for notes in track.sequence {
//                 let x: Vec<Note> = notes.into();
//                 next_notes.append(&mut notes);
//             }
//         }
//         // let notes_data = self.tracks.iter()
//         //     .map(|track| track.sequence.)
//         //     .flatten()
//         //     .collect();
//
//         if next_notes.is_empty() {
//             return None;
//         }
//         Some(
//             NotesData {
//                 notes: next_notes,
//                 notes_waveforms: self.track_waveforms.clone(),
//             }
//             )
//     }
// }

// impl<'a, SequenceType: Iterator<Item = Vec<Note>>> Iterator for TrackGrid<SequenceType> {
//     type Item = NotesData;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let mut next_notes = Vec::new();
//         for track in &self.tracks {
//             let sequence = track.sequence.next_notes();
//             for notes in sequence {
//                 let mut x: Vec<Note> = notes;
//                 next_notes.append(&mut x);
//             }
//         }
//
//         if next_notes.is_empty() {
//             return None;
//         }
//         Some(
//             NotesData {
//                 notes: next_notes,
//                 notes_waveforms: self.track_waveforms.clone(),
//             }
//         )
//     }
// }

// impl NotesWindow {
//     pub(crate) fn is_empty(&self) -> bool {
//         self.notes_data.notes.is_empty()
//     }
//
//     pub(crate) fn window_duration_ms(&self) -> f32 {
//         self.end_time_ms - self.start_time_ms
//     }
// }
//
// #[cfg(test)]
// mod test_sequence_grid {
//     use crate::track::TrackBuilder;
//     use crate::track_grid::TrackGridBuilder;
//     use crate::note::NoteBuilder;
//     use crate::oscillator;
//     use crate::grid_note_sequence::GridNoteSequenceBuilder;
//
//     #[test]
//     fn test_active_notes() {
//         // Create a sequence grid with a sequence with two notes, one on and one off
//         let mut track_grid = TrackGridBuilder::default()
//             .tracks(
//                 vec![
//                     TrackBuilder::default()
//                         .name(String::from("Track 1"))
//                         .sequence(
//                             GridNoteSequenceBuilder::default()
//                                 .sequence(vec![vec![
//                                     // See comment below in setup_note(), we set start_time_ms there
//                                     // because otherwise builder fails because end_time_ms depends on it
//                                     // Now set again here to set up the logic under test
//                                     setup_note()
//                                         .start_time_ms(0.0)
//                                         .build().unwrap(),
//                                     setup_note()
//                                         .start_time_ms(1.0)
//                                         .build().unwrap(),
//                                 ]]).build().unwrap()
//                         )
//                         .volume(0.9)
//                         .build().unwrap()
//                 ])
//             .track_waveforms(vec![vec![oscillator::Waveform::Sine]])
//             .sample_clock_index(0.0)
//             .build().unwrap();
//
//         // expect one note to be active when sample_clock_index is 0.0
//         let note_window = track_grid.next_notes_window();
//         assert_eq!(note_window.notes_data.notes.len(), 1);
//         assert_eq!(note_window.start_time_ms, 0.0);
//
//         // Now advance the sample_clock_index past both notes and expect no active notes
//         track_grid.sample_clock_index = 2.0 * oscillator::SAMPLE_RATE;
//         let notes_window = track_grid.next_notes_window();
//         assert_eq!(notes_window.notes_data.notes.len(), 0);
//     }
//
//     fn setup_note() -> NoteBuilder {
//         NoteBuilder::default()
//             // Unfortunately because end_time_ms() custom builder unwraps values from
//             // start_time_ms and duration_ms we have to set them before end_time_ms
//             .start_time_ms(0.0)
//             .duration_ms(1000.0)
//             .end_time_ms()
//             .frequency(440.0)
//             .volume(1.0)
//             .clone()
//     }
// }