# STATUS

## 2026-11-07 Shared original-resolution interface completed

The six menu families and HUD now resolve authored parent-relative
rectangles at 640x480 and 800x600 with the original integer conversion,
fixed glyph sizes and native/stretched picture rules. Save/Load keeps
its centered controls and original thumbnail size. Review also fixes the
separate Main-menu warning image and button drawing path.

The geometry/raster checks pass 7/7; the six-menu resize matrix and
modal matrix each pass 1/1. Viewer release compilation passes. Successful
checks are not repeated; no native launch or manual inspection is claimed.
Live window/device switching, pointer projection and credits remain open.
Estimates: M4 about 83%, M5 60%, M6 about 25%.
[Commands, results and limits](docs/original-interface-resolutions.md)

## 2026-11-07 Next shared interface batch has original reference rules

Graphics and credits studies now replace earlier assumptions. Interface
work can use original parent-relative rectangles, fixed glyph sizes,
picture rules, 640/800 art and the Save/Load centering exception.
Credits must use the original line-count timing, mixed fonts, final
photo and level/current-experience score. The authored 25-card sequence
has 134 lines and a nominal duration of 129 seconds. These are reference
findings; the Rust presentation changes are still pending.

The refreshed script audit identifies genuine remaining preSpecial,
reduceHealth, trap membership/removal and Tile2/Tile3 behavior. The
incorrectly capitalized clearstateChangeNotify is an original-script
spelling issue, not permission to invent an alias. Study continues on
shared combat/control commands. M4 about 80%, M5 60%, M6 about 25%.
No manual tests, and no completed milestone is claimed.

## 2026-11-07 Ordinary liquid damage completed; display rules established

Poison, fast poison and swamp now share the original amount, clock,
depth and immunity rules and dispatch ordinary world attacks. Armor,
reaction, sound and death remain in the existing combat path. Three
grouped core checks pass. The twelve-region data matrix and one native
Dead City fixture pass 2/2 on the first run in 49.15 seconds, checking
zero-loss protection, poison-sound cycling and fatal damage through the
normal death menu. Original Options.cs is unchanged. No successful
native run is repeated; earned campaign saves remain untouched.
The regional command audit catches a study-note naming error:
addImmunity is registered; addImmune is only the original usage-message
typo. The command handler and affected checks are corrected before
publication; arithmetic and native integration evidence are reused.
The corrected command matrix and original Wisp callback pass 2/2;
that script grants swamp immunity while leaving poison eligibility.
[Commands, evidence and limits](docs/terrain-liquid-damage.md)

The original-data script report first regenerates unchanged, then the
44.58-second rebuild after the naming correction marks addImmunity
built and reduces Swamp's missing commands from eight to seven. Its
acceptance/name-presence statuses are hints, not completed behavior.
The new graphics study establishes shared 640/800 layout, font, picture,
background and Save/Load rules; implementation and device switching
remain incomplete. Original ontop/special-status hazards and native save
semantics also remain open. Estimates: M4 about 80%, M5 60%, M6 about
25%. No manual testing.

## 2026-11-07 Shared quest-item application lifecycle completed

Applying a quest item now unapplies other applied quest items before
adding its own effects. One grouped matrix covers permanent/timed
replacement, statistics, timers, icons, holding, repeat application,
serialized continuation, dropping and expiry. All 16 existing inventory
tests pass. The new matrix first fails because its drop command lacks
the required namespace; after that test-only correction it passes all
four cases. Successful existing checks are not repeated. No native or
manual run is needed for this shared-core rule. Original campaign
application triggers remain untraced. Estimates remain M4 about 80%,
M5 60%, M6 about 25%.
[Commands, results and limits](docs/inventory-application.md)

## 2026-11-07 Inventory behavior fixed; remaining work grouped

The shared GUI item lookup now leaves an empty-cell right click silent
and retains normal left-click return of a held quest item. The bug was
reproduced in 10.62 seconds. The post-fix native flow completes in
11.02 seconds; its stale log-text assertion is corrected and checked
against the existing logs/save, then the target compiles. No successful
native sequence is repeated. [Evidence](docs/inventory-empty-slot.md)

The north-bank Invulnerable potion is earned through a normal camera
ray and survives reload alongside the GoodSkull and shelf replacement.
The 29.74-second test completes both native processes, then incorrectly
expects the removed-object list to remain identical. It should gain
NoName49 when that potion is picked up. The corrected condition and
remaining state checks pass on the existing saves; corrected source
compiles in 16.13 seconds. The final Options byte assertions in both
recorded tests were not reached, and remain not tested for these runs.

Newest campaign save: earned-skull-chamber-potion/1794048357391220700/
slots/slot-3.json, SkullChamberPotionReload at (32656,204036,864),
level-12 stats [39,85,27,11,2474]. The potion is carried, not consumed.
The direct return crosses deadly fast-poison; new original-engine
prose supplies the missing ordinary damage rules for implementation.
A wider numerical search finds a western-edge jump avoiding deep
liquid, but native return is not tested. Preserve this earned checkpoint.

Following the user's update, development proceeds by shared behavior
groups, combined checks and integrated chapter routes. Individual
objects get separate checks only for a distinct rule or failure.
Current groups: inventory input, ordinary terrain hazards, original
graphics modes, then broader campaign progression. Estimates remain
M4 about 80%, M5 60%, M6 about 25%. No manual testing.

## 2026-11-07 Earned good skull and shelf replacement survive reload

The native route from SkullRiverCrossedReload passes once in 37.41
seconds, following a 14.86-second compilation. At update 104 the
ordinary camera ray hits skull.22 first. Its original GSKULL.CS cue
runs once at update 110, removes the shelf skull and grants one
GoodSkull. SkullMsg advances to 1. The dynamically created clicker
NoName100 stays at (31583,206637,994), with identical identity and
placement after the fresh-process reload. No BadSkull is carried.

