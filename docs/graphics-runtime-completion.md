# Graphics runtime bridge

Implemented on 2026-11-08. The coordinated build passed in **3m56s** and
the targeted viewer batch passed **11/11** checks, including the five new
model matrices, the new original-data frontend matrix and the corrected
Software fixture. **The final production-default fullscreen gate now passes,
closing M4 at 100%.** The final build passed in 2m47s, three new checks in
0.01s, the affected frontend check in 0.41s and the native route in 3.68s.
Earlier failed attempts and the final console evidence are recorded next.
Earlier component
and windowed-display results in
[graphics-device-controls.md](graphics-device-controls.md) and
[live-display-transitions.md](live-display-transitions.md) remain their own
evidence, not proof of this new native path.

## Final console result and retained investigation

M4 is complete against its planned automated gates. On 2026-11-08, session 2 is the local console,
with a connected VG27A on DISPLAY2, desktop 2560x1440 at 60Hz and advertised
640x480/800x600 modes. No session transfer or driver installation was performed.

| Attempt | Result and evidence |
| --- | --- |
| First console route, 180-frame fixture | FAIL, 6.26s. Exact640 acknowledged, pending800 truncated before its five-second deadline. `live-display-native/1794117975439690400`. |
| Quit-driven fixture (build PASS, 1.96s) | FAIL, 11.89s. Exclusive800 token remained, but actual window/monitor became2560x1440; restoration640 failed too. `live-display-native/1794118345306277000`. |
| Changed-state diagnostic (build PASS, 2m16s) | FAIL, 12.07s. Actual800 reached at+262ms while camera640; by+575ms actual desktop dimensions returned while camera800. All snapshots remained focused and not minimized. Restoration showed the same sequence. `live-display-native/1794118605829582400`. |
| Existing DX12 HWND override, same binary/route | FAIL, 1.46s. Actual 640 acknowledged, then initial swapchain creation failed with 0x887A0001 before any 800 request. `live-display-native/1794118672782384000`. |
| Serial Vulkan diagnostic (build PASS, 1m55s) | FAIL, 12.09s. Sequential main/render execution still returns the display to desktop dimensions. Temporary switch removed. `live-display-native/1794118992710145600`. |
| Existing DX12 Visual override, same route | PASS, 3.22s. All three exact fullscreen confirmations, repeated 800 no-op, menu return/Quit and original-file integrity pass. This predates the new submitted-frame gate. `live-display-native/1794119188155204600`. |
| Final production defaults and submitted-frame gate | PASS, 3.68s (child route 3.23s). No renderer overrides; actual DX12/Visual defaults logged. Exact 640/800/640 plus submitted epochs 1/2/3, repeated 800 no-op, Options/Main/Quit, isolated Glide640 preferences and original Options unchanged. `live-display-native/1794119657462978500`. |

Paths above are under ignored `kq8-artifacts/`; each retains viewer/stdout/
stderr logs and isolated preferences. The ordinary exact-mode assertions
have not been relaxed. Original Options integrity is a final harness
assertion, so it was not reached by these failing routes; the program still
targets isolated writable preferences and reads the installation only.

The changed-state diagnostic logs actual host dimensions, Winit focus and
minimized state, Bevy window/mode/focus and camera target dimensions only
when they change. The Vulkan sequence implicates render reconfiguration
but does not establish the exact driver cause. Sequential rendering did not
fix it. The working DX12 Visual path is now the verified Windows game
default, together with a submitted-frame gate. Camera target agreement alone does not prove
successful presentation, as the failed DX12 HWND initialization demonstrates.

Windows game/ground Auto launches without WGPU_BACKEND choose DX12. Explicit
API/backend choices remain effective. Windows game processes supply Visual
only when WGPU_DX12_PRESENTATION_SYSTEM is absent; inherited values are never
replaced. Model inspection and non-Windows defaults are unchanged. Bevy0.20
exposes presentation selection only through its environment reader, so this
process-local default is set immediately after argument parsing, before
workers. This is independent of the game's logical Software/Direct3D/Glide
profiles and does not change original texture resolution.

Visual uses DirectComposition inside the Winit-owned physical mode. It does
not mean the DXGI swapchain itself owns exclusive fullscreen; Microsoft
documents that distinction for
[CreateSwapChainForComposition](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgifactory2-createswapchainforcomposition).
The acceptance requirement remains actual Winit Exclusive mode and exact
native window/monitor dimensions. Explicit Vulkan and DX12 HWND overrides
retain their observed failures on this host; no automatic resolution or
borderless fallback was added.

`graphics_present.rs` gives each native request/restoration a fresh epoch,
extracts that immutable request into the render world, observes an acquired
surface texture of the requested size before Bevy's render_system, and
acknowledges only after that system consumes it for presentation. The driver
still requires current host/window/camera/palette agreement. This establishes
frame submission, not GPU completion, scanout or original raster fidelity.
Old epochs, wrong-sized textures and acquisition without presentation cannot
close a request. The supplied-host component tests remain model evidence.

