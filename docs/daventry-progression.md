# Daventry campaign route

Daventry has two regional exits. Castle Daventry contains a required ingredient and leads back to Daventry. The gate in the churchyard tomb advances the story to the Dimension of Death, called `DeadCity` in the data. These prerequisites come from the original GOG game scripts. They describe the route to verify, independently of whether the Rust game has passed each part.

| Stage | What the player does | What establishes progress |
| --- | --- | --- |
| Start the story | Select New Game, choose Normal and let the cottage sequence finish. Walk to the wizard and speak to him. | In `wizard.cs`, the eighth line of the first conversation exchanges `Piece1A` for `Mask1` and `Piece1`. The thirteenth line starts the map sequence and marks the first meeting. Its later cue gives the Magic Map. |
| Learn what blocks the route | Reach the Shadow encounter in the churchyard tomb and return to the wizard. | `sdwtrap2.cs` and `shadow.cs` record the meeting. With the first wizard meeting and the Shadow encounter recorded, `wizard.cs` gives the ingredient conversation. Its fifth line changes the wizard's state to 2, where he will accept the ingredients. |
| Obtain the Candle | Reach and interact with the church candle. | `newworld.cs` places `candle.kq`. `candle.cs` grants one Candle, clears the full flag and removes the prop. |
| Obtain the dead hero’s ring | Enter the hero’s tomb, open the coffin and take the ring. | `tombdor2.cs` rejects fists and the dagger when breaking the tomb door. `coffnlid.cs` changes the coffin interaction when its fall ends. `worldinv.cs` places `RingOfDeadHero` inside. `ring.cs` supplies its pickup response. |
| Obtain the Ashes | Use the cave exit to Castle Daventry, take the Ashes and return. | Daventry’s `portaltp.cs` starts `portal1`. CastleD’s `worldinv.cs` places the Ashes. Its `portalt2.cs` and `portal2.kq` bring Connor back. |
| Make the Ring of Light | Present each ingredient to the wizard while he is in state 2 and finish the casting sequence. | `wizard.cs` consumes each offered item and remembers it. Its three flags are tested together. The final `castring4` callback grants the Ring of Light and changes the wizard’s state to 3. |
| Leave Daventry | Enter the churchyard tomb’s gate with the Ring of Light and accept the travel prompt. | On a first visit, `sdwtrap1.cs` checks Connor’s inventory and then records `Daventry::BeenThere::DoneThat`. `davgate4.kq` names Dead City and its arrival coordinates. Later visits use the recorded flag. |

### Verified through the dead hero's ring

The source chain starts at Normal New Game. Each later process loads
the preceding process's unchanged menu save. Connor acquired the
Crystal, shield and eight mushrooms before the Wizard meeting. The
windmill leg earned the axe, and the Henchman leg left a WestYard
save with 40 health, four mushrooms and the Crystal. Both legs used
enemy AI and ordinary damage. The Crystal windmill and Henchman
regressions passed in 184.20 and 131.66 seconds respectively.

At the tomb, the Crystal can restore health lost on the approach.
In one recorded fight it healed Connor from 8 to 40. Connor killed
the archer, struck the door with the axe, and the original script
opened it. After the Knight's dialogue returned control, Connor
walked inside and saved HeroDoor with 50 health. The archer was
gone, the door was unlocked and the coffin was still closed.

A fresh process loaded HeroDoor and approached the lid. Its original
Push event started the fall; the end of the animation marked the
coffin open and disabled its click blocker. On update 74 the walking
check observed that flag. Connor stepped to the southwest corner
and clicked the ring through gameplay pixel (822, 378) at update 93.
It was the first item hit, 1,396.4 units from Connor, within the
4,700-unit reach. The menu saved HeroRing. Another process loaded it
and saved again with exactly one ring, level 5, 50 health, the archer
gone and both tomb flags intact.

The checked-in two-process coffin and ring-reload regression also
passed from that earned HeroDoor checkpoint in 96.54 seconds.

