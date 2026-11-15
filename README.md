# KQ8-REA

KQ8-REA continues the [KQ8R project](https://github.com/Eric-lz/KQ8R) to recreate Sierra's *King's Quest: Mask of Eternity* in Rust. First, we aim to match the original Windows game. That version will be the foundation for later enhancements.

We plan to use [REA](https://github.com/morluto/rea) to study the original game and guide its conversion to Rust. REA is not yet integrated.

## Progress

| Phase | Status |
| --- | --- |
| M0 · Read original game data | Complete · 100% |
| M1 · Recreate worlds and graphics | Complete · 100% |
| M2 · Recreate movement and camera | Complete · 100% |
| M3 · Recreate Daventry gameplay | Complete · 100% |
| M4 · Recreate menus, inventory and dialogue | Complete · 100% |
| M5 · Recreate combat | In progress · ~65% |
| M6 · Recreate the full campaign and saves | In progress · ~28% |
| M7 · Verify and release the Windows recreation | Planned · 0% |

Percentages apply to each phase, not the whole game. All 12 regions load, but the campaign and visual fidelity are unfinished. See the [current status](STATUS.md) and [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Bring your own game

**No original game files are included.** Bring your own GOG copy of *Mask of Eternity* to build and run the Rust version. Development uses GOG version 1.03.

## Future enhancements

After the recreation, we plan to add modern widescreen and ultrawide resolutions, higher frame rates for animation, movement and camera, and support for other platforms, including macOS.

See [credits and sources](CREDITS.md).