Final logs are `m4-final-presentation-build.log`,
`m4-final-presentation-unit.log`, `m4-final-presentation-frontend.log` and
`m4-final-default-native.log` under `kq8-artifacts/logs`. The native viewer
stderr is empty; no viewer child remains, and the desktop returned to
2560x1440. Unrelated passing checks were retained, not replayed. M5/M6 remain
paused and manual playtesting remains deferred.

## Original rules and explicit recreation policy

The source behavior is [graphics-modes.md](behaviour/graphics-modes.md)
(device inventory, startup, selection, confirmation and original sizes)
and [brightness.md](behaviour/brightness.md) (active-surface palette
selection and restoration). Implementation uses those prose notes and
original data; this implementation session did not inspect the executable
or reverse-engineering output.

The original inventory is represented by a declared compatibility profile:

| Launch profile | Declared inventory | Original 800 capability |
| --- | --- | --- |
| `direct3d`, the recreation default | Software and `D3D-display` fullscreen devices | Neither device supports 800. |
| `glide` | Software and a Glide fullscreen device; emulated first board type 0 with 4 MB framebuffer | Active Glide supports 800; Software does not. |

The profile is separate from `--renderer auto|vulkan|dx12`, which selects
the host rendering API. Modern adapter memory, API name and desktop bit
depth do not establish legacy hardware or enumeration. The profiles never
combine Direct3D and Glide in the same inventory. Both still use the
recreation's shared rasterizer. Neither the menu label nor acceptance of a
host mode proves original Software, Direct3D or Glide pixel fidelity.

The Direct3D profile is a declared development default chosen to support
the installed GOG `D3D-display` preference. The original compiled saved
mode-name default remains `Software`.

Normal game launch requests fullscreen. `--windowed` is an explicit
640 × 480 diagnostic using the original **Software window device**, even
when the saved fullscreen name is `D3D-display` or `Glide`. The selected
fullscreen name and active logical device are separate state. Profile
controlled game windows cannot be resized by dragging; the original
Graphics menu owns resolution requests. The standalone model viewer keeps
its previous host-window behavior. These launch/window choices are
recreation policy, not newly established original behavior.

Startup first applies the source-backed saved-name validation: absent
Glide falls back to Software/640; a non-Glide saved name clears the 800
flag. Unknown or unavailable names produce an explicit error. A validated
Glide/800 request cannot use the Software window device: it reports that
640 is required, or that fullscreen should be used. No invalid name is
silently replaced by a newly invented Direct3D device. Validation changes
the in-memory request; it does not write the original or local Options
file just by launching.

## Requests, acknowledgement and failure

`graphics_runtime.rs` owns the declared inventory, selected fullscreen
device, acknowledged active surface and one serial-numbered pending
request. A same-family device choice or already-applied resolution is a
no-op. Retained disabled targets still reach the defensive lower-level
validation. Unsupported sizes or unavailable targets are reported without
silently lowering the requested size.

`graphics_runtime_driver.rs` applies requests on the main thread:

1. Choose an exactly matching 640 × 480 or 800 × 600 advertised monitor
   mode. Among exact sizes, the host policy chooses the refresh rate
   closest to the desktop's and prefers the higher bit depth on a tie.
2. Ask Bevy/Winit to apply that exact exclusive mode, or the explicit
   Software window presentation. A changed Bevy `WindowMode` is only a
   request: Bevy can otherwise fall back to borderless fullscreen.
3. Inspect the real Winit window. Fullscreen acceptance requires actual
   `Exclusive`, matching window pixels, matching monitor pixels and a
   matching mode size. Camera targets and the Bevy window dimensions must
   also agree. Require a matching fresh render epoch whose actual acquired
   texture was submitted for presentation. Borderless is never accepted as
   exclusive evidence; camera readiness alone is insufficient.
4. Apply the target surface's palette through the fallible palette path.
   Only then update active-device state, Graphics markers/capability,
   saved-name preference and interface dimensions. The world is retained;
   this path does not load a new world or replace gameplay state.

Initial simulation waits for native presentation acceptance. Existing
menu ownership pauses gameplay during later Graphics choices. Queued menu
input waits while a host request or restoration is pending.

A host failure deadline of five seconds prevents an indefinitely blocked
request. This is a recreation safeguard, not an original confirmation
dialog or an original timeout rule. A failed live request retains the
preceding palette/preferences, requests restoration of the preceding host
mode and waits for its actual host/camera acknowledgement. Failed initial
activation, or an unacknowledged restoration, exits with a reported error.
Original driver failure behavior is not claimed to match this safeguard.

