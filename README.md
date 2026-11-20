# KQ8-REA

KQ8-REA continues the [KQ8R project](https://github.com/Eric-lz/KQ8R), which recreates Sierra's *King's Quest: Mask of Eternity* in Rust. The first goal is to match the original Windows game. That faithful version will be the foundation for a modernized game later.

We aim to use [REA](https://github.com/morluto/rea) to help understand the original game and guide its conversion to Rust. REA is planned but not yet integrated.

## Progress

| Phase | Status |
| --- | --- |
| M0 · Read the original game data | Complete · 100% |
| M1 · Recreate the worlds and graphics | Complete · 100% |
| M2 · Recreate movement and camera | Complete · 100% |
| M3 · Recreate Daventry gameplay | Complete · 100% |
| M4 · Recreate menus, inventory and dialogue | Complete · 100% |
| M5 · Recreate combat | Complete · 100% |
| M6 · Recreate the full campaign and saves | In progress · ~97% |
| M7 · Verify and release the Windows recreation | Planned · 0% |
| M8 · Modern Improvements | In progress · ~90% |

These percentages describe individual phases, not overall game completion. All 12 regions load; campaign progression and final fidelity checks still need work. See the [current status](STATUS.md) and [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

M8 is being done early: Graphics offers windowed/fullscreen modes, seven aspect ratios including 3:2, standard resolution presets and a 15–120 Draw FPS cap. Options → Mouse offers camera sensitivity; brightness, sensitivity and volume bars support clicking and show percentages. Drawing can run faster than the game's unchanged 15 updates per second. If a saved fullscreen mode disappears, startup opens a temporary 640×480 window. Fullscreen resolutions depend on the monitor's available modes; native ultrawide output and the new cap's visual feel are not tested.

Graphics also offers Off, SMAA, and FXAA anti-aliasing for the 3D view; its selection is saved locally without changing game assets.

Graphics offers Off, 4x, 8x, and 16x anisotropic filtering for 3D surface textures; 4x is the default and original texture files are unchanged.

The default keyboard layout is W/S/D/A to move, R to walk or run, Space to jump, Q to switch weapons, C to center the camera, E to toggle the cursor, and Z to toggle the interface; saved custom bindings take precedence.

Automated combat checks cover enemy attacks, both camera views for melee, elemental shots, distinct deaths and drops, terrain hazards, and the Ice Dragon's chain appearance; the full campaign remains unfinished.
The automated campaign walker has completed Dead City's key quests, defeated the Skeleton King, crossed the lowered drawbridge, and reached Swamp with its inventory and quest progress saved; later campaign routes remain unfinished.
The [M5 combat audit](docs/m5-combat-completion-audit.md) records the completed phase and its remaining original-game comparison limits.

## Bring your own game

**This project contains no original game files.** You need your own GOG copy of *Mask of Eternity* to build a playable Rust version. Development uses the GOG 1.03 release.

## Future enhancements

After the faithful recreation, we plan to improve animation, movement and camera frame rates and support more platforms, including macOS.

See [credits and sources](CREDITS.md).
