# Azriel's Hammer hit and return

This unattended Windows check uses the player's Dead City game data and an
original Skeleton. It tests the Hammer's hit, return and catch, along with
the menu lock during the throw.

From the repository folder in PowerShell:

```powershell
$env:KQ8_GAME_DIR = "C:\path\to\King's Quest 8 GOG"
cargo test -p kq8-viewer --test hammer_hit_return -- --ignored --nocapture
```

Expected result: `1 passed; 0 failed`. On this machine it passed in 34.54
seconds. One Hammer hit reduced `skeleton1` from 60 to 23 health. The
projectile turned back on that contact, returned to Connor and was removed.
Connor's catch ended and released the input lock at update 76. A menu
request at update 63 was blocked; the menu saved successfully at update
80 with the Hammer ready and no projectile in flight.

Dead City's start script gives a Small Crossbow, so the fixture gives the
Hammer afterward through the ordinary item path. It disables enemy AI and
uses god mode to isolate this throw. A separate Daventry exploratory run
reached the floor-grid return; the existing core check covers range and
floor turn-back numerically. The normal menu cannot save during the Hammer
flight because the throw holds the input lock. This check does not compare
the original executable or cover an earned campaign encounter. No manual
testing was performed.
