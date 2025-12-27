//! 8-bit waveform synthesizers

use std::f32::consts::PI;

/// Sample rate for audio generation
pub const SAMPLE_RATE: u32 = 44100;

/// Waveform type for synthesis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Square,
    Triangle,
    Sawtooth,
    Noise,
}

/// A single oscillator that generates samples
#[derive(Debug, Clone)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub frequency: f32,
    pub amplitude: f32,
    phase: f32,
    noise_state: u32,
}

impl Oscillator {
    pub fn new(waveform: Waveform, frequency: f32, amplitude: f32) -> Self {
        Self {
            waveform,
            frequency,
            amplitude,
            phase: 0.0,
            noise_state: 0x1234,
        }
    }

    pub fn square(frequency: f32, amplitude: f32) -> Self {
        Self::new(Waveform::Square, frequency, amplitude)
    }

    pub fn triangle(frequency: f32, amplitude: f32) -> Self {
        Self::new(Waveform::Triangle, frequency, amplitude)
    }

    pub fn sawtooth(frequency: f32, amplitude: f32) -> Self {
        Self::new(Waveform::Sawtooth, frequency, amplitude)
    }

    pub fn noise(amplitude: f32) -> Self {
        Self::new(Waveform::Noise, 0.0, amplitude)
    }

    /// Set frequency (for note changes)
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
    }

    /// Generate the next sample
    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            Waveform::Square => self.square_sample(),
            Waveform::Triangle => self.triangle_sample(),
            Waveform::Sawtooth => self.sawtooth_sample(),
            Waveform::Noise => self.noise_sample(),
        };

        // Advance phase
        if self.waveform != Waveform::Noise {
            self.phase += self.frequency / SAMPLE_RATE as f32;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }

        sample * self.amplitude
    }

    fn square_sample(&self) -> f32 {
        if self.phase < 0.5 { 1.0 } else { -1.0 }
    }

    fn triangle_sample(&self) -> f32 {
        if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - 4.0 * self.phase
        }
    }

    fn sawtooth_sample(&self) -> f32 {
        2.0 * self.phase - 1.0
    }

    fn noise_sample(&mut self) -> f32 {
        // Linear feedback shift register for pseudo-random noise
        let bit = ((self.noise_state >> 0) ^ (self.noise_state >> 2)) & 1;
        self.noise_state = (self.noise_state >> 1) | (bit << 14);
        if self.noise_state & 1 == 1 { 1.0 } else { -1.0 }
    }
}

/// ADSR envelope for note shaping
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    pub attack: f32,  // seconds
    pub decay: f32,   // seconds
    pub sustain: f32, // level 0-1
    pub release: f32, // seconds
}

impl Envelope {
    pub fn quick() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.1,
        }
    }

    pub fn plucky() -> Self {
        Self {
            attack: 0.005,
            decay: 0.15,
            sustain: 0.3,
            release: 0.2,
        }
    }

    pub fn pad() -> Self {
        Self {
            attack: 0.1,
            decay: 0.2,
            sustain: 0.8,
            release: 0.3,
        }
    }

    pub fn sparkle() -> Self {
        Self {
            attack: 0.001,
            decay: 0.05,
            sustain: 0.1,
            release: 0.08,
        }
    }

    /// Long atmospheric pad envelope
    pub fn atmospheric() -> Self {
        Self {
            attack: 0.5,
            decay: 0.3,
            sustain: 0.6,
            release: 1.0,
        }
    }

    /// Get envelope value at a given time since note start
    /// note_duration is how long the note is held before release
    pub fn value_at(&self, time: f32, note_duration: f32) -> f32 {
        if time < 0.0 {
            return 0.0;
        }

        // Note is still held
        if time < note_duration {
            if time < self.attack {
                // Attack phase
                time / self.attack
            } else if time < self.attack + self.decay {
                // Decay phase
                let decay_progress = (time - self.attack) / self.decay;
                1.0 - decay_progress * (1.0 - self.sustain)
            } else {
                // Sustain phase
                self.sustain
            }
        } else {
            // Release phase
            let release_time = time - note_duration;
            if release_time < self.release {
                let release_progress = release_time / self.release;
                self.sustain * (1.0 - release_progress)
            } else {
                0.0
            }
        }
    }
}

/// Convert MIDI note number to frequency
pub fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Note names for reference (C4 = 60)
pub mod notes {
    pub const C3: u8 = 48;
    pub const D3: u8 = 50;
    pub const E3: u8 = 52;
    pub const F3: u8 = 53;
    pub const G3: u8 = 55;
    pub const A3: u8 = 57;
    pub const B3: u8 = 59;
    pub const C4: u8 = 60;
    pub const D4: u8 = 62;
    pub const E4: u8 = 64;
    pub const F4: u8 = 65;
    pub const G4: u8 = 67;
    pub const A4: u8 = 69;
    pub const B4: u8 = 71;
    pub const C5: u8 = 72;
}
