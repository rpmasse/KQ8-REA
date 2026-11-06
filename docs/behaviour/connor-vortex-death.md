# Connor's scripted death at the Temple 4 vortex

## The three-second hold

In Temple 4's `vortrap.cs`, the sword, bow and empty-hand versions of Connor's second vortex animation issue `KQMonster::setState Connor Die Now` when that animation finishes. The original executable schedules the game-over action for **3.0 seconds of simulation time after entering Die**. The script's comment agrees with the executable.

1. On entering Die, Connor's health and health display go to zero. The argument `Now` bypasses selection and playback of the usual character death movement. The previous vortex animation is a separate scripted object. Its end has already triggered the request.
2. With reincarnation off, Connor stays in Die. Gravity, `keepOnGround`, his collision type and his collision mask are turned off. The player loses control through the ordinary `handsOff` behavior. The world continues updating for the three-second hold.
3. The camera continues the shot set by the script. At vortex entry the script selects third person, enables the active camera and stops following anyone. At the first vortex movement's end it cuts to (99990.679688, 94458.148438, 7234.895508), rotation (0, 0, 3.14). The losing branches request no further shot. The `Now` handler requests no camera move or ordinary death animation during the hold.
4. At the scheduled time, the game checks that Connor is still in Die. If so, it clears `handsOff`, dismisses any gameplay popup, marks Connor dead, switches from gameplay to the main menu and applies the normal menu pause. New is selected. Save and Back to Game are disabled. The original menu state and its recovery choices are described in [main-menu.md, rule 25](main-menu.md). If Connor has left Die, the delayed action does nothing.
5. The trap explicitly sets `KQConner::reincarnate` to zero at its start. With that switch enabled elsewhere, the original instead restores Connor's health, resets the meter to full and returns him to Wait. It skips the hold and game-over menu in that branch.

## How to check

For each of `c_suck2.kqs`, `ci_suck2.kqs` and `cg_suck2.kqs`, record the simulation time at the end cue and entry to Die. Check zero health, the disabled collision and physics, and continued world updates in the scripted shot. Check that the first update processing the 3.0-second deadline opens the paused main menu. Assert that the three losing paths never play the victory movie. For ordinary combat deaths, the final recognized death movement's end initiates the hold; `dropweapon` is only the first stage of Fire/Ice death. See [connor-combat-death.md](connor-combat-death.md) (Q192) for state-entry gates, the earlier animation lock versus scripted hands-off and Escape/Save eligibility. The vortex branches explicitly skip the movement stage.

Static executable trace and original Temple 4 data. Original runtime **not tested**. Exact real-time presentation between simulation updates, a save/load in this hold, and general death-camera behavior outside this scripted shot were not checked.
