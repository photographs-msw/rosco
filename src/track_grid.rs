use derive_builder::Builder;
use crate::float_utils::{float_geq, float_leq};

use crate::note_sequence_trait::NextNotes;
use crate::oscillator::Waveform;
use crate::playback_note::{PlaybackNoteBuilder, PlaybackNote};
use crate::sample_effect_trait::{ApplyEffect, BuilderWrapper};
use crate::track::Track;

#[derive(Builder, Clone, Debug)]
pub(crate) struct TrackGrid<
    SequenceType: NextNotes + Iterator,
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Clone + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Send,
> {
    pub(crate) tracks: Vec<Track<SequenceType>>,
    
    pub(crate) track_waveforms: Vec<Vec<Waveform>>,

    #[builder(default = "vec![EnvelopeType::new(); self.tracks.clone().unwrap().len()]")]
    pub(crate) track_envelopes: Vec<EnvelopeType>,
    
    #[builder(default = "vec![vec![LFOType::new()]; self.tracks.clone().unwrap().len()]")]
    pub(crate) track_lfos: Vec<Vec<LFOType>>,
}

impl<
    SequenceType: NextNotes + Iterator,
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Clone + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Clone + Send,
>
TrackGrid<SequenceType, EnvelopeType, LFOType> {

    pub(crate) fn next_notes(&mut self) -> Vec<PlaybackNote<EnvelopeType, LFOType>> {
        let mut playback_notes = Vec::new();
        let mut min_start_time_ms = f32::MAX;
        let mut max_end_time_ms = 0.0;

        for (i, track) in self.tracks.iter_mut().enumerate() {
            for note in track.sequence.next_notes() {
                playback_notes.push(
                    PlaybackNoteBuilder::default()
                        .note(note)
                        .waveforms(self.track_waveforms[i].clone())
                        .envelope(self.track_envelopes[i].clone())
                        .lfos(self.track_lfos[i].clone())
                        .build().unwrap()
                );
                
                if float_leq(note.start_time_ms, min_start_time_ms) {
                    min_start_time_ms = note.start_time_ms;
                }
                if float_geq(note.end_time_ms(), max_end_time_ms) {
                    max_end_time_ms = note.end_time_ms();
                }
            }
        }
        
        for playback_note in playback_notes.iter_mut() {
            playback_note.playback_start_time_ms = min_start_time_ms;
            playback_note.playback_end_time_ms = max_end_time_ms;
        }
        
        playback_notes
    }
}

impl<
    SequenceType: NextNotes + Iterator,
    EnvelopeType: ApplyEffect + BuilderWrapper<EnvelopeType> + Clone + Send,
    LFOType: ApplyEffect + BuilderWrapper<LFOType> + Clone + Send,
>
Iterator for TrackGrid<SequenceType, EnvelopeType, LFOType> {
    type Item = Vec<PlaybackNote<EnvelopeType, LFOType>>;

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
    use crate::{envelope, lfo};
    use crate::grid_note_sequence::GridNoteSequenceBuilder;
    use crate::note::NoteBuilder;
    use crate::oscillator;
    use crate::track::TrackBuilder;
    use crate::track_grid::TrackGridBuilder;

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
                                        .build().unwrap(),
                                    setup_note()
                                        .start_time_ms(1.0)
                                        .duration_ms(1000.0)
                                        .build().unwrap(),
                                ]]).build().unwrap()
                        )
                        .volume(0.9)
                        // .default_playback_note_kind()
                        .build().unwrap()
                ])
            .track_waveforms(vec![vec![oscillator::Waveform::Sine]])
            .track_envelopes(vec![envelope::default_envelope()])
            .track_lfos(vec![vec![lfo::default_lfo()]])
            .build().unwrap();

        // expect one note to be active when sample_clock_index is 0.0
        let playback_notes = track_grid.next_notes();
        assert_eq!(playback_notes.len(), 2);
    }

    fn setup_note() -> NoteBuilder {
        NoteBuilder::default().clone()
    }
}
