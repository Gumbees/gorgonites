//! Procedural music composition engine

use std::time::Duration;
use std::collections::VecDeque;
use rand::prelude::*;
use rodio::Source;

use super::synth::{Oscillator, Waveform, Envelope, midi_to_freq, SAMPLE_RATE};
use super::melody::{MelodyGenerator, Scale, PatternType, Note, keys};
use crate::config::AudioConfig;

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
    volume: f32,
    enabled: bool,
}

impl Track {
    fn new(waveform: Waveform, envelope: Envelope, notes: Vec<Note>, bpm: f32, volume: f32, enabled: bool) -> Self {
        let samples_per_beat = (SAMPLE_RATE as f32 * 60.0 / bpm) as u64;
        Self {
            voice: Voice::new(waveform, envelope),
            notes,
            current_note_idx: 0,
            samples_until_next: 0,
            samples_per_beat,
            volume,
            enabled,
        }
    }

    fn update_notes(&mut self, notes: Vec<Note>) {
        self.notes = notes;
        self.current_note_idx = 0;
    }

    /// Get the current note's pitch (for syncing other tracks)
    fn current_pitch(&self) -> Option<u8> {
        if self.notes.is_empty() {
            return None;
        }
        let note = &self.notes[self.current_note_idx];
        if note.is_rest() { None } else { Some(note.pitch) }
    }

    /// Check if we just started a new note this sample
    fn just_triggered(&self) -> bool {
        self.samples_until_next == 0
    }

    fn sample(&mut self, current_sample: u64) -> f32 {
        if !self.enabled || self.notes.is_empty() {
            return 0.0;
        }

        // Check if it's time for the next note
        if self.samples_until_next == 0 {
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

        self.voice.sample(current_sample) * self.volume
    }
}

/// Simple delay/echo effect for reverb-like sound
struct DelayEffect {
    buffer: VecDeque<f32>,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl DelayEffect {
    fn new(delay_ms: f32, feedback: f32, mix: f32) -> Self {
        let delay_samples = (SAMPLE_RATE as f32 * delay_ms / 1000.0) as usize;
        let mut buffer = VecDeque::with_capacity(delay_samples);
        for _ in 0..delay_samples {
            buffer.push_back(0.0);
        }
        Self {
            buffer,
            delay_samples,
            feedback,
            mix,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer.pop_front().unwrap_or(0.0);
        let output = input + delayed * self.feedback;
        self.buffer.push_back(output);

        // Mix dry and wet
        input * (1.0 - self.mix) + delayed * self.mix
    }
}

/// Synced highs voice that plays along with melody
struct SyncedHighs {
    voice: Voice,
    octave_offset: u8,
    volume: f32,
    enabled: bool,
}

impl SyncedHighs {
    fn new(octave_offset: u8, volume: f32, enabled: bool) -> Self {
        Self {
            voice: Voice::new(Waveform::Square, Envelope::sparkle()),
            octave_offset,
            volume,
            enabled,
        }
    }

    /// Trigger highs when melody triggers
    fn trigger(&mut self, melody_pitch: u8, duration_samples: u64, current_sample: u64, velocity: f32) {
        if !self.enabled {
            return;
        }
        let high_pitch = melody_pitch.saturating_add(self.octave_offset);
        // Louder highs with slightly longer duration for more presence
        self.voice.play_note(high_pitch, duration_samples * 2 / 3, current_sample, velocity * 0.75);
    }

    fn sample(&mut self, current_sample: u64) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        self.voice.sample(current_sample) * self.volume
    }
}

/// Pad voice for atmospheric sustained notes
struct PadVoice {
    voice: Voice,
    current_root: u8,
    samples_until_change: u64,
    samples_per_bar: u64,
    volume: f32,
    enabled: bool,
}

impl PadVoice {
    fn new(bpm: f32, volume: f32, enabled: bool) -> Self {
        let samples_per_bar = (SAMPLE_RATE as f32 * 60.0 / bpm * 4.0) as u64;
        Self {
            voice: Voice::new(Waveform::Triangle, Envelope::atmospheric()),
            current_root: 48,
            samples_until_change: 0,
            samples_per_bar,
            volume,
            enabled,
        }
    }

