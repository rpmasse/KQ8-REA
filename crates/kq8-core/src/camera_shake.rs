//! Original camera-sync waveform, from camera-shake.md rules 1–6/12 (Q187).
//! The camera adapter decides how these angular samples affect each view.
use serde::{Deserialize, Serialize};

const PHASE_RATE: f32 = 0.293;
const SAMPLE_RATE: f32 = 15.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    pub seconds: f32,
    /// Pitch and heading maxima, in radians.
    pub maximum: [f32; 2],
    /// Phase-rate multipliers, despite the original command's period names.
    pub rate: [f32; 2],
}

impl Parameters {
    pub fn valid(&self) -> bool {
        self.seconds.is_finite()
            && self.maximum.iter().chain(&self.rate).all(|v| v.is_finite())
            && (self.seconds <= 0.0 || {
                let samples = self.seconds * SAMPLE_RATE;
                let half = self.seconds * (SAMPLE_RATE / 2.0);
                samples.is_finite()
                    && samples < u32::MAX as f32
                    && half > 0.0
                    && self.maximum.iter().all(|m| (m / half).is_finite())
                    && self
                        .rate
                        .iter()
                        .all(|r| (samples.ceil().max(1.0) * r * PHASE_RATE).is_finite())
            })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Shake {
    active: bool,
    parameters: Parameters,
    count: u32,
    amplitude: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub number: u32,
    pub amplitude: [f32; 2],
    /// Pitch and heading increments/parameters, in radians.
    pub angle: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tick {
    Idle,
    Sample(Sample),
    Stopped,
    Invalid,
}

impl Shake {
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn amplitude(&self) -> [f32; 2] {
        self.amplitude
    }

    pub fn valid(&self) -> bool {
        self.parameters.valid()
            && self.amplitude.iter().all(|v| v.is_finite())
            && if self.active {
                self.count <= (self.parameters.seconds.max(0.0) * SAMPLE_RATE).ceil() as u32
            } else {
                self.count == 0 && self.amplitude == [0.0; 2]
            }
    }

    /// Replaces the single effect without applying a camera update now.
    pub fn start(&mut self, parameters: Parameters) -> Result<(), &'static str> {
        if !parameters.valid() {
            return Err("invalid camera-shake parameters");
        }
        *self = Self {
            active: true,
            parameters,
            count: 0,
            amplitude: [0.0; 2],
        };
        Ok(())
    }

    /// One independent 15-Hz camera sync, including ticks with stationary Connor.
    pub fn tick(&mut self) -> Tick {
        if !self.active {
            return Tick::Idle;
        }
        if !self.valid() {
            *self = Self::default();
            return Tick::Invalid;
        }
        if self.count as f32 >= self.parameters.seconds * SAMPLE_RATE {
            self.active = false;
            self.count = 0;
            self.amplitude = [0.0; 2];
            return Tick::Stopped;
        }
        let half = self.parameters.seconds * (SAMPLE_RATE / 2.0);
        let sign = if (self.count as f32) < half {
            1.0
        } else {
            -1.0
        };
        for (a, maximum) in self.amplitude.iter_mut().zip(self.parameters.maximum) {
            *a += sign * (maximum / half);
        }
        self.count += 1;
        let angle = std::array::from_fn(|axis| {
            self.amplitude[axis]
                * (self.count as f32 * self.parameters.rate[axis] * PHASE_RATE).sin()
        });
        if !self.amplitude.iter().chain(&angle).all(|v| v.is_finite()) {
            *self = Self::default();
            return Tick::Invalid;
        }
        Tick::Sample(Sample {
            number: self.count,
            amplitude: self.amplitude,
            angle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEER: Parameters = Parameters {
        seconds: 4.0,
        maximum: [0.01, 0.01],
        rate: [2.0, 2.0],
    };
    const GONG: Parameters = Parameters {
        seconds: 4.0,
        maximum: [0.02, 0.03],
        rate: [4.0, 5.0],
    };
    fn sample(shake: &mut Shake) -> Sample {
        match shake.tick() {
            Tick::Sample(s) => s,
            other => panic!("expected a sample, got {other:?}"),
        }
    }
    fn near(actual: [f32; 2], expected: [f32; 2]) {
        for (a, e) in actual.into_iter().zip(expected) {
            assert!((a - e).abs() < 1e-7, "{actual:?} vs {expected:?}");
        }
    }

    #[test]
    fn original_beer_and_gong_values_have_sixty_samples_then_stop() {
        // Independent derived values in the study note, not runtime captures.
        for (p, first, peak) in [
            (
                BEER,
                [0.0001843443, 0.0001843443],
                [-0.0095496923, -0.0095496923],
            ),
            (
                GONG,
                [0.0006143527, 0.0009944087],
                [-0.0113337655, -0.0009687461],
            ),
        ] {
            let mut shake = Shake::default();
            shake.start(p).unwrap();
            near(sample(&mut shake).angle, first);
            for _ in 2..30 {
                sample(&mut shake);
            }
            let top = sample(&mut shake);
            assert_eq!(top.number, 30);
            near(top.angle, peak);
            near(top.amplitude, p.maximum);
            for _ in 31..60 {
                sample(&mut shake);
            }
            let last = sample(&mut shake);
            assert_eq!(last.number, 60);
            near(last.amplitude, [0.0; 2]);
            assert!(shake.active());
            assert_eq!(shake.tick(), Tick::Stopped);
            assert!(!shake.active() && shake.count() == 0 && shake.amplitude() == [0.0; 2]);
            assert_eq!(shake.tick(), Tick::Idle);
        }
    }

    #[test]
    fn nonintegral_duration_uses_the_unrounded_counter_limits() {
        let mut shake = Shake::default();
        shake
            .start(Parameters {
                seconds: 0.1,
                maximum: [0.02, 0.03],
                rate: [4.0, 5.0],
            })
            .unwrap();
        let first = sample(&mut shake);
        let second = sample(&mut shake);
        near(first.amplitude, [0.02 / 0.75, 0.03 / 0.75]);
        near(second.amplitude, [0.0; 2]);
        assert_eq!(second.number, 2);
        assert_eq!(shake.tick(), Tick::Stopped);
        shake
            .start(Parameters {
                seconds: 0.15,
                maximum: [0.02, 0.03],
                rate: [4.0, 5.0],
            })
            .unwrap();
        sample(&mut shake);
        sample(&mut shake);
        let third = sample(&mut shake);
        assert_eq!(third.number, 3);
        near(third.amplitude, [0.02 / 1.125, 0.03 / 1.125]);
        assert_eq!(
            shake.tick(),
            Tick::Stopped,
            "a nonintegral ramp need not return to zero before stopping"
        );
        for seconds in [0.0, -1.0] {
            shake.start(Parameters { seconds, ..GONG }).unwrap();
            assert_eq!(shake.tick(), Tick::Stopped);
        }
    }

    #[test]
    fn replacement_and_json_resume_keep_the_accumulated_ramp() {
        let mut uninterrupted = Shake::default();
        uninterrupted.start(GONG).unwrap();
        for _ in 0..29 {
            sample(&mut uninterrupted);
        }
        let mut restored: Shake =
            serde_json::from_slice(&serde_json::to_vec(&uninterrupted).unwrap()).unwrap();
        assert!(restored.valid());
        assert_eq!(sample(&mut restored), sample(&mut uninterrupted));
        assert_eq!(restored.count(), 30);
        for _ in 31..=61 {
            assert_eq!(restored.tick(), uninterrupted.tick());
        }
        restored.start(GONG).unwrap();
        for _ in 0..17 {
            sample(&mut restored);
        }
        restored.start(BEER).unwrap();
        let first = sample(&mut restored);
        assert_eq!(first.number, 1);
        near(first.angle, [0.0001843443; 2]);
    }

    #[test]
    fn zero_and_negative_axes_and_invalid_checkpoints_do_not_leak_nan() {
        let mut positive = Shake::default();
        positive.start(BEER).unwrap();
        let mut negative = Shake::default();
        negative
            .start(Parameters {
                maximum: [-0.01, 0.01],
                rate: [2.0, -2.0],
                ..BEER
            })
            .unwrap();
        let value = sample(&mut positive).angle;
        near(sample(&mut negative).angle, value.map(|v| -v));
        negative
            .start(Parameters {
                maximum: [0.0, 0.01],
                rate: [2.0, 0.0],
                ..BEER
            })
            .unwrap();
        near(sample(&mut negative).angle, [0.0; 2]);
        let retained = positive.clone();
        assert!(positive
            .start(Parameters {
                seconds: f32::NAN,
                ..BEER
            })
            .is_err());
        assert_eq!(
            positive, retained,
            "invalid replacement must leave the current effect intact"
        );
        let mut bad = retained.clone();
        bad.count = 999;
        assert!(!bad.valid());
        assert_eq!(bad.tick(), Tick::Invalid);
        assert_eq!(bad.tick(), Tick::Idle);
        bad = retained;
        bad.amplitude[0] = f32::INFINITY;
        assert!(!bad.valid());
    }

    #[test]
    #[ignore = "requires KQ8_GAME_DIR pointing to the GOG installation"]
    fn the_five_original_beer_and_gong_calls_use_the_documented_parameters() {
        let game =
            kq8_data::GameDir::open(std::env::var_os("KQ8_GAME_DIR").expect("set KQ8_GAME_DIR"))
                .unwrap();
        for (region, file, expected) in [
            ("Daventry", "beer42.cs", BEER),
            ("Daventry", "hibeer.cs", BEER),
            ("Daventry", "hibeer2.cs", BEER),
            ("Barren", "gongtp1.cs", GONG),
            ("Barren", "gsetup.cs", GONG),
        ] {
            let volume =
                kq8_data::VolArchive::open(&game.volume_path(region, "kq").unwrap()).unwrap();
            let bytes = volume.unpack(volume.find(file).unwrap()).unwrap().data;
            let text = String::from_utf8_lossy(&bytes);
            let calls: Vec<_> = text
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("KQCamera::shake "))
                .collect();
            assert_eq!(calls.len(), 1, "{region}/{file}");
            let words: Vec<_> = calls[0]
                .split_whitespace()
                .skip(1)
                .map(|s| s.parse::<f32>().unwrap())
                .collect();
            assert_eq!(words.len(), 5);
            let parameters = Parameters {
                seconds: words[0],
                maximum: [words[1], words[3]],
                rate: [words[2], words[4]],
            };
            assert_eq!(parameters, expected, "{region}/{file}");
            let mut shake = Shake::default();
            shake.start(parameters).unwrap();
            for number in 1..=60 {
                assert_eq!(sample(&mut shake).number, number);
            }
            assert_eq!(shake.tick(), Tick::Stopped);
        }
    }
}
