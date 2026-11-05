//! Original GOG cinematics. The three W32*.dll files contain RIFF AVI.
//! Video time uses the stream's rate/scale and the audio track is PCM.

use crate::aud::{wave_pcm, Sound};

pub struct Movie {
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub rate: u32,
    pub scale: u32,
    /// One entry per AVI display interval. A two-byte Indeo 5 null
    /// picture at this position retains the preceding picture.
    pub holds: Vec<bool>,
    pub audio: Sound,
}

fn word(b: &[u8], at: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        b.get(at..at + 4)
            .ok_or("short AVI header")?
            .try_into()
            .unwrap(),
    ))
}

/// RIFF chunks are length-prefixed and padded to an even byte boundary.
fn chunks(
    mut data: &[u8],
    mut read: impl FnMut(&[u8], &[u8]) -> Result<(), String>,
) -> Result<(), String> {
    while !data.is_empty() {
        let size = word(data, 4)? as usize;
        let end = 8usize.checked_add(size).ok_or("AVI chunk too large")?;
        read(&data[..4], data.get(8..end).ok_or("short AVI chunk")?)?;
        data = data.get(end + (size & 1)..).ok_or("short AVI padding")?;
    }
    Ok(())
}

fn collect_records(
    list: &[u8],
    stream: &[u8; 4],
    video: &[u8; 2],
    pcm: &mut Vec<u8>,
    holds: &mut Vec<bool>,
    level: usize,
) -> Result<(), String> {
    if level > 4 {
        return Err("nested AVI records".into());
    }
    chunks(list, |id, payload| {
        if id == stream {
            pcm.extend_from_slice(payload);
        } else if &id[..2] == video && matches!(&id[2..], b"iv" | b"dc" | b"db") {
            // IV50 uses a five-bit start code (31) followed by a
            // three-bit picture type (4 = null), least-significant bit first.
            let hold = payload.is_empty() || matches!(payload, [0x9f, _]);
            if (hold && holds.is_empty()) || (!hold && payload.len() < 3) {
                return Err("invalid short Indeo 5 picture".into());
            }
            holds.push(hold);
        } else if id == b"LIST" && payload.starts_with(b"rec ") {
            collect_records(&payload[4..], stream, video, pcm, holds, level + 1)?;
        }
        Ok(())
    })
}

impl Movie {
    pub fn read(data: &[u8]) -> Result<Self, String> {
        if !data.starts_with(b"RIFF") || data.get(8..12) != Some(b"AVI ") {
            return Err("expected an AVI movie".into());
        }
        let size = word(data, 4)? as usize;
        let body = data
            .get(8..)
            .and_then(|d| d.get(..size))
            .ok_or("short AVI file")?;
        let mut picture = None;
        let mut soundtrack = None;
        let mut pcm = Vec::new();
        let mut holds = Vec::new();
        chunks(body.get(4..).ok_or("short AVI type")?, |id, list| {
            if id != b"LIST" {
                return Ok(());
            }
            if list.starts_with(b"hdrl") {
                let mut index = 0;
                chunks(&list[4..], |id, stream| {
                    if id != b"LIST" || !stream.starts_with(b"strl") {
                        return Ok(());
                    }
                    let (mut header, mut format) = (Vec::new(), Vec::new());
                    chunks(&stream[4..], |id, bytes| {
                        match id {
                            b"strh" => header = bytes.to_vec(),
                            b"strf" => format = bytes.to_vec(),
                            _ => (),
                        }
                        Ok(())
                    })?;
                    if header.starts_with(b"vids") {
                        if format.get(16..20) != Some(b"IV50") {
                            return Err("expected Indeo 5 video".into());
                        }
                        if index >= 100 {
                            return Err("unsupported video stream number".into());
                        }
                        picture = Some((
                            [b'0' + index / 10, b'0' + index % 10],
                            word(&format, 4)?,
                            word(&format, 8)?,
                            word(&header, 32)?,
                            word(&header, 24)?,
                            word(&header, 20)?,
                        ));
                    } else if header.starts_with(b"auds") && index < 100 {
                        soundtrack = Some((
                            [b'0' + index / 10, b'0' + index % 10, b'w', b'b'],
                            format,
                            word(&header, 32)?,
                        ));
                    }
                    index = index.checked_add(1).ok_or("too many AVI streams")?;
                    Ok(())
                })?;
            } else if list.starts_with(b"movi") {
                let (stream, _, _) = soundtrack
                    .as_ref()
                    .ok_or("movie data before AVI audio header")?;
                let (video, ..) = picture
                    .as_ref()
                    .ok_or("movie data before AVI video header")?;
                collect_records(&list[4..], stream, video, &mut pcm, &mut holds, 0)?;
            }
            Ok(())
        })?;
        let (_, width, height, frames, rate, scale) = picture.ok_or("AVI has no video track")?;
        let (_, format, samples) = soundtrack.ok_or("AVI has no soundtrack")?;
        if width != 640 || height != 480 || frames == 0 || rate == 0 || scale == 0 {
            return Err("unsupported KQ8 movie dimensions or clock".into());
        }
        let audio = wave_pcm(&format, &pcm, pcm.len() as u64)?;
        if audio.rate == 0 || audio.frames() == 0 || audio.frames() != samples as usize {
            return Err("incomplete AVI soundtrack".into());
        }
        if holds.len() != frames as usize {
            return Err("AVI display intervals differ from the stream header".into());
        }
        Ok(Self {
            width,
            height,
            frames,
            rate,
            scale,
            holds,
            audio,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the original GOG movies"]
    fn all_three_gog_films_have_video_and_a_complete_pcm_track() {
        let game = crate::GameDir::open(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
        for (name, frames, holds, samples) in [
            ("W32opn_1.dll", 1609, 16, 2364663),
            ("W32mdl_1.dll", 255, 18, 373762),
            ("W32ndg_1.dll", 774, 0, 1136116),
        ] {
            let bytes = std::fs::read(game.find(name).unwrap()).unwrap();
            let movie = Movie::read(&bytes).unwrap();
            let seconds = f64::from(movie.frames) * f64::from(movie.scale) / f64::from(movie.rate);
            let audio_seconds = movie.audio.frames() as f64 / f64::from(movie.audio.rate);
            println!(
                "{name}: {} frames, {seconds:.3} s, {} stereo PCM frames ({audio_seconds:.3} s)",
                movie.frames,
                movie.audio.frames()
            );
            assert_eq!(
                (movie.width, movie.height, movie.frames),
                (640, 480, frames)
            );
            assert_eq!((movie.audio.rate, movie.audio.channels), (22050, 2));
            assert_eq!(movie.holds.iter().filter(|&&hold| hold).count(), holds);
            assert_eq!(movie.audio.frames(), samples);
            assert_eq!((movie.scale, movie.rate), (66667, 1000000));
            assert!(movie.audio.peak > 0);
            assert!((15. - f64::from(movie.rate) / f64::from(movie.scale)).abs() < 0.01);
        }
    }
}
