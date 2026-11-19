//! Modern viewer display choices, independent of the original Options.cs.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Anisotropic {
    pub level: u16,
}

impl Default for Anisotropic {
    fn default() -> Self { Self { level: 4 } }
}

impl Anisotropic {
    // A clamp of 1 is wgpu's ordinary, non-anisotropic sampling mode.
    pub const CHOICES: [u16; 4] = [1, 4, 8, 16];
    pub fn valid(self) -> bool { Self::CHOICES.contains(&self.level) }
    pub fn label(self) -> String {
        if self.level == 1 { "Off".to_owned() } else { format!("{}x", self.level) }
    }
    pub fn step(self, direction: i32) -> Self {
        let index = Self::CHOICES.iter().position(|&level| level == self.level).unwrap_or(0) as i32;
        Self { level: Self::CHOICES[(index + direction).rem_euclid(Self::CHOICES.len() as i32) as usize] }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AntiAliasing {
    #[default]
    Off,
    Smaa,
    Fxaa,
}

impl AntiAliasing {
    pub const ALL: [Self; 3] = [Self::Off, Self::Smaa, Self::Fxaa];
    pub fn label(self) -> &'static str {
        match self { Self::Off => "Off", Self::Smaa => "SMAA", Self::Fxaa => "FXAA" }
    }
    pub fn step(self, direction: i32) -> Self {
        let index = Self::ALL.iter().position(|&choice| choice == self).unwrap() as i32;
        Self::ALL[(index + direction).rem_euclid(Self::ALL.len() as i32) as usize]
    }
}

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
    ThreeTwo,
    SixteenNine,
    SixteenTen,
    TwentyOneNine,
    ThirtyTwoNine,
}

