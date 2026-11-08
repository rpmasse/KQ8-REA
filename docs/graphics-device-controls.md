# Graphics-device policy and controls

The original Graphics rules now have a shared core model taking an explicit
device inventory and active device. It implements fixed renderer-family rows,
fullscreen eligibility, Glide's suppression of Direct3D, the excluded
`D3D-dd3dfx` entry, last eligible Direct3D selection and the original 800-mode
capabilities. Selection markers and duplicate-selection rejection compare
renderer families, even when a different Direct3D driver name was retained.
Startup name/resolution validation preserves the note's unresolved cases
instead of inventing a fallback for unknown Direct3D names or empty lists.

The Graphics screen can consume that inventory and emit typed accepted
device actions. Ordinary selections await a presentation-owner acknowledgement
before changing the active marker or saved preferences. Direct3D to Software
opens the original confirmation with Cancel selected. Its known keyboard and
pointer behavior, sounds, question splitting, runtime geometry and pressed
picture/caption are implemented. Cancel returns to Graphics; Escape goes to
Options; Yes emits a restart action only on release. Opening the dialog
does not itself authorize the application exit action.

## Grouped evidence, 2026-11-08

The policy matrix passes first attempt, **1/1**. The original-data GUI
matrix initially failed because the fixture expected the TTAG placeholder
instead of the installed message resolved by tag 169. The implementation
already resolved the correct original message. The fixture now checks its
106-character length and independent split boundaries (37, 33 and 34
characters), reconstructing the full resolved message without embedding it
in source. The corrected GUI matrix passes **1/1 in 0.05 s**. The successful
policy matrix was not repeated.

The GUI matrix covers original device captions, selected/disabled pixels,
navigation at 640/800, request/acknowledgement separation, modal defaults,
sounds, input locking, Enter and mouse press/release, pointer coordinates,
Yes/Cancel/Escape and pressed button pixels from the installed artwork.
The popup-specific cases use its established 640 path; unavailable forced
800 popup behavior is not inferred.

```powershell
cargo test --locked --release -p kq8-core graphics_device_ -- --include-ignored --nocapture
```

Set `KQ8_GAME_DIR` to the absolute read-only GOG path. Retained failure and
success logs: `kq8-artifacts/logs/m4-graphics-device.log` and
`m4-graphics-device-corrected.log`. The retry compiled in 1m07s. The production
Windows build also succeeds in 2m33s, retaining two existing unused-method
warnings.

## Open production gates

This is a tested core/frontend component, **not completed device switching
in the running game**. The viewer still needs to supply an explicit supported
device inventory, consume live/restart decisions, route modal sounds/pointer
events, validate startup settings and acknowledge palette/presentation
updates. WGPU adapter enumeration cannot establish legacy driver availability.
Original renderer-family palette/raster behavior and fullscreen acceptance
remain separate open gates. No native graphics-device run or manual testing
was performed, and the successful 640/800 resize checks were not repeated.

Source: `behaviour/graphics-modes.md` rules 1–10 and its Q207 modal addendum,
written by the separate original study session, plus the installed GUI,
message, font and picture resources. No original executable was read by B.
