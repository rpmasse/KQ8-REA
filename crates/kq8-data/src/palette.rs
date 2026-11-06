//! Palettes: `.ppl` (chunked) and `.pal` (Windows RIFF).
//!
//! Sources are listed in docs/provenance/README.md. Layouts checked against
//! the real files; see `kq8 bitmap-verify`.

use crate::chunks::{read_chunks, tag_string, u16_at, u32_at};

pub type Rgb = [u8; 3];

/// Palette conversion path; brightness.md Q186 describes their differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    Direct3D,
    Glide,
    Software,
}

impl Output {
    /// Original assignGModeName names (brightness.md rule 5a).
    pub fn from_mode(name: &str) -> Option<Self> {
        if name == "Glide" {
            Some(Self::Glide)
        } else if name == "Software" {
            Some(Self::Software)
        } else if name.starts_with("D3D-") && name.len() > 4
        {
            Some(Self::Direct3D)
        } else {
            None
        }
    }
}

/// The original promoted-f32 endpoints, interpolated then stored as f32.
/// This is a palette multiplier, independent of the display gamma ramp.
pub fn brightness_gain(value: u8) -> f32 {
    let low = f64::from(0.3_f32);
    let high = f64::from(1.7_f32);
    (low + f64::from(value.min(100)) / (100.0 / (high - low))) as f32
}

/// Direct3D palette colour conversion under ordinary round-to-nearest
/// f32 stores. Original driver/control-state boundary output remains open.
pub fn brightness_color(rgb: Rgb, value: u8) -> Rgb {
    convert_color(rgb, brightness_gain(value), Output::Direct3D)
}

fn convert_color(rgb: Rgb, gain: f32, output: Output) -> Rgb {
    let largest = f64::from(*rgb.iter().max().unwrap());
    let gain = if output != Output::Software && largest * f64::from(gain) > 255.0 {
        (255.0 / largest) as f32
    } else {
        gain
    };
    std::array::from_fn(|axis| {
        let product = f64::from(rgb[axis]) * f64::from(gain);
        // Q186: cap comparison precedes the explicitly stored products.
        if product > 255.0 {
            return 255;
        }
        let stored = match output {
            Output::Glide => axis == 1,
            Output::Direct3D => axis != 1,
            Output::Software => false,
        };
        if stored {
            (product as f32) as u8
        } else {
            product as u8
        }
    })
}

pub fn for_output(colors: [Rgb; 256], value: u8, output: Output) -> [Rgb; 256] {
    let gain = brightness_gain(value);
    std::array::from_fn(|index| {
        let rgb = convert_color(colors[index], gain, output);
        if output == Output::Glide
            && rgb == [0; 3]
            && ((1..=9).contains(&index) || (246..=254).contains(&index))
        {
            [255, 0, 255]
        } else {
            rgb
        }
    })
}

/// The two hardware paths consult different stages for palette fog.
pub fn hardware_fog(rgb: Rgb, value: u8, output: Output) -> Rgb {
    match output {
        Output::Glide => rgb,
        _ => convert_color(rgb, brightness_gain(value), output),
    }
}

