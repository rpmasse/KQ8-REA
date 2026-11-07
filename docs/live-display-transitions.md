# Live original-size presentation

This M4 batch connects the original Graphics screen's 640 × 480 and
800 × 600 choices to the existing Windows renderer and loaded interface.
Its behavior comes from [graphics-modes.md](behaviour/graphics-modes.md),
especially rules 2, 5–6, 9 and 11–18. It builds on the completed static
geometry/art batch; it does not identify the host renderer as original
Glide, Direct3D or Software.

## Implementation

`display_mode.rs` owns the last applied size and a pending host request.
The Graphics action requests physical window dimensions immediately.
The preference and interface remain at the applied size until a matching
window acknowledgement and matching camera target dimensions arrive.
The open menu keeps gameplay paused, and menu input waits during this
short transition. A repeated request for the same applied or pending
size is a no-op. A supplied unavailable 800 mode is visibly checker-
disabled and cannot be selected by mouse or keyboard.

The capability is explicit input. The current viewer retains its existing
support for both sizes; modern adapter memory and API names do not stand
in for original hardware enumeration. The original renderer's startup
validation, device-family switching and Direct3D-to-DirectDraw restart
modal remain separate work.

An unexpected acknowledged size retains the old interface/preferences,
reports the mismatch and makes one request to restore the previous host
size. This is the study note's compatibility safeguard, not a claim about
original driver-failure semantics or an invented keep-mode dialog. There
is no timeout-based acceptance. Actual native rejection/restoration is
**not tested** by the controlled headless matrix.

The existing menu, Options, Graphics, Keyboard and Save/Load art resize
in place. They retain selection, capture/list/name/modal state and clock
origins; a normal animated menu frame may still advance with real time.
The loaded HUD resolves its immutable authored layout again, retaining
panel, inventory, map, timer and palette ownership. Only its image cache
is invalidated. Status positions were already recalculated each frame;
no extra status-position cache was necessary.

Captions retain text, fonts, palette and speech ownership but rewrap at
320/400 pixels. A separate layout revision invalidates a caption even
when resizing both output and canvas leaves their ratio unchanged.
Newly loaded HUD/caption resources are synchronized to the applied live
size before presentation. Original world textures are unchanged.

Both sizes are 4:3. The existing projection, FOV, camera transform and
game state remain in place. Native output and the camera target must
agree before new pointer coordinates are accepted. The existing logical
window mapping handles host DPI. These are consistency checks of the
recreation's current projection; the original projection and complete
native pointer chain remain unverified in the study note.

## Host API evidence

The pinned dependency is Bevy 0.20.0. Its [WindowResolution API](https://docs.rs/bevy_window/0.20.0/bevy_window/struct.WindowResolution.html)
records requested physical dimensions; it is not a native acceptance
result. Its [WindowResized message](https://docs.rs/bevy_window/0.20.0/bevy_window/struct.WindowResized.html)
and [Camera target dimensions](https://docs.rs/bevy_camera/0.20.0/bevy_camera/struct.Camera.html)
supply the acknowledgment boundary used here. The official
`bevy_winit-0.20.0` crate source confirms that a synchronous
`request_inner_size` result or later native resize callback updates the
window and emits that message. The downloaded dependency archive was
verified against Cargo.lock SHA256
`1ef0a5a55524e793ab13536ef244229fa66a4839fae6564df2aa4b3c13956a82`;
it remains ignored under `kq8-artifacts/display-work/`.

## Grouped automatic check

From the repository root, with the reference install read-only:

```powershell
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_SAVE_DIR = Join-Path $PWD 'kq8-artifacts/display-transition-saves'
cargo test --locked --release -p kq8-viewer --bin kq8-viewer display_transition -- --ignored --nocapture
```

One headless matrix uses the original Graphics layout/tags, original
menu art, Daventry HUD and caption assets, a real Bevy Window and Camera,
and retained Active/DoorSim state. It checks the requested-versus-applied
boundary, delayed camera readiness, 640→800→640 refresh, a caption whose
text and output ratio stay unchanged, the HUD pixel round trip, captured
keyboard row/state, original inclusive row boundary, explicit disabled
eligibility/checker rendering, duplicate selection, unexpected host-size
safeguard, and normalized picking at 100%/150% host scale.

Result on 2026-11-07: **passed on the first attempt**, 1/1 in 0.23 seconds,
after the shared release viewer test build completed in 1 minute 58
seconds. The retained command output is
`kq8-artifacts/display-transition.log`. The test executable is
`target/release/deps/kq8_viewer-4aebd42ecd1f894a.exe`, SHA256
`9d51f90238a283a2353ff5784c2ae7be004b8dcba13510f6db7cc11771f810f0`.
The independent pending tile-output matrix reused this executable; it
required no second compile. There were no compilation or matrix failures
in this batch.

Previously successful static geometry/raster/menu/modal, credits and
ending checks are reused. Cached Save/Load geometry uses its already-
tested resize helper; this matrix does not claim a new save-dialog flow.

## One native acknowledgement flow

After the headless matrix passed, `tests/live_display.rs` ran one bounded
unattended process using the real production executable. It starts the
Main menu, opens Options/Graphics, selects 800, repeats 800, returns to
640, then leaves Graphics/Options. Its queued inputs use the ordinary
handlers; the pending transition suspends further inputs until the real
window event and camera target agree. No game world or movie is loaded.

The process runs from its own ignored working directory with local
`kq8-artifacts/Options.cs` containing `set800mode 0` and a separate save
directory. A 180-frame limit normally exits the viewer; the harness has
a 45-second deadline and can terminate only its own child on timeout.
It compares the original Options bytes after exit, while the surrounding
check also records the original file's SHA256 before and after.

```powershell
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-viewer --test live_display -- --ignored --nocapture
```

The actual run split `--no-run` compilation from direct execution of the
resulting test target so other agents could resume source edits once the
production executable was settled. Build: **passed**, 1 minute 59 seconds,
with two unused-method warnings for `screen_size` getters used only by
the headless matrix. Native check: **passed on the first attempt**, 1/1
in 6.16 seconds; the viewer child ran 5.71 seconds and exited after 180
frames. No test or native flow was repeated.

Evidence:

- Production executable SHA256:
  `ce99711a9e9d5f9bb00a96b89a6ba55dd2799d8e07cbbfde55c49ea77e59bd5f`.
- Build and result logs: `kq8-artifacts/live-display-native-build.log`
  and `kq8-artifacts/live-display-native.log`.
- Isolated run: `kq8-artifacts/live-display-native/1794058351384620600/`.
  Its `kq8-artifacts/logs/viewer.log` records requested 800 → confirmed
  800/revision 1 → requested 640 → confirmed 640/revision 2, then normal
  Graphics-to-Options and Options-to-Main returns. Exactly two requests
  confirm that repeating the selected 800 size was a no-op.
- Final isolated preference: `set800mode 0`. Original Options SHA256
  unchanged before/after:
  `c4f0678e45cf42f8ddacb2e09f165bc65b847fb37fdc7e639b664a8cd540ae14`.
- Process absence was confirmed after the successful run.

This establishes one real Windows/window/camera acknowledgement round
trip on the current renderer. Native rejection/restoration, historical
device enumeration, fullscreen device switching, the original projection
and complete native pointer parity remain **not tested**. There was no
manual test, visual inspection or original executable access.
