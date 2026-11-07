# Original interface resolutions

This M4 batch implements the shared 640×480 and 800×600 interface
geometry/art rules from [graphics-modes.md, rules 11–18](behaviour/graphics-modes.md).
It does not implement native device enumeration, backend switching or
the Graphics screen's live-resolution action.

Authored GUI rectangles remain unchanged. A separate resolved tree
normalizes each child against its original parent and converts its edges
against that parent's new integer dimensions, adding 0.1 and truncating.
Converted dialog origins are then added as integers. Drawing and input
use these resolved rectangles; the returned tree is never used as the
source of another resize.

Shared GUI fonts have their PFT horizontal/vertical scale fields reset
to 1. Glyph data and spacing remain original. Unstretched control
pictures retain their own pixel dimensions; stretched pictures fill
their resolved rectangles. Caption widths remain 320/400 pixels, with
fixed rows and tiled stone. Map content transforms and fixed-size status
icons retain their separate behavior.

Main, Options, Graphics, Keyboard, Save and Load now render canvases at
the selected logical resolution. The startup-menu presentation and input
mapping use the current canvas dimensions rather than assuming 640×480.
This also leaves independent movie/credits canvases available to their
own presentation rules.

Main and Save/Load backgrounds select the matching original-sized source
bitmap without renaming files. Save/Load's named slot/editing controls
retain their 640-layout sizes and are centered by (+80,+60) at 800.
The title and popup controls remain on the general layout path. Save
also centers BAR and RED. Thumbnails remain 160×120.

## Grouped checks

From the repository root:

```powershell
cargo test --locked --release -p kq8-core gui_draw::tests -- --nocapture
$env:KQ8_GAME_DIR = (Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
cargo test --locked --release -p kq8-core original_interface_menu_matrix -- --ignored --nocapture
cargo test --locked --release -p kq8-core original_interface_modal_matrix -- --ignored --nocapture
cargo check --locked --release -p kq8-viewer
```

The geometry/raster family includes nested controls with fractional
edges, negative-coordinate truncation, immutable 640→800→640 conversion,
unscaled PFT glyphs, native/stretched picture footprints, captions,
meters, transparency and existing map drawing checks.

The original-data menu matrix covers all six menu types, resolved input
rectangles, Graphics' inclusive lower row boundary, Save/Load's centered
controls and uncentered exceptions, original background dimensions,
thumbnail size/placement, slot selection/scrolling and identical rendered
pixels when returning to 640.

Review found a separate Main-menu modal drawing path that had not passed
its authored picture control to the shared renderer. The correction now
uses control 1004's native-size `gm_win2.pbm` and the normal/highlight
button controls 1006/1007's stretched-picture flags. The original runtime
button positions remain unchanged (`main-menu.md`, rule 14). A targeted
original-data matrix covers two- and three-button dialogs at both sizes,
normal/highlight/pressed raster strips and resolved click dispatch. At
800 the warning image stays 360×120 pixels at (175,225), while its
button pictures fill their resized rectangles.

Results on 2026-11-07:

- `gui_draw::tests`: **7 passed, 0 failed**, 0.00 seconds after a
  57.88-second core build. Includes the grouped geometry/raster matrix
  and the affected existing drawing regressions.
- `original_interface_menu_matrix`: **1 passed, 0 failed**, 0.48 seconds;
  reused the compiled core (Cargo finished in 0.28 seconds).
- `cargo check --locked --release -p kq8-viewer`: **passed**, 22.75 seconds.
- `original_interface_modal_matrix`: **1 passed, 0 failed**, 0.04 seconds;
  reused the settled core build from the parallel scripted-character
  batch (Cargo finished in 0.21 seconds). Earlier successful geometry,
  six-menu and viewer checks were not repeated for this narrow fix.

Logs are `kq8-artifacts/interface-geometry.log`,
`kq8-artifacts/interface-original-menu.log` and
`kq8-artifacts/interface-viewer-check.log`; the review correction is in
`kq8-artifacts/interface-original-modal.log`. There were no failed attempts
and no successful check was repeated. Viewer compilation was a check,
not an executable rebuild. No native game launch or manual visual
inspection was performed.

The inherited 800-mode meter assertion expected 17 rows because it
rounded the authored top 466 to 583. The original rule converts 582.5
to 582, so the corrected extent is 18 rows through pixel 599. The
affected drawing regression now checks that extent.

## Remaining integration and limits

The saved resolution flag chooses logical interface dimensions on load.
Changing that flag in Graphics still does not perform the original live
window/device/GUI resize; that remains a separate integration gate with
renderer availability, startup validation and the Direct3D-to-DirectDraw
restart modal. This batch does not claim original 3D projection, Windows
pointer conversion, cursor-hotspot or failed-device recovery equivalence.
Credits timing/scoring/layout remains separately owned work.

Automated pixel checks establish data-driven geometry, not visual parity
against a running original executable. The original GOG files are read
only and are never renamed or modified.
