//! Melodic pattern and scale generation

use rand::prelude::*;

/// Musical scale types
#[derive(Debug, Clone, Copy)]
pub enum Scale {
    MinorPentatonic,
    MajorPentatonic,
    NaturalMinor,
    Dorian,
}

impl Scale {
    /// Get intervals from root (in semitones)
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            Scale::MinorPentatonic => &[0, 3, 5, 7, 10],
            Scale::MajorPentatonic => &[0, 2, 4, 7, 9],
            Scale::NaturalMinor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
        }
    }

    /// Get notes in scale starting from root note
    pub fn notes_from(&self, root: u8, octaves: u8) -> Vec<u8> {
        let intervals = self.intervals();
        let mut notes = Vec::new();

        for octave in 0..octaves {
            for &interval in intervals {
                notes.push(root + octave * 12 + interval);
            }
        }

        notes
    }
}

/// A note event with timing
#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: u8,      // MIDI note number
    pub duration: f32,  // in beats
    pub velocity: f32,  // 0.0-1.0
}

impl Note {
    pub fn new(pitch: u8, duration: f32, velocity: f32) -> Self {
        Self { pitch, duration, velocity }
    }

    pub fn rest(duration: f32) -> Self {
        Self { pitch: 0, duration, velocity: 0.0 }
    }

    pub fn is_rest(&self) -> bool {
        self.velocity == 0.0
    }
}

/// Pattern types for melodic generation
#[derive(Debug, Clone, Copy)]
pub enum PatternType {
    Arpeggio,
    StepUp,
    StepDown,
    Random,
    Pendulum,
}

/// Generates melodic patterns
pub struct MelodyGenerator {
    rng: StdRng,
    scale: Scale,
    root: u8,
    available_notes: Vec<u8>,
}

impl MelodyGenerator {
    pub fn new(seed: u64, scale: Scale, root: u8) -> Self {
        let available_notes = scale.notes_from(root, 2);
        Self {
            rng: StdRng::seed_from_u64(seed),
            scale,
            root,
            available_notes,
        }
    }

    /// Generate a melodic phrase
    pub fn generate_phrase(&mut self, length: usize, pattern: PatternType) -> Vec<Note> {
        match pattern {
            PatternType::Arpeggio => self.generate_arpeggio(length),
            PatternType::StepUp => self.generate_steps(length, true),
            PatternType::StepDown => self.generate_steps(length, false),
            PatternType::Random => self.generate_random(length),
            PatternType::Pendulum => self.generate_pendulum(length),
        }
    }

    fn generate_arpeggio(&mut self, length: usize) -> Vec<Note> {
        let mut notes = Vec::with_capacity(length);
        let pattern_len = self.available_notes.len().min(4);

        for i in 0..length {
            let idx = i % pattern_len;
            let pitch = self.available_notes[idx];
            let duration = self.random_duration();
            notes.push(Note::new(pitch, duration, 0.8));
        }

        notes
    }

    fn generate_steps(&mut self, length: usize, ascending: bool) -> Vec<Note> {
        let mut notes = Vec::with_capacity(length);
        let start_idx = if ascending { 0 } else { self.available_notes.len() - 1 };

        for i in 0..length {
            let idx = if ascending {
                (start_idx + i) % self.available_notes.len()
            } else {
                (start_idx + self.available_notes.len() - (i % self.available_notes.len()))
                    % self.available_notes.len()
            };
            let pitch = self.available_notes[idx];
            let duration = self.random_duration();
            notes.push(Note::new(pitch, duration, 0.8));
        }

        notes
    }

    fn generate_random(&mut self, length: usize) -> Vec<Note> {
        let mut notes = Vec::with_capacity(length);

        for _ in 0..length {
            let pitch = *self.available_notes.choose(&mut self.rng).unwrap();
            let duration = self.random_duration();
            let velocity = self.rng.gen_range(0.6..0.9);
            notes.push(Note::new(pitch, duration, velocity));
        }

        notes
    }

    fn generate_pendulum(&mut self, length: usize) -> Vec<Note> {
        let mut notes = Vec::with_capacity(length);
        let mut going_up = true;
        let mut current_idx = 0;

        for _ in 0..length {
            let pitch = self.available_notes[current_idx];
            let duration = self.random_duration();
            notes.push(Note::new(pitch, duration, 0.8));

            if going_up {
                if current_idx < self.available_notes.len() - 1 {
                    current_idx += 1;
                } else {
                    going_up = false;
                    current_idx = current_idx.saturating_sub(1);
                }
            } else {
                if current_idx > 0 {
                    current_idx -= 1;
                } else {
                    going_up = true;
                    current_idx += 1;
                }
            }
        }

        notes
    }

    fn random_duration(&mut self) -> f32 {
        // Common note durations in beats
        let durations = [0.25, 0.5, 0.5, 1.0, 1.0];
        *durations.choose(&mut self.rng).unwrap()
    }

    /// Generate a bass pattern (simpler, longer notes)
    pub fn generate_bass(&mut self, bars: usize, beats_per_bar: usize) -> Vec<Note> {
        let mut notes = Vec::new();
        let bass_notes: Vec<u8> = self.available_notes.iter()
            .filter(|&&n| n < self.root + 12)
            .copied()
            .collect();

        for _ in 0..bars {
            // One or two notes per bar for bass
            let notes_in_bar = self.rng.gen_range(1..=2);
            let beat_duration = beats_per_bar as f32 / notes_in_bar as f32;

            for _ in 0..notes_in_bar {
                let pitch = *bass_notes.choose(&mut self.rng).unwrap_or(&self.root);
                notes.push(Note::new(pitch, beat_duration, 0.9));
            }
        }

        notes
    }

    /// Mutate a phrase for evolution
    pub fn mutate_phrase(&mut self, phrase: &[Note], mutation_chance: f32) -> Vec<Note> {
        phrase.iter().map(|note| {
            if self.rng.gen::<f32>() < mutation_chance {
                // Mutate this note
                let new_pitch = *self.available_notes.choose(&mut self.rng).unwrap();
                Note::new(new_pitch, note.duration, note.velocity)
            } else {
                *note
            }
        }).collect()
    }
}

/// Common root notes for keys
pub mod keys {
    pub const C: u8 = 48;  // C3
    pub const D: u8 = 50;
    pub const E: u8 = 52;
    pub const F: u8 = 53;
    pub const G: u8 = 55;
    pub const A: u8 = 57;
    pub const B: u8 = 59;
}
