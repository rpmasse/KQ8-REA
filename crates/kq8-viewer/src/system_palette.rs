//! Read-only Windows adapter for the twenty static colors of an 8-bit DIB.
//! The original palette rule is brightness.md 12/12a. The host acquisition
//! is our adapter: modern non-palette displays use Windows' halftone palette.
use kq8_data::palette::{Output, Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    System,
    WindowsHalftone,
}

pub(crate) fn for_software_surface(output: Output, windowed: bool) -> Result<(Output, Option<Source>), String> {
    if !output.is_software() { return Ok((output, None)); }
    if !windowed { return Ok((Output::Software, None)); }
    let (system, source) = read()?;
    Ok((Output::SoftwareWindow { system }, Some(source)))
}

#[cfg(not(windows))]
pub(crate) fn read() -> Result<([Rgb; 20], Source), String> {
    Err("the original Software window palette requires Windows system colors".into())
}

#[cfg(windows)]
pub(crate) fn read() -> Result<([Rgb; 20], Source), String> {
    use std::ffi::c_void;
    type Handle = *mut c_void;
    #[derive(Clone, Copy, Default)]
    #[repr(C)]
    struct Entry { red: u8, green: u8, blue: u8, flags: u8 }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetDC(window: Handle) -> Handle;
        fn ReleaseDC(window: Handle, dc: Handle) -> i32;
    }
    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn GetSystemPaletteEntries(dc: Handle, start: u32, count: u32, entries: *mut Entry) -> u32;
        fn CreateHalftonePalette(dc: Handle) -> Handle;
        fn GetPaletteEntries(palette: Handle, start: u32, count: u32, entries: *mut Entry) -> u32;
        fn DeleteObject(object: Handle) -> i32;
    }
    struct Dc(Handle);
    impl Drop for Dc {
        fn drop(&mut self) {
            // SAFETY: this live desktop DC was obtained by GetDC(NULL).
            unsafe { ReleaseDC(std::ptr::null_mut(), self.0); }
        }
    }
    struct Palette(Handle);
    impl Drop for Palette {
        fn drop(&mut self) {
            // SAFETY: this is our owned, unselected logical palette.
            unsafe { DeleteObject(self.0); }
        }
    }
    // SAFETY: NULL requests a desktop DC; it does not alter display state.
    let dc = Dc(unsafe { GetDC(std::ptr::null_mut()) });
    if dc.0.is_null() { return Err("Windows did not provide a palette device context".into()); }
    let mut entries = [Entry::default(); 20];
    // SAFETY: the disjoint output slices each contain ten PALETTEENTRYs.
    let system = unsafe {
        GetSystemPaletteEntries(dc.0, 0, 10, entries.as_mut_ptr()) == 10
            && GetSystemPaletteEntries(dc.0, 246, 10, entries[10..].as_mut_ptr()) == 10
    };
    let source = if system { Source::System } else {
        // Windows documents this palette as including the twenty static
        // system colors. This keeps the platform-supplied colors on a host
        // without a physical palette; no game-palette fallback is invented.
        // SAFETY: dc remains alive for this call.
        let palette = Palette(unsafe { CreateHalftonePalette(dc.0) });
        if palette.0.is_null() { return Err("Windows did not provide a halftone palette".into()); }
        // SAFETY: palette is valid and each destination has ten entries.
        let copied = unsafe {
            GetPaletteEntries(palette.0, 0, 10, entries.as_mut_ptr()) == 10
                && GetPaletteEntries(palette.0, 246, 10, entries[10..].as_mut_ptr()) == 10
        };
        if !copied { return Err("Windows returned incomplete static palette colors".into()); }
        Source::WindowsHalftone
    };
    Ok((entries.map(|e| [e.red, e.green, e.blue]), source))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn windows_software_palette_provider_returns_static_edges_without_display_mutation() {
        let (colors, source) = read().expect("Windows system colors");
        assert_eq!(colors[0], [0, 0, 0]);
        assert_eq!(colors[19], [255, 255, 255]);
        assert!(colors.iter().any(|&c| c != colors[0] && c != colors[19]));
        assert_eq!(for_software_surface(Output::Software, false), Ok((Output::Software, None)));
        assert_eq!(for_software_surface(Output::Glide, true), Ok((Output::Glide, None)));
        assert_eq!(for_software_surface(Output::Direct3D, true), Ok((Output::Direct3D, None)));
        assert!(matches!(for_software_surface(Output::Software, true), Ok((Output::SoftwareWindow { .. }, Some(_)))));
        println!("Software DIB: 20 Windows static colors via {source:?}; fullscreen and hardware paths retained");
    }
}
