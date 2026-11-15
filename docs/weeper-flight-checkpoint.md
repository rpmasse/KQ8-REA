# Weeper flight across a disk save

The Windows check uses the player's GOG Dead City data. It saves while the
first Weeper bolt is still flying, loads that save in a new process, and
compares the completed drain with a continuous run.

From the repository folder in PowerShell:

```powershell
$env:KQ8_GAME_DIR = "C:\path\to\King's Quest 8 GOG"
cargo test --locked -p kq8-viewer --test weeper_flight_checkpoint -- --ignored --nocapture
```

Expected result: `1 passed; 0 failed`. On this machine the check passed in
122.81 seconds. Slot 1 held `bolt1` at flight update 2, before collision and
with Connor at 80 health. The fresh process hit on its first world update,
created eight links, applied pulses 1 through 12 exactly once, ended the
drain once and saved Connor at 68 health with no active bolt or drain.
The shot count and final combat state matched an uninterrupted route saved
at the same world update.

The fixture clears only the two Dead City intro traps' Connor collision
bits; otherwise the test teleport triggers their scripts before the Weeper
fires. The first test run failed on an assertion comparing JSON `3.0` with
integer `3`. Correcting the assertion produced the passing result.

This checks the Rust version using original data. It does not establish
the original executable's midflight save behavior, cover other projectile
types or prove a playable campaign encounter. No manual testing was done.
