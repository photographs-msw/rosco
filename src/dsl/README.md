# Overview of Script Structure and Processing

When the script is parsed, the parser first creates a @track.rs `Vec<Track>`.

The parser then processes macro substitution declarations at the top of the script, before the first `Outer Block`. These declarations use the `let` keyword to bind expressions to identifiers for later reuse. Macro names can then be referenced throughout the script using the `$` prefix syntax (e.g., `$env1`).

The parser also processes template definitions using the `template` keyword. Templates are similar to macros but can contain parameter placeholders in the form `{param_name}`. Templates are applied using the `apply` keyword with parameter values.

It then reads each `Outer Block`. For each one, the parser creates a new `FixedTimeNoteSequence` and a new `TrackEffects`. The envelope and effects declared in the script are converted to their corresponding structs, `Envelope`, `Flanger`, `Delay` and `LFO`. These are passed to the builder call to create the `TrackEffects`. Then a Track is built, setting its sequence to the new `FixedTimeNoteSequence` and its track_effects to the new `TrackEffects`.

After this the parser processes each line defining a new note declaration, constructing a `PlaybackNote` of either type `osc` for a `Note` based on its waveforms, or of type `samp` for `SampledNote`. Each note is added to the current sequence.

After the last outer block, the parser constructs a `TrackGrid`, setting its tracks to the `Vec<Track>` and returns it.

# DSL Syntax Specification

- Expressions are ALL_CAPS
- Terminals are a sequence upper- and/or lower-case characters and possibly other ASCII characters
- `+` means "one or more"
- `*` means "zero or more"
- `{1}` means "exactly one"
- `|` indicates alternation
- `.` represents any chracter

--- 

COMMENT -> #.*

DELAY -> delay mix f32 decay f32 interval_ms f32 duration_ms f32 num_repeats usize num_predelay_samples usize num_concurrent_delays uszie 
FLANGER -> flanger window_size usize mix f32
LFO -> lfo freq f32 amp f32 waveforms WAVEFORMS
EFFECT_DEF -> DELAY | FLANGER | LFO

WESTERN_PITCH -> C | CSharp | C#| DFlat | Db | D | DSharp | D#| EFlat | Eb| E | F | FSharp | F#| GFlat | Gb | G | GSharp | G# | AFlat | Ab | A | ASharp | A#| BFlat | Bb | B
OCTAVE -> 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
NOTE_FREQ -> f32 | OCTAVE,WESTERN_PITCH
WAVEFORM -> sine | sin | square | sqr | triangle | tri | sawtooth | saw | guassiannoise | noise
WAVEFORMS -> WAVEFORM, | WAVEFORM
VOLUME -> f32
FILE_PATH -> .+
STEP_INDEX -> usize
OSC_NOTE -> osc:WAVEFORMS:NOTE_FREQ:VOLUME:STEP_INDEX
SAMP_NOTE -> samp:FILE_PATH:VOLUME:STEP_INDEX
NOTE_DECLARATION -> OSC_NOTE | SAMP_NOTE

DURATION_TYPE -> Whole | Half | Quarter | Eighth | Sixteenth | ThirtySecond | SixtyFourth | 1 | 1/2 | 1/4 | 1/8 | 1/16 | 1/32 | 1/64
TEMPO -> u8
NUM_STEPS -> usize
SEQUENCE_DEF -> FixedTimeNoteSequence dur DURATION_TYPE tempo TEMPO num_steps NUM_STEPS

ENVELOPE_PAIR -> f32,f32
ENVELOPE_DEF -> envelope a ENVELOPE_PAIR d ENVELOPE_PAIR s ENVELOPE_PAIR r ENVELOPE_PAIR

IDENTIFIER -> `[a-zA-Z][a-zA-Z0-9\-_]{}*`
MACRO_REFERENCE -> $IDENTIFIER
EXPR -> ENVELOPE_DEF | EFFECT_DEF | SEQUENCE_DEF | NOTE_DECLARATION | MACRO_REFERENCE
ASSIGNMENT -> let IDENTIFIER = EXPR

TEMPLATE -> template IDENTIFIER = EXPR
APPLY_IDENTIFIER -> IDENTIFIER
APPLY_ARG -> APPLY_IDENTIFIER:EXPR
APPLY -> apply APPLY_ARG+ IDENTIFIER

OUTER_BLOCK -> SEQUENCE_DEF{1} ENVELOPE_DEF* EFFECT_DEF* NOTE_DECLARATION*

SCRIPT -> ASSIGNMENT* TEMPLATE* OUTER_BLOCK+

---

## Template and Apply Examples

### Template Definition
```
template osc_template1 = osc:sine:440.0:0.9:{step}
```

This defines a template named `osc_template1` with a parameter `{step}` that can be substituted.

### Apply Expression
```
apply step:0,8 $osc_template1
```

This applies the `osc_template1` template with the `step` parameter set to values `0` and `8`, generating:
```
osc:sine:440.0:0.9:0
osc:sine:440.0:0.9:8
```

### Multiple Parameters
```
template samp_template1 = samp:/path/to/file.wav:{volume}:{step}
apply volume:0.5,0.8 step:0,4 $samp_template1
```

This generates four lines:
```
samp:/path/to/file.wav:0.5:0
samp:/path/to/file.wav:0.5:4
samp:/path/to/file.wav:0.8:0
samp:/path/to/file.wav:0.8:4
```

### Envelope Definition
```
envelope a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
```

This defines an envelope with attack, decay, sustain, and release parameters.