/// Adjust a copied palette; retain stored colours for matching and future loads.
pub fn with_brightness(colors: [Rgb; 256], value: u8) -> [Rgb; 256] {
    for_output(colors, value, Output::Direct3D)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteFormat {
    /// `PPAL` tag followed by `head`, `data` and lighting tables.
    Chunked,
    /// Standard Windows `RIFF` / `PAL ` file.
    Riff,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub format: PaletteFormat,
    /// Colours as stored in the file. Missing entries are black.
    pub colors: [Rgb; 256],
    /// How many entries the file really contained.
    pub entry_count: usize,
    /// Chunk count from the `head` chunk (chunked palettes only).
    pub declared_chunks: Option<u32>,
    /// Tags of all chunks after the marker, in file order.
    pub chunk_tags: Vec<String>,
    /// Chunks after `data` (lighting, haze, colour-match tables), as
    /// (tag, size). Not decoded yet.
    pub extra_chunks: Vec<(String, usize)>,
}

impl Palette {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.starts_with(b"RIFF") {
            Self::parse_riff(bytes)
        } else if bytes.starts_with(b"PPAL") {
            Self::parse_chunked(bytes)
        } else {
            Err(format!(
                "unknown palette signature '{}'",
                tag_string(&first4(bytes))
            ))
        }
    }

    /// `PPAL` is only a marker: its size field does not describe the data,
    /// so chunk reading starts right after its 8 bytes.
    fn parse_chunked(bytes: &[u8]) -> Result<Self, String> {
        let chunks = read_chunks(bytes, 8).map_err(|e| e.to_string())?;
        let mut pal = Palette {
            format: PaletteFormat::Chunked,
            colors: [[0; 3]; 256],
            entry_count: 0,
            declared_chunks: None,
            chunk_tags: chunks.iter().map(|c| c.tag_str()).collect(),
            extra_chunks: Vec::new(),
        };
        let mut have_data = false;
        for c in &chunks {
            match &c.tag {
                b"head" if c.data.len() >= 4 => {
                    // low 24 bits: chunk count; high 8 bits: version
                    pal.declared_chunks = Some(u32_at(c.data, 0) & 0x00FF_FFFF);
                }
                b"data" if !have_data => {
                    // PALETTEENTRY: red, green, blue, flags
                    pal.entry_count = (c.data.len() / 4).min(256);
                    for i in 0..pal.entry_count {
                        pal.colors[i] = [c.data[i * 4], c.data[i * 4 + 1], c.data[i * 4 + 2]];
                    }
                    have_data = true;
                }
                _ => pal.extra_chunks.push((c.tag_str(), c.data.len())),
            }
        }
        if !have_data {
            return Err("palette has no 'data' chunk".into());
        }
        Ok(pal)
    }

    fn parse_riff(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[8..12] != b"PAL " {
            return Err("RIFF file is not a palette (no 'PAL ' form)".into());
        }
        let chunks = read_chunks(&bytes[..bytes.len().min(8 + u32_at(bytes, 4) as usize)], 12)
            .map_err(|e| e.to_string())?;
        let data = chunks
            .iter()
            .find(|c| &c.tag == b"data")
            .ok_or("RIFF palette has no 'data' chunk")?;
        if data.data.len() < 4 {
            return Err("RIFF palette 'data' chunk too small".into());
        }
        // LOGPALETTE: u16 version (0x300), u16 count, then entries.
        let count = u16_at(data.data, 2) as usize;
        let available = (data.data.len() - 4) / 4;
        if count > available {
            return Err(format!(
                "RIFF palette claims {count} colours, has room for {available}"
            ));
        }
        let mut colors = [[0; 3]; 256];
        for (i, slot) in colors.iter_mut().enumerate().take(count.min(256)) {
            let e = &data.data[4 + i * 4..];
            *slot = [e[0], e[1], e[2]];
        }
        Ok(Palette {
            format: PaletteFormat::Riff,
            colors,
            entry_count: count,
            declared_chunks: None,
            chunk_tags: chunks.iter().map(|c| c.tag_str()).collect(),
            extra_chunks: chunks
                .iter()
                .filter(|c| &c.tag != b"data")
                .map(|c| (c.tag_str(), c.data.len()))
                .collect(),
        })
    }

    /// The colours as the game shows them on a modern display: as stored,
    /// except entry 0, which is black (see [`as_shown`]).
    pub fn shown_colors(&self) -> [Rgb; 256] {
        as_shown(self.colors)
    }
}

/// A colour table as the game shows it on a modern display: as stored
/// (only a 256-colour screen swaps entries 0–9 and 246–255 for the system
/// colours, Q3), except entry 0, which is black on any screen (measured in
/// the running original, Q22; models-and-textures.md "System colours").
/// Also used for the colour tables inside textures, so every texture
/// follows one rule; that part was not measured (only a world palette was).
pub fn as_shown(mut colors: [Rgb; 256]) -> [Rgb; 256] {
    colors[0] = [0, 0, 0];
    colors
}

fn first4(bytes: &[u8]) -> [u8; 4] {
    let mut t = [0u8; 4];
    for (d, s) in t.iter_mut().zip(bytes) {
        *d = *s;
    }
    t
}

#[cfg(test)]
mod tests {
    #[test]
    fn brightness_preserves_neutral_colours_and_clipped_hue() {
        use super::brightness_color as b;
        assert_eq!(b([30, 100, 250], 50), [30, 100, 250]);
        assert_eq!(b([100, 200, 50], 0), [30, 60, 15]);
        assert_eq!(b([50, 100, 25], 100), [85, 170, 42]);
        assert_eq!(b([100, 200, 50], 100), [127, 254, 63], "D3D's green register product can fall just below 255 after the f32 reduced-gain store (Q186)");
        assert_eq!(b([255, 128, 64], 100), [255, 128, 64]);
        assert_eq!(b([0, 0, 0], 100), [0, 0, 0]);
        assert_eq!(b([50, 100, 25], 255), b([50, 100, 25], 100));
    }