A separate 905.43-second run, starting at Normal New Game with all
checkpoint inputs unset, earned a different HeroDoor save. Its first
straight approach hit the click blocker head-on and failed to open
the coffin. The route now approaches from the southwest. A glancing
contact can slide along the blocker while Connor remains within the
original scripted push's 60-degree facing limit. The physical push,
visible pickup and fresh reload passed from both HeroDoor saves in
111.38 and 111.68 seconds.

Two earlier routes are recorded under the road-to-the-tomb section.
One opened the coffin but kept trying to stand inside it. A second
hit the overlapping click blocker before touching the lid. The
original collision boxes and the recorded movement steps reproduced
the difference: the successful 173.2-unit forward movement reached
both objects, whereas the 146.3-unit movement on the other approach
reached only the blocker. The current route saves inside the open
door, loads that checkpoint and recognizes completion when the
original coffin script records it. It clicks from reachable ground.

Set `KQ8_GAME_DIR` to the GOG installation. The command can earn its
own prerequisites. To repeat a shorter leg, set `KQ8_CRYSTAL_YARD_SLOT`
to an earned WestYard save with the Crystal, or `KQ8_HERO_DOOR_SLOT`
to the live HeroDoor save with its coffin still closed.

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield earned_crystal_to_hero_ring_and_fresh_reload -- --ignored --exact --test-threads=1
```

### Other Daventry branches

These scripts are relevant to a complete recreation. Some can provide access or equipment for the main route. The ingredient test above names only the Candle, Ashes and dead hero’s ring.

| Branch | What to verify in play |
| --- | --- |
| Church and tomb access | Church roof and entry triggers, pushing the urn and its delayed opening of the tomb door, the archer encounter and crossbow reward. Sources include `churchr3.cs`, `urn1.cs`, `rooftrap.cs` and `sprgnbow.cs`. |
| Weapons and navigation | The dagger has a loose placement in `worldinv.cs`. The axe pickup gives the Small Axe and starts the henchman sequence. Hanging and ringing the bell three times brings out the lake lady, who can grant the Lake Sword. The mill tree checks the weapon and changes the crossing. The rope and hook has its own pickup and traversal interactions. Sources include `dagger.cs`, `ax.cs`, `henchman.cs`, `belstnd.cs`, `bell.cs`, `lady.cs`, `milltree.cs` and `ropenhok.cs`. |
| Wizard's book and the star globe | The book's animation gives a Spell Page. Offering it to the star globe leads to a Clarity potion. The page and the potion are separate inventory and object-state checks. Sources include `wizbook.cs` and `starball.cs`. |
| The beast, unicorn and pyramids | The beast handles the Unicorn Horn. The unicorn sequence can give the Dark Pyramid. The Crystal Pyramid has its own pickup. These branches need separate coverage when following the later regional story back through Daventry. Sources include `ugly.cs`, `unicorn.cs` and `pyramid.cs`. |
| Castle keep and travel | Using the Keep Key on the locked keep door earns teleport progress. The castle cave’s first discovery grants experience separately. Sources include `keepdoor.cs` and `portaltp.cs`. |
| Village, supplies and discoveries | Cottage ravens, baskets, fish, the inn, the girl, bows and barrels, the alchemist's obstruction, the waterfall, health and equipment need their own interaction coverage. Original files include `condoor.cs`, `ravencnr.cs`, `basket2.cs`, `girl.cs`, `alchdoor.cs`, `boardup.cs`, `bricks.cs` and `worldinv.cs`. |

Physical access can add prerequisites that an inventory check alone cannot show. A full Daventry-to-Dead City run must start with an empty save directory and use ordinary player controls across all three ingredient pickups, wizard exchanges and both travel prompts.

### Current continuation boundary after the Small Axe

The prepared route now defeats the Henchman with ordinary weapon and
consumable inputs and leaves the windmill after a post-victory reload.
Connor carried the shield and five cottage mushrooms to the Wizard,
collected five more mushrooms on the way to the mill, defeated the
archer and earned the Small Axe. One verified windmill slot had
30 health, four mushrooms, the unconsumed shield, the axe and map.
Enemy AI and normal damage were enabled throughout.

Connor used the shield after the original scene returned control and
attacked when the stationary guard was within the axe's 2,350-unit
reach. In the verified repeat the guard entered Die at 817. His
animation ended at 907, awarding 30 experience and raising Connor to
level 4. The original script returned control and removed clicker2,
and the game left the guard's corpse. A fresh process loaded the menu save, crossed
the west opening at 45 and saved with 40 health. Both removals stayed
recorded. The two-process check took 131.52 seconds from an earned
windmill slot. The windmill acquisition and fresh-process axe reload
took 178.21 seconds in a separate prepared-map test.

Set `KQ8_GAME_DIR` to the owned GOG installation and run the duel test.

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield earned_supplies_defeat_the_henchman_and_reopen_the_west_yard -- --ignored --exact --test-threads=1
```

