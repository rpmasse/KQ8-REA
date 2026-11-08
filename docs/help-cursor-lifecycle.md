# Help timing, cursor/map geometry and Graphics lifetime

This grouped M4 change uses Study A's `pickup-tutorials.md`,
`in-game-help.md`, `interface.md`, `magic-map.md` and `graphics-modes.md`.
The implementation session did not inspect the original executable.

Help notices use simulation time for their initial delays, staged death
notice and delivery retries. First hit, low health, ordinary enemy death
and standing-jump completion have separate receipt gates and one-shot bits.
Accepted combat hits and ordinary enemy death entry feed the scheduler;
pickup and restored-history paths use the same time source. Pausing at
the Main menu leaves these deadlines unchanged. The standing-jump hook's
shared movement boundary is also covered by the pending Talk batch.

The original cursor uses a top-left anchor, each frame's transparency and
a 100 ms real-time frame clock. The 640/800 display ratio is fractional.
Map content, arrow, pan and zoom remain display-pixel quantities, with the
original integer rounding and absolute follow limits; only the GUI area
scales. The rendered map position is retained after drag without waiting
for another simulation update.

The Graphics screen retains focus and resolved targets across Back,
Options and reopening Main. Keyboard activation temporarily places and
restores the pointer as studied; pointer release uses its current location.
These screen-lifetime checks do not prove the separate active-device and
exclusive-fullscreen integration.

## Automated evidence

The first shared build failed at a private `Placement` import in a new
test. After correcting that import, the release test build passed in
4 minutes 26 seconds. Both logs are retained:
`m4-admission-help-surface-build.log` and
`m4-admission-help-surface-build2.log` under `kq8-artifacts/logs/`.

The grouped core run passed **13/13 in 0.01 seconds**, covering tutorial
receipt/staging/delivery/history, accepted hit/death sources, original
Graphics defaults and map geometry. The viewer run passed **5/6 in
0.91 seconds**, including original-data cursor/map, actual popup
Escape/Main/Return actions, retained Graphics lifetime, immutable live-image
palette rebuilding and the Windows palette provider. Logs:

- `kq8-artifacts/logs/m4-help-map-default-core.log`
- `kq8-artifacts/logs/m4-help-map-surface-viewer.log`

The sixth viewer check incorrectly required Daventry's reserved palette
colors to differ between Software window and fullscreen. The installed
palette already matches the Windows static colors. The correction compares
fullscreen output to the original source palette instead; its retry remains
pending with the graphics runtime batch. The passing synthetic palette
test already covers genuinely different source/system colors.

These are unattended component and original-data checks. They do not
establish original native save-event equivalence, full Windows DPI/pointer
behavior, final legacy-driver pixels, or actual exclusive-fullscreen mode
acceptance. The help frontend check exercises real menu actions and the
production playing/running gate, but omits outer Connor admission and GPU
capture. No manual or new native game run was performed for this batch.

## Commands

From the repository, set `KQ8_GAME_DIR` to the absolute read-only GOG
installation and `KQ8_SAVE_DIR` to an isolated ignored scratch directory.
The shared compile command is:

```powershell
cargo test --locked --release -p kq8-core -p kq8-viewer --no-run --lib --bins --test conversation_checkpoint
```

The focused new checks can be selected with:

```powershell
cargo test --locked --release -p kq8-core non_pickup_tutorial_gate_stage_clock_and_pause_matrix -- --nocapture
cargo test --locked --release -p kq8-core tutorial_sources_use_accepted_hit_and_ordinary_death_entry_matrix -- --nocapture
cargo test --locked --release -p kq8-core map_display_coordinate_follow_drag_zoom_matrix -- --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer original_cursor_map_display_coordinate_and_frame_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer help_escape_main_return_and_game_time_wait_matrix -- --include-ignored --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer graphics_runtime_retained_screen_focus_and_modal_pointer_matrix -- --include-ignored --test-threads=1 --nocapture
```

Each command should report a passing check. The recorded grouped run used
the compiled test executables to combine these selectors with their affected
existing checks. Successful checks are retained rather than repeated.