Newest earned save: earned-good-skull/1794047529603851200/slots/
slot-3.json, GoodSkullReload at (31469,204559,864). Level-12 stats
[39,85,27,11,2474], other inventory, both archer deaths, portal flags
and the Rock-held plate are preserved. Original Options.cs is unchanged.
Evidence: good-skull-earned-native.log. The successful river jumps and
archer fights were not repeated. Next establish the return route and
reach the statue with this earned skull. No manual testing. Estimates
remain M4 about 80%, M5 60%, M6 about 25%.
[Commands and limits](docs/skull-river-access.md#earned-good-skull-and-shelf-replacement)

## 2026-11-07 Original skull-river jumps and saved crossing verified

Normal standing jumps reach block.32 at update 149, feet 1779, and
block.31 at update 60 of the next leg, feet 2689. The new diagnostic
running_jump option presses ordinary Run, Forward and Jump keys; the
running jump lands across the river at update 68, (34744,202931,864).
No gameplay physics or animation distances change. Health stays 39.

The three native legs pass and save, but the overall test initially
fails after 68.49 seconds: it incorrectly requires the final save to
remain above y=202900. The original walk-stop animation carries Connor
from the reached flat-bank target onto the safe slope at
(35363,202274,662). The corrected assertion uses the actual bank/hazard
boundary and is checked against that existing save. The continuation
then verifies only the outstanding fresh-process reload, passing once
in 10.96 seconds after a 14.57-second test compilation. No jump repeats.

Newest earned save: earned-skull-river-crossing/1794047351774964100/
slots/slot-2.json, SkullRiverCrossedReload. Stats [39,85,27,11,2474],
level 12, items, both archer deaths, portal flags and the held Rock
persist. Original Options.cs is unchanged. Evidence: the initial
skull-river-crossing-native.log and passing skull-river-crossing-reload.log.
Next recover the original good skull. No manual testing. Estimates:
M4 about 80%, M5 60%, M6 about 25%.
[Commands and limits](docs/skull-river-access.md#original-block-and-jump-crossing)

## 2026-11-07 Both skull-chamber archers defeated; reload passes

The healed portal revisit, eastern-archer encounter and fresh-process
reload pass once in 87.51 seconds, following a 12.15-second compilation.
Portal5 returns Connor to the chamber at update 239 without repeating
its 4,000-XP award. Ordinary Small Crossbow shots defeat skeleton16;
its death completes at update 218 of the encounter leg, awarding 400 XP.
A final incoming arrow leaves 39 health. Both deaths, inventory, quest
flags, the held Rock and stats [39,85,27,11,2474], level 12, survive reload.
Original Options.cs preservation passes. Evidence:
skull-eastern-archer-native.log.

Genuine continuation:
earned-skull-eastern-archer/1794046057966712000/slots/slot-4.json,
SkullEasternArcherReload at (36280,193254,864), first person, weapon
stowed. Slot-2 is HealedSkullChamber; slot-3 is the earned victory.
The western fight and well healing were not repeated. Next establish
the original block/jump route across the damaging river, then recover
the good skull. Runtime rules are unchanged. No manual testing.
Engineering estimates remain M4 about 80%, M5 60%, M6 about 25%.
[Commands and limits](docs/skull-river-access.md#healed-return-and-eastern-archer)

## 2026-11-07 Earned return-portal healing and reload pass

The continuation from the genuine SkullReturnPortal save passes once
in 40.30 seconds. Normal F2 selects third person while standing, then
the reached well route and 14 visible clicks heal 52 to 120. A fresh
process restores level-12 stats [120,85,27,11,2074], unchanged inventory
and quest flags, skeleton17's completed death, skeleton16 still present,
the Rock-held plate and the patched return-portal flags/masks.
Original Options.cs preservation passes. Evidence:
skull-return-well-third-person.log.

Genuine continuation:
earned-skull-return-well/1794045759136946000/slots/slot-3.json,
SkullReturnWellReload at (66544,104638,1584). The successful portal leg
was not repeated; its corrected Cue 5 assertion was checked against the
existing trap log (one occurrence). The log-selection failure and the
first-person aim failure below are retained. Next return through portal5
and defeat skeleton16 before the river crossing. No runtime changes or
manual testing. Estimates remain M4 80%, M5 60%, M6 25%.
[Commands and limits](docs/skull-river-access.md#return-portal-and-earned-healing)

## 2026-11-07 Well reached; first-person target selection failed

The resumed route reaches the original well approach at update 114,
(67035,105055,1584), but its automated aim cannot see well2 after a full
camera turn. It exits after 21.61 seconds without healing or a new save.
Evidence: skull-return-well-resumed.log and
earned-skull-return-well/1794045670622607100/SkullReturnWell.

The original return portal's paired setThirdPerson calls correctly
restore the preceding first-person preference (camera.md rule 7).
Earlier successful well routes used third person. The next check uses
the normal F2 key while standing after loading the genuine return save,
then follows the same reached path and visible clicks. No camera or
gameplay rule is changed; this is one failed well-view attempt. The
already successful portal leg is not repeated. M4 80%, M5 60%, M6 25%.

## 2026-11-07 Northwest return portal saved; assertion used the wrong log

The original patched portrap6 sequence returns Connor at update 161 to
(63691,110436,1584), with unchanged [52,85,27,11,2074] level-12 stats.
The native viewer reports PASS and writes SkullReturnPortal in
earned-skull-return-well/1794045576576558000/slots/slot-2.json. The test
then fails after 36.86 seconds because its Cue 5 assertion inspected
doors.log; the callback appears once in traps.log at update 161.

The assertion now reads the trap log. Resume from this genuine return
save for the well leg; do not repeat the successful portal traversal.
Evidence: skull-return-well-native.log and the run's trap/viewer logs.
No runtime rule changed. M4 80%, M5 60%, M6 25% remain estimates.

## 2026-11-07 Earned western skull archer and reload pass

After the projectile-path audit below, the revised encounter passes
once in 48.14 seconds, with an 11.66-second compilation. The earned
Small Crossbow kills skeleton17; its death completes at update 195,
awarding 400 XP. The saved and freshly restored level-12 stats are
[52,85,27,11,2074], at (33253,192808,864). Inventory, quest flags and
the Rock-held plate persist; original Options.cs is unchanged. The
other archer remains. Evidence: skull-western-archer-native.log.

Genuine continuation:
earned-skull-river-guards/1794045168852782700/slots/slot-3.json,
SkullWesternArcherReload. Do not repeat this successful encounter.
Next inspect earned healing via the return portal and Sylph's well,
then the second archer and block/jump crossing. The abandoned lethal
floor-only skull route is retained as a local prototype, not a working
test. Runtime rules were unchanged. No manual testing. Engineering
estimates remain M4 about 80%, M5 60%, M6 about 25%.
[Commands and limits](docs/skull-river-access.md)

## 2026-11-07 Skull-river guards: second attempt exposes blocked bolt path

The cover attempt compiled in 11.94 seconds and failed after 20.26
seconds, with Connor entering Die at update 238. Skeleton16 did not
damage Connor at this position, but skeleton17 did. The camera's ray
selected skeleton17 correctly. Every fired crossbow bolt instead hit
block.31 after 712 units, starting with bolt106 at update 113. Thus
camera visibility did not establish a clear projectile path. Neither
guard died and no new save was written. Evidence:
skull-river-guards-cover.log and
earned-skull-river-guards/1794044813029029800/SkullRiverGuards.

Two actual combat attempts have now failed. Stop native route variants
at this obstacle. The next required evidence is a numerical sweep of
the original projectile's spawn, radius and path against the oriented
block, plus the return shot paths. Keep both failures and the earlier
river-wall death. No gameplay rule was changed and no successful test
was repeated. M4 about 80%, M5 60%, M6 about 25% remain estimates.

## 2026-11-07 Skull-river guards: exposed ranged attempt failed

The first ranged attempt compiled in 11.77 seconds and failed after
16.90 seconds. From (30948,193885,864), Connor's normal camera rays and
crossbow projectiles hit skeleton17, but both archers hit him. He died
at update 190 before either enemy died. No new save was written.
Evidence: skull-river-guards-native.log and
earned-skull-river-guards/1794044546535599600/SkullRiverGuards.

The monster-list columns add to the type's stats: each guard has 160
health and awards 400 XP, not 100/200. Original block.31 is tall enough
to shield a south-bank position from skeleton16 while leaving a ray to
skeleton17. A revised route will test that cover, then address the
remaining archer from east of the block. This is one failed combat
attempt; the earlier river-wall death is a separate route obstruction.
No successful test was repeated. M4 80%, M5 60%, M6 25% remain estimates.

## 2026-11-07 Good-skull route: river-wall failure, prerequisites identified

The first native skull attempt compiled in 11.49 seconds and failed
after 12.90 seconds. It reached rivrtrap.01 at update 132, position
(27006,196473,703); the original 5,000-damage event killed Connor.
Skeleton16 and skeleton17 were also shooting across the river. No skull
pickup or new save was produced. Evidence: good-skull-native.log and
earned-good-skull/1794044067750593900/GoodSkull.

The original files put the bone bridge in the eastern chamber. The
northwest chamber's river-wall triggers and central fast-poison terrain
were absent from the earlier static route's eligibility checks. The next
action is an earned ranged encounter from the dry bank, then crossing
planning using the original jump arcs and authored blocks. Keep hazards
and combat active. This is one failed native attempt, not two; no new
crossing attempt has run yet. M4 about 80%, M5 60%, M6 about 25% remain
engineering estimates. No manual testing.

## 2026-11-07 Earned western portal crossing and reload pass

The revised three-process route passes once in 264.89 seconds. Connor
walks from the Rock-held plate to the well, saves WesternPortalApproach,
then enters portrap5 through ordinary movement. The original portal
sequence returns control at (29021,192412,864). Its one-time 4,000 XP
award produces level 12 and stats [120,85,27,11,1674]. A fresh process
restores that destination, inventory and flags without another award.
NoName98 still occupies triptrp5; all three involved portal masks are 4.
Original Options.cs preservation passes. Genuine continuation:
earned-western-portal/1794043427585987500/slots/slot-4.json,
WesternPortalCrossedReload. Slot-2 is the approach; slot-3 is the crossing.

The first route failure below is retained. Full segment planning fixed
the route without changing runtime collision, scripts or engine rules.
No successful native test was repeated. Only campaign test/documentation
code changed; no unrelated core suite was rerun. The separate reload-only
entry point compiled but was not run, because the full route already
includes a passing fresh-process reload. [Commands and limits](docs/western-portal.md)

Next is the good-skull shelf, then the statue/feather/scales and remaining
Sylph/campaign branches. M4 about 80%, M5 60%, M6 about 25% remain
engineering estimates, not checklist ratios. No manual testing.

## 2026-11-07 Western portal approach: first route stopped near the well

The first earned route reached the well at update 2879 but its next
waypoint collided head-on at (66735,105404,1584), update 2899. The replay
reported no progress at update 3078 and exited; no portal callback ran
and no new save was written. Evidence: western-portal-first-crossing.log
and earned-western-portal/1794043032595147200. This is a real failed run,
not a completed portal crossing. No engine behavior was changed.

The first plan sampled only approach endpoints. Full original static
terrain/building segment planning now identifies an approach via
(67250,105750), then (64000,110000) and (62500,110436), avoiding the
failed collision. The revised test saves at the well before attempting
the short approach, and accepts that genuine checkpoint for future
continuation. The revised route and portal/reload now pass as recorded
above. M4 80%, M5 60%, M6 25% remain engineering estimates. No manual testing.

## 2026-11-07 Earned Rock pickup releases the plate through reload

The normal visible pickup from WesternPlateRockReload passes once in
28.75 seconds. NoName98 merges into the existing carried stack; triptrp5
receives Off before the world identity is destroyed. Both portal masks
become zero and carried Rocks return to two. A fresh Windows process
preserves that state without another departure. Other inventory, stats,
quest flags and original Options.cs are unchanged. New genuine checkpoint:
earned-western-plate-rock-pickup/1794042294163512500/slots/slot-3.json,
WesternPlateRockPickupReload. The earlier hold route was not repeated.

Deletion requests retain names/transforms until occupied-trap callbacks
finish. Non-merged pickup retains identity at the origin and uses stay
checks; origin overlap can remain. Seven focused lifecycle cases pass
once each, including the spirit-trap handler added during final review.
The remaining core run has 706 passes and 17 ignored, plus one old raw
interpreter fixture corrected to complete the event and passing separately.
The first test build's type errors and that fixture failure are retained.
Final release viewer test-target compilation passes in 19.71 seconds.

Q203 remains partial: last-unit/non-Rock drops, saved-contact reattachment,
other registration paths and overrides remain. Q202/Q205 are open. No
manual testing. Estimates remain M4 about 80%, M5 60%, M6 about 25%, not
measured checklist ratios. [Commands and limits](docs/item-removal-traps.md)

## 2026-11-07 Earned Rock holds the western plate through save and reload

The previously failed earned-Rock route passes once in 46.00 seconds.
The ordinary throw consumes one of the two carried Rocks. Recoverable
world Rock NoName98 occupies triptrp5, enabling both portal masks to 4,
and a fresh Windows process restores that same membership and state.
Other carried items, stats and quest flags remain unchanged. Checkpoint:
earned-western-plate-rock/1794039587261976800/slots/slot-3.json,
WesternPlateRockReload. Earlier failure evidence is retained.

Fresh item registration now uses its oriented ball stack and invokes
eligible trap callbacks synchronously, preserving command-local final
NewInvItem values through nested creation. Existing item moves remain
stay-only. Nine registration cases pass, plus one chest-event-boundary
case. The remaining core suite passes once (697 passed, 17 ignored),
before the chest-only correction; successful cases were not repeated.
Final release viewer check passes. [Commands and limits](docs/item-registration-traps.md)

Q203 is partly implemented: pickup/deletion, last-unit/non-Rock identity
rules and broader registration/restore paths remain. Q202 aim/arc is
still open. No manual testing. Engineering completion estimates remain
M4 about 80%, M5 60%, M6 about 25%; these are not measured checklist ratios.

## 2026-11-07 Q203 studied; recoverable Rock implementation still pending

The original projectile does not occupy the plate: its collision-detail
pair with a trap is excluded. Rock-category dropItem always creates a
fresh quantity-1 world item, even for the last carried Rock. Registering
that active item's collision shape checks stationary overlaps and runs
ordinary trap entry synchronously. Existing-collider script placement
still has no destination entry probe. Callback/NewInvItem ordering,
pickup/deletion release and silent saved-name reattachment are now in
[traps.md](docs/behaviour/traps.md#recoverable-rock-registration-and-removal-q203-2026-11-07)
and [items.md](docs/behaviour/items.md). Q203 is answered, not implemented.
The last carried Rock has an additional original edge: the old identity
also enters world ownership at its stored carried placement before the
fresh landing Rock is created, with no cleanup found in this command
chain. Do not simplify that case into a pure replacement. Registration
rotates the local ball centres by the item's full orientation.

Static original study and installed-data arithmetic only; original
runtime/native save tests **not tested**. No Rust or campaign result
changed. The earned Rock check below remains a real failure. Next work
must implement the eligible-registration distinction and lifecycle from
the notes, then recheck that failure. Original aim/arc remains Q202.

## 2026-11-07 Script-moved props notify occupied traps after the event

Script placement now updates the prop's logical position immediately and
checks existing trigger occupancy after the outer script finishes. It
does not sweep the jump or enter destination triggers. Removed masks do
not suppress departure. Each trigger sees position/deletion changes from
the preceding callback; saved occupants rebuild their shape on loading.
The viewer also applies callback requests in the same update.

The final original Gnome boulder integration passes once in 2.30 seconds:
its actual trigger callback, departure, camera/Connor requests and cues
are correct, and serialized boulder placement survives a fresh original
data load. Model/collider positions agree, the Dropped flag remains set,
and the original absence of a click target is preserved. The loader now
registers the still KQBoulder class for saved prop positions. The release
viewer test target builds successfully; no successful test is repeated.

Eleven focused cases pass once each. The remaining core suite passed
once (686 passed, 17 ignored, eight focused cases excluded), before the
final overlap review correction. The three failed original-data runs are
retained: name capitalization, missing boulder save registration, and an
incorrect assumption that the original boulder has a click target. The
save failure was recorded before source inspection identified the class
filter; the test's save requirement was kept. A focused test's initial
string-type compile error is also retained. These are not counted as passes.
[Commands and exact limits](docs/scripted-prop-traps.md)

M4 remains approximately 80%, M5 60%, M6 20-25%. These are engineering
estimates, not a measured feature inventory. M4-M6 are not complete;
the earned Rock puzzle remains unresolved and no manual test was used.

## 2026-11-07 Physical pushes reach puzzle triggers; original book callback passes

Physical pushes now retain their actual collision path and notify
traps with their original box or ball-stack shape. Logical and saved
positions update before the script callback. Traps can resolve their
own position for checkDistance, which the original book, boulder and
girl scripts require.

Eight focused core checks now pass. Review removes an unsupported change
to the old logical-centre rule; only that revised case is rerun and
passes. The remaining core checks pass once (678 passed, 17 ignored).
The installed Temple2 book callback and controller-init integration
passes once in 1.74 seconds, disabling the trap, deleting the book,
creating pushbk2 and requesting the opening Special animation.

The final review adds a check for a callback changing the next prop's
type before its collider refreshes. The dispatcher now reads the live
type, and this eighth case passes once. The final viewer release check
passes; the successful Temple2 asset check is not repeated.

Earlier compile diagnostics (private field access, array-size inference)
and the incorrect zero-test module filter are retained and not counted
as passing evidence. The original-data check is an isolated push with
other obstacles omitted, not a campaign route. Exact mid-pass callbacks,
script-placement/removal occupancy and the asymmetric-box centre rule
(Q204) remain incomplete. Q203 is now studied above, but its
implementation remains pending; M4-M6 are not done.
[Commands and evidence](docs/pushed-object-traps.md)

## 2026-11-07 Tossed-item movement reaches the trap dispatcher

Seven focused synthetic checks pass for shaped item entry/exit along
the real mover path, wall clipping, masks, callback order, multiple
occupants and settled serialized occupancy. Ordinary placed items
still do not enter. The initial compile-only failure was a private
test import; the public import fixed it. The native earned Rock
puzzle remains the failure below, pending Q202 and implementation of
the now-studied Q203. Pickup/deletion
release and native campaign use of this new integration are not tested.
[Commands and limits](docs/tossed-item-traps.md)

The seventh case (wall sliding) initially expected the wrong contact
position, omitting the existing 25-unit pull-back. Its corrected
expectation passes without a physics change. The other six cases
were not rerun. The remaining core regression passes once (671
passed, 17 ignored), and the viewer release check passes.

## 2026-11-07 Earned Rock lands inside the plate; portal stays closed

The first native Rock attempt fails in 41.93 seconds. The visible
ground click at gameplay pixel (814,417) starts ThrowRock normally.
On update 80 the original rockHit.cs drops one recoverable Rock at
(114047.86,108670.29,0), inside the plate footprint. Carried Rocks
change from two to one, but triptrp5 has no occupant and both portal
masks remain zero. Save and reload processes finish; the assertion
requiring mask 4 fails. Evidence: kq8-artifacts/western-plate-rock-native.log
and earned-western-plate-rock/1794035062532059200. This is one real
failed attempt, not a successful puzzle check.

The new route diagnostic aims through an on-screen gameplay pixel,
rejects nearer aimable objects and checks the drawn-ground hit.
No projectile or trap rules changed in that attempt. Q203 is now
answered by the study above: a fresh eligible collider's registration
can enter without flight, unlike placement of an existing collider.
Its implementation remains pending; Rock aim and flight fidelity remain
Q202. Do not bypass the contact/mask/detail conditions with forced On,
or alter the pressure box to make this test pass.
The earlier passing campaign checkpoint remains WesternPlateReleasedReload.
[Command and limits](docs/dead-city-progression.md#earned-rock-placement-failure)

## 2026-11-07 Graveyard zombie and western plate On/Off verified

The ordinary return around the cleared roads and fresh reload pass
in 242.61 seconds. Fifteen visible well2 clicks on updates 2736-2750
run the original five-health event, restoring 39 to 110. There is no
incoming damage on the return. GraveyardWellReload ends at
(66728.32,104706.875,1584), with [110,80,25,10,7512], both Rocks,
unchanged items/quest flags/prop placements and all cleared guards
still gone. Evidence: earned-graveyard-well/1794033520956824200/slots/slot-3.json
and graveyard-well-native.log under kq8-artifacts.

The ordinary return to the graveyard and fresh reload also pass, in
242.48 seconds, without damage or other state changes.
HealedGraveyardReload ends at (111066.125,100055.83,0), with
[110,80,25,10,7512]. Evidence:
earned-healed-graveyard-return/1794033787998345400/slots/slot-3.json
and healed-graveyard-return-native.log. The earlier failed approaches
remain below.

The prepared encounter defeats zombie6 without damage: six normal
Mace hits, death on update 164 and the original 402 experience on
199. The first test fails in 69.79 seconds because its final
position expectation assumed Connor would stop outside the plate.
His unchanged stopping animation reaches (113745.9,108677.2,0),
activating triptrp5 on update 314. The original script enables both
portal masks, lowers the plate and adds the map icon. The failed
position assertion is retained in
earned-healed-graveyard-zombie/1794034064579530000 and
healed-graveyard-zombie-native.log. No new engine change is needed.

The corrected encounter and held-plate reload pass in 69.41 seconds.
WesternPlateHeldReload retains [110,80,25,10,7914], both Rocks,
Connor inside triptrp5 and both portal masks at 4. The reward and On
event do not repeat on reload. Evidence:
earned-healed-graveyard-zombie/1794034224062657900/slots/slot-3.json
and healed-graveyard-plate-native.log.

Walking off and reloading pass in 30.77 seconds. On update 51 the
original script clears both portal masks, raises the plate and
removes the restored map icon. The final WesternPlateReleasedReload
is at (112108.76,108774.945,0), with unchanged full health, experience,
inventory and quest flags; the plate is empty and both masks are 0.
Off does not repeat on reload. Evidence:
earned-western-plate-release/1794034374631528600/slots/slot-3.json
and western-plate-release-native.log. Leaving a Rock on this plate,
using its portal, the skull and later quests remain unverified.
M4-M6 remain incomplete.
[Commands and limits](docs/dead-city-progression.md#earned-graveyard-preparation-and-western-plate)

## 2026-11-07 Graveyard entrance passes; two zombie attempts stopped

The ordinary winding entrance and fresh-process reload pass in
109.36 seconds. GraveyardEntranceReload is at (111013.625,99472.47,0),
with unchanged [39,80,25,10,7512], both Rocks and all quest state.
Evidence: earned-graveyard-entrance/1794032840475660800/slots/slot-3.json
and graveyard-entrance-native.log under kq8-artifacts.

The first zombie6 attempt fails in 19.66 seconds. One attack misses,
but three subsequent hits take health 39 to 29 to 3 to -4. Connor
enters Die on update 239; zombie6 still has 179 health. The existing
retreat driver holds Back at those hits, extending the zombie's
900 reach to 1125; it did not withdraw far enough before the cue.
No victory, reward, plate arrival or new save is claimed.
Evidence: earned-graveyard-zombie/1794032972112326600 and
graveyard-zombie-native.log. This is the first real graveyard combat
failure; the entrance checkpoint remains valid.

The second attempt, using the earned Small Crossbow from 4861 units
away, fails in 22.72 seconds. No shot is dispatched: the normal
gameplay ray cannot target zombie6 and the camera search exhausts
its full turn. Native logs show Birth, not active, not drawn and
collider out throughout. character-movement.md rule 30 explains
that it rises only after seeing Connor within its 3000 alarm radius.
The earlier clear terrain/building sight line did not imply a live
target. Evidence: earned-graveyard-zombie/1794033128236711700 and
graveyard-zombie-crossbow-native.log.

Combat retries are stopped after these two failures. The retained
ignored zombie test is currently failing; only the entrance is
verified. No game-engine bug is established. The next independent
preparation is an ordinary return to the Sylph well, healing through
its existing interaction, and a fresh reload before returning.
graveyard-well-return-plan.log records a possible long cleared-road
route; that return is not tested. Any later ranged attempt must first
respect the original emergence condition. Plates and later quests
remain unverified. M4-M6 remain incomplete.

## 2026-11-07 Follow-up graveyard threat correction

The local proximity diagnostic had a numeric overload error:
PowerShell selected an integer Math.Clamp overload from bounds 0/1,
rounding projections onto route segments. Explicit double arguments
correct it. The old planned graveyard line passes within 690 units
of zombie6 at (113515,103640,0), with 410 health and alarm radius 3000.
The previously listed nearby enemies were not a complete assessment.
Evidence: graveyard-after-western-threat-audit.log. This corrects an
offline diagnostic; the published central/western native results
remain unchanged. No graveyard native attempt has started yet.

## 2026-11-07 Central and western passages verified

The independently healed SylphWellReload supports the central
encounter. The cleared approach, three guard deaths and fresh-process
reload pass in 179.31 seconds. CentralGuardsReload retains 23 health,
the original combined 500 experience once (7112 total), both Rocks
and unchanged inventory/quest state. The seven actual guard drops,
four Mushroom uses and another reload pass in 96.07 seconds.
CentralSuppliesReload retains [43,80,25,10,7112], 25 coins, one unused
Strength potion and both Rocks. Evidence: prepared-central-guards-native.log,
central-supplies-native.log, earned-prepared-central-guards/1794031235820492500
and earned-central-supplies/1794031529033744000 under kq8-artifacts.

Fourteen terrain sight-line samples block MaceBoy1/2 from the northern
bend. Native traversal reaches the western guard approach without
their damage. The first skeleton52 attempt fails in 45.22 seconds:
the potion correctly raises hand damage 80 to 120 on update 430,
but stationary charged attacks allow five ordinary hits. Health
drops below zero on 598; the final charged hit kills skeleton52 on
614, and pending Die starts on 622. No successful continuation or
reward/reload is claimed from that run. Evidence:
earned-western-passage-guard/1794031852408072000 and
western-passage-guard-native.log.

The retreat/counterattack attempt passes with a fresh-process reload
in 89.47 seconds. The guard misses at distances 1430 and 1370 against
his 1300 reach; three normal Mace hits defeat him, with only four
health lost by Connor. The original 400 experience is awarded once
on update 590. Strength starts on 430 and expires on 730, exactly
20 seconds later; hand damage returns from 120 to 80 and its empty
item leaves the inventory. No combat rules or source save values were edited.

Latest checkpoint: WesternPassageReload at
(107494.71,117003.98,192), level eleven [39,80,25,10,7512], with both
Rocks and 25 coins. Evidence:
earned-western-passage-guard/1794031981737135600/slots/slot-3.json
and western-passage-retreat-native.log under kq8-artifacts.
The graveyard floor, zombies, portal plates, skull and later quests
remain ahead. M4–M6 remain incomplete.
[Commands and limits](docs/dead-city-progression.md#prepared-central-and-western-passage)

## 2026-11-07 Earned return to the Sylph well and healing verified

The independent ordinary-input return from SecondRockReload passed
with a fresh-process reload in 169.75 seconds. Ten visible clicks on
well2 ran the original sylphwel.cs health event on updates 1176-1184,
restoring 61 health to the level-eleven maximum of 110. Both Rocks,
other inventory, quest flags and moved blocks remain unchanged.
The defeated Weeper stays gone; no healing event repeats on reload.
GOG Options.cs is unchanged and AI/damage cheats remain disabled.

The healed checkpoint is SylphWellReload at (66725.86,104720.02,1584),
level eleven stats [110,80,25,10,6612], in
earned-sylph-well/1794030725771876400/slots/slot-3.json under
kq8-artifacts. Evidence: sylph-well-native.log. The offline approach
uses (67000,105000); its first farther-east target was unreachable
and was not replayed. This verifies healing, not the central guard
encounter below. M4–M6 remain incomplete.
[Command and limits](docs/dead-city-progression.md#central-passage-combat-and-the-sylph-well)

## 2026-11-07 Central passage combat: two attempts failed; route stopped

The ordinary walking approach reached the eastern bend, but the
Small Crossbow attempt died on update 790 after defeating skeleton2.
Connor began with 61 health, lost 32 before his first successful
crossbow hits, and died to skeleton3. The same-team skeleton1 joined
the encounter; its authored zero-damage hits still interrupted shots.
Evidence: earned-central-archers/1794030315905121300 and
central-archers-native.log (56.40 seconds). No continuation save is
claimed. This was the first real failure of this section.

The Mace attempt also failed in 62.19 seconds. It defeated skeleton1
on update 796 (the heavy swing dealt 207 actual damage); its original
200-point reward completed on 811. Connor then died to an archer on
877 while still approaching skeleton2. Evidence:
earned-central-guards/1794030484043099000 and central-guards-native.log.
The route was stopped after these two real failures. That failing
test revision remains in commit 8fff486; the healed update passes above.
SecondRockReload remains the last verified checkpoint before these
attempts; the later independent well continuation is recorded above.

The numeric audit shows ordinary running after the first kill, with
archer hits interrupting the approach; it does not establish an engine
defect. MONSTERS.KQ lines 14-16 and the earned save identify all three
team members. Connor entered with only 61 of 110 maximum health and
no carried healing items. Original sylphwel.cs supplies five health
per ordinary ConnorAction, which prompted the independent healing
check above, followed by the successful healed continuation.
No combat rule or saved state was changed to alter either outcome.

## 2026-11-07 Southern approach, passage Weeper and second Rock verified

Three earned continuations and their fresh-process reloads pass:

- SylphKnowledgeReload to SouthernPassageReload: 136.28 seconds,
  arrival near (100150.47,70869.45,864), with unchanged stats and items.
- SouthernPassageReload to PassageWeeperReload: 69.48 seconds. Ordinary
  Small Crossbow shots defeat weeper16; its death finishes on update
  291 and awards the original 500 experience once, with no health loss.
- PassageWeeperReload to SecondRockReload: 62.28 seconds. A visible
  click on the original NoName44 Rock succeeds on update 272. Connor
  retains two Rocks after reload, with every other item unchanged.

The latest verified save is SecondRockReload at
(112862.055,70938.58,864), level eleven stats [61,80,25,10,6612].
Sylph topic 3 and her presence persist; the Weeper stays defeated,
its reward does not repeat on reload, and GOG Options.cs is unchanged.
All routes retain ordinary damage/AI and use no grants or teleports.

Evidence under kq8-artifacts: earned-southern-passage/1794029366147771600,
earned-passage-weeper/1794029563826960700 and
earned-second-rock/1794029819717808900. Logs are southern-passage-native,
passage-weeper-native and second-rock-final-native (.log).
The first Rock test completed pickup/reload but failed at 63.15 seconds
because its assertion read count instead of the saved quantity field.
That harness error is retained in second-rock-native.log and
earned-second-rock/1794029717660383500; the corrected full rerun passes.

The access investigation changed direction. Even with all obstacles
omitted, the terrain search cannot walk onto the western high plateau.
It can reach the graveyard floor through a winding ground-level route.
A navigation lead in RESEARCH prompted that distinction. Replacing
whole-building boxes with real collision sweeps for 25 matching original
placements finds the approach; the old broad-box search gave a false
unreachable result. Unmatched bounds, objects and closed doors remain
obstacles. The helper still uses terrain heights, so it is not a full
bridge-floor or live-mover proof. Only the sections above are verified
in gameplay. No original engine rules changed.

The graveyard, its four portal plates, skull, feather, scales and later
branches remain incomplete. M4–M6 are still active.
[Commands and limits](docs/dead-city-progression.md#southern-passage-and-second-rock)

## 2026-11-07 Earned Sylph conversations verified

The three original introductory conversations and final fresh-process
reload passed in 252.41 seconds, starting from SylphSummonedReload.
The route walks around the portal structures and clicks Sylph through
the gameplay camera. Her original final TalkComplete callbacks run
once per topic: cases 10/11/12, sequences 2/5/3, on updates 593/320/150.
The second topic awards 3000 experience once; the third and reload
repeat neither that award nor the appearance award.

SylphKnowledgeReload retains metConnor 3, isIn true, all four plates
up, unchanged moved blocks and inventory, and level eleven stats
[61,80,25,10,6112]. Connor is at (70721.22,107156.445,1351.8123).
God mode and AI-off remain false; GOG Options.cs is unchanged.
Evidence: earned-sylph-topics/1794028407978349500 and
sylph-topics-native.log under kq8-artifacts.

The first offline approach band had 86 excluded samples out of 1197;
it was not replayed. The subsequent search used a 1000-unit obstacle
margin and found the western detour used by the passing native test.
No game-engine rules changed. Her item quest remains **not tested**:
the original scale.cs spawns Mold only during the scales completion.
[Automated command and result](docs/dead-city-progression.md#the-sylph-conversations)

The follow-up at that point was to investigate actual building floors
on the graveyard approach; the subsequent result is recorded above.
An archive audit finds 856 distinct Dead City script/object names,
all in kq.vol, with no duplicates in the other three volumes; an
alternate archive copy does not explain the unresolved access route.
The original Dedtower model gives upward floor contacts along its
long western span, rising to Z5170, which the terrain-only planner
excluded as one large obstacle. This is geometry evidence, **not a
verified traversal**. The wider terrain search toward the bridge
still stops near (93750,102500), 5251 units short. Audit that approach
and the building collision surface before a native route. Evidence:
deadcity-text-copy-audit.log, dead-tower-floor-audit.log and
sylph-to-tower-plan.log. M4–M6 remain incomplete.

## 2026-11-07 All four Sylph plates and the appearance reward verified

The route clearance audit resolved the final-block approach without
changing movement or collision rules. The old driver heading remained
within its 0.13-radian tolerance while the long leg carried Connor
into torch.dt4. The revised route crosses at Y82500, using closer
marks beside the torches. Its 791-sample terrain/static-bound check
has no excluded samples; the subsequent live run passes the crossing.

The full continuation and fresh-process reload passed in 125.28
seconds from the genuine FirstSylphBlockReload. Block14 clears on
update240, block13 on670 and block12 on1054. The last plate's original
callback summons Sylph and awards 2000 experience exactly once.
SylphSummonedReload preserves all four B flags up, empty plate
triggers, moved blocks, Sylph's home at (70662,105677,1584), isIn and
FirstTime true, and metConnor 0. Final stats are level eleven
[61,80,25,10,3112]; inventory, AI/damage settings and GOG Options.cs
are unchanged. No reward is repeated on reload.

Evidence: earned-sylph-summoned/1794027970800884800,
sylph-summoned-audited-native.log and sylph-corridor-audit.log under
kq8-artifacts. The two earlier failed routes remain documented below.
SylphSummonedReload was the verified campaign boundary for this
step; the subsequent conversation check is recorded above. Her item
quest and the remaining Dead City campaign are still incomplete.
[Automated command and checkpoint](docs/dead-city-progression.md#the-sylph-pressure-blocks)

## 2026-11-07 First Sylph plate verified; full route remains open

The first block and fresh-process reload passed in 52.94 seconds
from WesternRockReload. Bound movement keys physically push block15
off trptrp15. Its original Off callback raises trip15, sets B15 up
and creates flame.port501. FirstSylphBlockReload retains the moved
block at (56803.633,120754.625,950), an empty plate trigger and the
tower flame. The other three B flags remain down; Sylph is absent
and no appearance reward is given. Stats [61,80,25,10,1112], inventory,
AI/damage settings and GOG Options.cs are unchanged. Evidence:
earned-sylph-first-block/1794027010744231100,
sylph-first-block-final-native.log.

The first test draft, 1794026925710997100, completed and saved the
push but failed at 43.45 seconds on incorrect expected log wording.
The corrected test checks the actual plate animation event and
saved flag. This was a harness error, not a failed push.

The remaining-three-block continuation has two real failures:

- 1794027170136062000, 64.90 seconds: block14's plate was released,
  but the departure path kept pushing it down the road until higher
  ground stopped the object. No continuation save was produced.
- 1794027397345330500, 73.42 seconds: sideways departure cleared
  blocks14 and13. The approach to block12 then repeatedly hit
  torch.dt4 head-on, ending on update1043 near (69650,80390,1248).
  The offline planned line missed the torch, but the live running
  path drifted into it. No continuation save or appearance award
  was produced.

Stopped native retries after these two failures and marked the
full-puzzle regression known failing pending a steering/clearance
audit. At that point FirstSylphBlockReload was the latest verified
save. The audit and subsequent successful continuation are recorded
above; no engine rules were changed in either step.
[Commands and evidence](docs/dead-city-progression.md#the-sylph-pressure-blocks)

## 2026-11-07 Earned western Rock and three archers

The two-process continuation passed in 155.92 seconds from the
genuine FerrymanKnowledgeReload. Connor exits the boathouse, uses
the earned Mace against BowGuys16, 15 and 17, then visibly clicks
the Rock placed by WORLDINV.CS at (56446,125813). Each archer awards
200 experience once, on updates 421, 525 and 621; pickup is on 886.
WesternRockReload retains one Rock, both weapons, the three defeats,
Ferryman topic 4 and KnowAboutFeather, with level eleven stats
[61,80,25,10,1112]. No Feather is granted. God mode and AI-off are
false, and GOG Options.cs is unchanged. Evidence:
earned-western-rock/1794026411163329300 and western-rock-final-native.log.

The first attempt, 1794026311032000700, failed after 46.32 seconds.
It killed BowGuys16, then tried to approach within 300 units of
BowGuys15's center and stopped against its body at distance 575.
The correction uses a 900-unit arrival radius for the two actor-center
marks, beginning normal attacks outside their colliders. Combat
and movement rules were not changed. The failed trace remains in
western-rock-native.log.

The original startup script explicitly loads trip5/triptrp5, and
rockme.cs accepts Rock for these pressure plates. This establishes
their presence, not a reachable route. The offline search was
extended to the full 262144-unit terrain domain with the documented
walkable slope limit. It still stops 7742 units from the trip5
approach and 15917 from the skull approach. These are numeric
diagnostics only: they do not prove that ordinary play cannot reach
either place. No portal or plate solution is claimed.
[Exact automated command](docs/dead-city-progression.md#the-western-rock)

## 2026-11-07 Installed regional patches now take priority

The original GOG installation's 22 regional patch files were being
bypassed by archive-only lookups. Both named reads and script/object
enumeration now select the installed correction first. The script
coverage tool uses the same folder index and includes GAME/patch.
The asset regression failed on all 22 selections before the fix and
passes after it, including Dead City's return callback reactivating
all four outward portals. The first draft of this test had a missing
String conversion; it was corrected before running the red check.

All 878 standard workspace tests pass. The native loading regression
passed in 57.95 seconds across all 12 regions, each rendering 30
frames. Logs: world-patches-before.log, world-patches-after.log,
world-patches-workspace.log, world-patches-regions.log and
world-patches-coverage.log in kq8-artifacts; native runs under
region-tests/23456-*.

Coverage now exposes KQConner::inTrap and removeFromTrap as missing
in the patched Barren elevator and IceWorld world scripts. Barren's
startup logs the former as unknown. Q201 asks the separate study
session for exact membership, return-value and removal behavior.
These elevator cases are **not verified**; the loading success does
not establish campaign correctness. Separate study remains unavailable
because its agents report the provider usage limit. No guessed engine
semantics were added.

The skull approach audit found a steep ridge rather than just an
overly conservative path cutoff: relaxing the offline planner to the
documented walkable normal still stops about 15,917 units short.
No new native skull route was attempted, no collision rules changed,
and no campaign state was granted. The last earned checkpoint remains
FerrymanKnowledgeReload; the portal/plate access route remains open.
[Source and exact automated commands](docs/world-file-selection.md)

## 2026-11-07 Ferryman introduction, knowledge and description lookup

The final earned five-process sequence passed in 319.03 seconds.
Connor enters the boathouse, clicks Ferryman through visible
gameplay rays and completes all four
introductory topics. Each ends through its original final line.
The first awards 2000 experience once; the third sets
KnowAboutFeather; the fourth awards 3000 once and raises Connor to
level eleven. FerrymanKnowledgeReload retains [110,80,25,10,512],
both weapons and metConnor 4. No Feather item is granted. AI and
damage remain enabled, and GOG Options.cs is unchanged. Evidence:
`earned-ferryman-topics/1794024289590037100`.

The earlier third-topic run rejected `invClick Feather` because Rust
required the full type name. Items rule 2 already defines the short
name, so this command now uses the existing prefix helper. The
script-dispatch regression failed before the fix and passes after
it; all 878 standard workspace tests pass. The final native check
also verifies the accepted original command on update 366. Its
message stays '76 0 0 1', the installed default. The earlier baseline
`1794023765220707000` passed the quest sequence in 317.13 seconds
before this command correction. Script coverage regenerated unchanged.

The initial sequence `1794023646379767500` completed and saved topic
one but failed a test assertion with the wrong log wording. Its
86.70-second failure was in the harness, not the conversation.
Later dialogue branches, item offers and the actual skull/feather
route remain open. The portal/plate source audit is diagnostic only;
it has not established the next playable route.
[Exact commands](docs/dead-city-progression.md#the-ferrymans-introductory-topics)

## 2026-11-07 Earned boathouse guard victory and reload

The two-process check passed in 72.82 seconds from RiverWeeperReload.
Connor changes view while standing, readies the earned Mace through
the melee HUD slot and follows the cleared road to skeleton38. One
ordinary Shift-click hit defeats the guard on update 282. Death
completes on 317 and awards 300 experience once. BoathouseGuardReload
retains level ten, stats [94,75,23,9,632], both weapons, third-person
view and the three defeated river-road enemies. AI and damage remain
enabled, and GOG Options.cs is unchanged. Local evidence:
`earned-boathouse-guard/1794023435988353000`.

This adds an earned campaign regression, with no engine-rule changes.
The Ferryman topics and the remaining Dead City quest are still open.
[Exact command](docs/dead-city-progression.md#the-boathouse-guard)

## 2026-11-07 Earned crossbow clears the river Weeper

The two-process campaign check passed in 76.51 seconds. From the
genuine `WesternPatrolReload` save, Connor selects the Small Crossbow
through the normal HUD, turns toward weeper15 in first person and
shoots through visible gameplay rays. Ten ordinary projectile hits
defeat it on update 395. Its death completes on 415 and awards 575
experience once: level nine becomes ten, health 11 becomes 100,
and the remaining experience is 332. The fresh `RiverWeeperReload`
retains stats [100,75,23,9,332], both weapons, first-person view and
the defeated Weeper, without another award. God mode and AI-off are
false, and the GOG Options.cs is unchanged. Evidence is local
`earned-river-weeper/1794022916871220500`.
All 877 standard workspace tests pass after the final input changes.

The new ranged route uses the normal ranged HUD slot and existing
shot path. Its visibility check needed the ranged aim mask; the
plain interaction mask excludes ordinary enemies. In first person,
the replay must turn Connor before requesting the visible shot.
Original damage, collision, animation and enemy rules are unchanged.

Earlier attempts never fired: third-person preflight encountered
doorway2 and used the wrong target mask; F2 during stoprun stalled
movement; standing F2 reached the firing point but the same mask
rejected the enemy; the corrected-mask run needed an ordinary turn
to bring it on screen. Logs remain in `dead-road-riverweeper`
(`1794022218511`, `1794022339903`, `1794022499420`) and
`earned-river-weeper/1794022717757448500`.

The mid-transition F2 case remains open as Q200. Forward stays active,
but the current Rust refresh replays stoprun after its transition
finishes and holds the last frame. The existing Q193 note is not
precise enough to change that behavior without study. This route
switches view while standing; it does not fix that separate case.
The reliable western-patrol replay and the boatman, skull, feather
and scales still remain open. [Route and command](docs/dead-city-progression.md#the-river-weeper)

## 2026-11-07 Western patrol victory, reload and repeat failures

The supplied exploratory encounter defeated skeleton55 on update 556;
its death completed on 591, awarding exactly 500 experience. The
genuine `WesternPatrol` save has 11 health and 2317 experience at
level nine. Its independent fresh-process reload passed in 10.14
seconds and retains both weapons without another award. The GOG
Options.cs remains unchanged. The verified saved checkpoint is now
`WesternPatrolReload`, but a reliable replay of this fight is open.

Two canonical encounter checks failed. Plain-swing retreat ended in
death on update 592 with the enemy at 15 health. The documented
Shift-click big swing was then checked through the existing input
path: two hits left the enemy at 10 health, but Connor's health fell
below zero on 586 and the route observed Die on 590. Stop further
melee variations. These do not establish an engine defect. Audit
ordinary ranged combat or a specific input issue before another
encounter attempt. The ignored route regression is explicitly
marked known failing; the reload test is a separate passing check.

Local evidence: successful exploratory route
`dead-road-suppliedpatrol/1794021424532`; failed canonical cases
`earned-western-patrol/1794021520583052600` and
`1794021685168491800`; passing reload
`western-patrol-victory-reload/1794021785994312500`.
[Checks and exact commands](docs/dead-city-progression.md#the-western-patrol)

## 2026-11-07 Boatman approach and input handoff

The supplied river run reached its final waypoint at update 1044
with 11 health, then waited for a save scheduled at 1500. A pursuing
skeleton38 killed Connor on 1108 during that idle wait. The process
was stopped after the recorded death; no new checkpoint was earned.

Two continuous boatman attempts reached the real Ferryman click and
his original 2,000-point award, restoring 100 health at level ten.
The first then timed out swinging at skeleton38, who had stayed at
his authored position 16,341 units away. The next used that original
road, but stopped at the talking spot: a scripted camera turn cleared
turn intent while the test kept holding Right. Original movement
rules require a release and re-press. Stop blind variations of these
routes; validate this specific input correction in a focused
conversation fixture before resuming. These runs are local
`dead-road-ferrymanroad/1794020163924` and `1794020324535`.
Neither completed the first topic or produced a campaign save.

The focused turn-input correction passed in 12.35 seconds. It
releases the held turn on update 13, leaves the conversation area
on 47 and sends TalkExit once on 85. The fixture positions Connor
at Ferryman and starts the original conversation directly. It uses
ordinary key events thereafter, with normal AI and damage. Its first
run reached the movement result but selected Options instead of
Save because Load was disabled in the empty save directory. The
corrected fresh-slot menu sequence passed and preserved GOG options.
All 877 standard workspace tests pass. The earned campaign
continuation has not passed yet.
The final focused rerun passed in 12.75 seconds.

The post-fix campaign run ended earlier on the river road: two
skeleton55 hits left 22 of the starting 47 health, and weeper15's
two drains killed Connor on update 753. It did not reach Ferryman.
The saved, verified boundary remains `CorridorSuppliesReload`.
Stop running-route variations. A supported next plan must address
the road enemies, using the earned weapons and the existing combat
studies, before claiming a reliable boatman continuation. The
Small Crossbow is carried, but a ranged encounter with these enemies
is **not tested**. Preserve the logs in
`dead-road-ferrymanroad/1794020859531`; no enemy rule was changed.

## 2026-11-07 First-hall supplies recovered

The three-process regression passed in 166.50 seconds. It uses
ordinary movement and visible clicks to recover the four remaining
Mushroom drops, saves in the hall, then consumes each on the return.
The final `CorridorSuppliesReload` has 47 health, level nine and
1,817 experience. The Mace and Small Crossbow remain carried; the
Mushrooms are gone and Cell04 stays closed. The GOG Options.cs is
unchanged. [Route and exact command](docs/dead-city-progression.md#recovering-the-remaining-first-hall-supplies)

The first southbound attempt hit flametow.10 repeatedly and stopped.
Original prop and door bounds identify the narrow west-side gap.
Walking through it at x 161885 with a 90-unit waypoint radius passed
in the subsequent exploratory run and the canonical regression.
No collider or combat rules changed. The river route with these
recovered supplies is the next check; the prior fatal route remains
documented below.

## 2026-11-07 River approach stopped by low health

The distinct running route toward the boathouse started from the
latest verified `OuterGuardReload` (27 health). It passed skeleton55
without attacking, but two hits on updates 403 and 420 left 2 health.
At the western turn, weeper15's bolt began draining Connor on update
607. Pulses on 611, 615 and 619 reduced health to 1, 0 and -1, then
requested death. The route did not reach the boatman or create a new
earned checkpoint. Evidence is local
`kq8-artifacts/dead-road-boathouse/1794019064701`.

The current save carries the Mace and Small Crossbow but no healing
items or active potion. Four uncollected Mushroom drops remain in
the already-cleared first hall, providing a possible ordinary supply
route before another river attempt; recovering them and continuing
is **not tested**. The northern side of the patrol is not an established
detour: the conservative offline planner cannot reach (105000,155500)
and stops at y 153250. Do not infer a route from its closest-point
output. Further western melee variations remain stopped as recorded
below. Enemy rules and health values were not changed.

Original reaper.cs gives 2,000 experience once on first interaction
and advances four conversation topics toward the Feather quest.
Ferryman is at (57071,193699,1050) in reaper.kq. The planner's solid
building bounds cannot establish the boathouse's real entrance.
That entrance, the conversations and their fresh-process save/reload
still need automated coverage. The campaign boundary is unchanged.

## 2026-11-07 Outer guard input audit and victory

The failed retreat approaches inherited the road’s enabled Run
toggle. The movement log shows running throughout the intended
walking approach. The replay now presses the configured Run key to
switch to walking before retreat input begins.

The distant approach then timed out following the guard. MONSTERS.KQ
assigns it a patrol, and its walking speed exceeds Connor’s. The
closer approach intercepts that patrol. The corrected exploratory
run defeated skeleton56 with all 27 starting health. The subsequent
two-process regression passed in 66.37 seconds and restored the
victory at 16 health, level nine and 1,817 experience. It verifies
missed enemy attacks, the Mace hits, one 450-point award and a fresh
reload. The original failed runs remain documented below.
[Check and route](docs/dead-city-progression.md#the-outer-mace-guard)

The next audit found that retreat input assumed every enemy hit at
halfway through its attack. It now waits for the actual `Do` cue in
the loaded animation. With both input corrections, the outer guard
regression passed in 66.89 seconds and restored all 27 starting health,
level nine and 1,817 experience. These changes affect the unattended
player, not the engine's damage or animation timing.

Two western patrol attempts from the earlier 16-health
`OuterGuardReload` ended in death. The first counterattacked before
skeleton55's actual hit cue and died on update 511, with the enemy
at 127 health. After the cue correction, more enemy attacks missed,
but two hits still killed Connor on update 616; the enemy had 28
health left. Stop variations of this encounter. Its original-runtime
comparison is **not tested**; these failures do not establish an
engine defect. Local `dead-road-westernpatrol` logs preserve both
attempts. The campaign boundary remains `OuterGuardReload`.
All 877 standard workspace tests passed after both input corrections.

## 2026-11-07 Junction equipment and the closed cells

The two-process equipment check passed in 36.12 seconds. Connor
takes the Mace and all three Mushroom drops, consumes each Mushroom,
and reloads with 27 health and melee strength 70. His broadsword
is on the ground where the Mace lay. Cell04 remains closed.

The first westward route stopped at Cell04. The offline planner had
omitted animated doors. Original ASwitch4.cs opens this gate from
a switch across the maze. Including the current save’s 59 closed
doors in the local planner found an outer northern route. The direct
Weeper route remains unverified. [Equipment check](docs/dead-city-progression.md#junction-equipment-and-the-closed-cells)

Two outer-corridor combat attempts failed. From the genuine
`NorthSuppliesReload` save, the first reached skeleton56 and
counterattacked, but Connor’s health fell below zero before the guard
died. The second started its approach on foot earlier and died on
update 142. The guard has 225 health, 15 hand damage and a 0.5-second
attack delay in the original data. Connor entered with 27 health.
The current replay’s retreat controller was built for the stationary
Daventry spear guard. Its earlier success provides limited evidence
for this pursuing enemy.

Stop variations of this encounter under the two-attempt rule.
The verified campaign ends at `NorthSuppliesReload`. Before another
attempt, audit the live input and movement record against the existing
combat notes and establish a distinct, supported plan for this enemy
or another route. Original-runtime comparison of this fight is
**not tested**. The failures are in local `dead-road-outerguard`
and `dead-road-outerguard2` logs. All 877 standard workspace tests
passed after the equipment check.

## 2026-11-07 Northern maze junction

The next group is a mace skeleton and two archers. The exploratory
run defeated them with 36 health left. The first canonical replay
died after circling outside an archer waypoint under fire. Its log
records repeated distances of 323–329 units against a 235-unit
arrival radius. Raising the three combat radii to 400 units allows
the player to start attacking earlier.

That two-process check passed in 66.44 seconds. It consumed the two
earned Mushrooms, awarded the group’s 650 experience once, and
reloaded all three enemies as defeated. The final save has 12 health
and 1,367 experience toward level ten. The Mace and three Mushrooms
they dropped are available for the western route. All 877 standard
workspace tests passed. [Route and command](docs/dead-city-progression.md#the-northern-junction)

## 2026-11-07 The three guards beyond Death’s gate

Two Windows processes verified that taking the first hall’s dropped
broadsword equips it, puts Connor’s Small Axe where the sword lay and
increases melee strength from 60 to 62. The crystal and exchanged axe
survived reload. The test passed in 28.98 seconds.

The equipped route clears both swordsmen and the archer. Connor uses
the crystal once, reaches level nine, and a third process restores
all three enemies as defeated with 80 health and 717 experience
toward the next level. The release test passed in 84.55 seconds.
The two-process continuation takes the guards’ two Mushrooms and
reloads at the northwest passage at the same 80 health. It passed in
47.97 seconds. All 877 standard workspace tests passed before this
additional replay. The two earlier fatal attempts
are documented below. The next northern junction has a separate group of
guards. [Automated route](docs/dead-city-progression.md#the-three-maze-guards)

## 2026-11-07 Beyond the spike gate

From the earned maze key, the two-process spike gate regression
passed in 90.29 seconds. `AcrossMazeGate` saves Connor on the far
side at y 105245 with 40 health. Two longer attempts to reach the
northern junction ended in combat, so that stretch remains unverified.
In the first, Connor killed the archer but was caught farther north.
In the second, the two melee skeletons and archer killed him near
the gate. The earlier hall fights left a healing crystal and a
broadsword available in the earned key save. [Test and route](docs/dead-city-progression.md#crossing-the-spike-gate)

## 2026-11-07 Death's hall saves and maze key

The first hall’s four pedestals open the doors to the letter tiles.
In the eight-launch regression from `FirstHallReload`, Connor
activated all four, saved in the hall, jumped to D, reloaded and
finished E, A, T and H. Death’s key sequence ran and a new process
restored the awarded key. `MazeKeyReload` has 80 health, level eight
and 1,197 experience. The regression passed in 740.44 seconds.

Before this fix, restoring an earned tile-entrance save left Connor
walking normally across the start strip. The disk format now retains
his tile controller and rebuilds its 64 stone references from the
restored world. A save during the jump holds both the puzzle state
and its playing movement. The compatibility path recognizes the
exact initial hall position in older saves. Later letter and jump
progress cannot be inferred in general from those older files.
The half-jump, original older entrance save and two-phase teleport
checks passed in 64.97, 36.27 and 98.67 seconds. All 877 standard
workspace tests passed. A single from-New invocation through the
maze key remains untested.
[Commands and evidence](docs/dead-city-progression.md)

## 2026-11-07 Dead City entrance archers and Paddle

The two-launch campaign replay passed in 158.21 seconds from
`DeadCityReload`. It restored one Paddle and both defeated archers,
with Connor at level seven, 58 health and 137 experience. The first
run walked back after the entrance script had already moved Connor
forward, and the archers killed him. The corrected route waits for
the conversation and fights from the script’s actual endpoint.
The road past the hall reached a locked spike gate. Original
`spmove.cs` requires the Key to Death’s Maze, supplied later by
`keycycle.cs`. The paddle pedestals and Death's hall come before
the skull on this route. [Commands and evidence](docs/dead-city-progression.md)

## 2026-11-07 Ring of Light and Dead City arrival

Five Windows launches from `AshesReload` verified the return portal,
the road back to the Wizard, the four casting movements and a
freshly reloaded Ring of Light in 396.95 seconds. The castle exit
first stopped short when its route relied on coasting into the
trigger. The passing regression crosses it with the normal forward
control. With the ring, the original tomb script sets its travel flag
and offers the Dead City portal. Dead City's `ShadBat.cs` removes the
earned ring and records the Shadow's defeat. The initial test wrongly
expected the ring to remain in Connor's inventory. The tomb approach,
gate crossing and fresh Dead City load passed in 214.44 seconds across three
Windows launches. The reloaded save has 60 health and 257 experience.
All 876 standard workspace tests passed. A single invocation from
New Game through Dead City remains untested. The campaign evidence combines a fresh Normal
start with its earned disk saves.

Dead City's good skull, the statue that produces the feather, and
the scales are the next campaign checks. The optional Daventry and
castle branches, later regions and complete ending remain open.
[Commands and evidence](docs/daventry-progression.md#return-the-ashes-and-earn-the-ring-of-light)

## 2026-11-07 Earned Castle Daventry entry and Ashes

The waterfall save now continues into CastleD through the cave and
the original region prompt. The three-process entry check passed in
203.98 seconds. The CastleD replay operated the outer light, waited
for the fireplace passage to open, collected one Ashes and reloaded
it in a fresh process. It passed in 118.38 seconds. The final
`AshesReload` save has 60 health and 207 experience. The Wizard
is still waiting for delivery. All 876 standard workspace tests passed.

The first approach reached the fireplace while its door was opening
and stayed pressed against it. Loading after it opened allowed passage.
A stone corner stopped the next route, which passed with greater
clearance. Neither issue required changing collision or disabling
enemies. Earlier Normal campaign saves supply the starting state.
A single default invocation from New Game through the Ashes remains
untested. [Campaign commands and original scripts](docs/daventry-progression.md#enter-castle-daventry-and-collect-the-ashes)

## 2026-11-07 The village fountain and the waterfall puzzle

Continuing the Wizard's two-ingredient save, Connor healed from 5 to
60 at the fountain. A second process restored his full health. The
road from there crossed the large bridge and entered the waterfall
rope trap. He climbed the rock face, pushed the bricks, gained 50
experience and regained control. A fresh load preserved 157 experience
and the change of waterfall. The three unattended checks passed in
135.27, 114.65 and 148.73 seconds. The route beyond the falls into
Castle Daventry and the return with the Ashes remain open in campaign
play. [Reproduction commands and inputs](docs/daventry-progression.md#the-waterfall-road-to-castle-daventry)

## 2026-11-07 Earned Shadow encounter and Wizard deliveries

The ordinary route from the urn now reaches the Shadow. In five
Windows launches Connor climbed out of the church, defeated the
archer and completed the twelve-movement encounter. The resulting
save had 46 health and 82 experience. A crossbow pickup assertion
initially counted two log lines for the same pickup. It now counts
the inventory event, and checks the saved crossbow quantity.
The first walking route out of the graveyard also exposed a real
in-game death at the hands of two zombies. Using the bound Run key
took Connor to the Wizard with 34 health and 107 experience in the
265.47-second two-process regression. From an earned
Wizard save the three-launch delivery and reload check passed in
239.54 seconds. All 876 standard tests passed.

The next required campaign segment goes through the waterfall cave to
Castle Daventry for the Ashes. The roof ambush variant, the Lake Sword
variant of the Shadow encounter, the Ring of Light and ordinary entry
into Dead City remain unverified. The continued campaign uses real
disk saves between processes. A single from-New invocation through
both deliveries has not been run. [Campaign route and commands](docs/daventry-progression.md#the-shadow-and-the-wizards-ingredients)

## 2026-11-07 Original menu selection masks

The installed main menu mask changes pixels only in the selected row,
wraps after 16 frames and uses a 100 ms real-time frame interval.
Four Windows launches covered Main, Options, Graphics and Keyboard.
The initial check missed the Graphics update because it logged only
timer-driven redraws. Checking the actual redraw path also covers
changes painted while an input event is being handled. The native test
passed in 15.54 seconds. [Menu details](docs/FEATURES.md#menu-selection-masks)

## 2026-11-07 Original HUD frame timing

The church Candle level-up now plays the installed flame and equipment
flashes at ten frames per second. The 17-frame effects show frame zero
again at 1700 ms and hide at 1800 ms, reproducing the original's final
interval. The Windows replay observed the first draw after expiry at
1846 ms. All 876 standard workspace tests passed. [Frame behavior](docs/FEATURES.md)

## 2026-11-07 Church urn opens the tomb door

The next two Windows launches continued the fresh campaign's Candle
save through the urn and a fresh-process reload. The door was open,
Connor retained the Candle and the urn award raised his experience
from 12 to 47. The test passed in 108.96 seconds. The next Daventry
checkpoint is the Shadow encounter. [Command and evidence](docs/daventry-progression.md#church-urn-and-tomb-door)

## 2026-11-07 Fresh Normal New Game through the church Candle

The default church regression passed in 1493.97 seconds. All 18 actual
Windows launches used only saves earned during this run. The final
`CandleReload` slot contained the dead hero’s ring, Rope and Hook and
Candle. Connor had 60 health at level 6. [Campaign evidence](docs/daventry-progression.md#church-candle)

## 2026-11-07 Earned the church Candle

From the earned church roof, Connor used the second rope to descend,
picked the Candle through the normal camera and kept it on a fresh
launch. The four-process test passed in 180.60 seconds. His save
contained the Candle, Rope and Hook and dead hero’s ring, with 60
health at level 6. The next story check is pushing the urn and opening
the route into the tomb. [Evidence and command](docs/daventry-progression.md#church-candle)

## 2026-11-07 First-person camera and combat

F2 selects the original actor-relative camera and movement lists.
Daventry's earned Small Axe struck the coffin lid with its first-person
animation. A separate Dead City weapon fixture fired one Small Crossbow
bolt with the first-person attack cue. Changing views while a script
owns a movement can leave its outgoing model active until the movement
ends. A menu save now retains both pieces of state. A church climb
started in first person forced third person and restored the original
view after a fresh-process dismount.

An integrated weapon-draw replay initially opened the menu three
updates later than requested while the renderer was busy. The outgoing
movement had already ended by then. Timed menu replays now allow at
most one simulation update per rendered frame, so the scheduled save
can inspect the intended animation. Near clipping, other weapons
and specialised controller branches remain open.

The five first-person Windows regressions passed in 367.63 seconds.
The standard workspace suite passed 875 tests. Three additional
original-file checks passed, and the release executable opened
New Game on Normal, entered Daventry and rendered 80 frames.
The earned Daventry scenarios use campaign saves. The crossbow
scenarios are focused fixtures with supplied equipment.
The commands are in the [README](README.md#automated-checks).

## 2026-11-07 Resumed a saved church climb and reached the roof

The original throw keeps the menu closed until Connor can accept input.
Once hanging, Connor saved on frame 22, restarted, advanced to frame 23
and climbed out onto the church roof. A first build resumed the motion
but triggered the upper trap while he was stationary. Retaining the
previous trap check position delayed that trigger until his actual move.
Two Windows launches passed in 82.10 seconds. The Candle remains ahead.
[Command and evidence](docs/daventry-progression.md#church-roof)

## 2026-11-07 Earned rope at the churchyard

The Windows test reached the churchyard from the previously earned mill
save with AI and damage enabled. A fresh process loaded the save made
inside the first church trigger. The upward hook appeared and the rope
could be selected. Two launches passed in 120.77 seconds. The earlier
build lost the active anchor at this point. Saving an active climb
and reaching the Candle are separate checks.
[Command and evidence](docs/daventry-progression.md#churchyard-rope-place)

## 2026-11-07 Fresh Normal New Game through the watermill rope

The unattended Windows test passed in 1405.41 seconds across 14 launches.
It earned every prerequisite from an empty save directory and checked two
ways of continuing after the tree fell. Connor collected the rope in the
running world and after loading a tree save. Both routes opened the mill
door, made the two jumps, entered the original rope trap and took the
placed rope through a visible click. Each route then reloaded the result
in a new process. Both final saves contained one rope and one dead hero’s
ring, with Connor at level 5, 35 health and 122 experience toward his next
level. The next campaign check is the church climb and the Candle.
[Command and evidence](docs/daventry-progression.md#watermill-rope-and-hook)

## 2026-11-07 From the hero’s ring to the watermill

A separate Windows regression took the fresh Normal campaign’s final
ring save to the mill in 131.20 seconds. Connor followed the northern
ridge and western descent, struck the tree with his Small Axe and
finished its scripted falling sequence. The resulting save has 50
health, the ring, the axe and the tree-down flag. The original script
loaded both fallen-tree objects and the dock. The Windows viewer now
registers their physical surfaces during play. The original-data test
matched both tree contacts and 375 walkable dock contacts against a
world reloaded with the same objects. The later rope test above covers
the mill door, stopped millstone and upper platform in play.
[Watermill route](docs/daventry-progression.md#watermill-tree)

## 2026-11-07 Fresh Normal New Game through the ring and disk reload

The full Windows regression passed in 951.45 seconds. It started with
an empty save directory and used nine viewer processes. The last one
loaded the ring save, saved again and exited through the menu. Connor
had one dead hero’s ring, 50 health at level 5, an open coffin, an
unlocked tomb door and the archer recorded as defeated. The game read
the supplied GOG installation throughout. Enemy AI and ordinary damage
were active. The original Options.cs passed the byte comparison.
[Route and reproduction command](docs/daventry-progression.md#verified-through-the-dead-heros-ring)

## 2026-11-07 Dead hero's ring earned and reloaded

Linked menu saves now cover Normal New Game through the dead hero's
ring. The original archer died in combat and the axe opened the
hero's tomb. From an earned HeroDoor save, Connor pushed the coffin,
clicked the ring from reachable ground and saved again. A fresh
process reloaded it at level 5 with 50 health. The ring, open coffin,
unlocked door and defeated archer all persisted. The [route and
command](docs/daventry-progression.md#verified-through-the-dead-heros-ring)
distinguish this campaign chain from the separate retreat diagnostic.
The Shadow, Candle, Ashes and Wizard exchanges still require their
own complete progression from this boundary.

## 2026-11-07 Fresh Normal New Game through the cottage Crystal and Wizard

The three-process garden regression passed in 399.88 seconds from a
fresh save directory. Connor earned the cottage shield, the Crystal,
eight mushrooms and then the original Wizard rewards. GardenWizard
held all three supplies. The Crystal provides a recovery item for
the hill archer. Two garden-only attempts had reached actual melee
range, but both ended in Connor's death before he earned the ring.

## 2026-11-07 Connor's original view-specific asset tables

The viewer's third-person base and weapon movement lists come from
the sections named by the installed Connor.kq. Automated checks also
resolved every first-person weapon file, the arm skin entries and
the authored idle and broadsword attack models from GAME/Common.
The README gives the source-check command. First-person camera and
controller integration remain open.

## 2026-11-07 Garden supplies through the windmill and Henchman

The GardenWizard save led to thirteen mushrooms at the hillside,
nine at the axe and two beyond the guard. The garden windmill test
and its fresh-process axe check passed in 215.34 seconds. The duel
and west-yard reload passed in 133.09 seconds. An independent run
from the earlier axe save passed in 132.91 seconds. Both loaded
WestYard with 40 health. The shield is now used at the outward
doorway after the last spoken line returns control.

## 2026-11-07 Eight mushrooms carried from the cottage to the Wizard

A two-process Windows test loaded the VillageShield checkpoint from
the verified Normal campaign. Connor opened Sarah's basket, picked
three more mushrooms outside, acknowledged the Jumping tutorial and
saved GardenMushrooms. A fresh process loaded it and walked to the
Wizard. The new save holds eight mushrooms, one shield, Piece1,
Mask1 and Magic Map. All three outdoor pickups stay gone. The test
passed in 327.90 seconds with damage and enemy AI enabled. The
[route notes](docs/daventry-progression.md#road-to-the-heros-tomb)
give the command. Carrying these supplies to the hero's tomb is open.

## 2026-11-07 Road from the windmill to the hero's tomb

Two automated runs loaded the WestYard slot earned from Normal New Game.
The first stopped against a pine. The second walked around it, made a
WestRoad menu save at 21 health and reached the hill below the tomb. The
archer killed Connor on update 982, before he reached the steps. The
hero's ring and the route into Dead City remain open. See
[the route notes](docs/daventry-progression.md#road-to-the-heros-tomb)
for the damage record and the original sources for more supplies.

## 2026-11-07 Earned Henchman defeat and west-yard reload

The two-process Windows test passed in 131.52 seconds. The prepared
windmill save contained a shield, four mushrooms, the Small Axe and
the map. Connor used the shield at 615, fought within the original
melee reach and exhausted the four mushrooms. The guard entered Die
at 817. At 907 his final animation ended, Connor received 30
experience and reached level 4, and the original script returned
control and removed clicker2. The GuardDuel menu save had 40 health.
A fresh process loaded it, walked beyond the guard at update 45
and saved WestYard with 40 health. The guard and click target stayed
removed. AI and damage stayed active and the GOG options byte check
passed. The two-process windmill acquisition and axe reload passed
in 178.21 seconds. The onward hero's ring route remains open.

The same duel test also passed without any supplied checkpoint in
640.92 seconds. Five Windows processes followed the original menu saves
from Normal New Game to WestYard, ending at level 4 with 40 health.
The guard entered Die at 813, completed at 903 and stayed removed
across the reload. This is the current verified campaign boundary.

## 2026-11-07 Post-Henchman save audit

The original death script completed, awarded 30 experience and created
hm_dead.kqs in the prepared duel. The old replay reported the removed
fighter as missing before saving. After using the recorded corpse,
the first integration still missed a 150-unit test waypoint and never
started its attacks. The guard's own position and Connor's melee reach
establish the correct encounter boundary.

At that boundary a second run defeated the guard, saved GuardDuel,
loaded it in a fresh process and crossed the west opening at update 46.
Both the guard and clicker2 stayed removed. The final assertion expected
identical health after standing in the yard. The original sprigwnd3
archer actually hit Connor five times, taking health from 40 to 23.
Verify the saved level, other stats and removal records, and account
for damage from the live world after loading. Save soon after crossing
instead of waiting in the archer's line of fire.

## 2026-11-07 Cottage basket and Wizard reward verified

The camera obstruction from the audit below is resolved. The walking
replay holds its click while the intended item is outside the viewport,
under a panel or behind another object. It uses the ordinary camera-drag
input until the normal pixel ray reaches the target first. In the
previously obstructed case the ray selected the stone mother at update
120, then the basket at 123. The original basket script gave five
mushrooms. The prepared Wizard slot kept them, one shield and the
original Wizard rewards. The two Windows tests passed in 391.26
seconds, covering the seeded continuation and the independent Normal
New Game shield pickup, fresh reload and timed expiration.

## 2026-11-07 Cottage basket replay audit

One uninterrupted supply route opened the basket, carried five mushrooms
and the shield to the Wizard, completed the original conversation and
saved the earned piece and map. The later windmill leg earned the Small
Axe and kept four mushrooms and the shield. The prepared run has not
passed on every fresh start.

The first attempted integration put the camera inside the petrified
mother's click box. The ray selected St_smom at distance zero, so the
basket remained full. A second western approach stopped at a real
cottage wall 355 units from its waypoint, before issuing the click.
Stop varying landmarks. The saved actor and camera traces from the
successful and failed runs, the original clickable boxes and cottage
BSP are available to locate a shared, visible pickup position. Only
count the branch after the normal on-screen ray and original basket
script succeed in a fresh run.

## 2026-11-07 Earned shield from Sarah's cottage

A Normal New Game route opened Sarah's door and collected the shield
in the east room. Both camera rays hit the intended object first.
The real menu saved the unconsumed potion. A fresh process loaded it
and used the normal slot-5 key. Protection rose from 0 to 50 at
update 12 and returned to 0 at 312. The three slots document the
earned potion, its active timer and its removal. The integrated test
passed in 146.04 seconds with damage and enemy AI on. The original
Options.cs byte comparison passed. The full four-supply detour and
the Henchman encounter remain open.

## 2026-11-07 West of the windmill

The two ordinary attempts to get past the stationary guard stopped.
The first jump hit the left fence. The second route crossed the
windmill doorway outward and continued towards a clear westward
opening. Three spear hits killed Connor before he reached that opening.
The hero's ring and its reload remain unverified. The campaign notes
record the two runs and the original village supplies that can be
earned before entering the windmill.

The separate four-item walking route stopped at the original cottage
door, then at an interior waypoint on the second run. Opening Sarah's
door and entering the east room are verified. Neither run clicked the
potion. See the Daventry campaign notes for its reach and visibility
check. Full encounter and campaign completion remain open.

## 2026-11-07 Ordinary combat death and delayed game over

Connor's pending death and his accepted death animation now survive
separate Windows processes. Equipped attacks and weapon swaps keep
their normal movement-end gate. The integrated two-process crossbow
and reincarnation check passed in 53.87 seconds. Temple 4's vortex
defeat regression passed in 48.69 seconds. Two Dead City cue schedules
verified whether a second hit arrives before or after Die has entered.
That final check passed in 48.21 seconds. The earlier failing
assertion is recorded in MODLOG.

657 core, 155 data, 7 audio and 41 viewer regular tests passed. The
ordinary and elemental death checkpoint tests also passed in the
implementation worktree. First-person exit, the remaining specialised
controllers and original Sierra save comparison are open.

## 2026-11-07 Pickup tutorial implementation

The integrated candle and watermill test passed in 66.11 seconds.
The first quest message opened 45 updates after pickup and the rope
message opened 68 updates after pickup. The original Dead City and
Temple 1 startup command ordering passed in 13.35 seconds. In a
separate three-process test, the cottage health tutorial opened at
401, exactly 45 updates after the mushroom receipt. All three saves
kept four remaining mushrooms and Connor's first-health flag across
the CastleD round trip. The portal test passed in 137.92 seconds.
The Normal New Game Wizard route passed in 229.76 seconds. It crossed
the farm after the help popup, received the quest piece during the
conversation, displayed the queued item and map help, and saved the map.

655 core, 155 data, 7 audio and 41 viewer regular tests passed.
The integrated viewer and its tests compile. Original runtime and
pending-event restore ordering are **not tested**. M4–M6 remain open.

## 2026-11-06 Integration build audit for pickup tutorials

The first combined cottage run loaded an older executable from the build
directory previously shared with the windmill worktree. Its slot had the
legacy flag but lacked the new help-history field. Rebuilding only the
local crates exposed a separate type error in an integration edit.
`world_build` owns an `Option<Box<Journey>>`. Two method-pointer forms
failed to compile. The implementation worktree's tested closure borrows
the boxed journey and calls its method through Rust's normal dereference.
That form is restored here. Keep the main checkout's build directory
exclusive to this checkout for subsequent tests.

## 2026-11-06 Daventry replay reaches the Henchman's attack

The earned-slot diagnostic passed in two independent Windows processes.
Both observed the original trigger and cutscene return, then crossed
west out of the doorway and reached the guard's first attack. A complete
integrated repeat passed in 138.50 s at updates 635 and 634. The full
duel and the onward hero-ring route remain open. Earlier attempts and
the original/source evidence for the separate branch around the guard
are recorded in MODLOG and docs/daventry-progression.md. No manual tests.

## 2026-11-06 Pickup tutorial source rules settled

Q196 establishes a three-second first-quest-item delay, the 4.5-second
rope delay, independent receipt and delivery gates, and stopHelp's
lasting effect on all seven flags. Development startup scripts actually
invoke that command. The earlier pickup prototype's immediate case 132
and seed-only suppression need correction before integration.

## 2026-11-06 Regional items survive travel and a Castle save

The two-process Windows test passed in 134.30 s. It starts on Normal and
finishes Daventry's original opening before leaving. A CastleD slot holds
six carried coins and seven coins left in Daventry under their original
name. A fresh process loads CastleD and returns to Daventry. Two clicks
give the seven coins once, bringing Connor's purse to thirteen. The next
item gets NoName24. The collected starting pile stays gone and the ring
retains its name, script and visibility. The original Options.cs byte check
passed. See MODLOG for the earlier fixture errors and reproduction commands.

650 core and 41 viewer regular tests passed. The installed Daventry data
check verified the restored model, click box and global name counter. The
viewer and tests compile without warnings. Original runtime and mid-flight
item continuation are **not tested**. M4–M6 and the campaign through the
ending remain open.

## 2026-11-06 Ordinary combat death studied

Q192 now has original static and installed-data evidence covering pending
Die requests, the accepted death animation, menu eligibility and the
three-second hold. An implementation worktree is using the prose note to
replace the temporary ordinary-death path and check old Rust saves.
Original runtime/native save comparison is **not tested**.

## 2026-11-06 Persistent traps return with their region

The Daventry–CastleD round trip now carries the departed region's trap
records through a save made in the other region. The native check passed
in 150.67 s across three processes. It observed one departure entry, the
silent removal of Connor before capture, mask 5 in both the Castle and
returned Daventry slots, and the original door, chest and mushroom state.
The first run exposed a second entry in the same handover. Moving removal
to region capture and gating subsequent simulation ticks fixed it using
the original route. MODLOG retains both runs and exact commands.

649 core and 40 viewer regular tests passed. The four original Dead City
spirit boxes passed regional save/restore, eligibility, old-format and
validation checks. The authored plate Lookup check passed. Script coverage
is unchanged. Original runtime, a dynamically created visitor on a full
return route and the Dead City altar campaign are **not tested**.
M4–M6 and the uninterrupted campaign remain open.

## 2026-11-06 Live indexed palette component verified

Live Graphics brightness now rebuilds indexed world images, menu art/backdrops, pointers and HUD/map/caption palettes from retained source inputs. Terrain, buildings, faces, late weapon images and ghost/dither variants share the active values; dimensions and true-colour/light-map paths are preserved. Graphics setup and film return reapply preferences. Q186's promoted-f32 gain, hardware channel-store differences, Glide exceptions, software clipping and source/adjusted haze/background rules are implemented. The separately authorized study supplied brightness.md rules 5a/7a; B used prose/readable data only.

All 851 regular tests pass (649/155/7/40 core/data/audio/viewer), plus both original-data checks. The final integrated Graphics run passes eight tests across 15 Windows processes in 426.76 s. D3D and Glide each change 0/100/0 live, save and resume from the real disk slot, and reopen Graphics with matching static artwork and unchanged original Options.cs. Both original films decode before returning through preference application. Viewer/test compilation is clean and regenerated script coverage is unchanged. Commands and local evidence are in MODLOG; no manual testing.

The earlier enhanced batch's two menu comparisons were a diagnostic error: its fixed crop included the reopened selection cursor. After stopping for a source audit, sampling excludes authored value rectangles and cursor columns. The same inputs then passed both reload cases and the complete batch. No engine constants, rendering or input times were changed to obtain the pass; MODLOG retains the failures and paths.

This completes the live indexed-palette component, not Graphics or M4–M6. Original runtime/native-original-save comparison, incoming x87 control state, final driver/texture output and complete film/true-colour/widget equivalence are **not tested**. Window-DIB system-colour substitution, graphics mode selection/live resolution, first-person/camera details, inventory/dialogue/credits scoring, remaining combat and ordinary Henchman/hero-ring routes, world-return state and uninterrupted campaign gates remain open. Next: publish this isolated component, then continue those milestone gates. The previous published spirit-trap/reaction component is c48a84c.

## 2026-11-06 Spirit traps and queued reactions verified

Q188/Q194/Q195 are implemented as a component: original spirit class/defaults/occupancy, queued stored health delta then ordinary force-1 hit, repeated Cue 1 and Cue 2 no-op, and optional trap/animation continuation. Ordinary/Choice/Talk retain ownership; lip requests, actual reaction playback, retained state name and input lock remain distinct. Wait/Move/Talk apply their own end rules. All eight Windows component checks pass across 14 processes (Wizard 52.92 s; the other seven 320.67 s). Full regular suites pass 649/153/7/38 (847 total); 52 focused controller checks and the four-box/two-cue-list original-data check pass. Final viewer/test compilation passes without Rust warnings. Generated coverage now names all four spirit traps. Existing unforced-hit and approach/speech/control-return regressions pass in 52.95/142.29 s. Both death functions pass eight scenarios in 535.49 s, including legacy Fire and interrupted equipped attacks. Total targeted Windows checks: 12 tests/36 fresh processes. Source/test/docs are ready for publication; M4–M6 remain incomplete.

Earlier failures are retained in MODLOG: the simultaneous-event fixture read the wrong log (22.33 s), the first roof save preceded original Choice entry (76.98 s), and the real Wizard lip code exposed an unconditional cut of uninterruptible react (27.99 s). The first two were corrected from existing event/script evidence; the third was corrected from the separately authorized Q195 study, with its attack/save times unchanged. The final component batch passes each case. Both death regression functions first failed an obsolete negative-health assertion (44.94 s); Q194 requires zero on dying entry. The corrected source-backed assertion and full death rerun pass, while the distinct simultaneous-event case still proves a later queued direct delta can leave health -1. No damage, protection, timing or movement parameters were tuned for a pass.

The source and automated procedures are in docs/provenance/README.md and MODLOG. No manual testing or executable inspection by B. Original-runtime/native-original-save comparisons and the complete Dead City altar route remain NOT TESTED. World-return spirit state, broader death eligibility/game-over timing, other controllers, ordinary Henchman/hero-ring and the uninterrupted campaign remain open. Raw study output, installed assets and artifacts are not committed.

## 2026-11-06 Third-person camera shake component

The original beer/gong waveform, shared follow/cut/flight calculations and optional disk continuation are implemented. The new three-process Windows check passes in 88.55 s; windmill flight/later-cut reload passes in 53.80 s; the New Game Wizard/earned-map route passes in 228.08 s. Final core/data/audio/viewer suites pass 639/153/7/38 tests (837 total); the five-call original-data check passes and viewer/test compilation has no warnings. Generated command coverage now recognizes KQCamera::shake. Final review preserved the existing conversation-around update causes while rebinding its retained stationary actor record; the repeated Wizard route/viewer/check pass.

The two new native fixture failures and the stopped/corrected actor-history fixtures are recorded in MODLOG, including source logs and failed roots. The latest combined shake test passes with numeric f32 comparison and follow's applied text read from viewer.log; production parameters were not changed to force a pass. Old destination/checkpoint and all sixteen supplied flight steps survive. Original GOG Options.cs byte equality passes in the shake and flight checks. Raw study material, extracted assets and build/test artifacts remain untracked. B used the original readable data and A's prose, never Mask.exe/disassembly. No manual testing.

Q193 is partly answered: A also settled first-person eye/support slope, F2/list/movement re-request/skin mapping, actor-space arm placement and shadow suppression; its initial body-hide claim was corrected. First-person implementation, exact near-plane/depth/near-wall clipping, physical drag/recentring, collision-probe/normal-length memory and universal original moved-versus-independent-timer deadlines remain incomplete/unverified. Two bounded traces stopped clipping, and two earlier traces stopped the timer-phase question; do not retry those blindly or claim full camera parity. Original-runtime and native-original-save comparison are NOT TESTED. M4–M6 remain incomplete. Continue the settled spirit-trap/live-brightness rules, ordinary Henchman/hero-ring/campaign route and the complete milestone gates.
## 2026-11-06 Original executable study handoff

The user-authorized separate study session A has completed its scoped handoff from the owned GOG executable/data. Q190 is answered in henchman.md (the Wait hit-deadline correction is already implemented/tested). Q188 is answered in traps.md: default state 2, double initial drain, integer floor zero at low levels, possibly lethal following Normal/forced-reaction hit, repeated Cue 1 and exact occupancy rules. Q187 is answered in camera-shake.md, linked from camera.md: angular 15-Hz waveform, accumulated envelope, fixed-cut/follow/flight behavior and saved continuation. These two controller implementations are still missing. Q186 is partly answered in brightness.md and corrected object-lighting rules 17/32: immediate slider palette rebuild, renderer-specific conversion/matching/haze/background, and no brightness-driven display-gamma call. Final driver/boundary quantization and full true-colour/menu/movie output remain open. Do not implement the superseded extra-gamma claim. Static original study did not run original runtime/native-save comparisons or manual testing. Raw study artifacts/tools remain ignored, with no game/extracted code published. M4–M6 are still incomplete; continue controller implementations and the ordinary-controls Henchman/campaign route, followed by all remaining milestone gates.

## 2026-11-06 Fixed-tick scripted holds recover the Henchman fixture

Synthetic --hold now samples before each fixed game tick, ahead of AI's backward-key test; the shared frame sampler keeps click/aim handling consistent. Physical keyboard handling and original game timing are unchanged. The identical stopped hold schedule now passes the three-process Henchman check in 95.56 seconds: damage 300 to 299 without flinch retains the unfinished attack/never deadline, its resumed Do misses at distance 2691 with backward released and reach 2500, its end schedules the next attack after 0.5 seconds, and the guard's feet/health remain unchanged. The complete check reaches and passes original Options.cs byte equality. Both roof-choice branches pass in 116.23 seconds, both nested Small Axe stages in 130.62 and moving-crossbow continuation in 58.12. Viewer passes 35 tests (21 ignored) and viewer/test compilation passes without warnings. The moving-shot test first failed at a hard-coded save remainder (0.3/0.8 rather than 0.366667/0.866667, 25.83 s); it now checks original intervals, their gap, both sides of the saved cooldown and run return from the actual second shot. No game timer changed. Artifacts: kq8-artifacts/hold-fixed-{henchman,choice,swaps,moving-shot,moving-shot-clocks,viewer,check,check-final}.log, passing roots henchman-combat/1793947345739307500, choice-checkpoint/1793947468357155400, choice-actions/1793947585597809100 and moving-shot-checkpoint/1793947876523785800. The two preceding failed input schedules remain below as history. M4–M6, full original-runtime parity and the winning Henchman/hero-ring campaign leg remain incomplete; this is protected/explicit-placement setup, not campaign acceptance. No manual testing was performed.

## 2026-11-06 Wait hit-deadline correction; retreat fixture stopped

Q190's separate executable study found that Wait repairs an attack's never deadline only when a surviving hit actually selects a reaction. The implementation now applies that gate and leaves the older non-Wait path unchanged. A five-case core check covers an uninterruptible movement, a reaction lock, insufficient roll, actual reaction and the unchanged Move path. Its baseline reproduced the wrong 10.5 deadline instead of never; the fix passes. Core/data/audio/viewer pass 632/153/7/35 tests (827 total).

Stop/handoff: two native attempts did not prove the retreat miss. The first held backward through Do and was correctly hit at distance 2837 inside extended reach 3125 (98.01 s; root henchman-combat/1793946598378617000). The second shortened the hold but still reached Do with backward held, distance 2544 (94.71 s; root 1793946752311744100). No more hold/tick variations: establish the replay input sampling/release path before trying this fixture again. Both runs reached the intended nonreacting hit/save boundary and resumed the attack; the combined test fails its miss assertion and must not be reported as passed. An additional log assertion was corrected to use the actual attack-end line without an invented fight prefix; the second run failed before reaching it. Logs are kq8-artifacts/wait-hit-deadline-{before,after,suites,viewer,native,native-released}.log. No engine range, movement speed, damage or health was changed to force a pass. M4–M6, the winning Henchman/hero-ring campaign leg and the retreat check remain incomplete. No manual testing was performed.

An explicit audit of the second run's generated slots/logs separately confirms one Do, one attack end with delay 0.5 s, never-to-finite deadline repair on completion, health 299 on both sides and the guard stationary at [137200,250000,15045]. This is partial evidence, not a passing combined fixture. New source evidence identifies the next action: --hold ranges are sampled in Update, so multiple FixedUpdate ticks can outpace a short press/release after a load. Move synthetic hold sampling into the fixed chain before the AI reads Connor's backward state, then check this same schedule; leave physical input and original combat rules unchanged.

## 2026-11-06 Choice animation-end callback

An ended one-shot movement now sends StartChoice when Choice owns Connor, without resetting the measured directions or decision. Ownership is checked before and after movement so a completed nested weapon swap cannot produce an extra entry call. The original-roof/gesture fixture reproduced zero callbacks in 60.56 seconds; the fix passed across three fresh Windows processes in 105.17 seconds, retaining identical Choice state through a second save and completing one exit choice after the next load. Both Small Axe swap stages passed across four more processes in 130.20 seconds, with one fresh entry each. Core/data/audio/viewer pass 631/153/7/35 tests; viewer/test compilation passes without warnings. Artifacts: kq8-artifacts/choice-movement-end-{before,after,core,viewer,check,swaps}.log, baseline root choice-movement-end/1793945675044424400, passing root 1793945780919659200 and swaps root choice-actions/1793945917509149300. This supplies placement and a KQAnimPlayEvent with god/no-AI; it is not an ordinary campaign route. M4–M6, full Choice quests, pre-choice waiting requests and rock/hammer continuation remain incomplete. No manual testing was performed.

## 2026-11-06 Directional-choice continuation

Choice list/cache/made state, queued calls and Free pose are now included in optional snapshots, with nested weapon swaps using their existing records. Core/data/audio pass 631/153/7 checks (19 ignored), viewer passes 35 (21 ignored), and final viewer/test compilation passes without warnings. All six original roof/basilisk/orc lists pass the data check. Five Windows checks cover seven scenarios across 14 fresh processes: both roof branches (117.42 s), a scripted pivot (56.11 s), both Small Axe swap stages (115.70 s), the chosen state with an original deferred NPC cue (59.38 s), and an explicitly supplied private pending-call record (56.04 s). Entry/cache are retained, pivots advance, the swap returns through one fresh StartChoice, and chosen outcomes do not replay. The Wizard approach/speech/control-return regression passes in 140.09 seconds. Original Options.cs bytes are unchanged. These are isolated original-trigger fixtures with god/no-AI, not campaign acceptance. Rock/hammer continuation, pre-choice waiting-request semantics, movement-end Choice callbacks and full quests remain open.

The two failed UI timing probes for a private pending DoChoice buffer remain a stop/handoff: at update 35 the original walk has already enabled handsOff and no slot is written; at update 34 a slot contains no decision. Artifacts: `kq8-artifacts/choice-native-actions.log` and `choice-native-pending.log`, roots `choice-actions/1793942742209455100/pending` and `1793943025076355300/pending`. Current FixedUpdate order emits and delivers that call before rendered-frame menu input. No further timing variations, menu-eligibility changes or artificial script delays were made. Ordinary UI capture of a nonempty private call buffer is NOT verified; the supplied-record case verifies restoration only. The reachable chosen/deferred-NPC-cue case uses the real Save menu and preserves one queued original event. MODLOG records this distinction and the duplicate entry, missed one-tick input, cursor-item and unflushed-log fixture failures. M4–M6 remain incomplete; no manual testing or executable inspection.

## 2026-11-06 Connor's combat-death continuation

An optional snapshot flag now retains the world's dying hit guard alongside the existing death Special list. Older slots derive it from the explicit final death movement/Kill marker; zero health does not imply death. Validation rejects inconsistent explicit records. Entering hit death also discards obsolete melee/ranged actions, preventing a waiting swing from replacing the death after its turn is cleared. Two unit checks cover current/legacy records, unchanged health after a later hit, zero health and invalid flag/list combinations. Core/data/audio passed 630/153/7 tests (18 ignored); viewer passed 32 (21 ignored); viewer/test compilation passed without warnings. Six original-asset death cases passed across 12 fresh processes in 362.43 seconds, covering Normal/Projectile, both Fire/Ice stages and a legacy Fire slot. Waiting Small Axe/Small Crossbow cases passed across four processes in 122.28 seconds. The native checks retain phase, ignore a later hit, avoid replaying the earlier drop sound and finish once into one death menu. An eight-case numeric audit matches the first resumed phase against the installed movement duration and speed at 15 Hz. Original Options.cs stays unchanged. MODLOG records exact commands, artifacts and the reproduced 326-point extra hit/restarted death, fixture type/module mistakes and unready-weapon case. These tests supply attacks/position with enemy AI off; they are not campaign or original-engine runtime comparisons. Q192 requests the untraced ordinary game-over timing, controls and save eligibility. Fall deaths, Die Now holds, other controllers, Q190/Q191 and M4–M6/campaign completion remain open. No manual testing or executable inspection was performed.

The living-hit regression passed in 52.22 seconds and still applies the expected later damage; the two Small Axe swap boundaries passed across four processes in 104.21 seconds. Artifacts are `kq8-artifacts/connor-death-{hit,weapon}-regression.log`. Both are regressions for living saves, not additional campaign acceptance.

## 2026-11-06 Live character collision flags and continuation

KQObject::collisionType and collision now resolve the live character before duplicate click boxes, update its actual type/mask, refresh its live click type and physical cylinders, and derive building, terrain-pass and Fall switches consistently. Placed, spawned and Special character checkpoints use the same setter; no saved format changed. Three core checks cover aiming with and without proxies, masks, zero refusal, JSON continuation, retired proxies and existing collider eligibility. Core/data/audio passed 630/153/7 tests (18 ignored); viewer passed 30 (21 ignored); final viewer/test compilation passed without warnings. The original Gnome fuse check passed across three fresh processes in 68.09 seconds: Dragon saves 0x4042 on, a deliberate temporary override saves 0x4002, and the next quest Init restores 0x4042; mask stays 0x281f. Its authored lighting, item consumption and single reward checks still pass, and original Options.cs bytes are unchanged. MODLOG records exact commands, artifacts and compile-fixture failures. This supplied-fuse/explicit-placement, god/no-AI setup is an isolated command/save check, not the full dragon fight or campaign. Q190, Q191, remaining controllers and M4–M6 remain incomplete. No manual testing or executable inspection was performed.

The original-skeleton and script-created-rat death-continuation regressions also passed across four fresh processes in 134.13 seconds (`kq8-artifacts/character-collision-deaths.log`).

## 2026-11-06 Authored interior lighting and restored Init notifications

useAlternateLightmap now switches each interior copy's authored .dil image and point-light baseline before dynamic lighting, clearing prior patches/haze factors and ignoring unchanged choices. All original regional .kq metadata contains exactly the Gnome13/Gnome14/Temple alternates; their 998/1295/2367 surfaces have identical layouts and 83,411/103,671/221,900 changed sequential texels. They remain permanent scenery; original defaults reload, and quest Init selects the saved choice. The Gnome three-process fuse-item test passed in 70.68 seconds with dynamic lights off, including independent temporary override and quest-driven reinitialization without duplicate reward. Temple 4's isolated on/repeat/off commands passed in 14.96 seconds with dynamic lights on. Two original-data checks passed in 7.84 seconds. Core/data/audio passed 627/153/7 tests (18 ignored); viewer passed 30 (21 ignored). Original Options.cs bytes were unchanged. The ordinary-board, goodbye/pause/return and timed scales reload regressions passed in 79.42/149.57/61.48 seconds. All 12 regions loaded and rendered 30 frames each in 183.52 seconds (`kq8-artifacts/alternate-interior-regions.log`); this is not campaign completion.

The reload test exposed queue replacement discarding fresh Init's zero-delay cue. Capture only immediate notices emitted during Active::start and reinsert after restoring saved events, keeping an equivalent retained notification's existing deadline; preserve new duplicate multiplicity, replace future Init timers as before, and consume capture on restore/first update. Core checks cover boundaries, saved deadlines and reconciliation. Q191 asks for native future Init/duplicate/listener evidence; broader queue parity remains unproven. MODLOG records the compile fixes, incorrect fixture/list assumptions, initial height rejection and lost-Init failure. Exact commands/artifacts are there, including the passing Gnome root kq8-artifacts/alternate-interior/1793936551388624600/. This is isolated equipment/position/god/no-AI setup, not the full fuse/dragon quest or altar narrative; dragon collision/controller behavior and visual parity remain open. No manual testing or executable inspection was performed. The Henchman two-attempt handoff below is unchanged and must not be retried without new evidence/strategy work. M4–M6 remain incomplete; continue their remaining interface, combat, controller and regional/campaign work.

## 2026-11-06 Daventry main-route continuation stops at the Henchman

Goal remains complete M4–M6; none is complete. The earned Normal New Game → wizard → windmill Small Axe route, plus its fresh-process reload, passed in 402.85 seconds (`kq8-artifacts/daventry-ring-foundation.log`, `windmill-route/1793933457485922800/`). Its slot-3 has health 30, Small Axe, Magic Map, AI on and god mode off, with the five gathered mushrooms consumed. A new ignored `daventry_hero_ring` campaign scenario defaults to earning that foundation, or accepts KQ8_AXE_SLOT to repeat only its new leg. It walks off the bale and runs the original Henchtp/ax/Henchman cutscene without forced flags or travel.

Two genuine continuation attempts failed at the same encounter. The charged-swing replay died at update 761 (75.39 s process, Henchman left at 238 health). After reading the original axe animation data (attack2 2.4 s, attack/attackb 0.8 s), a normal-swing replay died at update 762 (75.46 s, Henchman left at 205). Connor stood in reach without healing items; this is not proof of an engine bug. The standing-only combat replay may need a real strategy, and fighting.md rule 5 explicitly leaves this scripted character outside its coverage. Q190 requests the missing specific state/attack evidence. Per AGENTS.md, stop further variations of this encounter after these two attempts. STATUS-handoff.md named by that rule is absent; this record contains the handoff instead. Keep engine health/damage/timing unchanged until source evidence warrants a change. The ring, its guard, coffin push, later path landmarks and ring reload have NOT been reached or tested. Do not present the new scenario as passing campaign acceptance.

Artifacts: `kq8-artifacts/daventry-ring-route-first.log`, `hero-ring-route/1793933920724401900/`, `daventry-ring-route-normal-final.log`, `hero-ring-route/1793934160572852200/`. A compile-time borrow conflict while adding the replay's swing selection was fixed by copying the option before mutating replay state; it is recorded separately in `daventry-ring-route-normal.log`. The replay now also supports ordinary weapon dispatch against a named breakable with live stats. Its isolated original-door diagnostic passed in 29.92 seconds with explicit equipment/position, protected Connor and AI off. A normal axe swing at update 41 reduced original tombdor2 health 1 to -29, called Die Normal once and opened the door and removed its click bit. Artifact: `kq8-artifacts/daventry-ring-door-diagnostic.log`, `kq8-artifacts/tomb-door-replay/1793934385243112000/`. This cannot substitute for the failed campaign route. The regular viewer suite passed 29 tests (19 ignored), logged in `kq8-artifacts/daventry-ring-viewer-suite.log`. No manual testing or executable inspection was performed. Next: resolve the Henchman evidence/strategy and continue the incomplete interface, combat, controllers, region quests and uninterrupted campaign. Do not mark the overall goal blocked solely on this encounter while other implementation work remains available.

## 2026-11-06 Named ordinary copies and one-shot continuation

Runtime ordinary copies now retain source file, name and current placement, including multiple aliases and legally replaced startup names. Kept-world capture removes live replacements from prior removals. Ordinary persistent animations now retain clock, pause, playing/waiting/transition state and delete-when-done before Init; both one-shot callback paths use retained health-losing hit kind or Normal. Core/data/audio passed 624/153/7 tests (18 ignored); viewer passed 29 (19 ignored), including legacy/invalid slots and repeated deletion/recreation capture. The GOG loader reconstruction passed in 6.00 seconds. The final three-process original boards check passed in 79.66 seconds: a running saved phase 0.46666667 completed at restored update 7; the paused twin at 0.06666667 resumed at update 5 and completed at 18; each ran its original Die Fire/Ice branch once and made one original wreck. A third load retained both wreck identities/places, both removed effect names and the NoName counter. The bell-copy regression passed in 82.40 seconds; original-skeleton and script-created-rat death regressions passed across four processes in 153.41 seconds. Original Options.cs bytes stayed unchanged. Exact commands/artifacts and the corrected missing-environment coverage invocation are in MODLOG. This is isolated injected fixture state with protected Connor/no-AI, not the boards quest or an ordinary-input campaign. A named-copy portal round trip, specialized-controller cloning, other controller saves, Q186–Q189 and remaining interface/combat/region quests remain open. M4–M6 are incomplete; no manual testing or executable inspection was performed.

## 2026-11-06 Fresh stats for runtime-loaded objects and reused names

Original .kq stat prototypes are now separate from live/saved stats. Dormant activation and fresh template copies install the file's stats and clear old hit history before Init, after the existing name-use guards. Supported dormant ordinary objects retain recipes, and an already-loaded ordinary file can get a hidden recipe without changing its live/dormant identity. A live replacement also prevents its retired animation from marking that name gone in the next save. Three core checks pass for defaults, Init events, refused loads, legal reuse and both save/departure gone lists. Original-data checks confirm bell health 1, default door health 10,000 and ordinary-object zero defaults. Three Windows processes passed hanging the bell via the installed stand script, saving at zero after one hit, recreating it with fresh health/history and reloading that new bell for one Die Ice callback. Core/data/audio passed 621/153/7 tests (18 ignored total), viewer passed 27 (18 ignored); the original-data and Windows checks passed in 7.77/82.70 seconds. GOG Options.cs was unchanged. The original-skeleton and script-created-rat death reload regressions also passed across four processes. MODLOG records the exposed recipe gaps, retired-name bug and fixture fixes. This is an isolated creation/save/reuse check with protected Connor and AI disabled, not the Lady-of-the-Lake quest or full campaign. Other controllers' copy behavior, AnimOnce completion kinds and Q186–Q189 remain open. M4–M6 remain incomplete; no manual testing was performed.

## 2026-11-06 Persistent ordinary-object combat state

Objects loaded with the original persist manager enabled now retain their live stats and last health-losing hit kind in kept interactions. The loader restores these after original defaults and before Init; permanent scenery, absent/deleted targets and characters are filtered out. Old records retain original defaults. Stats-event Die callbacks now use the saved kind rather than always Normal. Three core checks passed for JSON/legacy records, persistence/liveness filtering, repeat capture, all four kinds and zero health. The original tomb-door two-process check passed in 60.03 seconds: a one-point Fire hit leaves zero health without Die, the slot keeps zero/Fire, and a negative stats event after loading sends one Die Fire; the original script opens the door and removes its click bit. Core/data/audio passed 618/153/7 tests (18 ignored total); viewer passed 27 (17 ignored). The cottage/castle save regression also passed after raising its frame cap; route inputs and assertions are unchanged. MODLOG records commands and the fixture corrections. History for fully blocked/zero-strength hits needs Q189; AnimOnce completion kind, new copies' initial stats, other controllers and the complete campaign remain open. M4–M6 remain incomplete. No manual testing was performed.

## 2026-11-06 Connor's stats-event death and hit-kind memory

Fatal stats events now request Connor's existing death sequence once, with the last accepted hit's kind or Normal before any hit. The below-zero check precedes caps and level-up healing; zero stays alive. The optional journey field retains the last kind through transfer/JSON saves, and old records default to no prior hit. Two core checks cover all kinds, ignored god/rope/dying hits, the old-record fallback, zero/below-zero, healing order and duplicate death requests. The three-process Windows check passed in 115.53 seconds: normal die finishes and opens one death menu; a nonfatal Fire hit saves health 79 and its kind, and a later stats event after loading plays dropweapon then firedie and opens one post-restore death menu. GOG Options.cs bytes stayed unchanged. Core/data/audio passed 615/153/7 tests (18 ignored total), viewer passed 27 (17 ignored). The existing two-process hit-reaction timing regression also passed. MODLOG records commands and two corrected log assertions. Ice/Projectile Windows sequences, saving midway through Connor's death, plain-object last-hit history and non-combat character deaths remain separate work. M4–M6 are still incomplete; no manual testing was performed.

## 2026-11-06 Live monster stats and Clarity continuation

Stats-change events now update combat characters' live fight stats rather than their static-object records. Below-zero health enters the existing death controller with the last hit kind; exactly zero remains alive. Core checks cover prior injury, stat caps, all four death kinds/default Normal, delayed Die callback and one experience award. The original Snowmane Clarity route passed across two fresh Windows processes: hotkey 7 consumes the original item, attack/protection change 350/30 to 200/0, the slot retains those values and the potion, and expiry at restored update 885 returns them to 350/30 and removes the potion. A separate original skeleton stats-event death passed disk continuation and awarded 200 experience once. Core/data/audio passed 613/153/7 tests (18 ignored total); viewer passed 27 (17 ignored). MODLOG records commands, artifacts and the failed direct-script console fixture. These isolated routes disable AI and protect Connor; they do not verify the complete Snowmane fight or uninterrupted campaign. Connor's fatal stat-event path, plain-object last-hit kind and non-combat character death handling remain separate gaps. Spirit traps need Q188 research before implementation. M4–M6 remain in progress; no manual testing was performed.

## 2026-11-06 Trap Lookup and Sylph character loading

Flagged generic traps now queue their Lookup callback after Init and after a disk restore, with live/script guards and pending-event deduplication. Runtime Init's own queued cues remain ahead of Lookup. The original Dead City check confirms all four plate flags and twelve makeActive requests. It also exposed a loader bug: Sylph's static shapeName prevented her valid 17-movement animFile from becoming a live character. Characters now follow their loaded animation list even when shapeName is supplied. Four fresh Windows processes passed both saved-flag cases: each Lookup runs once, all-up places Sylph at the well, and all-down moves her away; Options.cs bytes are unchanged. Core/data/audio passed 611/153/7 tests (18 ignored total), viewer passed 27 (17 ignored). Exact automated commands, artifacts and failed checks are in MODLOG. This is isolated plate-state reconciliation with AI disabled, not the full puzzle or campaign. Original queue-service timing, trigger subclasses, other controllers, world-return persistence and the remaining M4–M6 work are still open. No manual testing was performed.

## 2026-11-06 Scripted footsteps and occupied-trap checkpoint

KQObject::setFootStepSound now changes the live building footstep type, ahead of its loaded default, and optional disk fields retain those overrides. The Temple 1 route exposed repeated On after loading; generic trap saves now retain ordered occupants, shrunk state, live mask and the one-shot gone flag by name. The strengthened two-process route passed: the real sloshtrp sets temple_e to water, the slot keeps type 3 and Connor inside, reload produces water without repeating On, and exit sends one Off, changes to dirt and saves type 0 with an empty inside list. Core passed 609 tests (15 ignored), viewer passed 27 (16 ignored), the original On/Off script check passed, and the existing Wizard approach/speech four-process regression passed. Coverage now recognizes both original footstep commands. The initial harness read doors.log instead of traps.log; corrected logs then revealed the occupancy bug, which the new checkpoint fixes. Exact commands and artifacts are in MODLOG. GOG Options.cs stayed unchanged and no manual testing was performed. M4–M6 remain incomplete: trap Lookup callbacks, subclass counters, world-return/airborne/monster footstep behavior, other controllers, Graphics, inventory/dialogue, combat and the continuous campaign remain open. Camera shake details are unresolved in Q187; hardware audio and visual parity are not tested.

## 2026-11-06 Fresh-load palette brightness component

Fresh world textures, HUD/caption palettes, loading artwork and newly opened menu artwork now apply the documented palette stage of setBrightness. Raw palettes remain cached for matching; true-colour texture bytes and dimensions are unchanged. The neutral value 50 is byte-identical. Core/data suites passed 760 tests (18 ignored), the viewer suite passed 26 (15 ignored), the installed-data check passed for 393,216 decoded Connor texture pixels across Daventry/Dead City and values 0/50/100, and three fresh Windows Dead City runs passed. Those launches changed 254, zero and 246 entries in NEWDEAD.PPL respectively; GOG Options.cs bytes stayed unchanged. The first installed-data check assumed DeadCity.ppl; it was corrected to follow the original world.cs newdead.ppl setting. Exact commands and artifacts are in MODLOG. This implements only the palette component: display gamma and live brightness remain missing, and original byte rounding, matching order and API behavior need Q186 research. Screen-specific visual parity and hardware gamma are not tested. No manual testing was performed. M4–M6 remain incomplete: continue Graphics runtime effects, inventory/dialogue, combat actions, cutscene controllers, region quests and the uninterrupted campaign.

## 2026-11-06 Interrupted conversation and camera-state checks

Two new unattended regressions passed across six fresh Windows processes without requiring a runtime change. The first saves/reloads during the original Wizard goodbye, the paused queue and welcome back; it preserves voice offsets and both waiting lines, sends one delayed TalkExit, stays silent when the pause loads, restarts the interrupted normal line from its beginning on return, and completes each normal line once. The second preserves the original midpoint and suppressed camera cuts after the conversation-button action; Connor can move inside the stay area without carrying the camera, the next line's camera 3 remains ignored, and completion restores following, HUD and movement control. The existing `--right` harness does not inject physical mouse motion, so actual drag/swing continuation is not tested. These isolated two-line conversations do not verify other partners or the Wizard's reward scene. GOG Options.cs stayed unchanged. No manual testing was performed. Exact commands, artifacts and limits are in MODLOG. Continue M4–M6: remaining Graphics runtime effects, inventory/dialogue branches, weapon/enemy actions, other cutscene controllers, region quests and the uninterrupted campaign remain incomplete.

## 2026-11-06 Camera destination-list checkpoint

The scripted camera checkpoint now retains its ordered `addCamLoc` destination list as an optional field. Older records keep the previous loaded-world fallback, while an explicitly saved empty list clears startup destinations; non-finite coordinates are rejected before mutation. An unattended two-process check using the installed windmill camera coordinates and duration reproduced the missing-destination failure before the fix, then passed: a saved flight at 9/31 resumed at 10/31 from the next expected position, completed once and supplied the retained destination to a later cut. This is an isolated camera check, not the full windmill scene. The viewer suite passed 26 tests (14 ignored), the GOG Options.cs bytes were unchanged, and no manual testing was performed. Exact commands and artifacts are in MODLOG. The preceding conversation weapon-away/out fix was published as `da56d76`, with matching local and remote trees. Continue M4–M6: live orbit, goodbye/return, other cutscene controllers, remaining Graphics, combat, region quests and the uninterrupted campaign remain open. Hardware audio and visual fidelity are not tested.

## 2026-11-06 Conversation weapon-away/out checkpoint

The GOG Small Axe and Small Crossbow conversation checkpoints passed in eight cases across 16 unattended Windows processes: before and after the put-away sound cue, at the core/viewer list-change boundary, and midway through `out`. A slot at the boundary previously failed validation because the core had cleared its weapon while the viewer still held the completed drawn `away` pose. A narrowly checked exception now preserves that legitimate state for the next normal weapon synchronization; five invalid near-matches are still rejected. All eight reloads advanced by the installed loop's expected phase increment, delivered each remaining put-away cue once, completed one TalkExit, restored the HUD and accepted movement. The viewer bin suite passed 25 tests (14 ignored); compilation and diff checks passed. Original Options.cs bytes were unchanged, and no manual testing was performed. See MODLOG for commands and the initial Fists fixture mistake and boundary failure. M4–M6 remain incomplete. Next: the camera checkpoint currently omits GameCam's `spots` destination list even though original fly/cut commands retain it; preserve that list and check an ongoing flight plus a later cut across a fresh-process load. Live orbit, goodbye/return, other cutscene controllers, Graphics, combat and the uninterrupted campaign remain open. Hardware audio and visual fidelity are not tested.

## 2026-11-06 Conversation continuation checkpoint

Conversation, Talk pose, camera and HUD records now supplement speech saves. Original info objects and animation names are resolved after world/weapon loading; entry and old camera cues are not replayed. The GOG Wizard approach and mid-speech routes both passed across fresh Windows processes: the approach advances, camera 4 is retained without a repeated cut, one TalkExit occurs, the prior bar is restored and movement control returns. Each route checks an early resumed slot and a final slot. Core tests passed (607 passed, 15 ignored), viewer tests passed (24 passed, 14 ignored), and the earlier speech-origin/offset regression passed. Camera history and remaining flight/roll steps also passed synthetic core round trips. The initial live assertions needed corrections for a valid stage advance and f32 decimal formatting; details and exact commands are in MODLOG. No manual playtesting was performed. M4–M6 remain open: verify weapon-away/out, live orbit/flight, goodbye/return and other partner/controller saves; finish the remaining Graphics, combat and uninterrupted campaign work. Visual parity is not tested.

## 2026-11-06 Speech checkpoint

Speech queues now have an optional disk record with relative clocks, IDs, future mouth cues, delayed replies and goodbye pauses. Restore uses installed message/audio files and a Resume presentation event, avoiding old gestures and line-start camera cues. The mixer and sound clock seek to the saved offset; the original spatial source is preserved when a speaker moves. Core/data/audio checks passed (602/152/7, 18 ignored), all 12 speech tests passed and the viewer suite passed (24 passed, 14 ignored). The unattended Daventry route resumes tuple 1000/4/0/8/1 at 2.200 seconds, retains the old sound origin after moving the Wizard 10,000 units, completes once at update 83 and writes a second slot with speech finished. See MODLOG for the initial range, JSON-path and frame-budget test failures. No manual playtesting was performed. M4–M6 remain in progress: conversation approach, camera and control state still need a checkpoint, and Graphics, combat and campaign gaps remain. Hardware audio and visual parity are not tested.

## 2026-11-06 Scripted shadows and disk-save checkpoint

The original `allowShadows` command now suppresses every shadow category independently of Graphics preferences. The loader receives each world's Load default; live scripts update the active switch; disk saves optionally restore it after startup, while older saves retain the loaded world's default. All 12 original Load defaults passed; full Dead City/Gnome loading and the original upper/lower Dead City rope scripts passed. Two unattended Windows processes saved suppression with mask 15, restored it, then re-enabled Connor's and a nearby original skeleton's shadows and saved the enabled state. The Graphics mask and GOG Options.cs were unchanged. Core/data passed 751 tests (18 ignored), the viewer suite passed 24 (14 ignored), and the existing Graphics category-toggle regression passed. The regenerated script coverage marks allowShadows built. No manual playtesting was performed. Continue M4–M6: positive visible object projection, other world Load effects, live resolution, brightness/API modes, interface/dialogue, remaining combat actions and the uninterrupted campaign remain open. See MODLOG for commands and the initial Gnome loading-order failure that was corrected.

## 2026-11-06 Live shadow controls checkpoint

M4–M6 remain in progress. The Graphics panel now updates the active world shadow mask; Connor uses bit 1, and placed/copied characters keep their own mask and original opacity so categories can be enabled after loading. Turning a category off hides its existing shadow and invalidates plain-object projections for re-enabling. Compilation, the viewer bin suite (23 passed, 13 ignored), and all four unattended Windows Graphics checks passed. The live route gained, lost and regained Connor's and an original Dead City skeleton's shadow; object cache rebuild totals increased from 487 to 974, but this location has no visible object shadows, so positive object projection there is not tested. The GOG installation was read only. No manual playtesting was performed. Continue with the missing scripted `allowShadows` switch and its save state, then the remaining Graphics effects, combat actions and campaign work. The changes and test commands are recorded in MODLOG.

## 2026-11-06 Windows fork published through live Dynamic Lighting

The GOG graphics preference reader, original Graphics menu path and controls, live Dynamic Lighting application, and Small Axe hand-weapon checkpoint were published to `rpmasse/KQ8-REA` through remote commit `05f1c3c`. Installed-data checks, unattended Windows graphics preference and menu routes, an active Dead City lighting switch with changed ground-light samples and eight point-light patches, the existing Options regression, the viewer bin suite, and both Small Axe swap reloads passed. The GOG installation was used read only. The remote and local trees matched at `24b01e9`. The 1.16 MB `MODLOG.md` blob was uploaded as UTF-8 after base64 transport failures; noninteractive Git push also lacked a credential. Continue M4–M6; other graphics effects, other weapon types, live encounter saves and the campaign remain open. No manual playtesting was performed.

## 2026-11-06 previous Windows fork checkpoint

The GOG Small Crossbow weapon-swap and pending melee/ranged combat-cue checkpoints were published to `rpmasse/KQ8-REA` as remote commit `7951da0`; the local and remote trees matched. A previous GitHub connector transport failure was resolved. Automated Windows checks passed for the three fresh-process swap states, the standing-shot turn/release/post-impact states, the queued Small Axe hit, the moving shot, the Small Axe pre-cue regression, and the viewer bin suite (22 passed, 13 ignored). No manual playtesting was performed. Current M4 graphics preference work is in progress and is described in the README and MODLOG.

**Windows fork, November 5.** M4–M6 are in progress. The GOG reference and its executable fingerprint are recorded in [provenance](docs/provenance/README.md). Automated checks cover the three original movies, scored credits, scripted Help and F1, Lucreto's combat forms, and the Temple 4 scripted defeat menu. A New Game route reaches Daventry's wizard, earns the Magic Map, then reaches the windmill, defeats its archer and saves with the Small Axe. A fresh process restores the moved bale and axe. Other checks cover the castle round trip, the grave raven resuming mid-flight, the Dead City scales, the Tower Skeleton and timed puzzle saves. Graphics controls, live encounter saves, region quests and the uninterrupted campaign remain open. The [README](README.md) gives the checks and their limits. The notes below retain upstream history.

Hand-over in case the chat is lost. Brought up to date 2026-10-20 after
B built the fight (M5 5c, `fighting.md`). Read with `AGENTS.md`, `docs/DESIGN.md`, `docs/SESSIONS.md`,
`MODLOG.md` (newest entries at the top). The short public summary is
`README.md`; the detailed "what works" list is `docs/FEATURES.md`.

## Where we are (2026-10-31)

- **2026-10-31 (B): Q176–Q181 applied** (I-1 to I-7, logs only, not
  looked at by eye): the north tile jump lands (steps 325); the teleport
  camera flies at 5,000 a second and follows again when the flight ends,
  with the hold switch; scripts' weapon requests on the stones say the
  line, the Jump key jumps; Death's mouth codes; the camera commands read
  numbers as C's `atof`. **Not done:** I-2 (the stones' bob drift of
  +0.011 per cycle: left out by the user's choice, the stones stay
  level) and Q175 (the
  tower skeleton, a new feature). Also H-1 (armour skins), H-2 (deep
  water run-out) done; **open for A:** Q184 (armour skins), Q185
  (`gobcon1` put back home over and over).

- **2026-10-30 (A): Q175 to Q183 answered** (traced; nothing built yet).
  No question is open any more. What B should change:
  - **North tile jump (Q177):** keep step up / down at **325** during a
    tile jump (only gravity and keep-on-ground go off, back on when the
    jump movement ends); the step-up climbs the stone's edge.
    `tile-puzzle.md` rule 11. The 29 box-less stones are right.
  - **Stones' bob (Q176):** count the wrap step (`objects.md` rule 6a);
    then the cues cancel the leftover and B's stand-in can go.
    `tile-puzzle.md` rule 16.
  - **Teleport camera (Q178):** a `flyToCamLoc` with no time (distance
    ÷ 5,000 per second), starting when `teleportdown` ends; the hold
    switch keeps the camera's height and tilt. `tile-puzzle.md` rules 7a,
    13.
  - **Weapon line (Q181):** said on entering only with a weapon out (as
    built); also for scripts' `setDoMode` / `setRequiredWeaponMode`
    (B's are silent). The Jump key jumps too. `tile-puzzle.md` rules 7,
    8, 18.
  - **Camera roll (Q182, Q180):** G-5's sign is right (negative =
    picture clockwise); no scaling; numbers read with `atof`.
    `camera.md` rules 4a, 4b.
  - **Death's mouth (Q179):** code 16 starts `rest` (not `stop`); the
    poses hold; end of line does nothing. `talking.md` rule 18a.
  - **Death's key (Q183):** the spin twice is right. `anim-scripts.md`
    rule 3.
  - **Tower skeleton (Q175):** its standing, moving and falling states
    are in `fighting.md` section 7a (the stagger's ground probe is
    behind it: rule 23 corrected). The fall's camera and move offsets
    are as read, not checked in play.

- **2026-10-30 (B): G-1 to G-4, fixes after the user's check of F-4 to
  D-6** (logs and unit tests, **not looked at by eye**): on the stones
  the weapon is put away for good and every swap is refused with "A
  weapon will not help me here." (also said on entering: the user's
  check, Q181); the camera's roll is drawn (sign from the user's check,
  Q182); copies loaded while playing move their collider, so Connor
  rides the bobbing stones; an object a script loads starts its
  movement afresh, so Death's key flies to Connor and is given.
  **Open for A:** Q177 to Q182. **Next for B:** D-8 (batch 2 list below).

- **2026-10-30 (B): F-4 to F-6 and D-6** (logs and unit tests, **not
  looked at by eye**): Death is a character (`KQDeath`): his whole scene
  runs, his lines with mouth movements, the key, Connor sent outside;
  `flyToCamLoc … -.6` read (Q180); the torches and flames of his room
  are made; `changeBmp`, so the Clarity potion shows the five letters
  (and `tileshow`, loaded while playing, now gets the potion's calls).
  **Open for A:** Q177, Q178, Q179 (Death's mouth details), Q180.
  **Left in batch 2 for B:** D-8 `matchHeights` / `terAnimSwitch`, D-9
  `Lookup` and spirit traps, D-10 `ConnorCollision` and `Scales`, D-11
  `ThrowRock`, `KQWeepBolt`, `RiverControl`, `KQAttackEvent` on objects,
  `KQMonster::add`; `KQTowerSkeleton` waits for Q175. `changeBmp` on still
  objects, doors and characters is logged, not drawn.

- **2026-10-30 (B): fixes after the user's check of the death hall**
  (DESIGN "Fixes after the user's check", F-1 to F-3; logs only, **not
  looked at by eye**): the halls' doors now stand where `newWorld.cs`
  moves them (they were all in the hall's middle, blocking Connor);
  Death's trap is armed at the landing on H, so his scene starts at
  once; gravity is off while a tile jump plays, so Connor rises. Still
  wrong: north jumps are about 25 short of the next row (Q177); the
  camera flight after a teleport is too fast (Q178). **Next for B:** F-4
  Death as a character (he is not loaded, so his talk never starts),
  F-5 `flyToCamLoc` with a roll, F-6 the torches `azrieltp.cs` loads,
  then D-6 (the Reveal potion on the letters).

- **2026-10-30 (B): script coverage batch 2 (Dead City), first part
  built** (DESIGN "script coverage batch 2", D-1 to D-5 and D-7; logs and
  unit tests only, **not played by hand**): the death-hall stone puzzle
  runs from start to finish in a scripted test (jumps, lit stone, right
  and wrong stones, teleports, lines, the way out by the door);
  `KQMoveEvent` on objects; copies a script loads are solid; the small
  commands (react percents, Connor's flinch 0.5, required weapon mode,
  `invClick`, `lightOnFire`, `setCamRelConner`, `walkSlope`). Dead City
  now misses 10 commands (40 uses), was 19 (70).
  **Open for A:** Q176 (the stones' bob correction has the same sign as
  their leftover; a stand-in keeps them level), Q177 (north jumps can't
  climb the 500 between rows: Connor hits the stone's side and drops to
  the hall floor; the puzzle still resolves because the landing is judged
  by x and y). **Left in batch 2 for B:** D-6 `changeBmp` (Reveal
  letters), D-8 `matchHeights` / `terAnimSwitch`, D-9 `Lookup` and
  spirit traps, D-10 `ConnorCollision` and `Scales`, D-11 `ThrowRock`,
  `KQWeepBolt`, `RiverControl`, `KQDeath`, `KQAttackEvent` on objects,
  `KQMonster::add`; `KQTowerSkeleton` waits for Q175.

- **2026-10-28 (A): script coverage batch 2 (Dead City) studied, Q174
  answered.** New note `docs/behaviour/tile-puzzle.md`: the death-hall
  stones (jump one stone per key press toward Connor's facing, the
  target stone lit; D-E-A-T-H fixed in the program; a right letter
  teleports him next to the next one, a wrong stone back to the start,
  no damage; `tileExit`, `deadHallTeleport`) and the scales. The other
  batch items went into their notes (see MODLOG 2026-10-28): rock
  throwing, the weeper's bolt, spirit traps, river boxes, Death's lips,
  the tower skeleton, `KQMoveEvent` flags on objects, `changeBmp`,
  `matchHeights`, `terAnimSwitch`, `setCamRelConner`, `Lookup`, and more.
  Corrections: Connor's flinch value is 0.5, not 0.9 (`fighting.md`
  rule 29); the "unlit" switch is used (`object-lighting.md` rule 10).
  For the user to decide: spirit traps may drain on every entry once a
  spirit has passed (`traps.md`, possible original bug); Dead City's
  rats, snakes and spirits probably don't patrol in the original
  (`characters.md`, a data quirk). Next: B builds batch 2.

- **2026-10-27 (B): script coverage checklist** (the user's
  request): `kq8 script-coverage` writes `docs/SCRIPT-COVERAGE.md`,
  every command, Connor state, event, script call and class the game's
  files use, per world, and whether we have it. Study batches for A in
  `docs/DESIGN.md` "script coverage checklist" (batch 1: everywhere;
  batch 2: Dead City, with Q174).

- **2026-10-27 (B): Dead City's Paddle puzzle** (the user's report):
  the Paddle shows on its pedestal and can be clicked back; pushing it
  (Q173 built, P-2) plays Connor's push from his side, the tiki sinks,
  the Paddle comes back; the fourth starts the laser (user: the first
  pedestal works). Fix: the 2nd–4th Paddle showed nothing (a reused
  name found the deleted first one). The user's check (2026-10-27):
  working. Seen on
  the way, not done: the tiki lights' cue to `Terrain`,
  `KQCamera::setCamRelConner` (laser scene).

- **2026-10-27 (B): fix: the Shadow fight on the first arrival in
  Dead City** (the user's report: Connor only played `portalx`). The
  gate's script cues the object `ShadBat`, which our loader had dropped
  (no collision, so no box); scripted objects without a box are now kept
  for their script (`clicking.md` rule 21). The whole scene runs in the
  logs and plays only once; the user's look: it works. Then
  `makeTranslucent Connor` (the Shadow is part of his model there): see-
  through in movements 3, 5, 7 (logs only). The last shot (too close,
  user): Q172 built (P-1): a cut no longer ends a running flight, so it
  is pushed out like the scene's other cuts. The user's check
  (2026-10-27): the fight works perfectly.

- **2026-10-27 (B): fix: the caption stone looked checkered** (the
  user's report). `stontran.pbm` is a one-pixel checkerboard of
  see-through pixels; stretched by uneven amounts (the 800 × 600 layout
  × 1.25, then the window) it broke into stripes and blocks. The caption
  picture is now made at window pixels with each stone pixel a whole
  number of window pixels. The user's look (2026-10-27): looks great, and
  it adapts as the window is resized.

- **2026-10-26 (B): fix: Daventry's small waterfall was missing after a
  return** (the user's report): a kept world now also keeps the files
  scripts loaded while playing and loads them again on return
  (`world-change.md` rules 12, 14.5). **Then (K-1 to K-3):** animated and scripted objects that
  changed keep their movement, phase, place and collision type (the
  bricks stay down, the stream stays blocked; values as in the user's
  `save10.vol`). The user checked it in play (2026-10-27): working.

- **2026-10-25 (B): smooth joins (with `--smooth`, ours)** (DESIGN
  "smooth joins", J-1 to J-4): a change of movement is drawn mixed over
  0.15 s (`smooth join <s>`, 0 = off) where both files share the part.
  The user's look: good; the length stays tunable (`smooth join`), the
  user will fine-tune it later. Still jumps: Connor's
  walk ↔ run (run's body is numbered differently: a tolerant
  renumbering would be a further step), and zombies, spriggans, the
  wizard's other files, skeletons, orcs.

- **2026-10-25 (B): smooth animation (`--smooth`, ours, off by default)**
  (DESIGN "smooth animation", S-1 to S-6): model parts drawn part-way
  between frames, creatures, doors, chests, thrown items and bolts glided
  between updates, Connor's rope climb too. The user's look: all good
  (one fix: Connor's put-away movement). The rope climb's camera now
  glides between updates (always on; user-checked).
  Not done: shadows keep whole frames.

- **2026-10-25 (B): session A's Q167 and Q169 answers built** (plan in
  `docs/DESIGN.md` "building session A's Q167 and Q169 answers", steps
  C-1 to C-3, one commit each; **checked in the logs only, not played by
  hand or looked at by eye**):
  - C-1: model parts with their full placement; items laid on a part
    (`body` on the part, place and turn); `kq8 shape-parts <file>`.
  - C-2 (Q169): a dying character's weapon lies where its death movement
    leaves it (Daventry `sprgnbow`'s crossbow, Dead City `skeleton4`'s
    sword); its line test's blocking types are a stand-in (Q170).
  - C-3 (Q167): `Inv <item>` in Connor's Special list lays a real item;
    the roof kill's crossbow stays to be picked up.
  - Q168 needed nothing (the caption stone was already tiled).
- **2026-10-24 (B): session A's 2026-10-24 answers built** (plan in
  `docs/DESIGN.md` "building session A's 2026-10-24 answers", steps B-1
  to B-8, one commit each; **checked by the user 2026-10-24: all
  good**):
  - B-1 `getConInventory` (Q166): the bell rings with the small axe and
    the third hit brings in the Lady of the Lake.
  - B-2 `exist` (Q162): the roof choice's forward key plays the
    spriggan kill.
  - B-3 the loop speed (Q164) for Connor and characters: walk 1,124,
    run 2,558, back 1,070 units/s (measured 1,119–1,122 / 2,562 /
    1,066); character speeds as `character-movement.md`'s table.
  - B-4 `setState … Wait` (Q160): Connor stands up after `portalx` in
    the Shadow scene.
  - B-5 `setHaze` (Q161): the thick fog in the Shadow's home.
  - B-6 `dithered=` on every class (Q165): Daventry's arrow doors,
    Gnome's lock and characters.
  - B-7 panel pictures (Q108 b): stretched to their box, the bars' empty
    part `4black.pbm` (the caption stone tiled again: Q168 says tiled).
  - B-8 Q163 checked: one gap fixed (a character first seen in a real
    state now reports `0 -> new`).
- **2026-10-24 (B): session A's 2026-10-22 answers built** (steps A-1 to
  A-12; **checked by the user 2026-10-24**: the Shadow scene with fists
  out, the roof Choice, the ravens, the experience sound, the dither, the
  popup on Yes and the spinning mask work; the conversation camera looks
  the same as the original).
- **2026-10-23 (B):** the frame for a phase is cut down (Q153; the
  King's portrait stays pushed, user-checked); objects a script loads
  while playing get their click box (the key behind the portrait, user:
  works); the Shadow scene at the Dead City portal plays (user: works).
- **2026-10-22 (B, M6 6e): moving between worlds built** (the loading
  screen checked by the user, the rest in the logs; the popup's look
  "good enough"; Castle voices and the mirror scene user-checked).
  Console words `reload`, `goworld <World> [x y turn]`, `goworld portal
  <file>`. A world takes about 3.4 GB, so left worlds are not kept alive.
- **Earlier (2026-10-18 to 2026-10-21):** M2 and M3 done; M4's speech,
  interface, inventory, characters and conversations; M5's fight, ranged
  weapons, potions; M6's rope, hammer and magic map. Details:
  `docs/FEATURES.md` and `MODLOG.md`.
- **Later for A:** Dead City's flames (11,000 vs 12,100, open
  question 9); Connor against monsters.

## Goal

A Rust rewrite of *King's Quest: Mask of Eternity* that reads the player's
own install at runtime. Roadmap M0–M7 in `docs/DESIGN.md`. Current
milestone: **M4** (remarks and speech, the interface and the inventory,
characters and conversations done; the menus postponed) and **M5**
(characters moving on their own done; the fight next). M0–M3 are done.
M6 has started with the Rope and Hook (6a).

## Setup

- Game: GOG.com release, "Version 1.3" (`MASK/Version.txt`; `MASK/readme.txt`
  says 1.0.0.3), in `./MASK/` (read only, never committed). KQ8FixWin11
  installed there (Windows 11 fix, no-CD).
- Rust stable (MSVC), Bevy 0.20 for the viewer (`crates/kq8-viewer`).
- Two-session workflow (`docs/SESSIONS.md`): **A** studies `Mask.exe` in
  Ghidra and writes `docs/behaviour/*.md` (no code); **B** writes code from
  those notes and the data, never sees the exe.
- Ghidra 12.1.4 at `C:\Tools\Ghidra`, project `re/ghidra/KQ8`, program
  `re/Mask.exe`; GhidraMCP 2.0 via `.mcp.json` (session A only).
  `re/` (Ghidra project, scratch notes, throwaway scripts) is never
  committed.
- Measuring the running original: a read-only memory reader
  (`re/dbg/mem_fall.py`; start the game normally, run
  `python -I re\dbg\mem_fall.py <log>`, play, stop it **by its own
  process only**). It gave the fall speed, slopes, movement, turning,
  jumps, walls and floor heights. Running the game under the x32dbg
  debugger with stops did not work (the game crashed or froze when
  resumed from a stop); that route is not in use.
- Builds: close the viewer before building, so no side build folder
  (`target/side`) is needed. `cargo clean --profile dev` now and then
  keeps `target/` small.

## Code layout

- `crates/kq8-data` (114 unit tests): file formats, also fonts and
  `.pba` arrays (`font.rs`), screen layouts and the tag file (`gui.rs`). Sound files and
  their lookup (`aud.rs`). VOL archives, bitmaps,
  palettes, materials, `.kqs` shapes, terrain (surface and floor grids),
  `.dig` buildings, `.kq` text, world scripts (start-up, incl. item
  lines), `.sae` key maps, object placement rules, the `BOUNDS.KQ` box
  table, `.anm` transition lines, `envInfo.kq` ground flags, the ground's
  light and shadow maps (`lightmap.rs`), object lights (`lights.rs`),
  building light maps (`dil.rs`), emitter /
  particle catcher / ground animator settings, the item type list
  (`Invinfo.itl`), the haze rules (`haze.rs`: objects, ground, palette
  colour, `newSky`), object near / far / off rules (`objects.rs`), the
  lights a `.kqs` carries per frame (`shape.rs` `sequence_lights`), point
  lights on building surfaces (`wall_light.rs`).
- `crates/kq8-core` (406 unit tests; named `kq8-game` until 2026-10-13):
  game rules without drawing. Connor: `connor_mover.rs` (steps, gravity,
  landing, Fall, fall death, building and object hits, Jump, pushing),
  `connor_auto.rs` (scripted turns and walks: `walkTo`, `pointAndCue`),
  `connor.rs` (keys → movement, turning, banking, steps from the
  animation, start/stop transitions, crash, wall blocks, jumps, wading),
  `connor_update.rs` (one update of Connor), `collide.rs` (ball stack,
  building BSP sweep, object box sweep, type/mask rule, line tests),
  `water.rs` (the wading test). World: `camera.rs` (the following camera
  with the obstacle check), `camera_flight.rs` (cutscene camera flights
  and cuts), `script.rs` (the `.cs` script runner),
  `events.rs` (`sendEvent` queue), `door.rs`, `chest.rs`, `item.rs`
  (item objects and inventories), `trap.rs`, `click.rs` (picking and
  reach), `named_anim.rs` (an object's named animations, e.g. `open`),
  `active.rs` (doors, chests, items, scripted objects, buildings as click
  targets, accept lists, `EndLoop`/`getEndLoop`, `deleteObject`, the
  collider switch, events and scripts together), `emitter.rs` (particles and catchers), `ground_anim.rs`
  (flowing ground textures), `anim.rs` (looping animations of objects),
  `sound.rs` (sound rules: groups, distance, balance, mix limit, sound
  objects, ducking; no sound card), `talk.rs` (speech queues, line
  timing, mouth schedule; M4 4a; the conversation queues, 4e),
  `character.rs` (characters' states, M4 4d; walking, M5 5a), `character_ai.rs` (sight, the active list, wandering, fidgets, alarm and chase, M5 5a), `conversation.rs` (the
  conversation command and Connor's talk state, M4 4e).
  `kq8-data/src/msg.rs`: message,
  talker and lip-sync files. Viewer `talk_view.rs`: voices, captions,
  Connor's mouth. M4 4c: `caption.rs` (wrap, the box's place),
  `panels.rs` (shown/hidden, sliding, pop-ups), `meters.rs` (bar
  percents and widths), `gui_draw.rs` (a panel drawn from its layout),
  `inventory.rs` (slot pictures), `connor_inventory.rs` (adding,
  stacking, equipment, applying, slot clicks, the hand). M5 5g:
  `potions.rs` (timers, icon places, the four effects, pickup
  experience); M6 6a: `rope.rs` (rope places, the hook icons, the rope
  state's sequence, camera spots and ranges) and viewer `rope_view.rs`
  (runs it on Connor, the camera, the rope model); viewer `potion_view.rs` (the orbit, Connor's skin, faces
  and Invisible look, `dither`). Viewer
  `gui_view.rs`: the panels, the pointer, clicks and keys on the bars.
- `crates/kq8-audio` (5 unit tests): our mixer and the sound card (cpal).
  Runs at a fixed **15 updates per second**; the ground animators run on
  continuous game time.
- `crates/kq8-cli`: `kq8` checks (`vol-verify`, `bitmap-verify`,
  `bitmap-png`, `shape-verify`, `terrain-verify`, `building-verify`,
  `item-verify`, `lightmap-verify`, `lighting-verify`, `aud-verify
  [--wav <file>]`, `msg-verify`, `font-verify`, `gui-verify`,
  `character-verify`, `conv-verify`) and `kq8 text <file>` to print a game text file
  (`vol:<World>/<volume>/<NAME>` for a file inside an archive).
- `crates/kq8-viewer`: Bevy viewer. `[--world <W>]` shows a world (the
  default); `--model NAME` shows one model; `--renderer auto|vulkan|dx12` the
  graphics API (`renderer.rs`); `--light x1|x2|off` the
  light at start; `--haze on|off` the haze at start; `--dynamic on|off`
  overrides `Options.cs`'s `useDynamicLighting`; `--sound on|off` the
  sound card (`sound_view.rs`). The full option list with descriptions is in
  `docs/FEATURES.md` ("Launch options"). Test options: `--frames N`, `--lift N`, `--at X,Y,H`,
  `--hold F,T,R[,N]`, `--jump N` (repeatable), `--turn N[,1|-1]`, `--click NAME,N`
  (repeatable; skips the line-of-sight test), `--aim NAME,N` (a click
  with the line-of-sight test), `--item TYPE` (a real held `TestItem`),
  `--key NAME,N` (`space`, `1`–`8`, `left:`/`right:` + a layout control
  name); `--corners on|off` (the corner panels, F6). Haze and drawing: the
  ground's material `ground_haze.rs`, walls `building_light.rs`, models
  `object_light.rs` (each with its `.wesl` shader), near / far / off and
  the collider switch `draw_distance.rs`, point lights on models
  `scene_lights.rs`, on building surfaces `wall_lights.rs`.
- Logs go to `kq8-artifacts/logs/`: `viewer.log`, `connor.log` and
  `camera.log` (one line per update), `doors.log` (clicks, scripts,
  events), `talk.log` (speech), `conversation.log` (conversations),
  `characters.log`, `gui.log` (panels, stats sent, the bars'
  contents, the hand), `traps.log`, `items.log`, `effects.log`, `ground_anim.log`,
  `anim.log`, `sound.log`, `lightmap-verify.log`, `lighting-verify.log`,
  `aud-verify.log`. `viewer.log`
  also logs the ground light under Connor and the camera, and every
  second how many objects and wall surfaces the haze leaves out and how
  many objects are near / far / off (plus every change of state), every
  collider switch, and the scene's lights (how many, what the nearest
  gives Connor), and per point light what it does to building surfaces
  (tested, skipped, not in front, out of reach, blocked, lit).

## What works (tested)

- M1 (checked by eye in Daventry): ground with textures; 30 buildings; the
  objects the world scripts load; see-through objects. All 12 worlds load.
  Lighting of the ground, models and buildings, and the haze/fog (see
  `docs/FEATURES.md`). F7 (light) and F8 (haze) switch them; start options
  tested, the key presses not tested by a test option.
- Viewer helpers: H shows invisible, unused and hidden objects; U shows
  the floor grid under the water (key presses not tested by a test
  option); camera position shown bottom left; F switches to a free flying
  camera.
- M2 (all checked by eye unless marked): Connor at his start point; walk,
  run, back, turn (arrows, End = run on/off); slopes and falls with the
  measured gravity; buildings and objects are solid (stop, slide, steps,
  floors, crash when running head-on, the wall blocks); start and stop
  animations; standing, running and backward jumps; a held turn key keeps
  turning him at a wall (key repeat; checked in the logs only); water:
  Connor and objects stand on the floor grid under the water, he wades
  (`waterwalk`/`waterrun`) where the water is deep enough, and can do a
  standing jump while wading (that jump: checked in the log only).
  Banking: he leans into turns while running (up to 0.18 rad, as traced;
  checked by eye at 0.15, the 0.18 end value in the logs only), also while
  wading, and straightens up in a jump (Q59, the user's check).
- Camera (checked by eye): follows Connor on a leash (1,500–5,000),
  keeps at least 1,400 above the ground, right mouse swings and zooms
  (Shift: tilt), Page Up/Down tilt, Home resets, F11 fixed distance.
  It keeps out of walls: buildings, closed doors and hills block its
  view, trees and plain objects don't; when blocked it swings round to
  look along the way Connor came and comes closer if it has to; dragging
  it into a wall only creeps it closer. The view lets Connor drift about
  14° off the middle before it catches up; the right button held keeps
  him at his spot on screen; it points at about his chest (M3 3o,
  checked by the user against the original 2026-10-19).
- M3 (checked by eye unless marked):
  - Doors: left click with the mouse opens them within reach (4,700);
    from too far Connor's "too far" remark is only logged; the open door
    is out of the doorway but still solid; the castle keep's doors open
    only with the key (test option `--item`).
  - Clicks: walls, floors and ceilings in front take the click (and
    swallow it), the ground doesn't (A's Q63 answer, 2026-10-16; checked
    in the logs only, the older stand-in was checked by eye).
  - Emitters: Daventry's falling leaves and stream drips; leaves vanish
    inside the cottage (particle catcher); IceWorld's snow follows Connor
    and stays out of buildings.
  - Flowing ground textures (river, stream, Swamp, DeadCity blood) with
    each square keeping its own orientation, timed 0.3 s from each change.
  - Animated objects play their starting movement (fountain, windmill,
    mill wheel, signs, torches, fish). Gnome's three rolling rocks roll
    out and come back each cycle, and their collision boxes move with
    them. Gnome's pushable `boulder` stands still, as in the original.
  - Traps: Daventry's 52 trap boxes switch on and off and run their
    scripts; walk and run zones change Connor's pace and the camera's
    leash (1,000–2,500 inside). Teleporters and cutscenes are only
    logged (ropes: M6 6a).
  - Items: the 22 items Daventry puts out at the start are drawn and can
    be picked up (counted in `items.log`); item scripts run (farm coins).
  - Chests: the pub's big chest and the keep's chest of coins (open,
    take, "empty"); the wizard's hidden chest stays hidden.
  - Scripted objects: the small chests, the jar and the mushroom baskets;
    clicking a swimming fish makes it disappear (sound and health only
    logged); other scripted things (signs, stone villagers) run their
    scripts, their remarks logged.
- Checked by eye by the user (2026-10-16): point lights on building
  surfaces, `KQPointLight` always shining, reach to a building in its
  own axes. F9 (dynamic lighting) checked by the user 2026-10-17.
- Sound effects (M3 3h, 2026-10-17): see `docs/FEATURES.md`. Checked by
  ear by the user. World music (3h-5) too.
- Remarks and speech (M4 4a) and the click turn and turn keys (4a-7
  to 4a-9): checked by the user against the original, 2026-10-18.
- The interface and inventory (M4 4c): checked by the user against the
  original, 2026-10-18 (the corner panels option: looked at by the user,
  no complaint). The back flip at a wall: checked by the user.
- Conversations (M4 4e): built from the logs, then checked by the user
  (walking away and back, the cuts, the Uglybeast; `docs/FEATURES.md`).
- Health items and potions (M5 5g, 2026-10-21): checked in the logs only
  (`MODLOG.md`); not looked at by eye yet.
- The Rope and Hook (M6 6a, 2026-10-21): **checked by the user by eye
  at two rope places: "working perfectly"**.
- Tests: 618 unit tests pass (131 + 482 + 5), 2026-10-21.

## What doesn't work yet / next

- **Kept worlds after K-1 to K-3 (2026-10-26):** still fresh on a
  return: doors and chests opened without a variable (and what was taken
  from chests), characters' place and health, hidden switches, items on
  the ground, copies a `loadKQ` made with a name or place.

- **After C-1 to C-3 (2026-10-25), not checked by the user yet:** the
  roof kill's crossbow lying where the movement drops it (and picking it
  up); a dead bow spriggan's crossbow (e.g. console `kill sprgnbow` with
  Connor at the roof) and a Dead City skeleton's sword lying where the
  death leaves them, while coins fly. Not tested in the viewer: a burnt
  bow spriggan throws its crossbow (its fire death has no `SmCrossbow`
  part); the line test blocked by a wall (unit test only).
  Rule 6's speed and wrap now also for doors, chests and every animated
  object (2026-10-25, logs only).

- **Left over from B-1 to B-8 (the user's look: all good, 2026-10-24),
  not built:** our camera's projection far plane does not follow
  `setHaze` (the ground and objects stop at `visibleDistance` anyway);
  `setState Connor Wait` from his jump, push, rope, talk or throw states
  (not described, nothing done); the status icons and popup buttons keep
  their picture's own see-through flag; `KQObject::changeBmp` is a stub.

- **Allocator: `mimalloc`, purge delay 0 (chosen by the user
  2026-10-22).** Tour of all 10 worlds, then Daventry ↔ Castle twice
  (`--world Daventry --frames 30000 --sound off` with one
  `--console "goworld <World>",30` per world; ~4.5 minutes; summary in
  `kq8-artifacts/logs/bench-allocators.txt`). Daventry's read, fresh →
  3rd return: Windows 3.5 → 14.6 s; `mimalloc` 3.1 → 3.5 s; with purge
  delay 0, 3.2 → 3.7 s. Memory at the end (working set / private):
  3.3 / 5.4 GB, 3.5 / 5.4 GB, 3.0 / 5.1 GB. Purge delay 0 against
  1000 ms, 3 tours each (`bench-purge.txt`): same read times, ~350 MB
  less working set on average, ~550 MB less at the highest. To turn it
  off for one run: `$env:MIMALLOC_PURGE_DELAY = "1000"`. `memory.rs`
  is Windows-only (Linux needs `/proc/self/status`). The bigger saving
  is ours. Measured (`world.log` "memory by kind"): identical copies are
  already shared; Daventry's animation frames are 56,468 meshes, 955 MB
  of 1,063 MB of meshes (textures 233 MB), 95,441 entities. Options:
  animate by moving parts: **measured, not worth it** (`viewer.log`
  "parts count": 85-91 % of the triangles stay, over half the parts
  change form every frame). **Plan chosen (DESIGN "Animation memory"):
  step 1 (no normal-memory copy of animation meshes; **done 2026-10-22**,
  Daventry in play 2.64 → 1.51 GB, checked by the user). **2026-10-23:
  step 3 skipped, step 4 (frames on the graphics card) now, in sub-steps
  4a-4g (DESIGN). 4a done (the compact data checked in all worlds:
  Daventry 71 MB instead of 955 MB). 4b built (still animated objects
  drawn the new way; F5 compares) and 4c (doors, chests, scripted
  objects) and 4d (characters) checked by the user. 4e built (copy
  templates and script copies; items and effects are still models, left
  as they are) and 4f (Connor) checked by the user; **4g done: the old
  drawing removed. Daventry in play 787 MB (was 2.64 GB), 3,565 entities
  (was 133,200).** Drawing work is open again.**
- **Loading time (2026-10-23, DESIGN "loading time"):** Daventry reads in
  1.3 s (was 3.5 s), everything drawn about 2.8 s after the start;
  a local `play.cmd` (not in the repository) skips cargo's 0.7 s. The timeline is in `viewer.log`. Smooth
  animation after M7. Other drawing work waits until step 4 is done.**
- **Moving between worlds, what is missing (6e):** worlds kept in full
  (positions, chests without a variable, items dropped, bodies, terrain
  changes, the frozen world clock): with saved games. Reading slows down
  with each change: fixed with `mimalloc` (see "Allocator" above). Building after the read
  still stops the window 0.5–0.9 s. The full-armour arrival list (rule
  5.2); Escape on the popup doesn't open the main menu (not built); a
  Clarity potion running on arrival doesn't reveal the new world's
  objects. Not yet looked at by eye: the popup, the arrivals. Help
  popups (`doPopup`, the "Go" help 151) are logged only.
- **Suggested next:** the user plays a few world changes by eye
  (Daventry ↔ Castle through the cave, the keep's teleporter with
  `--kept-maps`); then saved games (the same kept-world record).
- **Stand-ins of 6d:** the follow margin (20 px inside every edge);
  layout pixels at 800 × 600 for the view and the mouse; Connor's place
  given to the map once per update; a world never left shown from its
  shipped map file; the mini-map's view taken from Connor's when the
  map is set up.

- **Stand-ins of 6b (Q148):** where the hammer turns back without a touch;
  the model drawn pointing along its flight; requests during the throw wait for the catch; the
  throw leaves the panels as they are. Not run: a throw from a real walk,
  Dead City's lever.

- **Stand-ins of 3m** (not traced; none reached in Daventry): a
  state request refused during an unfinished jump is tried again every
  update; objects later in the collision list after a push are not
  tested; `follow Connor` during a flight ends it.
- **Stand-ins of 3n and 3o:** one camera update on the first tick,
  looking straight at Connor (Q27 d); the view heading's state around
  cutscene cameras (Q113 a–c; the user's check matched ours after the
  watermill scene); the mouse's recentring near the window's edges is
  not built (not settled; the note says it may be left out).
- **Stand-ins of 4d (characters):** a character's facing taken as
  Connor's (−Y). Q116–Q124 are all built (Q123 on buildings with Q135,
  4d-7b-5). Q127 (doors and chests keep the old load names: fine as they are) and
  Q130 (Connor's shadow test) answered by A, not built yet; Q128 and Q129
  built in M5 5a. Our
  readings: the shadow's terrain search uses the model's points, not
  its box; multiply by 0.5 for "strength 0.5"; a transition's target
  first moves in the next update. Not built: `changeAnmFile`, the
  `endLoop` cue (the fight). Not tested: the
  characters scripts load while playing (the Lady, three ravens, the
  Unicorn), a real turn in the viewer. `sprdump`'s scene (the outhouse
  door) runs and the spriggan is drawn (fixed 2026-10-25; logs only).
- **temple4's `LucEnd` / `LucEndFar`** (class `KQLucreto`, the final
  Lucreto) are not loaded as characters: no model, never drawn (found
  2026-10-25).
- **Stand-ins of 4a:** the caption panel's look. Two details of `eval` (`anim-scripts.md` rule 18) are not
  built, as no data uses them: `eval` alone gives 1 (the note: 0), and
  its words are not read again as a new line.
- **Stand-ins of 4c** (Q108–Q111): the 800 × 600 screen and how the
  layout's rectangles scale with it (× 1.25; pictures stretched to them
  since B-7); the panels' animations at 15 frames per second; counts of 0
  not written; the rocks picture; the blink's order; a `give`'s
  pushed-out piece at Connor's feet; the
  pointer's click point at its top-left. Not built: the map (M6), the
  help popup and F1, the level-up flame (not traced). Escape still quits: the original's main menu pauses
  the game (a menu step must stop the updates and the panels' timers).
  When standing, an automatic turn plays `Stop` for its first update
  even if he was already turning on the spot (as rules 24–25 say).
  Q111 (Space's sounds, the mask's place and size, black behind the
  bars) built from the user's checks; the exact rules are not traced.
- **Stand-ins and gaps of 5g (potions):** the spinning mask takes the
  first free status place like a potion, and the potion icon's used
  part is its `.pba`'s frame 0 (Q143); `KQObject::dither` draws half
  see-through, `dithered=1` ignored (Q144); the orbit's angle 0 along +x
  and the shape turning with it (ours). Not built: the hurt line while
  invisible (rule 27), the same lowering of Connor's reported place
  while he talks (Q146 a) (cutscenes done in 5g-9: every handsOff, Q146
  b; the rope in M6 6a-6), first-person arms,
  armour skins (the temple armour's `con65`), the help pop-ups (logged).
  The helmet's face set `chf` exists in no world, so his face stays.
  Potions are not kept across saves or worlds (we have neither yet).
- **Stand-in of the back flip at a wall (Q112):** the game may test the
  back key or the movement asked for rather than the back wall block;
  it only differs in cases not seen yet.
- **Stand-ins of 4e (conversations):** the keys are ignored during the
  walk to the spot, no Jump or push while talking (Q125); the camera
  aims 1,400 above the partner and swings around it like the following
  camera (Q126); the talk state waits while a jump is unfinished
  (`conversation.md` open question 3); Connor's state changes one step
  after core decides (same update). Not built: weapons away (`away` /
  `out`, no weapons yet), first person, the right button's lock release
  on entering. Not run: the horn on the Uglybeast, `clearSpeak` on a
  partner, a real mouse drag during a talk.
- **Stand-ins of 5c (the fight):** his swing and flinch play as one-item
  Special lists (so a weapon request during a swing waits for its end:
  `ranged-weapons.md` open question 1); a swing click on nothing turns
  him toward where the line meets the level of his feet. Game over
  (Connor stays dead). The corpse has no draw-distance test and no skin
  swap (bossprig). The flinch pause is fixed at 2 s (Q136,
  `reactPause`). Not built: fleeing / keeping distance (rule 37; none
  in Daventry), the tips' texts, surprise lines (none in Daventry). Not
  run in the viewer: the real mouse swing, fire / ice / projectile
  deaths, picking up an item in the air, the other two bow spriggans.
  Seen in the logs: rule 26's alarm with "run" also reaches far
  characters that are active and see Connor (as written; not checked
  in the original).
- **Stand-ins and open points of 5f (ranged weapons):** the bolt's own
  shape is its box's ball stack (one ball, radius 235: Q140 a); a
  character aims at Connor + 975 (0.75 × his box top 1,300; the note
  says 1,200: Q140 b); the `arrow` part is taken from the frame at the
  `do` cue's own phase (Q140 c); a script's `walkTo` right after
  `setDoMode` cuts the swap (the axe cutscene, Q141); `setDoMode` doesn't
  end a jump; the right button held (open question 2) and first person
  not done; the run comes back the simpler way (1 s after the last
  moving shot); how objects take damage is still the barrel stand-in
  (Q139 e); fire / ice bolts' sparks (`emitter`) not drawn.
- **Stand-ins of 5a / 5b (characters moving):** the idle list kept in the
  same player as `idle:<name>` (the same result as the original's swap,
  rule 18); the stuck line 1 above the feet (the note's recommendation).
  Not run in the game yet: losing sight, leaving / re-entering the
  graveyard trap, `wanderAndStop`, random wander points, landing on a
  goblin's top (unit test only), ranged characters (`doMode=1`).
- **Left in 6a (the rope):** the top trap's `On` comes at his first
  move (in `taway`), not during the climb as in the original (cause not
  found: the note's open question 7); the grab tie's 0.5 tolerance is
  ours. Not built: first person (only logged), saved games (none
  yet), rock throws (none yet: "ignored on the rope" by construction).
  Not run: a ranged weapon held at the start, the cave rope
  (`ropetrp3/4`), other worlds' ropes, a chaser losing him on the rope
  (only the report line and far bats seen), `setRopeCam` spots and the
  camera limits in the viewer (unit tests only).
- **Next step, suggested:** for the user: the rope by eye in Daventry
  (`ropeandhook.md` "How to check"). Then: check ranged weapons by eye
  (`ranged-weapons.md` "How to check" 1–9) and the fight
  (`fighting.md` "How to check" 1–8) in ours and the original. For A:
  Q140, Q136, Q143, Q144, Q146, game over. For the user: potions by eye
  in Daventry (`potions.md` "How to check" 2–7). **The main
  menu is postponed (the user, 2026-10-20):** starting straight in
  Daventry keeps testing quick; when it comes, keep a launch option
  that skips it.
- Sound, not done: pause, and `setActiveCamera` / `makeActive` for
  sound objects.
- Speech, not done: the Daventry monster-death remark, helmets' and
  armour's faces (M5).
- Clicking: only doors, chests, items, scripted objects, buildings and
  characters with a script react. `checkInventory` by id number (Q62 b)
  not built.
- Camera, still missing: first person (F2), shake; the roll of `move
  KQCamera` and of flights is not drawn (logged).
- Cutscenes: a request from a cue reaches Connor one update after the
  cue (our order).
- Animated objects: scenery animations' cues (sounds) don't fire.
  `loadKQ` with a name or position while playing is not done; a copy
  that comes in is not solid or clickable.
- Collision, not done: Connor against monsters (none drawn);
  objects with no box in the game's table stay
  walk-through. Hidden, far and off objects are not solid (rule 41; not
  checked in the original). A deleted or far building's walls stay
  solid.
- Loops (`walk`, `run`) still spread their step evenly over the loop
  (Q24); one-shot animations use their frame steps.
- Lighting still lacks: the brightness setting (rule 17). Point lights
  on building surfaces light every surface of a drawn building, not only
  those in view (same picture, more work).
- Not drawn yet: the
  teleport pad's look (Q14b), arrow sparks and the dragon fire (they come
  with fighting).
- Nothing yet of: menus, fighting, saving, travel between worlds.
- **Not M3 (M3 is Daventry only, the user 2026-10-19):** things of later
  worlds wait for M6 (listed there in `docs/DESIGN.md`): Gnome's rolling
  rocks hurting Connor, Gnome's pushable boulder, damaging water.

## Open questions (`docs/behaviour/QUESTIONS.md`)

- **Built 2026-10-24 (B-1 to B-8):** Q160, Q161, Q162, Q163 (one gap
  fixed), Q164, Q165, Q166, Q108 b. Still open for A: Q108 a, c, d;
  whether a saved game keeps `setHaze`'s values (we have no saving yet).
- **Built 2026-10-25 (C-1 to C-3):** Q169, Q167; Q168 needed nothing.
  The user's look at B-1 to B-8 and the two fixes: all good (2026-10-24).
- **New for A (2026-10-25): Q170**: which collision types block the
  dropped weapon's line test (`fighting.md` rule 31a.2: "any solid
  object"); B uses every type as a stand-in.
- **New for A (2026-10-25): Q171**: the level-up flame and the slot
  flashes are see-through in the original although rule 20a says
  attribute 0x8 is solid; B uses their files' own transparent flag.
- **Built 2026-10-27 (P-1): Q172** (a cut during a running flight).
- **Built 2026-10-27 (P-2): Q173** (the Paddle's push).

- Inside `magic-map.md` (not in QUESTIONS.md): 2 and 3 answered by the
  user's check (2026-10-21); 4 and 5 answered in `world-change.md`
  (2026-10-21); 6 (800 × 600) built with a stand-in.
- Inside `world-change.md`: three small points (not blocking):
  left-world variables in memory, the rope / movement list after
  arrival, `worldState.kq`'s events.
- **Built 2026-10-24 (A-1 to A-12):** Q112, Q125, Q126, Q138 (e and the
  swap during a swing), Q140 c, Q143 c, Q144, Q145, Q146, Q148, Q149,
  Q150, Q151, Q152, Q153 (Connor's frames; the speed and wrap: Q164),
  Q154, Q155, Q156, Q157, Q158 a-b, Q159, Q42; Q14 b and Q136 needed
  nothing. Small parts left: the level flame's speed and length, whether
  `LEVEL.AUD` / `STATS.AUD` play at a world start (not traced); Q81 c's
  forward-wrap transition detail (cues skipped when a waiting
  transition's phase lies in the wrap) not built.

- Open points inside `object-lighting.md` (not in QUESTIONS.md): only
  the flames' 12,100 (traced and built) vs the user's ~11,000 (open
  question 9, kept for a later session) and the Direct3D renderer (6,
  not studied; Glide only). The rest was answered 2026-10-17.

- `talking.md` open questions 3 and 5 (folders, a line with no voice)
  are still open; 1 and 2 (conversations, shapes above 7) answered in
  `conversation.md`, 4 (the caption panel) in `interface.md`.
- `conversation.md` open question 3 (states that delay the talk
  state): small, not blocking (1 and 2 answered); B's stand-in: an
  unfinished jump.
- **Q147** answered by A and built by B (2026-10-21). Left open, small:
  `ropeandhook.md` open question 7 (what sets off the top trap
  mid-climb).
- `characters.md` open questions 1, 2 and 4 (`flags` bit 0x20, how the
  renderer blends shadows, `changeBmp` / `changeAnmFile`): not blocking;
  question 3 (the AI) answered 2026-10-19 in `character-movement.md`,
  whose own open questions are now: 1 water for walking characters
  (small), 3 the stuck line's exact ground test at foot height (not
  worth copying); 2 (the fight) answered 2026-10-20 in `fighting.md`.
- `fighting.md` open questions (not blocking the Daventry fight):
  Connor's death (the input side and the
  bolt's flight answered 2026-10-20 in `ranged-weapons.md`, built in M5
  5f).
- Q131–Q134: answered by A and built 2026-10-20 (M5 5b).
- Q106 (partly) and Q107 (mostly) answered 2026-10-18 in
  `interface.md` and built (4c-1); still open: index 0 in fonts, font
  flag bit 6, the picture attribute bits.
- For A (asked 2026-10-18 by B, M4 4c): **Q108** (the 800 × 600 screen:
  how rectangles and fonts scale; the panels' animation speed; (b) how
  pictures fill their box answered and built 2026-10-24), **Q109** (counts of 0 for
  gold and rocks, the rocks picture, the blink's order, a `give`'s
  pushed-out piece, the top bar in a window; (d) the potions' clock and
  (a) the slots' count 0 answered in `potions.md` and built in M5 5g), **Q110** (the scroll's and rock's remark
  numbers in `inventory.md` rule 9; the pointer's click point; the gold
  button while the coins are held; the rope's numbers and "held or not"
  answered 2026-10-21 in `ropeandhook.md`), **Q111** (answered by the user's checks
  and built; the exact rules are worth tracing).
- `interface.md` open questions 1–6 and `inventory.md` open questions
  1–4 (level-up flame, rocks picture, held item on world
  change, applied quest items).
- For A (asked 2026-10-19 by B, M3 3o; small, not blocking): **Q113**
  a–c (the view heading's state around cutscene cameras: rule 1 from
  idle only, the exact aim before unlocking, the mill wheel's turn past
  Connor); d and e were settled by the user's checks.
- For B (answered 2026-10-17, not built): Q81 (c) (stand-in already
  right; only the forward-wrap transition detail is new).
- `sound.md` open questions 5–6 (music after a video; whether a world
  change cuts the old tune): not traced, checks by ear suggested.
- Q41: jumps: (a) jump during the wait before a stop, (c) the 250 limit,
  (d) a running jump's landing with only a turn key held. Stand-ins.
- Q27 a, a detail: the step from the mouse's window fraction to the
  camera (signs, centring) was not found; the user's checked directions
  stand.
- Q24: partly answered (steps spread within each frame; not compared
  frame by frame).
- Q29 (e): why Connor climbs so little into a steep hillside (the rest
  answered).
- Q30 (b): what keeps Connor on a slope going downhill; "likely answered"
  by the stronger gravity, but the re-check A asked for was not recorded.
- All others are answered or closed (Q27, Q53, Q100, Q102 built
  2026-10-19 (3o), Q128 and Q129 built 2026-10-20 (M5 5a-3, 5a-9), Q101 and Q48 closed; Q105 answered and built
  2026-10-18 (4c-5); Q103–Q104 answered, built and
  checked by the user 2026-10-18; Q94–Q96 answered and built
  2026-10-18; Q84–Q93 answered 2026-10-17 and built 2026-10-18; Q80 answered 2026-10-17; Q79 answered by A 2026-10-17 and checked
  by B in 7k; Q75–Q78 answered by A and built
  2026-10-16; Q54, Q60–Q64, Q70–Q74 built 2026-10-16; Q73 checked;
  Q68–Q69 answered and built 2026-10-16; Q65–Q67 answered and built, the
  lighting session 2026-10-14/15; Q55 answered in
  `connor-collision.md`, not built yet; Q56 answered by A's trace and
  built 2026-10-13; Q59 closed by the user's check).

## Files that matter

- `docs/behaviour/*.md`: the rules (session A).
- `docs/behaviour/QUESTIONS.md`: B's questions and A's answers.
- `docs/provenance/README.md`: which note each piece of code follows, and
  how it was checked.
- `docs/DESIGN.md`: architecture, roadmap, step plans.
- `docs/FEATURES.md`: what works, in detail, and how each was checked.
- `MODLOG.md`: every change with its test.

## Prompt for a fresh session

See  [docs/SESSIONS.md](docs/SESSIONS.md)
