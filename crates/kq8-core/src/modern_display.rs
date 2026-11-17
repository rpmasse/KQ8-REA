//! Modern viewer display choices, independent of the original Options.cs.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DrawFps {
    pub fps: u16,
}

impl Default for DrawFps {
    fn default() -> Self { Self { fps: 60 } }
}

impl DrawFps {
    pub const CHOICES: [u16; 5] = [15, 30, 60, 90, 120];
    pub fn valid(self) -> bool { Self::CHOICES.contains(&self.fps) }
    pub fn label(self) -> String {
        if self.fps == 15 { "15 (Original)".to_owned() } else { self.fps.to_string() }
    }
    pub fn step(self, direction: i32) -> Self {
        let index = Self::CHOICES.iter().position(|&fps| fps == self.fps).unwrap_or(2) as i32;
        Self { fps: Self::CHOICES[(index + direction).rem_euclid(Self::CHOICES.len() as i32) as usize] }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Windowed,
    #[default]
    Fullscreen,
}

impl ViewMode {
    pub const ALL: [Self; 2] = [Self::Windowed, Self::Fullscreen];
    pub fn label(self) -> &'static str {
        match self { Self::Windowed => "Windowed", Self::Fullscreen => "Fullscreen" }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ratio {
    #[default]
    FourThree,
    FiveFour,
    SixteenNine,
    SixteenTen,
    TwentyOneNine,
    ThirtyTwoNine,
}

impl Ratio {
    pub const ALL: [Self; 6] = [Self::FourThree, Self::FiveFour, Self::SixteenNine,
        Self::SixteenTen, Self::TwentyOneNine, Self::ThirtyTwoNine];
    pub fn label(self) -> &'static str {
        match self {
            Self::FourThree => "4:3", Self::FiveFour => "5:4",
            Self::SixteenNine => "16:9", Self::SixteenTen => "16:10",
            Self::TwentyOneNine => "21:9", Self::ThirtyTwoNine => "32:9",
        }
    }
    fn value(self) -> f64 {
        match self {
            Self::FourThree => 4.0/3.0, Self::FiveFour => 5.0/4.0,
            Self::SixteenNine => 16.0/9.0, Self::SixteenTen => 16.0/10.0,
            // Common 3440x1440 and 2560x1080 panels are marketed as 21:9.
            Self::TwentyOneNine => 43.0/18.0, Self::ThirtyTwoNine => 32.0/9.0,
        }
    }
    pub fn matches_pixels(self, size: [u32; 2]) -> bool {
        if size[1] == 0 { return false; }
        let actual = size[0] as f64 / size[1] as f64;
        ((actual-self.value())/self.value()).abs() <= 0.02
    }
    pub fn nearest(size: [u32; 2]) -> Self {
        Self::ALL.into_iter().min_by(|a,b| {
            let actual = size[0] as f64 / size[1].max(1) as f64;
            (actual-a.value()).abs().total_cmp(&(actual-b.value()).abs())
        }).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub mode: ViewMode,
    pub ratio: Ratio,
    pub pixels: [u32; 2],
}

impl Choice {
    pub fn valid(self) -> bool {
        (640..=16_384).contains(&self.pixels[0])
            && (480..=16_384).contains(&self.pixels[1])
    }
}

impl Default for Choice {
    fn default() -> Self {
        Self { mode: ViewMode::Fullscreen, ratio: Ratio::FourThree, pixels: [640,480] }
    }
}

// Deliberately finite, familiar pixel sizes. The monitor's native size is
// appended when it falls into the closest ratio family.
const STANDARD: &[(Ratio, [u32; 2])] = &[
    (Ratio::FourThree,[640,480]), (Ratio::FourThree,[800,600]),
    (Ratio::FourThree,[1024,768]), (Ratio::FourThree,[1280,960]),
    (Ratio::FourThree,[1600,1200]), (Ratio::FourThree,[1920,1440]),
    (Ratio::FiveFour,[1280,1024]), (Ratio::FiveFour,[1600,1280]),
    (Ratio::SixteenNine,[854,480]), (Ratio::SixteenNine,[1280,720]),
    (Ratio::SixteenNine,[1600,900]), (Ratio::SixteenNine,[1920,1080]),
    (Ratio::SixteenNine,[2560,1440]), (Ratio::SixteenNine,[3840,2160]),
    (Ratio::SixteenTen,[1280,800]), (Ratio::SixteenTen,[1440,900]),
    (Ratio::SixteenTen,[1680,1050]), (Ratio::SixteenTen,[1920,1200]),
    (Ratio::SixteenTen,[2560,1600]),
    (Ratio::TwentyOneNine,[1920,800]), (Ratio::TwentyOneNine,[2560,1080]),
    (Ratio::TwentyOneNine,[3440,1440]), (Ratio::TwentyOneNine,[3840,1600]),
    (Ratio::ThirtyTwoNine,[2560,720]), (Ratio::ThirtyTwoNine,[3840,1080]),
    (Ratio::ThirtyTwoNine,[5120,1440]), (Ratio::ThirtyTwoNine,[7680,2160]),
];

/// `fullscreen_modes` contains actual host video modes, not speculative
/// resolutions. Exclusive fullscreen can only show those pixel sizes.
pub fn choices(native: [u32; 2], fullscreen_modes: &[[u32; 2]], mode: ViewMode, ratio: Ratio) -> Vec<[u32; 2]> {
    let mut result: Vec<_> = STANDARD.iter().filter_map(|&(family,pixels)| {
        (family == ratio && pixels[0] <= native[0] && pixels[1] <= native[1]
            && (mode == ViewMode::Windowed || fullscreen_modes.contains(&pixels))).then_some(pixels)
    }).collect();
    if ratio == Ratio::nearest(native)
        && (mode == ViewMode::Windowed || fullscreen_modes.contains(&native)) {
        result.push(native);
    }
    result.sort_by_key(|pixels| (pixels[0] as u64 * pixels[1] as u64, pixels[0]));
    result.dedup();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_catalog_respects_ratio_native_and_exclusive_modes() {
        let native = [3440,1440];
        assert!(Choice::default().valid());
        assert!(!Choice { pixels: [0,0], ..Choice::default() }.valid());
        assert_eq!(Ratio::nearest(native), Ratio::TwentyOneNine);
        assert!(Ratio::TwentyOneNine.matches_pixels(native));
        assert!(!Ratio::SixteenNine.matches_pixels([2580,1328]));
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::FourThree),
            vec![[640,480],[800,600],[1024,768],[1280,960],[1600,1200],[1920,1440]]);
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::TwentyOneNine),
            vec![[1920,800],[2560,1080],[3440,1440]]);
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::ThirtyTwoNine),vec![[2560,720]]);
        assert_eq!(choices(native,&[[3440,1440],[2560,1080]],ViewMode::Fullscreen,Ratio::TwentyOneNine),
            vec![[2560,1080],[3440,1440]]);
        assert!(choices(native,&[[3440,1440]],ViewMode::Fullscreen,Ratio::FourThree).is_empty());
    }
}
