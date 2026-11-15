# KQ8-REA

KQ8-REA is an unofficial Rust recreation of Sierra's *King's Quest: Mask of Eternity*. It continues work from the [KQ8R project](https://github.com/Eric-lz/KQ8R). The immediate goal is to recreate the original game as faithfully as possible.

We plan to use [REA](https://github.com/morluto/rea) to study the original game and guide a Rust converter. REA has not yet been integrated. The completed recreation will be the foundation for a modernized version.

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
| M7 · Verify and release the Windows recreation | Planned · 0% |

These are estimates for each phase. All 12 game regions load, but the game cannot yet be played from beginning to end. See the [current status](STATUS.md) and [full plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Bring your own game

**No original game files are included.** To build and play the Rust version, you need your own GOG copy of *Mask of Eternity*. Development uses GOG version 1.03.

## What comes next

After the original game is recreated, planned enhancements include modern widescreen and ultrawide resolutions, higher frame rates for animation, movement and camera, and support for other platforms, including macOS.

See [credits and sources](CREDITS.md).
