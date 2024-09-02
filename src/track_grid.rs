use derive_builder::Builder;
use crate::float_utils::{float_geq, float_leq};

use crate::note::Note;
use crate::note_sequence_trait::NextNotes;
use crate::oscillator;
use crate::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid<SequenceType: NextNotes + Iterator> {
    pub(crate) tracks: Vec<Track<SequenceType>>,
    pub(crate) track_waveforms: Vec<Vec<oscillator::Waveform>>,
}

// Notes from all tracks, with associated waveforms for each note
#[derive(Clone, Debug)]
pub(crate) struct NotesData {
    pub(crate) notes: Vec<Note>,
    pub(crate) notes_waveforms: Vec<Vec<oscillator::Waveform>>,
    pub(crate) window_duration_ms: f32,
}

impl<SequenceType: NextNotes + Iterator> TrackGrid<SequenceType> {

    pub(crate) fn next_notes(&mut self) -> NotesData {
        let mut notes = Vec::new();
        let mut notes_waveforms = Vec::new();
        let mut min_start_time_ms = f32::MAX;
        let mut max_end_time_ms = 0.0;

        for (i, track) in self.tracks.iter_mut().enumerate() {
            let mut note_count = 0;
            for note in track.sequence.next_notes() {
                notes.push(note);
                if float_leq(note.start_time_ms, min_start_time_ms) {
                    min_start_time_ms = note.start_time_ms;
                }
                if float_geq(note.end_time_ms, max_end_time_ms) {
                    max_end_time_ms = note.end_time_ms;
                }
                note_count += 1;
            }
            for _ in 0..note_count {
                notes_waveforms.push(self.track_waveforms[i].clone());
            }
        }

        NotesData {
            notes,
            notes_waveforms,
            window_duration_ms: max_end_time_ms - min_start_time_ms,
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

#[cfg(test)]
mod test_sequence_grid {
    use crate::track::TrackBuilder;
    use crate::track_grid::TrackGridBuilder;
    use crate::note::NoteBuilder;
    use crate::oscillator;
    use crate::grid_note_sequence::GridNoteSequenceBuilder;

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
                                .sequence(vec![vec![
                                    // See comment below in setup_note(), we set start_time_ms there
                                    // because otherwise builder fails because end_time_ms depends on it
                                    // Now set again here to set up the logic under test
                                    setup_note()
                                        .start_time_ms(0.0)
                                        .build().unwrap(),
                                    setup_note()
                                        .start_time_ms(1.0)
                                        .build().unwrap(),
                                ]]).build().unwrap()
                        )
                        .volume(0.9)
                        .build().unwrap()
                ])
            .track_waveforms(vec![vec![oscillator::Waveform::Sine]])
            .build().unwrap();

        // expect one note to be active when sample_clock_index is 0.0
        let note_window = track_grid.next_notes();
        assert_eq!(note_window.notes.len(), 2);
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
            .clone()
    }
}
