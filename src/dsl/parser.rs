use std::str::FromStr;
use std::collections::HashMap;
use regex;

use crate::audio_gen::oscillator::Waveform;
use crate::effect::delay::{DelayBuilder};
use crate::effect::flanger::{FlangerBuilder};
use crate::effect::lfo::{LFOBuilder};
use crate::envelope::envelope::{EnvelopeBuilder};
use crate::envelope::envelope_pair::EnvelopePair;
use crate::meter::durations::DurationType as MeterDurationType;
use crate::note::note::{NoteBuilder};
use crate::note::playback_note::{NoteType, PlaybackNote, PlaybackNoteBuilder};
use crate::note::sampled_note::{SampledNoteBuilder};
use crate::note::scales::WesternPitch;
use crate::sequence::fixed_time_note_sequence::{FixedTimeNoteSequence, FixedTimeNoteSequenceBuilder};
use crate::sequence::note_sequence_trait::AppendNote;
use crate::track::track::{Track, TrackBuilder};
use crate::track::track_effects::{TrackEffects, TrackEffectsBuilder};
use crate::track::track_grid::{TrackGrid, TrackGridBuilder};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DslDurationType {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    Fraction(f32), // For 1, 1/2, 1/4, etc.
}

impl FromStr for DslDurationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Whole" => Ok(DslDurationType::Whole),
            "Half" => Ok(DslDurationType::Half),
            "Quarter" => Ok(DslDurationType::Quarter),
            "Eighth" => Ok(DslDurationType::Eighth),
            "Sixteenth" => Ok(DslDurationType::Sixteenth),
            "ThirtySecond" => Ok(DslDurationType::ThirtySecond),
            "SixtyFourth" => Ok(DslDurationType::SixtyFourth),
            "1" => Ok(DslDurationType::Fraction(1.0)),
            "1/2" => Ok(DslDurationType::Fraction(0.5)),
            "1/4" => Ok(DslDurationType::Fraction(0.25)),
            "1/8" => Ok(DslDurationType::Fraction(0.125)),
            "1/16" => Ok(DslDurationType::Fraction(0.0625)),
            "1/32" => Ok(DslDurationType::Fraction(0.03125)),
            "1/64" => Ok(DslDurationType::Fraction(0.015625)),
            _ => Err(format!("Unknown duration type: {}", s)),
        }
    }
}

impl DslDurationType {
    #[allow(dead_code)]
    fn to_factor(&self) -> f32 {
        match self {
            DslDurationType::Whole => 1.0,
            DslDurationType::Half => 0.5,
            DslDurationType::Quarter => 0.25,
            DslDurationType::Eighth => 0.125,
            DslDurationType::Sixteenth => 0.0625,
            DslDurationType::ThirtySecond => 0.03125,
            DslDurationType::SixtyFourth => 0.015625,
            DslDurationType::Fraction(f) => *f,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WaveformType {
    Sine,
    Sin,
    Square,
    Sqr,
    Triangle,
    Tri,
    Sawtooth,
    Saw,
    GaussianNoise,
    Noise,
}

impl FromStr for WaveformType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sine" | "sin" => Ok(WaveformType::Sine),
            "square" | "sqr" => Ok(WaveformType::Square),
            "triangle" | "tri" => Ok(WaveformType::Triangle),
            "sawtooth" | "saw" => Ok(WaveformType::Sawtooth),
            "gaussiannoise" | "noise" => Ok(WaveformType::GaussianNoise),
            _ => Err(format!("Unknown waveform: {}", s)),
        }
    }
}

impl WaveformType {
    fn to_waveform(&self) -> Waveform {
        match self {
            WaveformType::Sine | WaveformType::Sin => Waveform::Sine,
            WaveformType::Square | WaveformType::Sqr => Waveform::Square,
            WaveformType::Triangle | WaveformType::Tri => Waveform::Triangle,
            WaveformType::Sawtooth | WaveformType::Saw => Waveform::Saw,
            WaveformType::GaussianNoise | WaveformType::Noise => Waveform::GaussianNoise,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WesternPitchType {
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
}

impl FromStr for WesternPitchType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(WesternPitchType::C),
            "CSharp" | "C#" => Ok(WesternPitchType::CSharp),
            "DFlat" | "Db" => Ok(WesternPitchType::DFlat),
            "D" => Ok(WesternPitchType::D),
            "DSharp" | "D#" => Ok(WesternPitchType::DSharp),
            "EFlat" | "Eb" => Ok(WesternPitchType::EFlat),
            "E" => Ok(WesternPitchType::E),
            "F" => Ok(WesternPitchType::F),
            "FSharp" | "F#" => Ok(WesternPitchType::FSharp),
            "GFlat" | "Gb" => Ok(WesternPitchType::GFlat),
            "G" => Ok(WesternPitchType::G),
            "GSharp" | "G#" => Ok(WesternPitchType::GSharp),
            "AFlat" | "Ab" => Ok(WesternPitchType::AFlat),
            "A" => Ok(WesternPitchType::A),
            "ASharp" | "A#" => Ok(WesternPitchType::ASharp),
            "BFlat" | "Bb" => Ok(WesternPitchType::BFlat),
            "B" => Ok(WesternPitchType::B),
            _ => Err(format!("Unknown western pitch: {}", s)),
        }
    }
}