    fn update_root(&mut self, root: u8, current_sample: u64) {
        if self.samples_until_change == 0 {
            // Play a long sustained note based on root
            self.voice.play_note(root, self.samples_per_bar * 2, current_sample, 0.6);
            self.samples_until_change = self.samples_per_bar * 2;
            self.current_root = root;
        }

        if self.samples_until_change > 0 {
            self.samples_until_change -= 1;
        }
    }

    fn sample(&mut self, current_sample: u64) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        self.voice.sample(current_sample) * self.volume
    }
}

/// The main procedural music generator
pub struct ProceduralMusic {
    melody_track: Track,
    bass_track: Track,
    synced_highs: SyncedHighs,
    pad: PadVoice,
    delay: DelayEffect,
    melody_gen: MelodyGenerator,
    root_note: u8,
    current_sample: u64,
    bpm: f32,
    bars_played: u32,
    bars_until_evolution: u32,
    evolution_min_bars: u32,
    evolution_max_bars: u32,
    mutation_chance: f32,
    master_volume: f32,
    rng: StdRng,
}

impl ProceduralMusic {
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, &AudioConfig::default())
    }

    pub fn with_config(seed: u64, config: &AudioConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Random BPM from config range
        let bpm = rng.gen_range(config.min_bpm..config.max_bpm);

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
            config.melody_volume,
            config.melody_enabled,
        );

        let bass_track = Track::new(
            Waveform::Triangle,
            Envelope::pad(),
            bass_notes,
            bpm,
            config.bass_volume,
            config.bass_enabled,
        );

        // Synced highs - triggers exactly with melody
        let synced_highs = SyncedHighs::new(
            config.highs_octave_offset,
            config.highs_volume,
            config.highs_enabled,
        );

        // Atmospheric pad
        let pad = PadVoice::new(bpm, 0.25, true);

        // Delay effect for reverb-like sound (150ms delay, 40% feedback, 30% wet)
        let delay = DelayEffect::new(150.0, 0.4, 0.3);

        Self {
            melody_track,
            bass_track,
            synced_highs,
            pad,
            delay,
            melody_gen,
            root_note: root,
            current_sample: 0,
            bpm,
            bars_played: 0,
            bars_until_evolution: config.evolution_min_bars,
            evolution_min_bars: config.evolution_min_bars,
            evolution_max_bars: config.evolution_max_bars,
            mutation_chance: config.mutation_chance,
            master_volume: config.master_volume * config.music_volume,
            rng,
        }
    }

    fn evolve(&mut self) {
        // Mutate melody
        let current_melody: Vec<Note> = self.melody_track.notes.clone();
        let mutated = self.melody_gen.mutate_phrase(&current_melody, self.mutation_chance);
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

        self.bars_until_evolution = self.rng.gen_range(self.evolution_min_bars..self.evolution_max_bars + 1);
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

        // Check if melody is about to trigger (for syncing highs)
        let melody_triggered = self.melody_track.just_triggered();
        let melody_pitch = self.melody_track.current_pitch();

        // Sample all tracks
        let melody = self.melody_track.sample(self.current_sample);
        let bass = self.bass_track.sample(self.current_sample);

        // Trigger highs in sync with melody
        if melody_triggered {
            if let Some(pitch) = melody_pitch {
                let duration = (self.melody_track.samples_per_beat as f32 * 0.5) as u64;
                self.synced_highs.trigger(pitch, duration, self.current_sample, 0.7);
            }
        }
        let highs = self.synced_highs.sample(self.current_sample);

        // Update pad with root note
        self.pad.update_root(self.root_note, self.current_sample);
        let pad = self.pad.sample(self.current_sample);

        self.current_sample += 1;

        // Mix melody and highs, apply delay for reverb effect
        let melodic = melody + highs;
        let wet_melodic = self.delay.process(melodic);

        // Final mix: bass (dry) + wet melodic + pad
        let mixed = (wet_melodic + bass + pad) * self.master_volume;
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
