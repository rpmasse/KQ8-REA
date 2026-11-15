# KQ8-REA

KQ8-REA is an unofficial Rust recreation of Sierra's *King's Quest: Mask of Eternity*, continuing the earlier [KQ8R project](https://github.com/Eric-lz/KQ8R). The goal is to match the original game first, then use that foundation for a modernized version.

We aim to use [REA](https://github.com/morluto/rea) to study the original game and guide a Rust converter. REA is planned; it is not yet integrated.

## Progress

| Phase | Status |
| --- | --- |
| M0 · Read game data | Complete · 100% |
| M1 · World and graphics | Complete · 100% |
| M2 · Movement and camera | Complete · 100% |
| M3 · Daventry gameplay | Complete · 100% |
| M4 · Menus, inventory and dialogue | Complete · 100% |
| M5 · Combat | In progress · ~65% |
| M6 · Full campaign and saves | In progress · ~28% |
| M7 · Final verification and Windows release | Planned · 0% |

These estimates describe each phase, not overall game completion. All 12 regions load, but a start-to-finish playthrough is not yet possible. See the [current status](STATUS.md) and [full plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Bring your own game

**This repository contains no original game files.** You need your own GOG copy of *Mask of Eternity* to build and run the Rust version. Development uses GOG version 1.03.

## After the recreation

Planned enhancements include modern widescreen and ultrawide resolutions, higher frame rates for animation, movement and camera, and portability to other platforms, including macOS.

See [credits and sources](CREDITS.md).
