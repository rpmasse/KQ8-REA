# KQ8-REA

**KQ8-REA** is an unofficial Rust recreation of Sierra's 1998 game *King's Quest: Mask of Eternity*. It grew from [Eric Peracchi Pisoni's KQ8R](https://github.com/Eric-lz/KQ8R) project. The first goal is to match the original Windows game from start to finish.

The plan is to use [REA](https://github.com/morluto/rea) to help understand the original game and convert its behavior into Rust. REA has not yet been used in this project. The completed Rust recreation will provide the foundation for a modernized edition.

## Progress

| Phase | Status |
| --- | --- |
| M0 · Read original game data | Complete (100%) |
| M1 · Render the world | Complete (100%) |
| M2 · Movement and camera | Complete (100%) |
| M3 · Daventry gameplay | Complete (100%) |
| M4 · Menus, inventory and dialogue | Complete (100%) |
| M5 · Combat | In progress (~65%) |
| M6 · Full campaign and saves | In progress (~28%) |
| M7 · Final comparison and Windows release | Planned (0%) |

Percentages describe each phase, not the whole project. The Windows build loads all 12 regions; a full playthrough is not yet possible. See [current status](STATUS.md) and the [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Build and run on Windows

**This repository contains no original game files.** Bring your own GOG copy of *Mask of Eternity* to build and run the Rust version. The project has been tested with GOG version 1.03.

Install Rust and the Visual Studio C++ build tools. In PowerShell, run these commands from the repository folder, using the path to your GOG installation:

```powershell
$env:KQ8_GAME_DIR = "C:\path\to\King's Quest 8 GOG"
cargo build --locked --release -p kq8-viewer
.\target\release\kq8-viewer.exe --windowed
```

## Future enhancements

After the original game is recreated, planned enhancements include modern widescreen and ultrawide resolutions, higher frame rates for animation, movement and camera, and support for more platforms, including macOS.

See [credits](CREDITS.md) and [developer checks](docs/automated-checks.md).
