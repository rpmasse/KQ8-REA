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
Verify acquisition and survival with ordinary controls before counting
this as a campaign route.

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
The route took 146.04 seconds. This establishes a resource the player
can earn before visiting the windmill. Carrying it to the encounter
and surviving the guard remain open.

With `KQ8_GAME_DIR` set to the GOG folder, reproduce with
`cargo test --locked -p kq8-viewer --test daventry_cottage_shield -- --ignored --test-threads=1`.

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
