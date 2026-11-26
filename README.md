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
| M6 · Recreate the full campaign and saves | In progress · ~97% locally |
| M7 · Verify and release the Windows recreation | Planned · 0% |
| M8 · Modern Improvements | In progress · ~90% |

- **Phase percentages:** These describe each phase, not overall game completion. M6 was revised from ~99.8% because the full campaign and ending remain unfinished.
- **Campaign (M6):** All 12 regions load. Local automated play earned the Ice Crossbow and Fire Gem and returned to Barren's third bridge. Ice World's remaining puzzles, later quests and the ending are unfinished; the latest M6 code is not yet on GitHub main.
- **Combat (M5):** Automated checks cover enemy attacks, elemental shots, deaths, drops and hazards. See the [combat audit](docs/m5-combat-completion-audit.md).
- **Display (M8):** Windowed and fullscreen modes, seven aspect ratios, standard resolutions and a 15–120 Draw FPS cap are available.
- **Graphics and controls (M8):** Anti-aliasing, anisotropic filtering, mouse sensitivity and clickable percentage sliders are available.
- **Keyboard:** W/S/D/A move, R toggles walk/run, Space jumps, Q switches weapons, C centers the camera, E toggles the cursor and Z opens the interface; saved bindings take precedence.
- **Current limits:** The game still updates at 15 Hz even when drawing faster. Fullscreen modes depend on the monitor; native ultrawide output and the higher drawing caps' visual feel are not tested.
- **More detail:** See the [current status](STATUS.md) and [completion plan](docs/DESIGN.md#kq8-rea-completion-plan-2026-11-05).

## Bring your own game

**This project contains no original game files.** You need your own GOG copy of *Mask of Eternity* to build a playable Rust version. Development uses the GOG 1.03 release.

## Future enhancements

After the faithful recreation, we plan to improve animation, movement and camera frame rates and support more platforms, including macOS.

See [credits and sources](CREDITS.md).
