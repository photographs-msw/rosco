# Overview of Script Structure and Processing

When the script is parsed, the parser first creates a @track.rs Vec<Track>.

It then reads each outer block. For each outer block the parser creates a new @fixed_time_note_sequence.rs FixedTimeNoteSequence and a new @track_effects.rs TrackEffects. The envelope and effects declared in the script are converted to their corresponding structs, @envelope.rs Envelope, @flanger.rs Flanger, @delay.rs Delay, @lfo.rs LFO. These are passed to the builder call to create the TrackEffects. Then a @track.rs Track is built, setting its sequence to the new FixedTimeNoteSequence and its track_effects to the new TrackEffects.

After this the parser processes each line defining a new note declaration, constructing a @note.rs @playback_note.rs @sampled_note.rs @oscillator.rs PlaybackNote of either type osc for a Note based on its waveforms, or of type samp for SampledNote. Each note is added to the current Sequence

After the last outer block, the parser constructs a @track_grid.rs TrackGrid, setting its tracks to the Vec<Track> and returns it

# Syntax Specification

Expressions are ALL_CAPS. Terminals are normal case or lower case. '+' = 'one or more'. '*' = 'zero or more'. '{1}' = 'exactly one'. '|' is alternation. '.' is any chracter.

COMMENT -> #.*

DURATION_TYPE -> Whole | Half | Quarter | Eighth | Sixteenth | ThirtySecond | SixtyFourth | 1 | 1/2 | 1/4 | 1/8 | 1/16 | 1/32 | 1/64
TEMPO -> u8
NUM_STEPS -> usize
ENVELOPE_PAIR -> f32,f32
DELAY -> delay mix f32 decay f32 interval_ms f32 duration_ms f32 num_repeats usize num_predelay_samples usize num_concurrent_delays uszie 
FLANGER -> flanger window_size usize mix f32
WAVEFORM -> sine | sin | square | sqr | triangle | tri | sawtooth | saw | guassiannoise | noise
WAVEFORMS -> WAVEFORM, | WAVEFORM
LFO -> lfo freq f32 amp f32 waveforms WAVEFORMS
EFFECT_DEF -> DELAY | FLANGER | LFO
WESTERN_PITCH -> C | CSharp | C#| DFlat | Db | D | DSharp | D#| EFlat | Eb| E | F | FSharp | F#| GFlat | Gb| G | GSharp | G# | AFlat | Ab | A | ASharp | A#| BFlat | Bb | B
OCTAVE -> 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
NOTE_FREQ -> f32 | OCTAVE,WESTERN_PITCH
VOLUME -> f32
STEP_INDEX -> usize
FILE_PATH -> .+
OSC_NOTE -> osc:WAVEFORMS:NOTE_FREQ:VOLUME:STEP_INDEX
SAMP_NOTE -> samp:FILE_PATH:VOLUME:STEP_INDEX

SEQUENCE_DEF -> FixedTimeNoteSequence dur DURATION_TYPE tempo TEMPO num_steps NUM_STEPS
ENVELOPE_DEF -> a ENVELOPE_PAIR d ENVELOPE_PAIR s ENVELOPE_PAIR r ENVELOPE_PAIR
NOTE_DECLARATION -> OSC_NOTE | SAMP_NOTE

OUTER_BLOCK -> SEQUENCE_DEF{1} ENVELOPE_DEF* EFFECT_DEF* NOTE_DECLARATION*

SCRIPT -> OUTER_BLOCK+
