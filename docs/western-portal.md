# Earned western portal traversal

The original triptrp5 script enables portrap5 and portrp5b near Sylph's
room while the plate has an occupant. The genuine WesternPlateRockReload
save supplies that occupant: the earned world Rock NoName98. Connor must
walk back from the graveyard along the cleared roads; the portal is not
beside the plate.

The installed portrap5.cs/portrp5b.cs scripts use pointAndCue, the portal
animation and its state-change notice, delayed camera/Connor placement,
portalx, then return control. The authored arrival is (29021,192412,889).
FirstFive changes to true and the first crossing awards 4,000 experience.
From this earned checkpoint that crosses level 11's 10,240 threshold:
the expected new stats are [120,85,27,11,1674] at level 12. Inventory and
quest flags outside the portal state must be preserved. The Rock should
remain on the distant plate.

These are original data/script rules, not newly invented engine behavior.
No Rust runtime behavior changes in this step. The replay uses normal
movement keys and waits for the original FirstFive flag; it never calls
the portal script directly, changes flags, places Connor or grants items.
Implementation B did not read the original executable or decompilation.

## Failure and evidence-based route correction

The first run reached the well at update 2879. Its next proposed waypoint
crossed a head-on collision at (66735,105404,1584), update 2899. With no
further progress the replay stopped at update 3078; no portal callback
ran and no save was written. The test failed after 209.88 seconds. The
complete evidence is retained in `kq8-artifacts/western-portal-first-crossing.log`
and `earned-western-portal/1794043032595147200/WesternPortalCrossed`.

The initial static check sampled endpoints only, which did not prove
that the connecting segment was clear. The existing local planner's
full original BSP/terrain segment check gives a route through
(67250,105750), (64000,110000) and (62500,110436). Evidence is
`western-portal-approach-road.log`; intermediate waypoints divide the
long segment without changing its path. This planning check omits live
actors and saved object changes and does not replace native verification.

The revised test writes WesternPortalApproach at the well before the
short portal leg. An optional genuine approach checkpoint lets later
attempts avoid replaying the completed road. Review also corrected a
log-format assertion before it was reached: script set logs include
an equals sign and quotes. This was not the first run's failure.

## Reproduction commands

Run from the repository root. These are automated checks, not a manual
playtest request; successful native legs should not be repeated.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_WESTERN_PLATE_ROCK_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-western-plate-rock/1794039587261976800/slots/slot-3.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_rock_opens_the_western_portal_and_first_crossing_reloads -- --ignored --exact --nocapture
```

If a genuine WesternPortalApproach save exists, set
`KQ8_WESTERN_PORTAL_APPROACH_SLOT` to it to skip the long road. If a
genuine WesternPortalCrossed save already exists, a separate test can
verify only the fresh-process reload:

```powershell
# Also set KQ8_WESTERN_PORTAL_CROSSED_SLOT to that genuine save.
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::saved_western_portal_crossing_reloads_without_another_award -- --ignored --exact --nocapture
```

## Result and remaining work

The revised three-process route **passes once in 264.89 seconds**, in
`kq8-artifacts/western-portal-planned-crossing.log`. Its genuine saves are
under `earned-western-portal/1794043427585987500/slots`: slot-2 is
WesternPortalApproach, slot-3 WesternPortalCrossed, and slot-4
WesternPortalCrossedReload. The portal returns control at update 255
of the short crossing leg. The final position is (29021,192412,864), on
the chamber's floor, with the expected level-12 stats. FirstFive is true,
FiveOn false and SixOn true; all three relevant portal masks are 4.
NoName98 remains the plate's sole occupant. Inventory and other quest
flags are preserved, reload awards nothing again, and original Options.cs
is unchanged. No successful native case was repeated. No runtime source
changed, so unrelated core tests were not rerun. The separate reload-only
entry point compiled but was not run; the full test already passed that
fresh-process check.

The skull, feather, scales, Sylph item quest and later regions remain
incomplete. This route does not settle Q202 aim/arc,
Q203's remaining item lifecycle rules, or Q205 spatial ordering. Manual
testing and original-engine runtime comparison are **not tested**.