The default path earns the prerequisites through Normal New Game.
`KQ8_PREPARED_AXE_SLOT` repeats only the duel and reload from an earned
windmill save. The retreat tactic and the full hero's ring route retain
their own acceptance checks.

The default test passed in 640.92 seconds across five fresh Windows
processes with all checkpoint-input variables unset. The first two
processes earned the shield, mushrooms and map. The third earned the
axe, the fourth defeated the guard and saved, and the fifth loaded
that victory and walked west. The guard entered Die at 813 and
awarded his experience at 903. Connor crossed the west opening at 41,
with level 4 and 40 health in the final slot.

#### Road to the hero's tomb

Two attempts started from that WestYard save with enemy AI active,
normal damage and no consumables. The first aimed directly from the
eastern bend to the western hills. Connor stopped at (156061, 262320)
on update 397 against the pine at (157296, 261811).

For the second run, the route followed the open ground north of the
pine, then turned east. Connor saved WestRoad at (175502, 310208)
with 21 health. He reached (220560, 306502) below the hero's hill on
update 932. On the way, two spriggans and a goblin had reduced his
health from 40 to 21. The tomb archer hit on updates 891, 936, 961 and
982. The last bolt reduced Connor's health from 6 to −2, before he
reached the steps or fought the archer at melee range. The checkpoint
proves the trail can be walked. Opening the tomb and obtaining the
ring still require their own successful run.

Two original Daventry scripts narrow the next test. The Small Crossbow
reward is authored in `sprgnbow.cs` for the church roof encounter.
The guard Connor killed inside the windmill was `sprigwnd1`; he came
away from that fight with the Small Axe, without a crossbow.
`worldinv.cs` also places a second shield potion at (251242, 259475)
and groups of three single mushrooms near (266000, 221000) and
(280000, 203000). Their placements are verified in the GOG archive.
The cottage garden pickup is described below. The second shield,
the other group of mushrooms and carrying extra supplies all the way
to the tomb fight remain to be checked.

**Garden supplies.** The three mushrooms near (280000, 203000) are
immediately outside Sarah's cottage. Starting with the earned cottage
shield, Connor opened the basket inside for five mushrooms, then walked
north outside its west wall. Each of the three individual mushrooms was
the first object hit through a visible gameplay pixel. Their original
pickup scripts increased the stack to six, seven and eight. The game
also opened the Jumping tutorial (case 133). The automated player
acknowledged it and used the menu to save GardenMushrooms.

A second Windows process loaded that save. Connor walked to the Wizard
and received the original fragment exchange and map. The GardenWizard
save still had eight mushrooms and one shield. The three garden items
remained gone. The two-process test took 327.90 seconds from the earned
VillageShield checkpoint.

