//! Procedural music composition engine

use std::time::Duration;
use rand::prelude::*;
use rodio::Source;

use super::synth::{Oscillator, Waveform, Envelope, midi_to_freq, SAMPLE_RATE};
use super::melody::{MelodyGenerator, Scale, PatternType, Note, keys};

/// A voice playing a single note with envelope
struct Voice {
    oscillator: Oscillator,
    envelope: Envelope,
    note_start_sample: u64,
    note_duration_samples: u64,
    active: bool,
}

impl Voice {
    fn new(waveform: Waveform, envelope: Envelope) -> Self {
        Self {
            oscillator: Oscillator::new(waveform, 440.0, 0.3),
            envelope,
            note_start_sample: 0,
            note_duration_samples: 0,
            active: false,
        }
    }

    fn play_note(&mut self, pitch: u8, duration_samples: u64, current_sample: u64, velocity: f32) {
        self.oscillator.set_frequency(midi_to_freq(pitch));
        self.oscillator.amplitude = velocity * 0.3;
        self.note_start_sample = current_sample;
        self.note_duration_samples = duration_samples;
        self.active = true;
    }

    fn sample(&mut self, current_sample: u64) -> f32 {
        if !self.active {
            return 0.0;
        }

        let time_in_note = (current_sample - self.note_start_sample) as f32 / SAMPLE_RATE as f32;
        let note_duration = self.note_duration_samples as f32 / SAMPLE_RATE as f32;

        let env_value = self.envelope.value_at(time_in_note, note_duration);

        if env_value <= 0.0 && time_in_note > note_duration + self.envelope.release {
            self.active = false;
            return 0.0;
        }

        self.oscillator.next_sample() * env_value
    }
}

/// Track holding a sequence of notes
struct Track {
    voice: Voice,
    notes: Vec<Note>,
    current_note_idx: usize,
    samples_until_next: u64,
    samples_per_beat: u64,
}

impl Track {
    fn new(waveform: Waveform, envelope: Envelope, notes: Vec<Note>, bpm: f32) -> Self {
        let samples_per_beat = (SAMPLE_RATE as f32 * 60.0 / bpm) as u64;
        Self {
            voice: Voice::new(waveform, envelope),
            notes,
            current_note_idx: 0,
            samples_until_next: 0,
            samples_per_beat,
        }
    }

    fn update_notes(&mut self, notes: Vec<Note>) {
        self.notes = notes;
        self.current_note_idx = 0;
    }

    fn sample(&mut self, current_sample: u64) -> f32 {
        // Check if it's time for the next note
        if self.samples_until_next == 0 && !self.notes.is_empty() {
            let note = &self.notes[self.current_note_idx];

            if !note.is_rest() {
                let duration_samples = (note.duration * self.samples_per_beat as f32) as u64;
                self.voice.play_note(note.pitch, duration_samples, current_sample, note.velocity);
            }

            self.samples_until_next = (note.duration * self.samples_per_beat as f32) as u64;
            self.current_note_idx = (self.current_note_idx + 1) % self.notes.len();
        }

        if self.samples_until_next > 0 {
            self.samples_until_next -= 1;
        }

        self.voice.sample(current_sample)
    }
}

/// The main procedural music generator
pub struct ProceduralMusic {
    melody_track: Track,
    bass_track: Track,
    melody_gen: MelodyGenerator,
    current_sample: u64,
    bpm: f32,
    bars_played: u32,
    bars_until_evolution: u32,
    rng: StdRng,
}

impl ProceduralMusic {
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Random BPM between 100-130
        let bpm = rng.gen_range(100.0..130.0);

        // Random key
        let root_notes = [keys::C, keys::D, keys::E, keys::G, keys::A];
        let root = *root_notes.choose(&mut rng).unwrap();

        // Random scale (favor pentatonic for safer melodies)
        let scales = [Scale::MinorPentatonic, Scale::MinorPentatonic, Scale::MajorPentatonic, Scale::Dorian];
        let scale = *scales.choose(&mut rng).unwrap();

        let mut melody_gen = MelodyGenerator::new(seed, scale, root);

        // Generate initial patterns
        let pattern_types = [PatternType::Arpeggio, PatternType::Pendulum, PatternType::Random];
        let pattern = *pattern_types.choose(&mut rng).unwrap();
        let melody_notes = melody_gen.generate_phrase(8, pattern);
        let bass_notes = melody_gen.generate_bass(4, 4);

        let melody_track = Track::new(
            Waveform::Square,
            Envelope::plucky(),
            melody_notes,
            bpm,
        );

        let bass_track = Track::new(
            Waveform::Triangle,
            Envelope::pad(),
            bass_notes,
            bpm,
        );

        Self {
            melody_track,
            bass_track,
            melody_gen,
            current_sample: 0,
            bpm,
            bars_played: 0,
            bars_until_evolution: 4,
            rng,
        }
    }

    fn evolve(&mut self) {
        // Mutate melody
        let current_melody: Vec<Note> = self.melody_track.notes.clone();
        let mutated = self.melody_gen.mutate_phrase(&current_melody, 0.3);
        self.melody_track.update_notes(mutated);

        // Occasionally regenerate bass
        if self.rng.gen_bool(0.3) {
            let new_bass = self.melody_gen.generate_bass(4, 4);
            self.bass_track.update_notes(new_bass);
        }

        // Occasionally change pattern type entirely
        if self.rng.gen_bool(0.15) {
            let pattern_types = [PatternType::Arpeggio, PatternType::Pendulum, PatternType::Random, PatternType::StepUp];
            let pattern = *pattern_types.choose(&mut self.rng).unwrap();
            let new_melody = self.melody_gen.generate_phrase(8, pattern);
            self.melody_track.update_notes(new_melody);
        }

        self.bars_until_evolution = self.rng.gen_range(4..8);
    }

    fn check_evolution(&mut self) {
        let samples_per_bar = (SAMPLE_RATE as f32 * 60.0 / self.bpm * 4.0) as u64;
        let current_bar = (self.current_sample / samples_per_bar) as u32;

        if current_bar > self.bars_played {
            self.bars_played = current_bar;

            if self.bars_played % self.bars_until_evolution == 0 {
                self.evolve();
            }
        }
    }
}

impl Iterator for ProceduralMusic {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.check_evolution();

        let melody = self.melody_track.sample(self.current_sample);
        let bass = self.bass_track.sample(self.current_sample);

        self.current_sample += 1;

        // Mix and apply master volume
        let mixed = (melody + bass * 0.8) * 0.5;
        Some(mixed.clamp(-1.0, 1.0))
    }
}

impl Source for ProceduralMusic {
    fn current_frame_len(&self) -> Option<usize> {
        None // Infinite stream
    }

    fn channels(&self) -> u16 {
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Infinite
    }
}
