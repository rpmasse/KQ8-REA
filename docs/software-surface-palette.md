# Software surface palette

Software output now distinguishes the original fullscreen 8-bit DirectDraw
palette from the window DIB palette. Both clip brightness channels independently;
the window path first substitutes Windows colors at entries 0–9 and 246–255.
Immutable source palettes, image dimensions, indices and transparency are retained.

The Windows adapter reads system palette entries when available. On this modern
desktop that API returns zero. The adapter then asks Windows for a halftone
palette and extracts its twenty static colors; it never invents them from the
game palette. This is a modern platform adapter, not a claim that the host
swapchain runs at 8-bit depth. Acquisition failures retain the prior display
colors and report the failure.

The palette rule comes from Study A's `behaviour/brightness.md` rules 11–12a.
The acquisition API is documented by Microsoft:
[GetSystemPaletteEntries](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getsystempaletteentries)
and [CreateHalftonePalette](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createhalftonepalette).

## Evidence and scope

The data boundary/gain/restoration matrix passes first attempt, **1/1**,
after a **10.71s** build. Log: `kq8-artifacts/logs/m4-software-palette-data.log`.
The two affected hardware-conversion checks also pass **2/2**; log:
`kq8-artifacts/logs/m4-software-palette-affected.log`. The new data matrix was
not repeated.
The Windows-provider and live-image checks pass in the shared viewer run.
The original-data menu/GUI check failed because it incorrectly expected
Daventry's twenty reserved colors to differ from the Windows static colors;
they are equal in the installed data. Replace that inequality with equality
against the original fullscreen source conversion. Its focused retry passes
in the 11/11 viewer batch (`m4-final-admission-graphics-viewer.log`). The initial failure is retained in
`kq8-artifacts/logs/m4-help-map-surface-viewer.log`.

```powershell
cargo test --locked --release -p kq8-data software_window_palette_edges_gain_and_source_restoration_matrix
cargo test --locked --release -p kq8-viewer --bin kq8-viewer software_surface_ -- --include-ignored --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer windows_software_palette_provider_returns_static_edges_without_display_mutation -- --nocapture
```

Set `KQ8_GAME_DIR` to the absolute read-only GOG directory for the original-data
check. Successful commands should report passing tests. No visual inspection or
manual testing is required.

This component selects the palette **within Software presentation**. The
separate logical active-device/fullscreen bridge is now integrated and its
model/frontend checks pass (see graphics-runtime-completion.md). In the
original, the saved renderer name selects the fullscreen device, while the window
device remains Software. A hardware path presented in this viewer's host window
does not prove original window-device fidelity. Hardware palette conversion is
retained; original software rasterization and final driver pixel equivalence are
not established by these palette checks. No texture-resolution changes were made.
