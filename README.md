# KQ8-REA

KQ8-REA is an unofficial Rust recreation of Sierra's 1998 game *King's Quest: Mask of Eternity*. It grew from [Eric Peracchi Pisoni's KQ8R project](https://github.com/Eric-lz/KQ8R). The first goal is to match the original game before making enhancements.

## Goal

The plan is to use [REA](https://github.com/morluto/rea) to study the original game and guide its conversion to Rust. REA has not yet been used in this project. The completed Rust version will provide the foundation for a modernized edition.

## Progress

| Phase | Status |
| --- | --- |
| M0 · Read the original game data | Complete · 100% |
| M1 · Recreate the world and graphics | Complete · 100% |
| M2 · Recreate movement and camera | Complete · 100% |
| M3 · Recreate Daventry gameplay | Complete · 100% |
| M4 · Recreate menus, inventory and dialogue | Complete · 100% |
| M5 · Recreate combat | In progress · ~65% |
| M6 · Recreate the full campaign and saves | In progress · ~28% |
| M7 · Compare with the original and release for Windows | Planned · 0% |

Percentages are estimates for each phase. All 12 regions load in the current Windows build, but a complete playthrough is not yet possible. See the [current status](STATUS.md) and [full plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Bring your own game

**This project does not include original game files.** To build and run the Rust version, you need your own GOG copy of *Mask of Eternity*. The project has been checked with GOG version 1.03 and reads game data from that installation.

## Later enhancements

After the recreation is complete, planned improvements include modern widescreen and ultrawide resolutions, higher frame rates for animation, movement and camera, and support for more platforms, including macOS.

See [credits and sources](CREDITS.md).