    #[test]
    fn the_original_gain_and_explicit_hardware_product_stores_differ() {
        assert_eq!(
            [
                brightness_gain(0),
                brightness_gain(50),
                brightness_gain(100)
            ],
            [0.3_f32, 1.0, 1.7_f32]
        );
        let mut source = [[0; 3]; 256];
        source[20] = [102, 153, 204];
        source[21] = [120, 200, 250];
        for output in [Output::Direct3D, Output::Glide] {
            assert_eq!(for_output(source, 0, output)[20], [30, 45, 61]);
            assert_eq!(for_output(source, 50, output)[20], [102, 153, 204]);
            assert_eq!(for_output(source, 100, output)[20], [127, 191, 255]);
        }
        assert_eq!(
            for_output(source, 100, Output::Direct3D)[21],
            [122, 203, 255],
            "D3D explicitly stores red/blue products as f32"
        );
        assert_eq!(
            for_output(source, 100, Output::Glide)[21],
            [122, 204, 254],
            "Glide explicitly stores the green product as f32"
        );
        assert_eq!(
            for_output(source, 100, Output::Software)[20],
            [173, 255, 255],
            "software clips channels independently"
        );
        assert_eq!(hardware_fog(source[20], 0, Output::Glide), source[20]);
        assert_eq!(hardware_fog(source[20], 0, Output::Direct3D), [30, 45, 61]);
        assert_eq!(source[21], [120, 200, 250]);
    }

    #[test]
    fn glide_changes_only_the_eighteen_converted_black_entries() {
        let source = [[1, 2, 3]; 256];
        let shown = for_output(source, 0, Output::Glide);
        assert_eq!(
            shown.iter().filter(|&&rgb| rgb == [255, 0, 255]).count(),
            18
        );
        for index in [1, 9, 246, 254] {
            assert_eq!(shown[index], [255, 0, 255]);
        }
        for index in [0, 10, 245, 255] {
            assert_eq!(shown[index], [0, 0, 0]);
        }
        assert_eq!(for_output(source, 0, Output::Direct3D), [[0; 3]; 256]);
        assert_eq!(Output::from_mode("Glide"), Some(Output::Glide));
        assert_eq!(Output::from_mode("D3D-display"), Some(Output::Direct3D));
        assert_eq!(Output::from_mode("Software"), Some(Output::Software));
        assert_eq!(Output::from_mode("software"), None, "the original software-name distinction is case-sensitive");
        assert_eq!(Output::from_mode("other"), None);
    }

    use super::*;

    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut v = tag.to_vec();
        v.extend_from_slice(&(data.len() as u32).to_le_bytes());
        v.extend_from_slice(data);
        v
    }

    #[test]
    fn chunked_palette() {
        let mut entries = vec![0u8; 1024];
        entries[100 * 4..100 * 4 + 3].copy_from_slice(&[1, 2, 3]);
        entries[0..3].copy_from_slice(&[9, 9, 9]);
        let mut f = b"PPAL\xff\xff\xff\xff".to_vec();
        f.extend(chunk(b"head", &1u32.to_le_bytes()));
        f.extend(chunk(b"data", &entries));
        f.extend(chunk(b"pcmt", &[0; 32]));
        let p = Palette::parse(&f).unwrap();
        assert_eq!(p.entry_count, 256);
        assert_eq!(p.colors[100], [1, 2, 3]);
        assert_eq!(p.declared_chunks, Some(1));
        assert_eq!(p.extra_chunks, vec![("pcmt".to_string(), 32)]);
        // Stored colours are used as they are (Q3), except entry 0: black (Q22).
        assert_eq!(p.shown_colors()[100], [1, 2, 3]);
        assert_eq!(p.shown_colors()[0], [0, 0, 0]);
        assert_eq!(p.colors[0], [9, 9, 9]);
    }

    #[test]
    fn riff_palette() {
        let mut data = vec![0x00, 0x03, 2, 0];
        data.extend_from_slice(&[10, 20, 30, 0, 40, 50, 60, 0]);
        let body = chunk(b"data", &data);
        let mut f = b"RIFF".to_vec();
        f.extend_from_slice(&((4 + body.len()) as u32).to_le_bytes());
        f.extend_from_slice(b"PAL ");
        f.extend(body);
        let p = Palette::parse(&f).unwrap();
        assert_eq!(p.format, PaletteFormat::Riff);
        assert_eq!(p.entry_count, 2);
        assert_eq!(p.colors[1], [40, 50, 60]);
        assert_eq!(p.shown_colors()[0], [0, 0, 0]);
        assert_eq!(p.shown_colors()[1], [40, 50, 60]);
    }
}
