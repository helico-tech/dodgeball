//! Chiptune synth. All sound effects and music are generated at start-up as
//! WAV bytes: pulse and triangle waves, noise drums, and an echo on the lead
//! like the SNES sound chip's echo buffer.

use crate::game::Sfx;

pub const SR: f32 = 44100.0;

pub const ALL_SFX: [Sfx; 14] = [
    Sfx::Throw,
    Sfx::Super,
    Sfx::Hit,
    Sfx::BigHit,
    Sfx::Catch,
    Sfx::Bounce,
    Sfx::Jump,
    Sfx::Whistle,
    Sfx::Blip,
    Sfx::Confirm,
    Sfx::Ko,
    Sfx::Cheer,
    Sfx::Pickup,
    Sfx::Pass,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Track {
    Title,
    Match,
    Champion,
}

pub const ALL_TRACKS: [Track; 3] = [Track::Title, Track::Match, Track::Champion];

pub fn wav(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let mut v = Vec::with_capacity(44 + samples.len() * 2);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + n * 2).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&44100u32.to_le_bytes());
    v.extend_from_slice(&88200u32.to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&(n * 2).to_le_bytes());
    for s in samples {
        v.extend_from_slice(&((s.clamp(-1.0, 1.0) * 30000.0) as i16).to_le_bytes());
    }
    v
}

#[derive(Clone, Copy)]
enum Wave {
    Square(f32),
    Tri,
    Sine,
}