impl Ratio {
    pub const ALL: [Self; 7] = [Self::FourThree, Self::FiveFour, Self::ThreeTwo, Self::SixteenNine,
        Self::SixteenTen, Self::TwentyOneNine, Self::ThirtyTwoNine];
    pub fn label(self) -> &'static str {
        match self {
            Self::FourThree => "4:3", Self::FiveFour => "5:4",
            Self::ThreeTwo => "3:2",
            Self::SixteenNine => "16:9", Self::SixteenTen => "16:10",
            Self::TwentyOneNine => "21:9", Self::ThirtyTwoNine => "32:9",
        }
    }
    fn value(self) -> f64 {
        match self {
            Self::FourThree => 4.0/3.0, Self::FiveFour => 5.0/4.0,
            Self::ThreeTwo => 3.0/2.0,
            Self::SixteenNine => 16.0/9.0, Self::SixteenTen => 16.0/10.0,
            // Common 3440x1440 and 2560x1080 panels are marketed as 21:9.
            Self::TwentyOneNine => 43.0/18.0, Self::ThirtyTwoNine => 32.0/9.0,
        }
    }
    pub fn matches_pixels(self, size: [u32; 2]) -> bool {
        if size[1] == 0 { return false; }
        let actual = size[0] as f64 / size[1] as f64;
        let compatible = |expected: f64| ((actual-expected)/expected).abs() <= 0.005;
        if self == Self::TwentyOneNine {
            [7.0/3.0,64.0/27.0,43.0/18.0,12.0/5.0].into_iter().any(compatible)
        } else { compatible(self.value()) }
    }
    pub fn nearest(size: [u32; 2]) -> Self {
        if let Some(family) = Self::ALL.into_iter().find(|family|family.matches_pixels(size)) {
            return family;
        }
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
// appended only when its shape matches the selected ratio family.
const STANDARD: &[(Ratio, [u32; 2])] = &[
    (Ratio::FourThree,[640,480]), (Ratio::FourThree,[800,600]),
    (Ratio::FourThree,[1024,768]), (Ratio::FourThree,[1280,960]),
    (Ratio::FourThree,[1440,1080]), (Ratio::FourThree,[1600,1200]),
    (Ratio::FourThree,[1920,1440]), (Ratio::FourThree,[2048,1536]),
    (Ratio::FiveFour,[960,768]), (Ratio::FiveFour,[1280,1024]),
    (Ratio::FiveFour,[1350,1080]), (Ratio::FiveFour,[1600,1280]),
    (Ratio::FiveFour,[1920,1536]), (Ratio::FiveFour,[2160,1728]),
    (Ratio::FiveFour,[2560,2048]), (Ratio::FiveFour,[2700,2160]),
    (Ratio::FiveFour,[2880,2304]), (Ratio::FiveFour,[4320,3456]),
    (Ratio::FiveFour,[5120,4096]), (Ratio::FiveFour,[5760,4608]),
    (Ratio::ThreeTwo,[720,480]), (Ratio::ThreeTwo,[1080,720]),
    (Ratio::ThreeTwo,[1440,960]), (Ratio::ThreeTwo,[1920,1280]),
    (Ratio::ThreeTwo,[2160,1440]), (Ratio::ThreeTwo,[3000,2000]),
    (Ratio::ThreeTwo,[3240,2160]), (Ratio::ThreeTwo,[4320,2880]),
    (Ratio::ThreeTwo,[4500,3000]),
    (Ratio::SixteenNine,[854,480]), (Ratio::SixteenNine,[1280,720]),
    (Ratio::SixteenNine,[1366,768]), (Ratio::SixteenNine,[1600,900]),
    (Ratio::SixteenNine,[1920,1080]), (Ratio::SixteenNine,[2560,1440]),
    (Ratio::SixteenNine,[3840,2160]), (Ratio::SixteenNine,[5120,2880]),
    (Ratio::SixteenNine,[7680,4320]),
    (Ratio::SixteenTen,[1280,800]), (Ratio::SixteenTen,[1440,900]),
    (Ratio::SixteenTen,[1680,1050]), (Ratio::SixteenTen,[1920,1200]),
    (Ratio::SixteenTen,[2560,1600]), (Ratio::SixteenTen,[2880,1800]),
    (Ratio::SixteenTen,[3456,2160]), (Ratio::SixteenTen,[3840,2400]),
    (Ratio::TwentyOneNine,[1680,720]), (Ratio::TwentyOneNine,[1920,800]),
    (Ratio::TwentyOneNine,[2560,1080]), (Ratio::TwentyOneNine,[3440,1440]),
    (Ratio::TwentyOneNine,[3840,1600]), (Ratio::TwentyOneNine,[5120,2160]),
    (Ratio::TwentyOneNine,[6880,2880]), (Ratio::TwentyOneNine,[7680,3200]),
    (Ratio::TwentyOneNine,[10240,4320]),
    (Ratio::ThirtyTwoNine,[2560,720]), (Ratio::ThirtyTwoNine,[3200,900]),
    (Ratio::ThirtyTwoNine,[3840,1080]), (Ratio::ThirtyTwoNine,[5120,1440]),
    (Ratio::ThirtyTwoNine,[6400,1800]), (Ratio::ThirtyTwoNine,[7680,2160]),
    (Ratio::ThirtyTwoNine,[10240,2880]), (Ratio::ThirtyTwoNine,[15360,4320]),
];

/// `fullscreen_modes` contains actual host video modes, not speculative
/// resolutions. Exclusive fullscreen can only show those pixel sizes.
pub fn choices(native: [u32; 2], fullscreen_modes: &[[u32; 2]], mode: ViewMode, ratio: Ratio) -> Vec<[u32; 2]> {
    let mut result: Vec<_> = STANDARD.iter().filter_map(|&(family,pixels)| {
        (family == ratio && ratio.matches_pixels(pixels)
            && pixels[0] <= native[0] && pixels[1] <= native[1]
            && (mode == ViewMode::Windowed || fullscreen_modes.contains(&pixels))).then_some(pixels)
    }).collect();
    if ratio.matches_pixels(native)
        && (Choice { mode, ratio, pixels: native }).valid()
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
    fn anisotropic_choices_cycle_and_reject_other_levels() {
        assert_eq!(Anisotropic::default(), Anisotropic { level: 4 });
        assert_eq!(Anisotropic::default().step(1), Anisotropic { level: 8 });
        assert_eq!(Anisotropic { level: 8 }.step(1), Anisotropic { level: 16 });
        assert_eq!(Anisotropic { level: 16 }.step(1), Anisotropic { level: 1 });
        assert_eq!(Anisotropic { level: 1 }.step(1), Anisotropic { level: 4 });
        assert_eq!(Anisotropic { level: 4 }.step(-1), Anisotropic { level: 1 });
        assert!(Anisotropic { level: 1 }.valid());
        assert_eq!(Anisotropic { level: 1 }.label(), "Off");
        assert!(!Anisotropic { level: 2 }.valid());
    }

    #[test]
    fn anti_aliasing_cycles_in_graphics_order() {
        assert_eq!(AntiAliasing::Off.step(1), AntiAliasing::Smaa);
        assert_eq!(AntiAliasing::Smaa.step(1), AntiAliasing::Fxaa);
        assert_eq!(AntiAliasing::Fxaa.step(1), AntiAliasing::Off);
        assert_eq!(AntiAliasing::Off.step(-1), AntiAliasing::Fxaa);
    }

    #[test]
    fn modern_catalog_respects_ratio_native_and_exclusive_modes() {
        let native = [3440,1440];
        assert!(Choice::default().valid());
        assert!(!Choice { pixels: [0,0], ..Choice::default() }.valid());
        assert_eq!(Ratio::nearest(native), Ratio::TwentyOneNine);
        assert!(Ratio::TwentyOneNine.matches_pixels(native));
        assert!(Ratio::TwentyOneNine.matches_pixels([1680,720]));
        assert!(Ratio::ThreeTwo.matches_pixels([2160,1440]));
        assert!(!Ratio::SixteenNine.matches_pixels([2580,1328]));
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::FourThree),
            vec![[640,480],[800,600],[1024,768],[1280,960],[1440,1080],
                [1600,1200],[1920,1440]]);
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::TwentyOneNine),
            vec![[1680,720],[1920,800],[2560,1080],[3440,1440]]);
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::ThirtyTwoNine),
            vec![[2560,720],[3200,900]]);
        assert_eq!(choices(native,&[],ViewMode::Windowed,Ratio::ThreeTwo),
            vec![[720,480],[1080,720],[1440,960],[1920,1280],[2160,1440]]);
        assert_eq!(choices(native,&[[3440,1440],[2560,1080]],ViewMode::Fullscreen,Ratio::TwentyOneNine),
            vec![[2560,1080],[3440,1440]]);
        assert!(choices(native,&[[3440,1440]],ViewMode::Fullscreen,Ratio::FourThree).is_empty());
        let unusual_native = [2580,1328];
        assert!(Ratio::ALL.into_iter().all(|family|
            !choices(unusual_native,&[],ViewMode::Windowed,family).contains(&unusual_native)),
            "a native size must not be appended under a merely nearest ratio");
    }

    #[test]
    fn requested_standard_sizes_are_only_in_their_aspect_family() {
        let requested: &[(Ratio,&[[u32;2]])] = &[
            (Ratio::FourThree,&[[640,480],[800,600],[1024,768],[1280,960],
                [1440,1080],[1600,1200],[1920,1440],[2048,1536]]),
            (Ratio::FiveFour,&[[960,768],[1280,1024],[1350,1080],[1920,1536],
                [2160,1728],[2560,2048],[2700,2160],[2880,2304],
                [4320,3456],[5120,4096],[5760,4608]]),
            (Ratio::ThreeTwo,&[[720,480],[1080,720],[1440,960],[1920,1280],
                [2160,1440],[3000,2000],[3240,2160],[4320,2880],[4500,3000]]),
            (Ratio::SixteenNine,&[[1280,720],[1366,768],[1920,1080],
                [2560,1440],[3840,2160],[5120,2880],[7680,4320]]),
            (Ratio::SixteenTen,&[[1280,800],[1440,900],[1680,1050],[1920,1200],
                [2560,1600],[2880,1800],[3456,2160],[3840,2400]]),
            (Ratio::TwentyOneNine,&[[1680,720],[2560,1080],[3440,1440],
                [3840,1600],[5120,2160],[6880,2880],[7680,3200],[10240,4320]]),
            (Ratio::ThirtyTwoNine,&[[2560,720],[3200,900],[3840,1080],
                [5120,1440],[6400,1800],[7680,2160],[10240,2880],[15360,4320]]),
        ];
        let native = [16384,16384];
        for &(family,sizes) in requested {
            let available = choices(native,&[],ViewMode::Windowed,family);
            for &size in sizes {
                assert!(available.contains(&size),"{} missing {size:?}",family.label());
                assert!(family.matches_pixels(size),"{} rejects its own {size:?}",family.label());
                for other in Ratio::ALL {
                    if other != family {
                        assert!(!choices(native,&[],ViewMode::Windowed,other).contains(&size),
                            "{size:?} leaked into {}",other.label());
                    }
                }
            }
        }
        assert_eq!(choices([3440,1440],&[[1680,720],[2560,1080],[3440,1440]],
            ViewMode::Fullscreen,Ratio::TwentyOneNine),
            vec![[1680,720],[2560,1080],[3440,1440]]);
        assert_eq!(choices([3440,1440],&[[720,480],[1920,1280]],
            ViewMode::Fullscreen,Ratio::ThreeTwo),vec![[720,480],[1920,1280]]);
    }
}
