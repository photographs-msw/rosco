use derive_builder::Builder;
use crate::float_utils::{float_geq, float_leq};

use crate::envelope;
use crate::envelope::Envelope;
use crate::note_sequence_trait::NextNotes;
use crate::oscillator;
use crate::playback_note::{PlaybackNoteBuilder, PlaybackNoteKind};
use crate::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid<SequenceType: NextNotes + Iterator> {
    pub(crate) tracks: Vec<Track<SequenceType>>,
    pub(crate) track_waveforms: Vec<Vec<oscillator::Waveform>>,
    pub(crate) track_envelopes: Vec<Option<Envelope>>,
}

impl<SequenceType: NextNotes + Iterator> TrackGrid<SequenceType> {

    pub(crate) fn next_notes(&mut self) -> Vec<PlaybackNoteKind> {
        let mut playback_notes = Vec::new();
        let mut min_start_time_ms = f32::MAX;
        let mut max_end_time_ms = 0.0;

        for (i, track) in self.tracks.iter_mut().enumerate() {
            for note in track.sequence.next_notes() {
                playback_notes.push(
                    PlaybackNoteKind::WithOscillatorAndEnvelope(
                        PlaybackNoteBuilder::default()
                            .note(note)
                            .build().unwrap(),
                        self.track_waveforms[i].clone(),
                        self.track_envelopes[i].unwrap_or(envelope::default_envelope()) 
                    )
                );
                
                if float_leq(note.start_time_ms, min_start_time_ms) {
                    min_start_time_ms = note.start_time_ms;
                }
                if float_geq(note.end_time_ms, max_end_time_ms) {
                    max_end_time_ms = note.end_time_ms;
                }
            }
        }
        
        for playback_note_kind in playback_notes.iter_mut() {
            playback_note_kind.set_playback_start_time_ms(min_start_time_ms);
            playback_note_kind.set_playback_end_time_ms(max_end_time_ms);
            playback_note_kind.set_playback_duration_ms();
        }
        
        playback_notes
    }
}

impl<SequenceType: NextNotes + Iterator> Iterator for TrackGrid<SequenceType> {
    type Item = Vec<PlaybackNoteKind>;

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
                                        .duration_ms(1000.0)
                                        .end_time_ms()
                                        .build().unwrap(),
                                    setup_note()
                                        .start_time_ms(1.0)
                                        .duration_ms(1000.0)
                                        .end_time_ms()
                                        .build().unwrap(),
                                ]]).build().unwrap()
                        )
                        .volume(0.9)
                        // .default_playback_note_kind()
                        .build().unwrap()
                ])
            .track_waveforms(vec![vec![oscillator::Waveform::Sine]])
            .track_envelopes(vec![None])
            .build().unwrap();

        // expect one note to be active when sample_clock_index is 0.0
        let playback_notes = track_grid.next_notes();
        assert_eq!(playback_notes.len(), 2);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default()
            // Unfortunately because end_time_ms() custom builder unwraps values from
            // start_time_ms and duration_ms we have to set them before end_time_ms
            .frequency(440.0)
            .volume(1.0)
            .track_num(1)
            .clone()
    }
}
