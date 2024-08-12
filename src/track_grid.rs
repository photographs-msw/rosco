use crate::note::Note;
use crate::oscillator;
use crate::track::Track;

pub(crate) struct TrackGrid {
    pub(crate) tracks: Vec<Track>,
    sample_clock_index: f32,
}

#[allow(dead_code)]
pub(crate) struct NoteWindow {
    notes: Vec<Note>,
    start_time_ms: f32,
    end_time_ms: f32,
}

#[allow(dead_code)]
impl TrackGrid {
    pub(crate) fn get_sample_rate_index(&self) -> f32 {
        self.sample_clock_index % oscillator::SAMPLE_RATE
    }

    pub(crate) fn get_current_time_ms(&self) -> f32 {
        (self.sample_clock_index / oscillator::SAMPLE_RATE) * 1000.0
    }

    pub(crate) fn active_notes(&mut self) -> NoteWindow {
        let cur_time_ms = self.get_current_time_ms();
        let mut window_end_time_ms = f32::INFINITY;
        let mut active_notes = Vec::new();
        for track in &mut self.tracks.iter_mut() {
            for note in track.sequence.iter_mut()  {
                if note.is_playing(cur_time_ms) {
                    window_end_time_ms = f32::min(window_end_time_ms, note.end_time_ms);
                    note.volume *= track.volume;
                    active_notes.push(note.clone());
                }
            }
        }

        NoteWindow {
            notes: active_notes,
            start_time_ms: cur_time_ms,
            end_time_ms: window_end_time_ms,
        }
    }
}

mod test_sequence_grid {

    #[cfg(test)]
    mod test_sequence_grid {
        use crate::track::TrackBuilder;
        use crate::track_grid::TrackGrid;
        use crate::note::NoteBuilder;
        use crate::oscillator;
        use crate::sequence::SequenceBuilder;

        #[test]
        fn test_active_notes() {
            // Create a sequence grid with a sequence with two notes, one on and one off
            let mut track_grid = TrackGrid {
                tracks:
                    vec![
                        TrackBuilder::default()
                            .name(String::from("Track 1"))
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
                    ],
                    sample_clock_index: 0.0,
            };

            // expect one note to be active when sample_clock_index is 0.0
            let note_window = track_grid.active_notes();
            assert_eq!(note_window.notes.len(), 1);
            assert_eq!(note_window.start_time_ms, 0.0);

            // Now advance the sample_clock_index past both notes and expect no active notes
            track_grid.sample_clock_index = 2.0 * oscillator::SAMPLE_RATE;
            let note_window = track_grid.active_notes();
            assert_eq!(note_window.notes.len(), 0);
        }

        fn setup_note() -> NoteBuilder {
            NoteBuilder::default()
                // Unfortunately because end_time_ms() custom builder unwraps values from
                // start_time_ms and duration_ms so we have to set them before end_time_ms
                .start_time_ms(0.0)
                .duration_ms(1000.0)
                .end_time_ms()
                .frequency(440.0)
                .volume(1.0)
                .clone()
        }
    }
}