From GardenWizard, a separate windmill run picked the original five
hillside mushrooms, bringing eight to thirteen. Connor defeated the
archer, earned the Small Axe and saved with nine mushrooms and
30 health. That run, including the fresh-process axe and bale check,
passed in 215.34 seconds. In the next run he consumed the shield at
the windmill doorway after the Henchman's scene, defeated the guard
and saved again. A fresh launch reached WestYard with 40 health and
two mushrooms. This pair of processes took 133.09 seconds.

The duel replay now presses the shield key when Connor reaches the
outward doorway. In one earlier run the final spoken line held controls
until update 616, so a key scheduled at 615 was correctly refused.
Both the garden save and the earlier windmill save passed when the
key followed the end of the scene.

**Fighting the hill archer.** Two further runs loaded the earned
two-mushroom WestYard save. In the first, Connor reached the steps
and swung the Small Axe in range. A bolt had already reduced his
health below zero at 957. His pending swing killed the archer at 969,
then Connor entered Die at 992. The first mushroom had been consumed
while Connor still had full health. In the second run one mushroom
was reserved for close combat. Connor hit the archer for 100 at
961, leaving it with 30 health. A bolt reduced Connor to −1 at 985.
Neither run earned a living checkpoint beyond the archer.

**The cottage Crystal.** Daventry's original item script places one
50-health Crystal at (290629, 200036, 13916). An orbit from the north
room failed to find a visible pixel. Sampling the original cottage
walls showed clear views from the east room. In a separate trial, a
real click at pixel (9, 465) hit the Crystal first; Connor picked it
up from 3,327 units away, within the 4,700-unit limit, and saved it.

The checked-in garden route now also acquires the Crystal. A fresh
Normal New Game earned the shield, then a second process picked up
the Crystal at update 46, opened the basket, picked three garden
mushrooms and saved. A third process loaded that inventory, met
the Wizard and saved with one Crystal, one shield, eight mushrooms
and the original story items. This run used no supplied checkpoint
and passed in 399.88 seconds.

A later start stopped in Fall beside the east-room wall after getting
the Crystal. The current approach aims from (291725, 197646) and then
crosses open floor into the passage. Walking from each of two actual
shield saves reached the basket and saved the Crystal, shield and
five mushrooms. Original fall behavior at the earlier wall is
tracked as Q197.