impl WesternPitchType {
    #[allow(dead_code)]
    fn to_western_pitch(&self) -> WesternPitch {
        match self {
            WesternPitchType::C => WesternPitch::C,
            WesternPitchType::CSharp => WesternPitch::CSharp,
            WesternPitchType::DFlat => WesternPitch::DFlat,
            WesternPitchType::D => WesternPitch::D,
            WesternPitchType::DSharp => WesternPitch::DSharp,
            WesternPitchType::EFlat => WesternPitch::EFlat,
            WesternPitchType::E => WesternPitch::E,
            WesternPitchType::F => WesternPitch::F,
            WesternPitchType::FSharp => WesternPitch::FSharp,
            WesternPitchType::GFlat => WesternPitch::GFlat,
            WesternPitchType::G => WesternPitch::G,
            WesternPitchType::GSharp => WesternPitch::GSharp,
            WesternPitchType::AFlat => WesternPitch::AFlat,
            WesternPitchType::A => WesternPitch::A,
            WesternPitchType::ASharp => WesternPitch::ASharp,
            WesternPitchType::BFlat => WesternPitch::BFlat,
            WesternPitchType::B => WesternPitch::B,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DelayDef {
    pub mix: f32,
    pub decay: f32,
    pub interval_ms: f32,
    pub duration_ms: f32,
    pub num_repeats: usize,
    pub num_predelay_samples: usize,
    pub num_concurrent_delays: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FlangerDef {
    pub window_size: usize,
    pub mix: f32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LFODef {
    pub freq: f32,
    pub amp: f32,
    pub waveforms: Vec<WaveformType>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EffectDef {
    Delay(DelayDef),
    Flanger(FlangerDef),
    LFO(LFODef),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnvelopeDef {
    pub attack: (f32, f32),
    pub decay: (f32, f32),
    pub sustain: (f32, f32),
    pub release: (f32, f32),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SequenceDef {
    pub dur: DslDurationType,
    pub tempo: u8,
    pub num_steps: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NoteDeclaration {
    Oscillator {
        waveforms: Vec<WaveformType>,
        note_freq: f32,
        volume: f32,
        step_index: usize,
    },
    Sample {
        file_path: String,
        volume: f32,
        step_index: usize,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OuterBlock {
    pub sequence_def: SequenceDef,
    pub envelope_defs: Vec<EnvelopeDef>,
    pub effect_defs: Vec<EffectDef>,
    pub note_declarations: Vec<NoteDeclaration>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MacroDef {
    pub name: String,
    pub expression: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TemplateDef {
    pub name: String,
    pub expression: String,
    pub parameters: Vec<String>, // List of parameter names like "step"
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApplyArg {
    pub parameter: String, // e.g., "step"
    pub values: Vec<String>, // e.g., ["0", "8"]
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApplyExpression {
    pub args: Vec<ApplyArg>,
    pub template_name: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Script {
    pub macro_defs: HashMap<String, String>,
    pub template_defs: HashMap<String, TemplateDef>,
    pub outer_blocks: Vec<OuterBlock>,
}

#[allow(dead_code)]
pub struct Parser {
    tokens: Vec<String>,
    current: usize,
}

impl Parser {
    #[allow(dead_code)]
    pub fn new(input: &str) -> Self {
        let expanded_input = Self::expand_macros(input).unwrap_or_else(|e| {
            panic!("Failed to expand macros: {}", e);
        });
        let tokens = Self::tokenize(&expanded_input);
        Self {
            tokens,
            current: 0,
        }
    }

    fn expand_macros(input: &str) -> Result<String, String> {
        let mut expanded = input.to_string();
        let mut macro_defs = HashMap::new();
        let mut template_defs = HashMap::new();
        
        // First pass: collect all macro and template definitions
        let lines: Vec<&str> = input.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("let ") {
                // Parse macro definition
                if let Some((name, value)) = Self::parse_macro_definition_line(line)? {
                    macro_defs.insert(name, value);
                }
            } else if line.starts_with("template ") {
                // Parse template definition
                if let Some((name, value)) = Self::parse_template_definition_line(line)? {
                    let parameters = Self::extract_template_parameters(&value);
                    template_defs.insert(name, (value, parameters));
                }
            } else if line.starts_with("FixedTimeNoteSequence") {
                // Stop at first outer block
                break;
            }
            i += 1;
        }
        
        // Multi-pass expansion: repeat until no changes
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_expanded = expanded.clone();
            
            // Expand macros
            for (name, value) in &macro_defs {
                let pattern = format!("${}", name);
                if new_expanded.contains(&pattern) {
                    new_expanded = new_expanded.replace(&pattern, value);
                    changed = true;
                }
            }
            
            // Expand apply expressions
            let (new_expanded_apply, apply_changed) = Self::expand_apply_expressions(&new_expanded, &template_defs)?;
            new_expanded = new_expanded_apply;
            if apply_changed {
                changed = true;
            }
            
            expanded = new_expanded;
        }
        
        // Check for any remaining $name that is not in macro_defs and panic with details
        let re = regex::Regex::new(r"\$([a-zA-Z][a-zA-Z0-9\-_]*)").unwrap();
        for (line_idx, line) in expanded.lines().enumerate() {
            for cap in re.captures_iter(line) {
                let macro_name = &cap[1];
                if !macro_defs.contains_key(macro_name) && !template_defs.contains_key(macro_name) {
                    panic!(
                        "Undefined macro or template '${}' encountered on line {}: \n  {}",
                        macro_name,
                        line_idx + 1,
                        line.trim()
                    );
                }
            }
        }
        Ok(expanded)
    }

    fn parse_macro_definition_line(line: &str) -> Result<Option<(String, String)>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 || parts[0] != "let" || parts[2] != "=" {
            return Ok(None);
        }
        
        let name = parts[1].to_string();
        let value = parts[3..].join(" ");
        
        Ok(Some((name, value)))
    }

    fn parse_template_definition_line(line: &str) -> Result<Option<(String, String)>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 || parts[0] != "template" || parts[2] != "=" {
            return Ok(None);
        }
        
        let name = parts[1].to_string();
        let value = parts[3..].join(" ");
        
        Ok(Some((name, value)))
    }

    fn expand_apply_expressions(input: &str, template_defs: &HashMap<String, (String, Vec<String>)>) -> Result<(String, bool), String> {
        let mut expanded = input.to_string();
        let mut new_expanded = expanded.clone();
        let mut apply_changed = false;
        
        let lines: Vec<&str> = input.lines().collect();
        let mut new_lines = Vec::new();
        
        for line in lines {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("apply ") {
                // Parse and expand apply expression
                let expanded_lines = Self::parse_and_expand_apply_line(trimmed_line, template_defs)?;
                new_lines.extend(expanded_lines);
            } else {
                new_lines.push(line.to_string());
            }
        }
        
        let new_expanded_apply = new_lines.join("\n");
        if new_expanded_apply != expanded {
            apply_changed = true;
        }
        
        Ok((new_expanded_apply, apply_changed))
    }

    fn parse_and_expand_apply_line(line: &str, template_defs: &HashMap<String, (String, Vec<String>)>) -> Result<Vec<String>, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 || parts[0] != "apply" {
            return Err("Invalid apply expression".to_string());
        }
        
        let mut args = Vec::new();
        let mut template_name = String::new();
        let mut i = 1;
        
        // Parse apply arguments
        while i < parts.len() {
            let part = parts[i];
            if part.starts_with("$") {
                // This is the template name
                template_name = part[1..].to_string();
                break;
            } else if part.contains(':') {
                // This is an apply argument
                let arg_parts: Vec<&str> = part.split(':').collect();
                if arg_parts.len() != 2 {
                    return Err(format!("Invalid apply argument format: {}", part));
                }
                let parameter = arg_parts[0].to_string();
                let values: Vec<String> = arg_parts[1].split(',').map(|s| s.to_string()).collect();
                args.push((parameter, values));
            } else {
                return Err(format!("Unexpected token in apply expression: {}", part));
            }
            i += 1;
        }
        
        if template_name.is_empty() {
            return Err("No template name found in apply expression".to_string());
        }
        
        // Get template definition
        let (template_expr, template_params) = template_defs.get(&template_name)
            .ok_or_else(|| format!("Template '{}' not found", template_name))?;
        
        // Generate all combinations of parameter values
        let mut expanded_lines = Vec::new();
        let combinations = Self::generate_parameter_combinations(&args)?;
        
        for combination in combinations {
            let mut expanded_expr = template_expr.clone();
            for (param, value) in combination {
                let pattern = format!("{{{}}}", param);
                expanded_expr = expanded_expr.replace(&pattern, &value);
            }
            expanded_lines.push(expanded_expr);
        }
        
        Ok(expanded_lines)
    }

    fn generate_parameter_combinations(args: &[(String, Vec<String>)]) -> Result<Vec<Vec<(String, String)>>, String> {
        if args.is_empty() {
            return Ok(vec![vec![]]);
        }
        
        let (param, values) = &args[0];
        let mut combinations = Vec::new();
        
        for value in values {
            let mut combination = vec![(param.clone(), value.clone())];
            combinations.push(combination);
        }
        
        // If there are more args, recursively combine them
        if args.len() > 1 {
            let sub_combinations = Self::generate_parameter_combinations(&args[1..])?;
            let mut new_combinations = Vec::new();
            
            for combination in combinations {
                for sub_combination in &sub_combinations {
                    let mut new_combination = combination.clone();
                    new_combination.extend(sub_combination.clone());
                    new_combinations.push(new_combination);
                }
            }
            combinations = new_combinations;
        }
        
        Ok(combinations)
    }

    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_comment = false;
        let mut in_file_path = false;
        let mut chars = input.chars().peekable();
        let mut at_line_start = true;
        let mut line_buffer = String::new();

        while let Some(ch) = chars.next() {
            // Buffer the line for blank line detection
            if ch == '\n' {
                if !in_comment {
                    // If the line is blank (only whitespace), skip it
                    if line_buffer.trim().is_empty() {
                        at_line_start = true;
                        line_buffer.clear();
                        continue;
                    }
                }
                at_line_start = true;
                line_buffer.clear();
            } else {
                line_buffer.push(ch);
                if !ch.is_whitespace() {
                    at_line_start = false;
                }
            }

            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
                continue;
            }

            if at_line_start && ch == '#' {
                in_comment = true;
                continue;
            }

            if in_file_path {
                if ch == ':' {
                    in_file_path = false;
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(":".to_string());
                } else {
                    current_token.push(ch);
                }
                continue;
            }

            // Detect start of file path after 'samp' and ':'
            if current_token == "samp" && chars.peek() == Some(&':') {
                tokens.push(current_token.clone());
                current_token.clear();
                chars.next(); // consume the ':'
                tokens.push(":".to_string());
                in_file_path = true;
                continue;
            }

            match ch {
                ':' | ',' | ' ' | '\n' | '\r' | '\t' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    if ch != ' ' && ch != '\n' && ch != '\r' && ch != '\t' {
                        tokens.push(ch.to_string());
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens.retain(|token| !token.is_empty());
        tokens
    }

    pub fn parse(&mut self) -> Result<TrackGrid<FixedTimeNoteSequence>, String> {
        let script = self.parse_script()?;
        self.build_track_grid(script)
    }

    fn parse_script(&mut self) -> Result<Script, String> {
        let mut macro_defs = HashMap::new();
        let mut template_defs = HashMap::new();
        let mut outer_blocks = Vec::new();
        
        // Parse macro definitions first
        while self.current < self.tokens.len() && self.peek() == "let" {
            let (name, expression) = self.parse_assignment()?;
            macro_defs.insert(name, expression);
        }
        
        // Parse template definitions
        while self.current < self.tokens.len() && self.peek() == "template" {
            let template_def = self.parse_template_definition()?;
            template_defs.insert(template_def.name.clone(), template_def);
        }
        
        // Parse outer blocks
        while self.current < self.tokens.len() {
            let block = self.parse_outer_block()?;
            outer_blocks.push(block);
        }

        Ok(Script { 
            macro_defs,
            template_defs,
            outer_blocks 
        })
    }

    fn parse_outer_block(&mut self) -> Result<OuterBlock, String> {
        let sequence_def = self.parse_sequence_def()?;
        let mut envelope_defs = Vec::new();
        let mut effect_defs = Vec::new();
        let mut note_declarations = Vec::new();

        // Parse optional envelope definitions
        while self.current < self.tokens.len() && self.peek() == "a" {
            let envelope_def = self.parse_envelope_def()?;
            envelope_defs.push(envelope_def);
        }

        // Parse optional effect definitions
        while self.current < self.tokens.len() && self.is_effect_start() {
            let effect_def = self.parse_effect_def()?;
            effect_defs.push(effect_def);
        }

        // Parse note declarations
        while self.current < self.tokens.len() && self.is_note_declaration_start() {
            let note_declaration = self.parse_note_declaration()?;
            note_declarations.push(note_declaration);
        }

        Ok(OuterBlock {
            sequence_def,
            envelope_defs,
            effect_defs,
            note_declarations,
        })
    }

    fn parse_sequence_def(&mut self) -> Result<SequenceDef, String> {
        self.expect("FixedTimeNoteSequence")?;
        self.expect("dur")?;
        let dur = self.parse_duration_type()?;
        self.expect("tempo")?;
        let tempo = self.parse_u8()?;
        self.expect("num_steps")?;
        let num_steps = self.parse_usize()?;

        Ok(SequenceDef {
            dur,
            tempo,
            num_steps,
        })
    }

    fn parse_duration_type(&mut self) -> Result<DslDurationType, String> {
        let token = self.advance();
        DslDurationType::from_str(&token)
    }

    fn parse_envelope_def(&mut self) -> Result<EnvelopeDef, String> {
        self.expect("a")?;
        let attack = self.parse_envelope_pair()?;
        self.expect("d")?;
        let decay = self.parse_envelope_pair()?;
        self.expect("s")?;
        let sustain = self.parse_envelope_pair()?;
        self.expect("r")?;
        let release = self.parse_envelope_pair()?;

        Ok(EnvelopeDef {
            attack,
            decay,
            sustain,
            release,
        })
    }

    fn parse_envelope_pair(&mut self) -> Result<(f32, f32), String> {
        let first = self.parse_f32()?;
        self.expect(",")?;
        let second = self.parse_f32()?;
        Ok((first, second))
    }

    fn parse_effect_def(&mut self) -> Result<EffectDef, String> {
        if self.peek() == "delay" {
            self.parse_delay_def()
        } else if self.peek() == "flanger" {
            self.parse_flanger_def()
        } else if self.peek() == "lfo" {
            self.parse_lfo_def()
        } else {
            Err(format!("Unknown effect type: {}", self.peek()))
        }
    }

    fn parse_delay_def(&mut self) -> Result<EffectDef, String> {
        self.expect("delay")?;
        self.expect("mix")?;
        let mix = self.parse_f32()?;
        self.expect("decay")?;
        let decay = self.parse_f32()?;
        self.expect("interval_ms")?;
        let interval_ms = self.parse_f32()?;
        self.expect("duration_ms")?;
        let duration_ms = self.parse_f32()?;
        self.expect("num_repeats")?;
        let num_repeats = self.parse_usize()?;
        self.expect("num_predelay_samples")?;
        let num_predelay_samples = self.parse_usize()?;
        self.expect("num_concurrent_delays")?;
        let num_concurrent_delays = self.parse_usize()?;

        Ok(EffectDef::Delay(DelayDef {
            mix,
            decay,
            interval_ms,
            duration_ms,
            num_repeats,
            num_predelay_samples,
            num_concurrent_delays,
        }))
    }

    fn parse_flanger_def(&mut self) -> Result<EffectDef, String> {
        self.expect("flanger")?;
        self.expect("window_size")?;
        let window_size = self.parse_usize()?;
        self.expect("mix")?;
        let mix = self.parse_f32()?;

        Ok(EffectDef::Flanger(FlangerDef {
            window_size,
            mix,
        }))
    }

    fn parse_lfo_def(&mut self) -> Result<EffectDef, String> {
        self.expect("lfo")?;
        self.expect("freq")?;
        let freq = self.parse_f32()?;
        self.expect("amp")?;
        let amp = self.parse_f32()?;
        self.expect("waveforms")?;
        let waveforms = self.parse_waveforms()?;

        Ok(EffectDef::LFO(LFODef {
            freq,
            amp,
            waveforms,
        }))
    }

    fn parse_waveforms(&mut self) -> Result<Vec<WaveformType>, String> {
        let mut waveforms = Vec::new();
        
        loop {
            let waveform = self.parse_waveform()?;
            waveforms.push(waveform);
            
            if self.peek() == "," {
                self.advance(); // consume comma
            } else {
                break;
            }
        }

        Ok(waveforms)
    }

    fn parse_waveform(&mut self) -> Result<WaveformType, String> {
        let token = self.advance();
        WaveformType::from_str(&token)
    }

    fn parse_note_declaration(&mut self) -> Result<NoteDeclaration, String> {
        if self.peek() == "osc" {
            self.parse_osc_note()
        } else if self.peek() == "samp" {
            self.parse_samp_note()
        } else {
            Err(format!("Unknown note type: {}", self.peek()))
        }
    }

    fn parse_osc_note(&mut self) -> Result<NoteDeclaration, String> {
        self.expect("osc")?;
        self.expect(":")?;
        let waveforms = self.parse_waveforms()?;
        self.expect(":")?;
        let note_freq = self.parse_note_freq()?;
        self.expect(":")?;
        let volume = self.parse_f32()?;
        self.expect(":")?;
        let step_index = self.parse_usize()?;

        Ok(NoteDeclaration::Oscillator {
            waveforms,
            note_freq,
            volume,
            step_index,
        })
    }

    fn parse_samp_note(&mut self) -> Result<NoteDeclaration, String> {
        self.expect("samp")?;
        self.expect(":")?;
        let file_path = self.parse_file_path()?;
        self.expect(":")?;
        let volume = self.parse_f32()?;
        self.expect(":")?;
        let step_index = self.parse_usize()?;

        Ok(NoteDeclaration::Sample {
            file_path,
            volume,
            step_index,
        })
    }

    fn parse_note_freq(&mut self) -> Result<f32, String> {
        let token = self.advance();
        
        // Try to parse as octave,western_pitch format first
        if let Ok(octave) = token.parse::<u8>() {
            if self.peek() == "," {
                self.advance(); // consume comma
                let pitch_token = self.advance();
                if let Ok(pitch) = WesternPitchType::from_str(&pitch_token) {
                    let western_pitch = pitch.to_western_pitch();
                    return Ok(western_pitch.get_frequency(octave));
                } else {
                    return Err(format!("Invalid western pitch: {}", pitch_token));
                }
            }
        }
        
        // Try to parse as western pitch (default octave 4)
        if let Ok(pitch) = WesternPitchType::from_str(&token) {
            let western_pitch = pitch.to_western_pitch();
            // Default to octave 4 (middle C)
            return Ok(western_pitch.get_frequency(4));
        }
        
        // Try to parse as float
        token.parse::<f32>().map_err(|_| format!("Invalid note frequency: {}", token))
    }

    fn parse_file_path(&mut self) -> Result<String, String> {
        let mut file_path = String::new();
        
        while self.current < self.tokens.len() && self.peek() != ":" {
            if !file_path.is_empty() {
                file_path.push(':');
            }
            file_path.push_str(&self.advance());
        }
        
        if file_path.is_empty() {
            Err("Empty file path".to_string())
        } else {
            Ok(file_path)
        }
    }

    fn is_effect_start(&self) -> bool {
        self.peek() == "delay" || self.peek() == "flanger" || self.peek() == "lfo"
    }

    fn is_note_declaration_start(&self) -> bool {
        self.peek() == "osc" || self.peek() == "samp"
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        let token = self.advance();
        if token == expected {
            Ok(())
        } else {
            Err(format!("Expected '{}', got '{}'", expected, token))
        }
    }

    fn advance(&mut self) -> String {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            token
        } else {
            String::new()
        }
    }

    fn peek(&self) -> &str {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else {
            ""
        }
    }

    fn parse_f32(&mut self) -> Result<f32, String> {
        let token = self.advance();
        token.parse::<f32>().map_err(|_| format!("Invalid float: {}", token))
    }

    fn parse_u8(&mut self) -> Result<u8, String> {
        let token = self.advance();
        token.parse::<u8>().map_err(|_| format!("Invalid u8: {}", token))
    }

    fn parse_usize(&mut self) -> Result<usize, String> {
        let token = self.advance();
        token.parse::<usize>().map_err(|_| format!("Invalid usize: {}", token))
    }

    fn build_track_grid(&self, script: Script) -> Result<TrackGrid<FixedTimeNoteSequence>, String> {
        let mut tracks = Vec::new();

        for block in script.outer_blocks {
            let track = self.build_track_from_block(block)?;
            tracks.push(track);
        }

        TrackGridBuilder::default()
            .tracks(tracks)
            .build()
            .map_err(|e| format!("Failed to build TrackGrid: {:?}", e))
    }

    fn build_track_from_block(&self, block: OuterBlock) -> Result<Track<FixedTimeNoteSequence>, String> {
        // Build FixedTimeNoteSequence
        let sequence = self.build_fixed_time_note_sequence(&block.sequence_def)?;
        
        // Build TrackEffects
        let track_effects = self.build_track_effects(&block.envelope_defs, &block.effect_defs)?;
        
        // Add notes to sequence
        let mut sequence_with_notes = sequence;
        for note_decl in &block.note_declarations {
            let playback_note = self.build_playback_note(note_decl, &block.sequence_def)?;
            sequence_with_notes.append_note(playback_note);
        }

        // Build Track
        TrackBuilder::default()
            .sequence(sequence_with_notes)
            .effects(track_effects)
            .build()
            .map_err(|e| format!("Failed to build Track: {:?}", e))
    }

    fn build_fixed_time_note_sequence(&self, sequence_def: &SequenceDef) -> Result<FixedTimeNoteSequence, String> {
        let duration_type = match sequence_def.dur {
            DslDurationType::Whole => MeterDurationType::Whole,
            DslDurationType::Half => MeterDurationType::Half,
            DslDurationType::Quarter => MeterDurationType::Quarter,
            DslDurationType::Eighth => MeterDurationType::Eighth,
            DslDurationType::Sixteenth => MeterDurationType::Sixteenth,
            DslDurationType::ThirtySecond => MeterDurationType::ThirtySecond,
            DslDurationType::SixtyFourth => MeterDurationType::SixtyFourth,
            DslDurationType::Fraction(_) => MeterDurationType::Quarter, // Default fallback
        };

        FixedTimeNoteSequenceBuilder::default()
            .duration_type(duration_type)
            .tempo(sequence_def.tempo)
            .num_steps(sequence_def.num_steps)
            .build()
            .map_err(|e| format!("Failed to build FixedTimeNoteSequence: {:?}", e))
    }

    fn build_track_effects(&self, envelope_defs: &[EnvelopeDef], effect_defs: &[EffectDef]) -> Result<TrackEffects, String> {
        let mut envelopes = Vec::new();
        let mut delays = Vec::new();
        let mut flangers = Vec::new();
        let mut lfos = Vec::new();

        // Build envelopes
        for env_def in envelope_defs {
            let envelope = EnvelopeBuilder::default()
                .attack(EnvelopePair(env_def.attack.0, env_def.attack.1))
                .decay(EnvelopePair(env_def.decay.0, env_def.decay.1))
                .sustain(EnvelopePair(env_def.sustain.0, env_def.sustain.1))
                .release(EnvelopePair(env_def.release.0, env_def.release.1))
                .build()
                .map_err(|e| format!("Failed to build Envelope: {:?}", e))?;
            envelopes.push(envelope);
        }

        // Build effects
        for effect_def in effect_defs {
            match effect_def {
                EffectDef::Delay(delay_def) => {
                    let delay = DelayBuilder::default()
                        .id(0) // Default ID
                        .mix(delay_def.mix)
                        .decay(delay_def.decay)
                        .interval_ms(delay_def.interval_ms)
                        .duration_ms(delay_def.duration_ms)
                        .num_repeats(delay_def.num_repeats)
                        .num_predelay_samples(delay_def.num_predelay_samples)
                        .num_concurrent_sample_managers(delay_def.num_concurrent_delays)
                        .build()
                        .map_err(|e| format!("Failed to build Delay: {:?}", e))?;
                    delays.push(delay);
                }
                EffectDef::Flanger(flanger_def) => {
                    let flanger = FlangerBuilder::default()
                        .window_size(flanger_def.window_size)
                        .mix(flanger_def.mix)
                        .build()
                        .map_err(|e| format!("Failed to build Flanger: {:?}", e))?;
                    flangers.push(flanger);
                }
                EffectDef::LFO(lfo_def) => {
                    let waveforms: Vec<Waveform> = lfo_def.waveforms.iter()
                        .map(|w| w.to_waveform())
                        .collect();
                    let lfo = LFOBuilder::default()
                        .frequency(lfo_def.freq)
                        .amplitude(lfo_def.amp)
                        .waveforms(waveforms)
                        .build()
                        .map_err(|e| format!("Failed to build LFO: {:?}", e))?;
                    lfos.push(lfo);
                }
            }
        }

        TrackEffectsBuilder::default()
            .envelopes(envelopes)
            .delays(delays)
            .flangers(flangers)
            .lfos(lfos)
            .build()
            .map_err(|e| format!("Failed to build TrackEffects: {:?}", e))
    }

    fn build_playback_note(&self, note_decl: &NoteDeclaration, sequence_def: &SequenceDef) -> Result<PlaybackNote, String> {
        let step_duration_ms = (60000.0 / sequence_def.tempo as f32) * sequence_def.dur.to_factor();
        let start_time_ms = note_decl.get_step_index() as f32 * step_duration_ms;
        let end_time_ms = start_time_ms + step_duration_ms;

        match note_decl {
            NoteDeclaration::Oscillator { waveforms, note_freq, volume, .. } => {
                let waveforms: Vec<Waveform> = waveforms.iter()
                    .map(|w| w.to_waveform())
                    .collect();

                let note = NoteBuilder::default()
                    .frequency(*note_freq)
                    .volume(*volume)
                    .start_time_ms(start_time_ms)
                    .end_time_ms(end_time_ms)
                    .waveforms(waveforms)
                    .build()
                    .map_err(|e| format!("Failed to build Note: {:?}", e))?;

                PlaybackNoteBuilder::default()
                    .note_type(NoteType::Oscillator)
                    .note(note)
                    .playback_start_time_ms(start_time_ms)
                    .playback_end_time_ms(end_time_ms)
                    .build()
                    .map_err(|e| format!("Failed to build PlaybackNote: {:?}", e))
            }
            NoteDeclaration::Sample { file_path, volume, .. } => {
                let sampled_note = SampledNoteBuilder::default()
                    .file_path(file_path.clone())
                    .volume(*volume)
                    .start_time_ms(start_time_ms)
                    .end_time_ms(end_time_ms)
                    .build()
                    .map_err(|e| format!("Failed to build SampledNote: {:?}", e))?;

                PlaybackNoteBuilder::default()
                    .note_type(NoteType::Sample)
                    .sampled_note(sampled_note)
                    .playback_start_time_ms(start_time_ms)
                    .playback_end_time_ms(end_time_ms)
                    .build()
                    .map_err(|e| format!("Failed to build PlaybackNote: {:?}", e))
            }
        }
    }

    fn parse_assignment(&mut self) -> Result<(String, String), String> {
        self.expect("let")?;
        let name = self.parse_identifier()?;
        self.expect("=")?;
        let expression = self.parse_expression()?;
        Ok((name, expression))
    }

    fn parse_template_definition(&mut self) -> Result<TemplateDef, String> {
        self.expect("template")?;
        let name = self.parse_identifier()?;
        self.expect("=")?;
        let expression = self.parse_expression()?;
        
        // Extract parameters from the expression (look for {param} patterns)
        let parameters = Self::extract_template_parameters(&expression);
        
        Ok(TemplateDef {
            name,
            expression,
            parameters,
        })
    }

    fn extract_template_parameters(expression: &str) -> Vec<String> {
        let mut parameters = Vec::new();
        let re = regex::Regex::new(r"\{([a-zA-Z][a-zA-Z0-9\-_]*)\}").unwrap();
        
        for cap in re.captures_iter(expression) {
            let param_name = cap[1].to_string();
            if !parameters.contains(&param_name) {
                parameters.push(param_name);
            }
        }
        
        parameters
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let token = self.advance();
        if token.chars().next().map_or(false, |c| c.is_alphabetic()) &&
           token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            Ok(token)
        } else {
            Err(format!("Invalid identifier: {}", token))
        }
    }

    fn parse_expression(&mut self) -> Result<String, String> {
        let mut expression_tokens = Vec::new();
        
        // Parse until we reach the end of the line or encounter another 'let'
        while self.current < self.tokens.len() {
            let token = self.peek();
            
            // Stop if we encounter another 'let' (start of next macro definition)
            if token == "let" {
                break;
            }
            
            // Stop if we encounter 'FixedTimeNoteSequence' (start of outer block)
            if token == "FixedTimeNoteSequence" {
                break;
            }
            
            expression_tokens.push(self.advance());
        }
        
        if expression_tokens.is_empty() {
            return Err("Empty expression".to_string());
        }
        
        // Reconstruct the original text by joining tokens intelligently
        let mut expression = String::new();
        for (i, token) in expression_tokens.iter().enumerate() {
            if i > 0 {
                // Add space before token, except for certain punctuation
                let prev = &expression_tokens[i - 1];
                if token != "," && token != ":" && prev != "," && prev != ":" {
                    expression.push(' ');
                }
            }
            expression.push_str(token);
        }
        
        Ok(expression.trim().to_string())
    }
}

impl NoteDeclaration {
    fn get_step_index(&self) -> usize {
        match self {
            NoteDeclaration::Oscillator { step_index, .. } => *step_index,
            NoteDeclaration::Sample { step_index, .. } => *step_index,
        }
    }
}

pub fn parse_dsl(input: &str) -> Result<TrackGrid<FixedTimeNoteSequence>, String> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_script() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            a 0.1,0.8 d 0.3,0.6 s 0.8,0.4 r 1.0,0.0
            delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            osc:sine:440.0:0.5:0
            osc:square:880.0:0.3:4
            samp:/Users/markweiss/RustroverProjects/osc_bak/src/dsl/test_data/test_sample.wav:0.005:2
        "#;

        let result = parse_dsl(input);
        if let Err(e) = &result {
            println!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_parse_multiple_blocks() {
        let input = r#"
            FixedTimeNoteSequence dur Eighth tempo 140 num_steps 8
            a 0.05,0.9 d 0.2,0.7 s 0.9,0.5 r 1.0,0.0
            osc:sine:220.0:0.4:0
            
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            flanger window_size 8 mix 0.3
            samp:/Users/markweiss/RustroverProjects/osc_bak/src/dsl/test_data/test_sample.wav:0.005:2
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 2);
    }

    #[test]
    fn test_parse_western_pitch() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:C:0.5:0
            osc:triangle:F#:0.3:4
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_octave_western_pitch() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:4,C:0.5:0
            osc:triangle:5,F#:0.3:4
            osc:square:3,A:0.7:8
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complex_effects() {
        let input = r#"
            FixedTimeNoteSequence dur Half tempo 100 num_steps 32
            a 0.1,0.9 d 0.4,0.6 s 0.8,0.3 r 1.0,0.0
            delay mix 0.8 decay 0.6 interval_ms 80.0 duration_ms 40.0 num_repeats 5 num_predelay_samples 15 num_concurrent_delays 3
            flanger window_size 12 mix 0.4
            lfo freq 2.5 amp 0.3 waveforms sine,triangle
            osc:sine,square:440.0:0.7:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        let track = &track_grid.tracks[0];
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
        assert_eq!(track.effects.flangers.len(), 1);
        assert_eq!(track.effects.lfos.len(), 1);
    }

    #[test]
    fn test_parse_macro_definitions() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            let flanger1 = flanger window_size 8 mix 0.3
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:440.0:0.5:0
        "#;

        let mut parser = Parser::new(input);
        let script = parser.parse_script().unwrap();
        
        // Verify macro definitions are parsed correctly
        assert_eq!(script.macro_defs.len(), 3);
        
        // Check that env1 contains the envelope definition
        assert_eq!(script.macro_defs.get("env1").unwrap(), "a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0");
        
        // Check that delay1 contains the delay definition
        assert_eq!(script.macro_defs.get("delay1").unwrap(), "delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2");
        
        // Check that flanger1 contains the flanger definition
        assert_eq!(script.macro_defs.get("flanger1").unwrap(), "flanger window_size 8 mix 0.3");
        
        // Verify that outer blocks are still parsed correctly
        assert_eq!(script.outer_blocks.len(), 1);
    }

    #[test]
    fn test_parse_macro_definitions_with_whitespace() {
        let input = r#"
            let env1 =   a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0   
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:440.0:0.5:0
        "#;

        let mut parser = Parser::new(input);
        let script = parser.parse_script().unwrap();
        
        // Verify that whitespace is trimmed from expressions
        assert_eq!(script.macro_defs.get("env1").unwrap(), "a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0");
        assert_eq!(script.macro_defs.get("delay1").unwrap(), "delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2");
    }

    #[test]
    fn test_parse_multiple_macro_definitions() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let env2 = a 0.1,0.9 d 0.2,0.7 s 0.9,0.4 r 0.8,0.1
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            let lfo1 = lfo freq 2.0 amp 0.3 waveforms sine,triangle
            let note1 = osc:sine:440.0:0.5:0
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:880.0:0.3:4
        "#;

        let mut parser = Parser::new(input);
        let script = parser.parse_script().unwrap();
        
        // Verify all macro definitions are parsed
        assert_eq!(script.macro_defs.len(), 5);
        
        // Check each macro definition
        assert_eq!(script.macro_defs.get("env1").unwrap(), "a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0");
        assert_eq!(script.macro_defs.get("env2").unwrap(), "a 0.1,0.9 d 0.2,0.7 s 0.9,0.4 r 0.8,0.1");
        assert_eq!(script.macro_defs.get("delay1").unwrap(), "delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2");
        assert_eq!(script.macro_defs.get("lfo1").unwrap(), "lfo freq 2.0 amp 0.3 waveforms sine,triangle");
        assert_eq!(script.macro_defs.get("note1").unwrap(), "osc:sine:440.0:0.5:0");
        
        // Verify that outer blocks are still parsed correctly
        assert_eq!(script.outer_blocks.len(), 1);
    }

    #[test]
    fn test_parse_identifier_validation() {
        let input = r#"
            let valid-name = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let valid_name = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            let validName123 = flanger window_size 8 mix 0.3
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:440.0:0.5:0
        "#;

        let mut parser = Parser::new(input);
        let script = parser.parse_script().unwrap();
        
        // Verify that valid identifiers with hyphens, underscores, and numbers are accepted
        assert_eq!(script.macro_defs.len(), 3);
        assert!(script.macro_defs.contains_key("valid-name"));
        assert!(script.macro_defs.contains_key("valid_name"));
        assert!(script.macro_defs.contains_key("validName123"));
    }

    #[test]
    fn test_macro_expansion_basic() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            $delay1
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope and one delay from the expanded macros
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_macro_expansion_multiple_uses() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            let flanger1 = flanger window_size 8 mix 0.3
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            $flanger1
            osc:sine:440.0:0.5:0
            
            FixedTimeNoteSequence dur Eighth tempo 140 num_steps 8
            $env1
            $flanger1
            osc:square:880.0:0.3:4
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 2);
        
        // Both tracks should have the same envelope and flanger from expanded macros
        for track in &track_grid.tracks {
            assert_eq!(track.effects.envelopes.len(), 1);
            assert_eq!(track.effects.flangers.len(), 1);
        }
    }

    #[test]
    fn test_macro_expansion_nested() {
        let input = r#"
            let base_env = a 0.1,0.9 d 0.2,0.7 s 0.9,0.4 r 0.8,0.1
            let env1 = $base_env
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope from the expanded macro
        assert_eq!(track.effects.envelopes.len(), 1);
    }

    #[test]
    fn test_macro_expansion_in_sequence() {
        let input = r#"
            let my_tempo = 140
            let my_steps = 8
            FixedTimeNoteSequence dur Quarter tempo $my_tempo num_steps $my_steps
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // The sequence should have been expanded with the macro values
        assert_eq!(track.sequence.tempo, 140);
    }

    #[test]
    #[should_panic(expected = "Undefined macro '$undefined_macro' encountered on line 3: \n  $undefined_macro")]
    fn test_macro_expansion_undefined() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $undefined_macro
            osc:sine:440.0:0.5:0
        "#;

        // The panic happens inside Parser::new, which is called by parse_dsl.
        // We don't need to check the result, just confirm that the call panics.
        let _ = parse_dsl(input);
    }

    #[test]
    fn test_macro_expansion_complex() {
        let input = r#"
            let attack_params = 0.2,0.8
            let decay_params = 0.3,0.6
            let sustain_params = 0.8,0.5
            let release_params = 1.0,0.0
            let env1 = a $attack_params d $decay_params s $sustain_params r $release_params
            let delay1 = delay mix 0.5 decay 0.7 interval_ms 100.0 duration_ms 50.0 num_repeats 3 num_predelay_samples 10 num_concurrent_delays 2
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            $delay1
            osc:sine:440.0:0.5:0
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope and one delay from the expanded macros
        assert_eq!(track.effects.envelopes.len(), 1);
        assert_eq!(track.effects.delays.len(), 1);
    }

    #[test]
    fn test_template_definition() {
        let input = r#"
            template osc_template1 = osc:sine:440.0:0.9:{step}
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:880.0:0.3:4
        "#;

        let mut parser = Parser::new(input);
        let script = parser.parse_script().unwrap();
        
        // Verify template definition is parsed correctly
        assert_eq!(script.template_defs.len(), 1);
        
        let template = script.template_defs.get("osc_template1").unwrap();
        assert_eq!(template.expression, "osc:sine:440.0:0.9:{step}");
        assert_eq!(template.parameters, vec!["step"]);
        
        // Verify that outer blocks are still parsed correctly
        assert_eq!(script.outer_blocks.len(), 1);
    }

    #[test]
    fn test_template_with_multiple_parameters() {
        let input = r#"
            template samp_template1 = samp:/path/to/file.wav:{volume}:{step}
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            osc:sine:880.0:0.3:4
        "#;

        let mut parser = Parser::new(input);
        let script = parser.parse_script().unwrap();
        
        // Verify template definition is parsed correctly
        assert_eq!(script.template_defs.len(), 1);
        
        let template = script.template_defs.get("samp_template1").unwrap();
        assert_eq!(template.expression, "samp:/path/to/file.wav:{volume}:{step}");
        assert_eq!(template.parameters, vec!["volume", "step"]);
    }

    #[test]
    fn test_apply_expression_basic() {
        let input = r#"
            template osc_template1 = osc:sine:440.0:0.9:{step}
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            apply step:0,8 $osc_template1
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have two notes from the expanded template
        // We can't directly access the notes, but we can verify the track was created successfully
        assert_eq!(track.sequence.tempo, 120);
    }

    #[test]
    fn test_apply_expression_multiple_parameters() {
        let input = r#"
            template samp_template1 = samp:/Users/markweiss/RustroverProjects/osc_bak/src/dsl/test_data/test_sample.wav:{volume}:{step}
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            apply volume:0.5,0.8 step:0,4 $samp_template1
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have four notes from the expanded template (2 volumes × 2 steps)
        // We can't directly access the notes, but we can verify the track was created successfully
        assert_eq!(track.sequence.tempo, 120);
    }

    #[test]
    fn test_template_and_macro_together() {
        let input = r#"
            let env1 = a 0.2,0.8 d 0.3,0.6 s 0.8,0.5 r 1.0,0.0
            template osc_template1 = osc:sine:440.0:0.9:{step}
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            $env1
            apply step:0,8 $osc_template1
        "#;

        let result = parse_dsl(input);
        assert!(result.is_ok());
        
        let track_grid = result.unwrap();
        assert_eq!(track_grid.tracks.len(), 1);
        
        let track = &track_grid.tracks[0];
        // Should have one envelope and two notes
        assert_eq!(track.effects.envelopes.len(), 1);
        // We can't directly access the notes, but we can verify the track was created successfully
        assert_eq!(track.sequence.tempo, 120);
    }

    #[test]
    #[should_panic(expected = "Template 'undefined_template' not found")]
    fn test_apply_undefined_template() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            apply step:0,8 $undefined_template
        "#;

        // The panic happens inside Parser::new, which is called by parse_dsl.
        let _ = parse_dsl(input);
    }

    #[test]
    fn test_debug_undefined_template() {
        let input = r#"
            FixedTimeNoteSequence dur Quarter tempo 120 num_steps 16
            apply step:0,8 $undefined_template
        "#;

        // Let's see what happens during expansion
        let result = Parser::expand_macros(input);
        println!("Expand result: {:?}", result);
        
        // The panic should happen here
        let _ = parse_dsl(input);
    }
} 