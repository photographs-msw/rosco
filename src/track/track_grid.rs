use derive_builder::Builder;

use crate::note::playback_note;
use crate::note::playback_note::{PlaybackNoteBuilder, PlaybackNote, NoteType};
use crate::sequence::note_sequence_trait::NextNotes;
use crate::track::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid<SequenceType: NextNotes + Iterator> {
    pub(crate) tracks: Vec<Track<SequenceType>>,
}

impl<SequenceType: NextNotes + Iterator> TrackGrid<SequenceType> {

    pub(crate) fn next_notes(&mut self) -> Vec<PlaybackNote> {
        let mut playback_notes = Vec::new();

        for track in self.tracks.iter_mut() {
            for playback_note in track.sequence.next_notes() {
                let mut playback_note_builder = PlaybackNoteBuilder::default();
                    playback_note_builder
                        .envelopes(track.effects.envelopes.clone())
                        .lfos(track.effects.lfos.clone())
                        .flangers(track.effects.flangers.clone())
                        .playback_start_time_ms(playback_note.playback_start_time_ms)
                        .playback_end_time_ms(playback_note.playback_end_time_ms);
                
                match playback_note.note_type {
                    NoteType::Oscillator => {
                        playback_notes.push(
                            playback_note_builder
                                .note_type(NoteType::Oscillator)
                                .note(playback_note.note)
                                .build().unwrap()
                        );
                    }
                    NoteType::Sample => {
                        playback_notes.push(
                            playback_note_builder
                                .note_type(NoteType::Sample)
                                .sampled_note(playback_note.sampled_note)
                                .build().unwrap()
                        );
                    }
                }
            }
        }
        
        playback_notes
    }
}

impl<SequenceType: NextNotes + Iterator> Iterator for TrackGrid<SequenceType> {
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