The locked Winit Windows backend can panic when its display-mode API
fails. Exact-mode prefiltering and recovery from an observed mismatch do
**not** guarantee recovery from that internal panic. The initial unsupported
mode rejection is observed in the first failed native run. Live mismatch
restoration was subsequently attempted on the console and **FAILED** as
recorded above. Winit internal-panic recovery remains **NOT TESTED**.

## Palette and restart integration

Menu entry, brightness changes, film return and world/display restoration
use the active logical surface. A saved fullscreen D3D name cannot change
an active Software window to hardware palette colors. The latter obtains
the twenty Windows static colors through the read-only adapter described
in `system_palette.rs`; the current host's Windows halftone fallback is a
declared adapter behavior, not a game-palette substitution. Palette access
is resolved before image/menu/HUD mutation, and a failed acquisition cannot
count as a successful graphics acknowledgement.

The Direct3D-to-Software confirmation remains the original restart path.
Cancel leaves the device and preferences unchanged. Accept writes the
accepted Software name to the recreation's local Options file and then
exits. A failed local write restores the previous in-memory graphics
preferences, reports the error and leaves the current device usable. It
does not exit or pretend to have live-switched to Software. Original game
files are read-only throughout.

## Automated checks and commands

No manual play or visual inspection is required by this batch. Set the
read-only reference path before running the original-data checks:

```powershell
$env:KQ8_GAME_DIR = "C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG"
```

The final changed/affected group (three new checks plus one frontend check)
was run after one shared release build. Expect 3/3 and 1/1:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer -- graphics_present_ windows_game_renderer_policy_precedence_matrix --test-threads=1 --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer graphics_profile_frontend_live_size_window_palette_and_restart_matrix -- --ignored --test-threads=1 --nocapture
```

The production-default native command below expects WGPU_BACKEND and
WGPU_DX12_PRESENTATION_SYSTEM to be absent from the launching environment.
It logs the selected policy and actual adapter. The recorded final run
temporarily removed inherited overrides and restored them afterward.

The five pure model matrices cover profile/default/startup policy, actual
window-vs-saved-fullscreen identity, live/restart guards, stale and failed
acknowledgement, 800 capability, exact monitor selection and rejection of
borderless or dimension-mismatched host reports:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer graphics_runtime_ -- --nocapture
```

The original-data frontend matrix calls the production menu and palette
handlers with explicitly supplied host replies. It checks actual menu/HUD
Software window colors despite saved D3D, fresh-world palette restoration,
live Software-to-D3D and Software-to-Glide requests, camera gating, the
640/800/640 round trip, repeat-size no-op, and restart Cancel/write-failure/
write-success. Its successful supplied replies would establish connected
frontend behavior, not native-mode acceptance:

```powershell
cargo test --locked --release -p kq8-viewer --bin kq8-viewer graphics_profile_frontend_live_size_window_palette_and_restart_matrix -- --ignored --nocapture
cargo test --locked --release -p kq8-viewer --bin kq8-viewer software_surface_original_data_window_fullscreen_refresh_matrix -- --ignored --nocapture
```

The second command is the corrected Software fixture, which passed on its
targeted retry. Its
original inequality assertion was invalid: Daventry's twenty palette-edge
colors equal this host's Windows halftone colors. It now compares the full
fullscreen palette with the parsed original palette's expected output.
The prior failure remains recorded; no production palette defect was
established by that inequality. The already-passing synthetic palette
check separately uses deliberately different reserved colors.

The one new native route is the existing `live_display` target, adapted
to explicit Glide compatibility and exact fullscreen:

```powershell
cargo test --locked --release -p kq8-viewer --test live_display live_original_exclusive_resolution_round_trip -- --ignored --nocapture
```

It creates an isolated ignored working directory with local
`assignGModeName Glide` / `set800mode 0` and its own save directory, then
launches the production binary with `--display-profile glide --fullscreen`.
Ordinary queued menu input opens Graphics, chooses 800, repeats 800,
returns to 640 and leaves Options. Pending transitions suspend later keys.
The child finishes through ordinary Main-menu Quit/Yes after the route;
a 45-second harness deadline can terminate only its own child. The earlier
180-frame cap was removed because it could truncate pending host acceptance.

Acceptance requires exactly three actual host confirmations in order:
initial640, 800, 640. Each log must report Winit `Exclusive` and matching
window/monitor pixels, with matching submitted-frame epochs 1/2/3. The harness verifies no extra request for repeated
800, menu return, final isolated Glide/640 preferences, successful process
  termination through Quit and unchanged original Options bytes. No world is loaded by
this representative presentation test. It must not be described as a
campaign, animation or legacy-driver fidelity test.

For coordinated builds, the unit and native targets can be compiled once
with `--no-run` and their reported executable paths run directly afterward.
No native launch should overlap another agent's active native test.

## Evidence and remaining limits