With `KQ8_GAME_DIR` set to the GOG installation, the following
check starts from Normal New Game by default. Set `KQ8_SHIELD_SLOT`
to an earned VillageShield menu save to repeat from the cottage.

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield garden_mushrooms_survive_the_trip_to_the_wizard -- --ignored --exact --test-threads=1
```

#### Earlier attempts at passing the guard

Two subsequent tests examined walking around the guard. Both started
with the same earned Small Axe save, 30 health, enemy AI and normal
damage. The first attempted a northern jump and collided with the
separate left fence. Connor died at update 860. The second used the
original building collision data to select a westward opening. The
cutscene returned control at 603 and Connor crossed the outer doorway
at 614. At 660 he had reached (138307, 250671) on the way to
(133850, 251500). Three spear hits dealt 16, 9 and 15 damage. He died
at 691. Neither run reached the hero's tomb. Stop changing these yard
landmarks until there is new evidence.

An independent way to prepare for the encounter is to collect the
healing and armour supplies placed by the original game. Daventry's
WorldInv.cs includes three individual mushrooms near the beginning,
and shield potions at (292519, 197716, 15246) and (251242, 259475, 13540).
The shield lasts 20 seconds and raises protection to at least 50.

Two attempts to collect all four village supplies stopped before any
item click. The first ran into Sarah's closed cottage door at 678.
The second opened the door using its original toggle script and crossed
the threshold at 550. Connor reached the east room, then stopped 397
units short of a replay waypoint at 869. A sweep against the original
cottage geometry confirms a clear line from the recorded camera to the
potion center. A separate two-process route now verifies that pickup.
The normal click ray first hit Sarah's door at update 528, then the
shield in the east room at 710. Its box was 1,427 units from Connor,
within the original 4,700-unit reach. The first health-item notice
opened at 755. A fresh process loaded the earned potion, used it with
key 5 and recorded protection increasing from 0 to 50 for 20 seconds.
The route took 146.04 seconds. The prepared duel above then carried
the potion to the guard and survived the encounter.

With `KQ8_GAME_DIR` set to the GOG folder, reproduce with
`cargo test --locked -p kq8-viewer --test daventry_cottage_shield -- --ignored --test-threads=1`.

The cottage-to-Wizard test continues from an earned, unconsumed shield
slot. Connor collects the five mushrooms in Sarah's basket, leaves
through the open door and walks north of the petrified girl to reach
the established Wizard route. A fresh process initially found the
petrified mother's clickable box in front of the basket. The final
walking replay pauses and uses the normal camera-drag input until a
visible gameplay pixel selects the requested object. At update 120 the
pixel selected St_smom, so the replay held its click. At 123 a new pixel
selected basket2, and its original script granted all five mushrooms.
Connor then earned Mask1, Piece1 and the Magic Map through the original
conversation. His named save kept all five mushrooms and the shield.
The optional `KQ8_SHIELD_SLOT` input repeats the continuation from
a previously earned VillageShield slot.

The New Game → wizard → windmill Small Axe route and reload passed again on
2026-11-06 in 402.85 seconds. The new `daventry_hero_ring` scenario continues
from that earned state into the original Henchtp/ax/Henchman cutscene, with
AI and damage enabled. Its standing charged-attack and normal-attack attempts
both died at the Henchman, at updates 761/762. No ring was obtained; later
path landmarks, the tomb guard, coffin and ring reload remain untested.
Q190 subsequently established how this stationary guard starts and completes
an attack, the reach check at its hit cue and the Wait attack deadline.

Two later unattended attempts used the same earned windmill save with 30
health, one Small Axe, zero mushrooms, normal damage and enemy AI enabled.
The first attempted retreat made the spear miss at update 756 at a distance
of 2,663 units with a reach of 2,500. Connor returned with a normal axe
swing at update 771 and dealt 38 damage, bringing the Henchman from 300 to
262 health. A spear hit during Connor's recovery dealt 20 damage at update
779. The next hit at 802 dealt 12. Connor was still backing at 2,578 units,
inside the extended 3,125 reach, and died.

The second attempt changed the return and recovery timing. It ended at a
different obstacle before the first spear swing. Both runs activated the
original Henchtp at update 49, before the walking route recorded its trigger
landmark at updates 641 and 646 respectively. In the first run Connor crossed
the exit westward. In the second he counted the exit landmark while running
east into the mill. His stopping animation reached the wall at 731, he hit
it head-on at 753, and the replay stopped at 906 at (142576, 249434, 15338).
The cause of the different arrival heading is undetermined. The timing
revision has no observed combat result. Further duel attempts stopped at the
two-attempt limit.

The replay now follows the authored trigger instead of returning to its
coordinates after the scene. It first observes the live trigger, steers
toward it, then releases movement as its script consumes it. The final
speech returns player control and removes the axe's scene object. From there
the replay accepts the exit only when Connor is outside and moving west.
Two independent Windows processes loaded copies of the same earned save.
Both recorded Henchtp activating at update 49, the scene finishing at 603
and an outward crossing at 614. The Henchman's first live attack began at
updates 635 and 629 respectively. The navigation check ends at attack
startup. The full campaign route retains the earlier measured retreat; the
revised return timing from the second attempt remains unverified.

With `KQ8_GAME_DIR` and the earned `KQ8_AXE_SLOT` set, reproduce just the
navigation check with:

```powershell
cargo test --locked -p kq8-viewer --test daventry_hero_ring earned_windmill_exit -- --ignored --nocapture
```

The tomb door and ring scripts do not test whether the Henchman died.
Walking around the guard after the scene is a separate, untested approach.
The duel, overland path, tomb guard, coffin, ring pickup and disk reload are
still open.

To reproduce the full candidate chain, run the following from the repository
root with `KQ8_GAME_DIR` set. It is an ignored diagnostic. An optional
`KQ8_AXE_SLOT` can point to an earned windmill slot-3.json to repeat only the
new leg. This repeats the continuation from a previously earned save.

```powershell
cargo test --locked -p kq8-viewer --test daventry_hero_ring earned_small_axe -- --ignored --nocapture
```

The separate `breakable_route_dispatches` test in that file passed against
the original tomb door through an ordinary axe swing. It explicitly sets up
position/equipment and protects Connor with AI off; it is a replay-tool
diagnostic, not campaign acceptance.

### Verified through the first wizard meeting

The unattended Windows run started at New Game → Normal with a fresh save directory. Connor completed the opening sequence and walked from Sarah's cottage to the wizard. The replay pressed his configured forward and turn keys, leaving movement, collision, health and enemy AI active. A first route hit the farmhouse wall. The final route walked around its south side. On reaching the wizard the replay submitted a named world click, which ran his normal distance and script handling.

The eighth spoken response exchanged the initial fragment for `Mask1` and `Piece1`. The thirteenth started the five-part map sequence. Its fourth animation cue awarded `MagicMap`. A slot written after the final line contained all three items, recorded the first meeting and confirmed removal of `Piece1A`. The replay acknowledges the original introduction, movement, camera and map help. It checks that the map tip displays the bound Tab key. The ingredient and portal stages above await their own continuous run.

With `KQ8_GAME_DIR` set to the original GOG folder, run:

```sh
cargo test --locked -p kq8-viewer --test daventry_wizard -- --ignored
```

The test opens and closes the Rust game automatically. Its input route is in `crates/kq8-viewer/tests/routes/daventry_wizard.json`, with logs and the resulting slot under `kq8-artifacts/wizard-route/`.

### Verified from the wizard to the windmill axe

Connor loaded his earned Magic Map slot and walked east of the goblin patrol. He picked up the original stacks of two and three mushrooms, climbed the hill and opened the windmill door. He drew his fists, defeated the spriggan archer, put his fists away and tried the axe from the floor. Its script refused him at a distance of 2,415 units. After he pushed the hay bale under the axe and jumped onto it, the same check measured 1,528 units, within the original 1,700-unit reach. The script gave one Small Axe, awarded 25 experience and placed the trigger for the henchman encounter. Connor was alive with the axe in the saved game.

The replay presses the bound movement, draw, away and jump keys. Like the wizard route, it uses named targets for original world clicks and melee attacks. Normal combat, healing, attack range, collision and the axe's distance test all run. The resulting slot marks the beginning of the henchman branch. A fresh-process reload now preserves the moved bale and axe positions. Completing the encounter and reaching the water mill are subsequent checks.

The lake bell requires the Small Axe in hand. `bell.cs` checks the equipped short weapon against item 1020 before counting a strike. The bell comes from the wizard's hidden chest. The tower book gives the Spell Page, the star globe exchanges it for Clarity, and revealing the chest makes the Lady Bell available. Three counted strikes summon the Lady. These dependencies have been identified in the game data. Their complete sequence still needs a continuous campaign run.

```sh
cargo test --locked -p kq8-viewer --test daventry_windmill -- --ignored
```

The default test earns the Magic Map through New Game in its first process, then loads it from disk in the second. To repeat only the windmill leg, set `KQ8_WIZARD_SLOT` to the Magic Map save produced by the wizard route. Each test creates a fresh working directory under `kq8-artifacts/windmill-route/`. Original game assets and save files stay local.

After the windmill save, another process loads Connor on the bale. It checks that his feet stay on the raised surface, that the axe is still at the position set by its pickup script, and that trying it again leaves his inventory and experience intact. It saves again to check both positions survive another save. The original persistence manager determines which props belong in the save. For those props, the Rust game records the position and orientation after a push or a script move. A prop missing from an older Rust save starts at its original location.