fn osc(w: Wave, phase: f32) -> f32 {
    let p = phase.fract();
    match w {
        Wave::Square(d) => {
            if p < d {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Tri => 4.0 * (p - 0.5).abs() - 1.0,
        Wave::Sine => (p * std::f32::consts::TAU).sin(),
    }
}

fn midi_hz(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

struct Buf {
    s: Vec<f32>,
    noise: u32,
}

impl Buf {
    fn new(secs: f32) -> Self {
        Buf { s: vec![0.0; (secs * SR) as usize + 1], noise: 0xACE1 }
    }
    fn noise(&mut self) -> f32 {
        let mut x = self.noise;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.noise = x;
        (x >> 8) as f32 / (1u32 << 23) as f32 - 1.0
    }
    /// Tone sweeping from f0 to f1 (Hz). Envelope: attack, then decay with power `p`.
    #[allow(clippy::too_many_arguments)]
    fn tone(&mut self, start: f32, dur: f32, f0: f32, f1: f32, w: Wave, vol: f32, attack: f32, p: f32) {
        let i0 = (start * SR) as usize;
        let n = (dur * SR) as usize;
        let mut phase = 0.0f32;
        for k in 0..n {
            let t = k as f32 / SR;
            let u = k as f32 / n as f32;
            let f = f0 + (f1 - f0) * u;
            phase += f / SR;
            let env = (t / attack.max(1e-4)).min(1.0) * (1.0 - u).powf(p);
            if let Some(x) = self.s.get_mut(i0 + k) {
                *x += osc(w, phase) * vol * env;
            }
        }
    }
    /// Filtered noise burst; `lp` in 0..1 (lower = darker).
    fn hiss(&mut self, start: f32, dur: f32, vol: f32, lp: f32, attack: f32, p: f32) {
        let i0 = (start * SR) as usize;
        let n = (dur * SR) as usize;
        let mut y = 0.0;
        for k in 0..n {
            let t = k as f32 / SR;
            let u = k as f32 / n as f32;
            let r = self.noise();
            y += (r - y) * lp;
            let env = (t / attack.max(1e-4)).min(1.0) * (1.0 - u).powf(p);
            if let Some(x) = self.s.get_mut(i0 + k) {
                *x += y * vol * env;
            }
        }
    }
    fn hat(&mut self, start: f32, dur: f32, vol: f32) {
        let i0 = (start * SR) as usize;
        let n = (dur * SR) as usize;
        let mut lp = 0.0;
        for k in 0..n {
            let u = k as f32 / n as f32;
            let r = self.noise();
            lp += (r - lp) * 0.3;
            if let Some(x) = self.s.get_mut(i0 + k) {
                *x += (r - lp) * vol * (1.0 - u).powi(2);
            }
        }
    }
}

pub fn sfx_samples(s: Sfx) -> Vec<f32> {
    let mut b = Buf::new(match s {
        Sfx::Cheer => 1.6,
        Sfx::Whistle | Sfx::Ko | Sfx::BigHit => 0.6,
        _ => 0.35,
    });
    match s {
        Sfx::Throw => {
            b.hiss(0.0, 0.18, 0.45, 0.35, 0.05, 1.0);
            b.tone(0.0, 0.09, 700.0, 250.0, Wave::Square(0.5), 0.1, 0.002, 1.0);
        }
        Sfx::Super => {
            for (k, n) in [72.0, 76.0, 79.0, 84.0, 88.0, 91.0].iter().enumerate() {
                b.tone(k as f32 * 0.032, 0.05, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.18, 0.002, 0.5);
            }
            b.hiss(0.0, 0.33, 0.4, 0.5, 0.08, 1.0);
        }
        Sfx::Hit => {
            b.hiss(0.0, 0.15, 0.7, 0.6, 0.001, 2.0);
            b.tone(0.0, 0.12, 220.0, 60.0, Wave::Square(0.5), 0.3, 0.001, 1.0);
        }
        Sfx::BigHit => {
            b.hiss(0.0, 0.4, 0.8, 0.45, 0.001, 1.5);
            b.tone(0.0, 0.32, 160.0, 40.0, Wave::Tri, 0.7, 0.001, 1.0);
            b.tone(0.0, 0.25, 110.0, 30.0, Wave::Square(0.5), 0.25, 0.001, 1.0);
        }
        Sfx::Catch => {
            b.tone(0.0, 0.05, midi_hz(88.0), midi_hz(88.0), Wave::Square(0.25), 0.2, 0.001, 0.3);
            b.tone(0.05, 0.12, midi_hz(95.0), midi_hz(95.0), Wave::Square(0.25), 0.2, 0.001, 1.0);
            b.tone(0.0, 0.05, 180.0, 90.0, Wave::Tri, 0.4, 0.001, 1.0);
        }
        Sfx::Bounce => b.tone(0.0, 0.07, 240.0, 110.0, Wave::Tri, 0.45, 0.001, 1.0),
        Sfx::Jump => b.tone(0.0, 0.12, 330.0, 760.0, Wave::Square(0.5), 0.1, 0.002, 1.0),
        Sfx::Whistle => {
            for (st, du) in [(0.0f32, 0.12f32), (0.17, 0.36)] {
                let i0 = (st * SR) as usize;
                let n = (du * SR) as usize;
                let mut ph = 0.0f32;
                for k in 0..n {
                    let t = k as f32 / SR;
                    let f = 2500.0 + if (t * 30.0 * std::f32::consts::TAU).sin() > 0.0 { 160.0 } else { 0.0 };
                    ph += f / SR;
                    let env = (t / 0.01).min(1.0) * if k > n - 400 { (n - k) as f32 / 400.0 } else { 1.0 };
                    b.s[i0 + k] += osc(Wave::Square(0.5), ph) * 0.12 * env;
                }
            }
        }
        Sfx::Blip => b.tone(0.0, 0.035, 1046.0, 1046.0, Wave::Square(0.25), 0.15, 0.001, 0.5),
        Sfx::Confirm => {
            for (k, n) in [76.0, 83.0, 88.0].iter().enumerate() {
                b.tone(k as f32 * 0.05, 0.08, midi_hz(*n), midi_hz(*n), Wave::Square(0.25), 0.17, 0.001, 0.6);
            }
        }
        Sfx::Ko => {
            let n = (0.55 * SR) as usize;
            let mut ph = 0.0;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let f = (900.0 - 780.0 * u) * (1.0 + 0.04 * (t * 40.0).sin());
                ph += f / SR;
                b.s[k] += osc(Wave::Square(0.5), ph) * 0.15 * (1.0 - u);
            }
        }
        Sfx::Cheer => {
            // crowd: noise with a wobbling filter, plus a few whistles
            let n = (1.5 * SR) as usize;
            let mut y = 0.0;
            let mut y2 = 0.0;
            for k in 0..n {
                let t = k as f32 / SR;
                let u = k as f32 / n as f32;
                let r = b.noise();
                let lp = 0.12 + 0.08 * (t * 13.0).sin() * (t * 7.0).sin();
                y += (r - y) * lp;
                y2 += (y - y2) * 0.5;
                let env = (t / 0.25).min(1.0) * (1.0 - u).powf(1.4);
                b.s[k] += (y - y2 * 0.5) * 0.9 * env;
            }
            for (st, f0) in [(0.2f32, 1300.0f32), (0.45, 1600.0), (0.7, 1450.0)] {
                b.tone(st, 0.18, f0, f0 * 1.25, Wave::Square(0.5), 0.035, 0.03, 0.5);
            }
        }
        Sfx::Pickup => b.tone(0.0, 0.05, 520.0, 780.0, Wave::Tri, 0.35, 0.001, 0.5),
        Sfx::Pass => b.tone(0.0, 0.07, 440.0, 560.0, Wave::Square(0.125), 0.12, 0.002, 1.0),
    }
    b.s
}

// ---------------------------------------------------------------- music

struct Song {
    bpm: f32,
    chords: &'static [[i32; 3]],
    melody: &'static [&'static str],
    bass: [i32; 16],
    drums: &'static str,
    lead_duty: f32,
}

const A: [i32; 3] = [57, 61, 64];
const FSM: [i32; 3] = [54, 57, 61];
const D: [i32; 3] = [50, 54, 57];
const E: [i32; 3] = [52, 56, 59];
const EM: [i32; 3] = [52, 55, 59];
const C: [i32; 3] = [48, 52, 55];
const B: [i32; 3] = [59, 63, 66];
const R: i32 = 99;

const TITLE: Song = Song {
    bpm: 132.0,
    chords: &[A, FSM, D, E, A, FSM, D, E],
    melody: &[
        "E5 - - C#5 - E5 - A5 - - - - G#5 - F#5 -",
        "E5 - - - C#5 - - - A4 - B4 - C#5 - - -",
        "D5 - - F#5 - - A5 - - - F#5 - A5 - B5 -",
        "B5 - - - G#5 - - - E5 - - - . . . .",
        "E5 - - C#5 - E5 - A5 - - - - G#5 - F#5 -",
        "E5 - - - C#5 - E5 - F#5 - - - E5 - C#5 -",
        "D5 - F#5 - A5 - D6 - C#6 - - - B5 - A5 -",
        "B5 - - - - - G#5 - A5 - - - - - . .",
    ],
    bass: [0, R, 12, R, 0, R, 12, R, 0, R, 12, R, 7, R, 12, R],
    drums: "k.h.s.hkk.h.s.h.",
    lead_duty: 0.25,
};

const MATCH: Song = Song {
    bpm: 150.0,
    chords: &[EM, EM, C, D, EM, EM, C, B],
    melody: &[
        "E5 - . E5 . G5 - . A5 - B5 - . . A5 G5",
        "E5 - - - - - D5 - E5 - - - . . . .",
        "E5 - . E5 . G5 - . C6 - B5 - . . A5 G5",
        "A5 - - - - - F#5 - D5 - - - . . . .",
        "B5 - . B5 . A5 - . G5 - A5 - . . G5 E5",
        "G5 - - - E5 - - - D5 - E5 - . . . .",
        "C5 - E5 - G5 - C6 - B5 - - - G5 - E5 -",
        "D#5 - - - F#5 - - - B5 - - - A5 - F#5 -",
    ],
    bass: [0, 0, 12, 0, 0, 0, 12, 0, 0, 0, 12, 0, 7, 7, 10, 12],
    drums: "k.h.s.hkk.hks.hh",
    lead_duty: 0.5,
};

const CHAMP: Song = Song {
    bpm: 120.0,
    chords: &[A, D, E, A],
    melody: &[
        "A4 - C#5 - E5 - A5 - - - E5 - A5 - - -",
        "F#5 - - - A5 - - - D6 - - - C#6 - B5 -",
        "B5 - - - G#5 - E5 - B5 - - - C#6 - D6 -",
        "C#6 - - - - - - - A5 - - - - - - -",
    ],
    bass: [0, R, R, 0, R, R, 0, R, 7, R, R, 7, R, R, 12, R],
    drums: "k...s..kk...s.hh",
    lead_duty: 0.25,
};

fn parse_note(tok: &str) -> Option<i32> {
    let b = tok.as_bytes();
    let base = match b.first()? {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };
    let (acc, rest) = if b.get(1) == Some(&b'#') { (1, &tok[2..]) } else { (0, &tok[1..]) };
    let oct: i32 = rest.parse().ok()?;
    Some(12 * (oct + 1) + base + acc)
}

fn render_song(song: &Song) -> Vec<f32> {
    let step = 60.0 / song.bpm / 4.0;
    let bars = song.chords.len();
    let len = bars as f32 * 16.0 * step;
    let mut lead = Buf::new(len + 0.1);
    let mut b = Buf::new(len + 0.1);

    // melody: collect note events
    let toks: Vec<&str> = song.melody.iter().flat_map(|bar| bar.split_whitespace()).collect();
    let mut k = 0;
    while k < toks.len() {
        if let Some(n) = parse_note(toks[k]) {
            let mut l = 1;
            while k + l < toks.len() && toks[k + l] == "-" {
                l += 1;
            }
            let start = k as f32 * step;
            let dur = l as f32 * step * 0.95;
            let i0 = (start * SR) as usize;
            let cnt = (dur * SR) as usize;
            let mut ph = 0.0f32;
            let f = midi_hz(n as f32);
            for j in 0..cnt {
                let t = j as f32 / SR;
                let vib = if t > 0.15 { 1.0 + 0.006 * (t * 34.0).sin() } else { 1.0 };
                ph += f * vib / SR;
                let env = (t / 0.004).min(1.0) * (1.0 - 0.35 * (t / 0.2).min(1.0)) * ((cnt - j) as f32 / 300.0).min(1.0);
                if let Some(x) = lead.s.get_mut(i0 + j) {
                    *x += osc(Wave::Square(song.lead_duty), ph) * 0.12 * env;
                }
            }
            k += l;
        } else {
            k += 1;
        }
    }
    // echo: three 16ths later, quieter, twice
    let delay = (step * 3.0 * SR) as usize;
    let dry = lead.s.clone();
    for (i, x) in lead.s.iter_mut().enumerate() {
        if i >= delay {
            *x += dry[i - delay] * 0.35;
        }
        if i >= delay * 2 {
            *x += dry[i - delay * 2] * 0.15;
        }
    }

    let drums: Vec<u8> = song.drums.bytes().collect();
    for bar in 0..bars {
        let ch = song.chords[bar];
        for s in 0..16 {
            let t = (bar * 16 + s) as f32 * step;
            // arpeggio
            let n = ch[s % 3] + 12;
            b.tone(t, step * 0.9, midi_hz(n as f32), midi_hz(n as f32), Wave::Square(0.125), 0.035, 0.002, 1.5);
            // bass
            let off = song.bass[s];
            if off != R {
                let f = midi_hz((ch[0] - 12 + off) as f32);
                b.tone(t, step * 0.85, f, f, Wave::Tri, 0.28, 0.002, 0.4);
            }
            // drums
            match drums[s % drums.len()] {
                b'k' => {
                    b.tone(t, 0.12, 130.0, 42.0, Wave::Sine, 0.6, 0.001, 1.2);
                    b.hiss(t, 0.01, 0.2, 0.9, 0.0005, 1.0);
                }
                b's' => {
                    b.hiss(t, 0.15, 0.32, 0.6, 0.001, 1.6);
                    b.tone(t, 0.06, 200.0, 160.0, Wave::Tri, 0.25, 0.001, 1.0);
                }
                b'h' => b.hat(t, 0.035, 0.18),
                _ => {}
            }
        }
    }
    for (x, l) in b.s.iter_mut().zip(lead.s.iter()) {
        *x = (*x + l) * 0.9;
    }
    b.s.truncate((len * SR) as usize);
    b.s
}

pub fn music_samples(t: Track) -> Vec<f32> {
    render_song(match t {
        Track::Title => &TITLE,
        Track::Match => &MATCH,
        Track::Champion => &CHAMP,
    })
}
