# KQ8-REA

KQ8-REA is an unofficial Rust recreation of Sierra's 1998 game *King's Quest: Mask of Eternity*. It builds on [Eric Peracchi Pisoni's KQ8R project](https://github.com/Eric-lz/KQ8R) and aims to recreate the original game from beginning to end.

The plan is to use [REA](https://github.com/morluto/rea) to study the original game and guide its conversion to Rust. REA has not yet been used in this project. The finished Rust recreation will be the foundation for a modernized edition.

## Project phases

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

These are estimates for each phase, not an overall completion percentage. The current Windows build loads all 12 regions, but a complete playthrough is not yet possible. See the [current status](STATUS.md) and [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Bring your own game

**This repository does not contain original game files.** You need your own GOG copy of *Mask of Eternity* to build a playable Rust version. The project has been tested with GOG version 1.03 and reads its game data from that installation.

On Windows, install Rust and the Visual Studio C++ build tools. From the repository folder, run this in PowerShell with your GOG installation path:

```powershell
$env:KQ8_GAME_DIR = "C:\path\to\King's Quest 8 GOG"
cargo build --locked --release -p kq8-viewer
.\target\release\kq8-viewer.exe --windowed
```

## After the recreation

Future enhancements include modern widescreen and ultrawide resolutions, higher frame rates for animations, movement and camera, and ports to other platforms, including macOS.

See [credits and sources](CREDITS.md).
