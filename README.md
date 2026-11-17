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
| M6 · Recreate the full campaign and saves | In progress · ~28% |
| M7 · Verify and release the Windows recreation | Planned · 0% |
| M8 · Modern Improvements | In progress · ~90% |

These percentages describe individual phases, not overall game completion. All 12 regions load; campaign progression and final fidelity checks still need work. See the [current status](STATUS.md) and [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

M8 is being done early: Graphics offers View Mode, Aspect Ratio and Resolution, and mouse camera movement is gentler. Fullscreen uses an available monitor mode and may update the selected aspect ratio and resolution; native ultrawide output is not yet tested. The original game still runs at 15 updates per second.

Automated combat checks cover enemy attacks, both camera views for melee, elemental shots, distinct deaths and drops, terrain hazards, and the Ice Dragon's chain appearance; the full campaign remains unfinished.
The [M5 combat audit](docs/m5-combat-completion-audit.md) records the completed phase and its remaining original-game comparison limits.

## Bring your own game

**This project contains no original game files.** You need your own GOG copy of *Mask of Eternity* to build a playable Rust version. Development uses the GOG 1.03 release.

## Future enhancements

After the faithful recreation, we plan to improve animation, movement and camera frame rates and support more platforms, including macOS.

See [credits and sources](CREDITS.md).
