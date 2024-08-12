use crate::note::Note;
use crate::oscillator;
use crate::sequence::Sequence;

pub(crate) struct SequenceGrid {
    pub(crate) sequences: Vec<Sequence>,
    sample_clock_index: f32,
}

#[allow(dead_code)]
impl SequenceGrid {
    pub(crate) fn get_sample_rate_index(&self) -> f32 {
        self.sample_clock_index % oscillator::SAMPLE_RATE
    }

    pub(crate) fn get_current_time_ms(&self) -> f32 {
        (self.sample_clock_index / oscillator::SAMPLE_RATE) * 1000.0
    }

    pub(crate) fn active_notes(&mut self) -> Vec<Note> {
        let cur_time_ms = self.get_current_time_ms();
        let mut active_notes = Vec::new();
        for sequence in &mut self.sequences.iter_mut() {
            for note in sequence.iter_mut()  {
                if note.is_playing(cur_time_ms) {
                    note.cur_playing_time_ms(cur_time_ms);
                    active_notes.push(note.clone());
                }
            }
        }
        active_notes
    }
}

mod test_sequence_grid {
    #[cfg(test)]
    mod test_sequence_grid {
        use crate::sequence_grid::SequenceGrid;
        use crate::sequence::SequenceBuilder;
        use crate::note::NoteBuilder;
        use crate::oscillator;

        #[test]
        fn test_active_notes() {
            // Create a sequence grid with a sequence with two notes, one on and one off
            let mut sequence_grid = SequenceGrid {
                sequences: vec![SequenceBuilder::default()
                    .notes(vec![
                        setup_note()
                            // See comment below in setup_note(), we set start_time_ms there
                            // because otherwise builder fails because end_time_ms depends on it
                            // Now set again here to set up the logic under test
                            .start_time_ms(0.0)
                            .build().unwrap(),
                        setup_note()
                            .start_time_ms(1.0)
                            .build().unwrap(),
                    ])
                    .build().unwrap()
                ],
                sample_clock_index: 0.0,
            };

            // expect one note to be active when sample_clock_index is 0.0
            let active_notes = sequence_grid.active_notes();
            assert_eq!(active_notes.len(), 1);

            // Now advance the sample_clock_index past both notes and expect no active notes
            sequence_grid.sample_clock_index = 2.0 * oscillator::SAMPLE_RATE;
            let active_notes = sequence_grid.active_notes();
            assert_eq!(active_notes.len(), 0);
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
                .cur_playing_time_ms(None)
                .clone()
        }
    }
}