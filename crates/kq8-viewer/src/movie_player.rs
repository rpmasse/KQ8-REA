//! Playback of the original Indeo movies. Rust keeps the AVI clock and
//! PCM audio. A local FFmpeg process decodes the video into a bounded queue.
use kq8_data::{aud::Sound, avi::Movie};
use std::{
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        mpsc::{sync_channel, Receiver, TryRecvError},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub struct Playback {
    child: Child,
    video: Mutex<Receiver<Result<Vec<u8>, String>>>,
    pub audio: Arc<Sound>,
    pub frames: u32,
    pub shown: u32,
    pub decoded: u32,
    holds: Vec<bool>,
    frame_seconds: f64,
    pub started: Option<Instant>,
}

pub enum Update {
    Wait,
    Frame(Vec<u8>),
    Finished,
}

fn decoder() -> PathBuf {
    if let Some(path) = std::env::var_os("KQ8_FFMPEG") {
        return path.into();
    }
    if let Ok(exe) = std::env::current_exe() {
        let adjacent = exe.with_file_name("ffmpeg.exe");
        if adjacent.is_file() {
            return adjacent;
        }
    }
    "ffmpeg".into()
}

impl Playback {
    pub fn open(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let movie = Movie::read(&bytes)?;
        let mut command = Command::new(decoder());
        // Skip only the short repeat records validated by the AVI reader.
        // Rust presents those at their original positions. Other decoding
        // errors must fail, and raw output must not add duplicate pictures.
        command
            .args([
                "-nostdin",
                "-v",
                "error",
                "-xerror",
                "-threads",
                "1",
                "-bsf:v",
                r"noise=drop=lte(size\,2)",
                "-i",
            ])
            .arg(path)
            .args([
                "-map",
                "0:v:0",
                "-an",
                "-sn",
                "-dn",
                "-fps_mode",
                "passthrough",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgba",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let child = command
            .spawn()
            .map_err(|e| format!("opening FFmpeg for {}: {e}", path.display()))?;
        Ok(Self::from_decoder(movie, child))
    }

    /// The same bounded reader serves the live decoder and controlled child
    /// processes used to verify media failure cleanup without replaying films.
    pub(crate) fn from_decoder(movie: Movie, mut child: Child) -> Self {
        let frame_bytes = movie.width as usize * movie.height as usize * 4;
        let mut output = child.stdout.take().expect("piped FFmpeg stdout");
        let (send, recv) = sync_channel(2);
        std::thread::spawn(move || loop {
            let mut frame = vec![0; frame_bytes];
            let read = match output.read(&mut frame) {
                Ok(0) => break,
                Ok(count) => output.read_exact(&mut frame[count..]),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => Err(e),
            };
            match read {
                Ok(()) => {
                    if send.send(Ok(frame)).is_err() {
                        break;
                    }
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    let _ = send.send(Err("FFmpeg truncated an RGBA picture".into()));
                    break;
                }
                Err(e) => {
                    let _ = send.send(Err(e.to_string()));
                    break;
                }
            }
        });
        Self {
            child,
            video: Mutex::new(recv),
            audio: Arc::new(movie.audio),
            frames: movie.frames,
            shown: 0,
            decoded: 0,
            holds: movie.holds,
            frame_seconds: f64::from(movie.scale) / f64::from(movie.rate),
            started: None,
        }
    }

    pub fn hold_count(&self) -> usize {
        self.holds.iter().filter(|&&hold| hold).count()
    }

    pub fn step(&mut self, now: Instant) -> Result<Update, String> {
        let elapsed = self
            .started
            .map_or(Duration::ZERO, |s| now.saturating_duration_since(s))
            .as_secs_f64();
        let due = ((elapsed / self.frame_seconds) as u32)
            .saturating_add(1)
            .min(self.frames);
        let mut last = None;
        while self.shown < due {
            if self.holds[self.shown as usize] {
                self.shown += 1;
                continue;
            }
            match self.video.get_mut().unwrap().try_recv() {
                Ok(frame) => {
                    last = Some(frame?);
                    self.shown += 1;
                    self.decoded += 1;
                    self.started.get_or_insert(now);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let status = self.child.try_wait().map_err(|e| e.to_string())?;
                    return Err(format!(
                        "FFmpeg ended after {} of {} intervals (status {status:?})",
                        self.shown, self.frames
                    ));
                }
            }
        }
        if let Some(frame) = last {
            return Ok(Update::Frame(frame));
        }
        let audio_end = self.audio.frames() as f64 / f64::from(self.audio.rate);
        let end = (f64::from(self.frames) * self.frame_seconds).max(audio_end);
        if self.shown == self.frames && elapsed >= end {
            match self.video.get_mut().unwrap().try_recv() {
                Ok(_) => return Err("FFmpeg produced more pictures than the AVI contains".into()),
                Err(TryRecvError::Disconnected) => {
                    if let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? {
                        if !status.success() {
                            return Err(format!("FFmpeg exited with {status}"));
                        }
                        return Ok(Update::Finished);
                    }
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        Ok(Update::Wait)
    }
}

impl Drop for Playback {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the original GOG files and KQ8_FFMPEG"]
    fn original_movies_display_every_interval_at_the_stream_clock() {
        let game = kq8_data::GameDir::open(std::env::var_os("KQ8_GAME_DIR").unwrap()).unwrap();
        for (name, pictures) in [
            ("W32opn_1.dll", 1593),
            ("W32mdl_1.dll", 237),
            ("W32ndg_1.dll", 774),
        ] {
            let mut m = Playback::open(&game.find(name).unwrap()).unwrap();
            let timeout = Instant::now() + Duration::from_secs(90);
            let origin = Instant::now();
            for i in 0..m.frames {
                let at = origin
                    + Duration::from_secs_f64(f64::from(i) * m.frame_seconds)
                    + Duration::from_micros(u64::from(i != 0));
                // No clock advance, new image or decoder consumption until due.
                if i > 0 {
                    assert!(matches!(
                        m.step(at - Duration::from_micros(2)).unwrap(),
                        Update::Wait
                    ));
                }
                loop {
                    match m.step(at).unwrap() {
                        Update::Frame(pixels) => {
                            assert!(!m.holds[i as usize]);
                            assert_eq!(pixels.len(), 640 * 480 * 4);
                            assert_eq!(m.shown, i + 1);
                            break;
                        }
                        Update::Wait if m.shown == i + 1 => {
                            assert!(m.holds[i as usize]);
                            break;
                        }
                        Update::Wait => {
                            assert!(Instant::now() < timeout, "waiting for {name} picture {i}");
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        Update::Finished => panic!("{name} finished before interval {i}"),
                    }
                }
            }
            assert_eq!(m.decoded, pictures);
            let duration = (f64::from(m.frames) * m.frame_seconds)
                .max(m.audio.frames() as f64 / f64::from(m.audio.rate));
            let end = origin + Duration::from_secs_f64(duration);
            assert!(matches!(
                m.step(end - Duration::from_millis(1)).unwrap(),
                Update::Wait
            ));
            loop {
                match m.step(end + Duration::from_micros(1)).unwrap() {
                    Update::Finished => break,
                    Update::Wait => {
                        assert!(Instant::now() < timeout);
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Update::Frame(_) => panic!("unexpected final picture"),
                }
            }
        }
    }
}
