# Rock projectile provenance

Study A's [rock-projectiles.md](../behaviour/rock-projectiles.md) supplies
Rock-specific aiming, direct and script-owned turning, launch-part placement,
two-segment flight progression, contact/fall/floor rules and Die-time recovery.
The installed prrock.kq, Conner.anm, c_rktoss.kqs and rockHit.cs supply the
parameters, cue frame, full part transform and item callback. The existing
shared sweep supplies contact normals; this change does not establish a
more precise geometry solver.

Implementation B uses those prose rules and original data only. Defined
falling, missing-part and malformed-input handling are explicit safeguards;
older approximate in-flight arc snapshots are rejected. Grouped core/data
checks pass and viewer hooks compile. The running viewer's complete throw
sequence and original exceptional save/world departure were not established
by that grouped check.

The 2026-11-15 Windows follow-up uses the installed Daventry files through
the production viewer, not the original executable. It verifies one Rock
throw and recovery, then a save after launch, fresh-process continuation
through landing, and equality with an uninterrupted route. `items.md` rule
11 supplies the original last-Rock identity behavior checked at the end.
Original-runtime midflight save parity, hammer/Weeper flight and broader
campaign encounters remain unverified. [Evidence and commands](../rock-projectile-recreation.md).