- Coordinated runtime/model/frontend/native-target compilation: **PASS**,
  3m56s, run by the coordinating implementation session.
- Five new pure runtime/model matrices: **PASS 5/5**.
- Original-data profile frontend matrix: **PASS 1/1**.
- Corrected Software fixture: **PASS 1/1** on its targeted retry. The
  previous failure remains recorded as an invalid inequality assertion.
- The coordinated targeted viewer batch, including those seven graphics
  checks plus three Talk checks and one help/menu check: **PASS 11/11**.
- Initial exact fullscreen native 640/800/640 route: **FAIL**, first launch,
  1.26s. Startup rejected the unavailable 640 mode and exited with error
  after one rendered frame; no exclusive presentation was acknowledged.
  The isolated Glide/640 preference bytes remained exactly as created by
  the harness, and no viewer process remained after the attempt.
- Final production-default exact fullscreen native route: **PASS**, 3.68s,
  with the new submission gate and all original assertions. See the console
  table above for the intervening failures and successful comparison.
- A rejected unadvertised mode before switching is observed. The later
  console attempts above exercise actual live mismatch restoration, which
  **FAILED**. Winit internal-panic recovery remains **NOT TESTED** and is
  not guaranteed by this bridge.
- Original rasterizer/driver pixel fidelity, full pointer/projection
  equivalence and complete gameplay-state comparison remain separate
  fidelity work. This change does not establish those results.

The native attempt's evidence is under
`kq8-artifacts/live-display-native/1794101649164562000/`, including
`kq8-artifacts/logs/viewer.log`, `stderr.log` and the isolated local Options
file. Its decisive log is:

```text
graphics runtime: request 1 failed: monitor Some("\\\\.\\DISPLAY193") has no exclusive fullscreen 640x480 mode; no borderless fallback was accepted; preferences retained
graphics runtime: initial presentation could not be created; exiting with failure
```

## Read-only host investigation after the native failure

The read-only inventory script at
`kq8-artifacts/scratch/graphics-host-inventory.ps1` queried the same Windows
monitor and mode APIs used by the locked Winit 0.30.13 backend. Its output
is `kq8-artifacts/graphics-host-inventory.json`; session evidence is
`kq8-artifacts/graphics-host-sessions.txt`. Monitor coordinates were queried
with a temporary per-monitor DPI-aware thread context, restored afterward;
no display configuration was changed.

Observed facts:

The inventory was rechecked after the controller/list completion batch at
2026-11-08 03:14 UTC. Session 2, DISPLAY193, and its sole 3600x2096 at 32 Hz
mode remain unchanged, including raw mode enumeration. The earlier JSON is
preserved as `graphics-host-inventory-before-owner-batch.json`; no display
setting was changed and the known-failing native test was not repeated.

- The process is in session **2**, `rdp-tcp#0`, with
  `GetSystemMetrics(SM_REMOTESESSION) = 1`.
- Windows enumerates exactly **one attached monitor**, also the primary:
  `\\.\DISPLAY193`, backed by **Microsoft Remote Display Adapter**.
- Its current and sole advertised mode is **3600 × 2096, 32-bit, 32 Hz**.
  Raw-mode enumeration returns that same single mode.
- `DISPLAY194` through `DISPLAY208` are detached Remote Display Adapter
  placeholders. They have no current mode or attached monitor handle and
  each lists only 800 × 600. None supplies a 640 mode or an alternative
  attached monitor for this test.
- The AMD GPU is available to Vulkan for rendering, as the viewer log
  reports, but this session's display inventory exposes no attached AMD
  output. Rendering-adapter availability and display-mode availability are
  different facts.

The selected monitor is therefore the only available attached target,
not a demonstrated selection bug. An explicit monitor argument or a
different WGPU rendering API cannot select an unexposed physical target.
Microsoft documents the remote-session metric in
[GetSystemMetrics](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsystemmetrics),
and separately documents that display-configuration queries can lack
console access from a remote session in
[QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig).

The next-step recommendation at that historical point was to use the **same unattended harness in a
local console session** with an attached display whose advertised modes
include both 640 × 480 and 800 × 600. Enumerate that session's modes first.
If its primary display lacks either size, select another attached display
with both modes explicitly, or use an appropriately configured display
adapter/test display that actually advertises and accepts both. Any future
monitor-selection option must still require exact supported sizes and real
exclusive acknowledgement. No session transfer, display reconfiguration,
driver installation or second native launch was attempted here.

`--windowed` at 640 × 480 remains an explicit diagnostic host policy for
Remote Desktop; for example, the default Direct3D profile with the saved
`D3D-display`/640 preference uses an actual logical Software window. That
option is not evidence for, or a substitute for, the exact fullscreen
gate, now passed separately on the console. The requirement remains original-size fullscreen without silently
accepting a desktop-sized borderless fallback.
